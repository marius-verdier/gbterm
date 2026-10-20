use std::io::Result;
use std::path;

const CARTRIDGE_TYPE_OFFSET: usize = 0x0147;
const ROM_SIZE_OFFSET: usize = 0x0148;
const CGB_FLAG_OFFSET: usize = 0x0143;
const TITLE_START_OFFSET: usize = 0x0134;
const RAM_SIZE_OFFSET: usize = 0x0149;

const HEADER_CHECKSUM_START: usize = 0x0134;
const HEADER_CHECKSUM_END: usize = 0x014C;
const HEADER_CHECKSUM_OFFSET: usize = 0x014D;

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

pub struct Cartridge {
    rom: Vec<u8>,
    header: CartridgeHeader,
}

impl Cartridge {
    pub fn new(rom: Vec<u8>) -> Option<Self> {
        let header = CartridgeHeader::parse(&rom)?;

        Some(Self { rom, header })
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

    pub fn rom_len(&self) -> usize {
        self.rom.len()
    }

    pub fn header_checksum_match(&self) -> bool {
        self.header().checksum_valid()
    }
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

#[derive(Debug)]
pub enum CgbSupport {
    DmgOnly,
    Supported,
    Required,
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
