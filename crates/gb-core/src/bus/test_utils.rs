use super::Bus;
use crate::cartridge::Cartridge;
use crate::cartridge::test_utils::make_test_rom;

pub(crate) fn make_test_bus() -> Bus {
    let rom = make_test_rom();
    let cartridge = Cartridge::new(rom).unwrap();

    let mut bus = Bus::new(cartridge);
    bus.vram = [0xAA; 0x2000];
    bus.wram = [0xBB; 0x2000];
    bus.hram = [0xCC; 0x7F];

    bus
}
