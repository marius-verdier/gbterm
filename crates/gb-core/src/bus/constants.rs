// VRAM -> Video Data
pub(super) const VRAM_START: u16 = 0x8000;
pub(super) const VRAM_END: u16 = 0x9FFF;

// WRAM -> Working Memory
pub(super) const WRAM_START: u16 = 0xC000;
pub(super) const WRAM_END: u16 = 0xDFFF;
// Mirrored WRAM
pub(super) const ECHO_RAM_START: u16 = 0xE000;
pub(super) const ECHO_RAM_END: u16 = 0xFDFF;

// HRAM -> Speedy Memory
pub(super) const HRAM_START: u16 = 0xFF80;
pub(super) const HRAM_END: u16 = 0xFFFE;

// OAM -> Object Attribute Memory
pub(super) const OAM_START: u16 = 0xFE00;
pub(super) const OAM_END: u16 = 0xFE9F;

// Interrupt / Enable
pub(super) const INTERRUPT_ENABLE_ADDRESS: u16 = 0xFFFF;

// Joypad
pub(super) const JOYPAD_ADDRESS: u16 = 0xFF00;

// Timer
pub(super) const DIV_ADDRESS: u16 = 0xFF04;
pub(super) const TIMA_ADDRESS: u16 = 0xFF05;
pub(super) const TMA_ADDRESS: u16 = 0xFF06;
pub(super) const TAC_ADDRESS: u16 = 0xFF07;
