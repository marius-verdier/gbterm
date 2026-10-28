mod constants;

#[cfg(test)]
pub(crate) mod test_utils;

use crate::{
    cartridge::Cartridge,
};
use constants::*;

pub struct Bus {
    cartridge: Cartridge,

    vram: [u8; 0x2000],
    wram: [u8; 0x2000],
    hram: [u8; 0x7F],

    oam: [u8; 0xA0],
    interrupt_enable: u8,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,

            vram: [0u8; 0x2000],
            wram: [0u8; 0x2000],
            hram: [0u8; 0x7F],

            oam: [0u8; 0xA0],
            interrupt_enable: 0u8,
        }
    }

    pub fn read8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.cartridge.read(address).unwrap_or(0xFF),
            VRAM_START..=VRAM_END => {
                let index = usize::from(address - VRAM_START);

                self.vram.get(index).copied().unwrap_or(0xFF)
            }
            0xA000..=0xBFFF => self.cartridge.read(address).unwrap_or(0xFF),
            WRAM_START..=WRAM_END => {
                let index = usize::from(address - WRAM_START);

                self.wram.get(index).copied().unwrap_or(0xFF)
            }
            ECHO_RAM_START..=ECHO_RAM_END => {
                let index = usize::from(address - ECHO_RAM_START);

                self.wram.get(index).copied().unwrap_or(0xFF)
            }
            HRAM_START..=HRAM_END => {
                let index = usize::from(address - HRAM_START);

                self.hram.get(index).copied().unwrap_or(0xFF)
            }
            OAM_START..=OAM_END => {
                let index = usize::from(address - OAM_START);

                self.oam.get(index).copied().unwrap_or(0xFF)
            }
            0xFEA0..=0xFEFF => 0xFF, // Unused memory space
            INTERRUPT_ENABLE_ADDRESS => self.interrupt_enable,
            _ => 0xFF,
        }
    }

    pub fn write8(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => self.cartridge.write(address, value),
            VRAM_START..=VRAM_END => {
                let index = usize::from(address - VRAM_START);

                self.vram[index] = value
            }
            0xA000..=0xBFFF => self.cartridge.write(address, value),
            WRAM_START..=WRAM_END => {
                let index = usize::from(address - WRAM_START);

                self.wram[index] = value
            }
            ECHO_RAM_START..=ECHO_RAM_END => {
                let index = usize::from(address - ECHO_RAM_START);

                self.wram[index] = value
            }
            HRAM_START..=HRAM_END => {
                let index = usize::from(address - HRAM_START);

                self.hram[index] = value
            }
            OAM_START..=OAM_END => {
                let index = usize::from(address - OAM_START);

                self.oam[index] = value
            }
            0xFEA0..=0xFEFF => {} // Unused memory space
            INTERRUPT_ENABLE_ADDRESS => self.interrupt_enable = value,
            _ => {}
        }
    }

}

#[cfg(test)]
mod tests {
    use super::test_utils::make_test_bus;
    use super::*;

    #[test]
    fn read_is_well_directed_to_components() {
        let bus = make_test_bus();

        assert_eq!(bus.read8(0x4000), 0x01); // Default Cartridge ROM Bank
        assert_eq!(bus.read8(0xA000), 0xFF); // Default because RAM is not enabled
        assert_eq!(bus.read8(0x8000), 0xAA); // routed to VRAM
        assert_eq!(bus.read8(0xC000), 0xBB); // routed to WRAM
        assert_eq!(bus.read8(0xFF80), 0xCC); // routed to HRAM
    }

    #[test]
    fn bus_routes_writes_to_cartridge() {
        let mut bus = make_test_bus();

        assert_eq!(bus.read8(0x4000), 0x01);

        bus.write8(0x2000, 0x05);

        assert_eq!(bus.read8(0x4000), 0x05);
    }

    #[test]
    fn bus_reads_back_internal_memory_writes() {
        let mut bus = make_test_bus();

        bus.write8(0x8000, 0x11);
        bus.write8(0xC000, 0x22);
        bus.write8(0xFF80, 0x33);

        assert_eq!(bus.read8(0x8000), 0x11);
        assert_eq!(bus.read8(0x8001), 0xAA);

        assert_eq!(bus.read8(0xC000), 0x22);
        assert_eq!(bus.read8(0xC001), 0xBB);

        assert_eq!(bus.read8(0xFF80), 0x33);
        assert_eq!(bus.read8(0xFF81), 0xCC);
    }

    #[test]
    fn echo_ram_mirrors_partial_wram() {
        let mut bus = make_test_bus();

        bus.write8(0xC123, 0x42);
        assert_eq!(bus.read8(0xE123), 0x42);

        bus.write8(0xE456, 0x69);
        assert_eq!(bus.read8(0xC456), 0x69);

        bus.write8(0xDDFF, 0x99);
        assert_eq!(bus.read8(0xFDFF), 0x99);
    }

    #[test]
    fn bus_reads_writes_to_oam() {
        let mut bus = make_test_bus();

        bus.write8(0xFE00, 0x11);
        bus.write8(0xFE9F, 0x22);

        assert_eq!(bus.read8(0xFE00), 0x11);
        assert_eq!(bus.read8(0xFE9F), 0x22);
    }

    #[test]
    fn reads_and_writes_interrupt_enable_register() {
        let mut bus = make_test_bus();

        assert_eq!(bus.read8(0xFFFF), 0x00);

        bus.write8(0xFFFF, 0x15);

        assert_eq!(bus.read8(0xFFFF), 0x15);
    }

}
