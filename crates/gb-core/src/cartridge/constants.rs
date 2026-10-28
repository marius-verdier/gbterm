pub(super) const CARTRIDGE_TYPE_OFFSET: usize = 0x0147;
pub(super) const ROM_SIZE_OFFSET: usize = 0x0148;
pub(super) const CGB_FLAG_OFFSET: usize = 0x0143;
pub(super) const TITLE_START_OFFSET: usize = 0x0134;
pub(super) const RAM_SIZE_OFFSET: usize = 0x0149;

pub(super) const HEADER_CHECKSUM_START: usize = 0x0134;
pub(super) const HEADER_CHECKSUM_END: usize = 0x014C;
pub(super) const HEADER_CHECKSUM_OFFSET: usize = 0x014D;

pub(super) const ROM_BANK_0_START: u16 = 0x0000;
pub(super) const ROM_BANK_0_END: u16 = 0x3FFF;
pub(super) const ROM_SWITCHABLE_BANK_START: u16 = 0x4000;
pub(super) const ROM_SWITCHABLE_BANK_END: u16 = 0x7FFF;
pub(super) const ROM_BANK_SIZE: usize = 0x4000;

// ROM BANK
pub(super) const MBC3_ROM_BANK_SELECT_START: u16 = 0x2000;
pub(super) const MBC3_ROM_BANK_SELECT_END: u16 = 0x3FFF;
pub(super) const MBC3_ROM_BANK_MASK: u8 = 0x7F;

// RAM
pub(super) const MBC3_RAM_ENABLE_START: u16 = 0x0000;
pub(super) const MBC3_RAM_ENABLE_END: u16 = 0x1FFF;
pub(super) const MBC3_RAM_ENABLE_MASK: u8 = 0x0F;
pub(super) const MBC3_RAM_ENABLE_VALUE: u8 = 0x0A;

pub(super) const EXTERNAL_RAM_START: u16 = 0xA000;
pub(super) const EXTERNAL_RAM_END: u16 = 0xBFFF;

pub(super) const RAM_BANK_SIZE: usize = 0x2000;
pub(super) const MBC3_RAM_BANK_SELECT_START: u16 = 0x4000;
pub(super) const MBC3_RAM_BANK_SELECT_END: u16 = 0x5FFF;
