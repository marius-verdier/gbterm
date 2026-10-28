mod constants;

use constants::*;
use std::io::Result;
use std::path;

pub fn rom_size_bytes(code: u8) -> Option<usize> {
    if code > 0x08 {
        return None;
    }

    Some((32 * 1024) << code)
}

pub fn ram_size_bytes(code: u8) -> Option<usize> {
    match code {
        0x00 => Some(0),
        0x02 => Some(8 * 1024),
        0x03 => Some(32 * 1024),
        0x04 => Some(128 * 1024),
        0x05 => Some(64 * 1024),
        _ => None,
    }
}

#[derive(Debug)]
pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc3RamBattery,
    Unknown(u8),
}

pub struct Cartridge {
    rom: Vec<u8>,
    ram: Vec<u8>,
    header: CartridgeHeader,
    //,
    active_bank: u8,
    ram_enabled: bool,
}

pub struct CartridgeHeader {
    pub title: String,
    pub cartridge_type: CartridgeType,
    pub rom_size: usize,
    pub ram_size: usize,
    pub cgb_support: CgbSupport,

    pub computed_checksum: u8,
    pub stored_checksum: u8,
}

#[derive(Debug)]
pub enum CgbSupport {
    DmgOnly,
    Supported,
    Required,
}

impl CartridgeType {
    pub fn from_code(code: u8) -> Self {
        match code {
            0x00 => Self::RomOnly,
            0x01 => Self::Mbc1,
            0x13 => Self::Mbc3RamBattery,
            _ => Self::Unknown(code),
        }
    }
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Option<Self> {
        let header = CartridgeHeader::parse(&rom)?;

        let ram = vec![0x00; header.ram_size];
        Some(Self {
            rom,
            ram,
            header,
            //
            active_bank: 1u8,
            ram_enabled: false,
        })
    }

    pub fn from_rom_file(path: impl AsRef<path::Path>) -> Result<Self> {
        let rom = std::fs::read(path)?;

        let cartridge = Self::new(rom)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid ROM"))?;

        Ok(cartridge)
    }

    pub fn header(&self) -> &CartridgeHeader {
        &self.header
    }

    pub fn active_bank(&self) -> u8 {
        self.active_bank
    }

    pub fn rom_len(&self) -> usize {
        self.rom.len()
    }

    pub fn header_checksum_match(&self) -> bool {
        self.header().checksum_valid()
    }

    pub fn read(&self, address: u16) -> Option<u8> {
        match address {
            ROM_BANK_0_START..=ROM_BANK_0_END => self.rom.get(usize::from(address)).copied(),

            ROM_SWITCHABLE_BANK_START..=ROM_SWITCHABLE_BANK_END => {
                let bank_start = ROM_BANK_SIZE * usize::from(self.active_bank);
                let offset_in_bank = usize::from(address - ROM_SWITCHABLE_BANK_START);
                let index = bank_start + offset_in_bank;

                self.rom.get(index).copied()
            }

            _ => None,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        if let CartridgeType::Mbc3RamBattery = &self.header.cartridge_type {
            match address {
                MBC3_ROM_BANK_SELECT_START..=MBC3_ROM_BANK_SELECT_END => {
                    let usable_value = match value & MBC3_ROM_BANK_MASK {
                        0x00 => 1u8,
                        v => v,
                    };

                    self.active_bank = usable_value;
                }

                MBC3_RAM_ENABLE_START..=MBC3_RAM_ENABLE_END => {
                    let usable_value = value & MBC3_RAM_ENABLE_MASK;

                    self.ram_enabled = usable_value == MBC3_RAM_ENABLE_VALUE;
                }

                _ => {}
            }
        }
    }
}

impl CartridgeHeader {
    pub fn parse(rom: &[u8]) -> Option<Self> {
        if rom.len() < HEADER_CHECKSUM_OFFSET + 1 {
            return None;
        }

        let cartridge_type = CartridgeType::from_code(rom[CARTRIDGE_TYPE_OFFSET]);
        let rom_size = rom_size_bytes(rom[ROM_SIZE_OFFSET])?;
        let ram_size = ram_size_bytes(rom[RAM_SIZE_OFFSET])?;
        let cgb_support = CgbSupport::from_flag(rom[CGB_FLAG_OFFSET]);

        let title_bytes = &rom[TITLE_START_OFFSET..=cgb_support.title_end()];
        let title_end = title_bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(title_bytes.len());
        let title = String::from_utf8_lossy(&title_bytes[..title_end]).to_string();

        let computed_checksum = Self::compute_checksum(rom)?;
        let stored_checksum = rom[HEADER_CHECKSUM_OFFSET];

        Some(Self {
            title,
            cartridge_type,
            rom_size,
            ram_size,
            cgb_support,
            computed_checksum,
            stored_checksum,
        })
    }

    fn compute_checksum(rom: &[u8]) -> Option<u8> {
        let mut checksum = 0u8;

        for byte in &rom[HEADER_CHECKSUM_START..=HEADER_CHECKSUM_END] {
            checksum = checksum.wrapping_sub(*byte).wrapping_sub(1);
        }

        Some(checksum)
    }

    pub fn checksum_valid(&self) -> bool {
        self.stored_checksum == self.computed_checksum
    }
}

impl CgbSupport {
    pub fn from_flag(flag: u8) -> Self {
        match flag {
            0x80 => Self::Supported,
            0xC0 => Self::Required,
            _ => Self::DmgOnly,
        }
    }

    pub fn title_end(&self) -> usize {
        match self {
            Self::DmgOnly => CGB_FLAG_OFFSET,
            Self::Supported | Self::Required => CGB_FLAG_OFFSET - 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_rom() -> Vec<u8> {
        let mut rom = vec![0u8; 8 * ROM_BANK_SIZE];

        for bank in 0..8 {
            let start = bank * ROM_BANK_SIZE;
            let end = start + ROM_BANK_SIZE;

            rom[start..end].fill(bank as u8);
        }

        rom[0x0147] = 0x13;
        rom[0x0148] = 0x02;
        rom[0x0149] = 0x00;
        rom[0x0143] = 0x00;

        rom
    }

    #[test]
    fn reads_default_rom_bank() {
        let rom = make_test_rom();
        let cartridge = Cartridge::new(rom).unwrap();

        assert_eq!(cartridge.read(0x4000), Some(0x01));
    }

    #[test]
    fn switches_rom_bank() {
        let rom = make_test_rom();
        let mut cartridge = Cartridge::new(rom).unwrap();

        assert_eq!(cartridge.read(0x4000), Some(0x01));

        cartridge.write(0x2000, 0x05);

        assert_eq!(cartridge.read(0x4000), Some(0x05));
    }

    #[test]
    fn rom_bank_zero_maps_to_one() {
        let rom = make_test_rom();
        let mut cartridge = Cartridge::new(rom).unwrap();

        cartridge.write(0x2000, 0x00);

        assert_eq!(cartridge.read(0x4000), Some(0x01));
    }

    #[test]
    fn rom_bank_number_is_masked() {
        let rom = make_test_rom();
        let mut cartridge = Cartridge::new(rom).unwrap();

        cartridge.write(0x2000, 0x85);

        assert_eq!(cartridge.read(0x4000), Some(0x05));
    }

    #[test]
    fn reads_fixed_bank_zero() {
        let rom = make_test_rom();
        let mut cartridge = Cartridge::new(rom).unwrap();

        cartridge.write(0x2000, 0x05);

        assert_eq!(cartridge.read(0x0000), Some(0x00));
    }
}
