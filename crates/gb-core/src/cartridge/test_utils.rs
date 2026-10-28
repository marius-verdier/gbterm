use super::constants::*;

pub(crate) fn make_test_rom() -> Vec<u8> {
    let mut rom = vec![0u8; 8 * ROM_BANK_SIZE];

    for bank in 0..8 {
        let start = bank * ROM_BANK_SIZE;
        let end = start + ROM_BANK_SIZE;

        rom[start..end].fill(bank as u8);
    }

    rom[CARTRIDGE_TYPE_OFFSET] = 0x13;
    rom[ROM_SIZE_OFFSET] = 0x02;
    rom[RAM_SIZE_OFFSET] = 0x03; // RAM SIZE
    rom[CGB_FLAG_OFFSET] = 0x00;

    rom
}
