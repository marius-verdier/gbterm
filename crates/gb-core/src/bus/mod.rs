mod constants;

#[cfg(test)]
pub(crate) mod test_utils;

use crate::{
    cartridge::Cartridge,
    joypad::{Button, Joypad},
    timer::Timer,
};
use constants::*;

pub struct Bus {
    cartridge: Cartridge,
    joypad: Joypad,
    timer: Timer,

    vram: [u8; 0x2000],
    wram: [u8; 0x2000],
    hram: [u8; 0x7F],

    oam: [u8; 0xA0],
    interrupt_enable: u8,
    interrupt_flag: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interrupt {
    VBlank,  // 0
    LcdStat, // 1
    Timer,   // 2
    Serial,  // 3
    Joypad,  // 4
}

impl Interrupt {
    pub(crate) fn bit(&self) -> u8 {
        match self {
            Self::VBlank => 0,
            Self::LcdStat => 1,
            Self::Timer => 2,
            Self::Serial => 3,
            Self::Joypad => 4,
        }
    }

    pub(crate) fn vector(&self) -> u16 {
        match self {
            Self::VBlank => 0x0040,
            Self::LcdStat => 0x0048,
            Self::Timer => 0x0050,
            Self::Serial => 0x0058,
            Self::Joypad => 0x0060,
        }
    }
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            joypad: Joypad::new(),
            timer: Timer::new(),

            vram: [0u8; 0x2000],
            wram: [0u8; 0x2000],
            hram: [0u8; 0x7F],

            oam: [0u8; 0xA0],
            interrupt_enable: 0u8,
            interrupt_flag: 0u8,
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
            INTERRUPT_FLAG_ADDRESS => self.interrupt_flag,
            JOYPAD_ADDRESS => self.joypad.read(),
            DIV_ADDRESS => self.timer.read_div(),
            TIMA_ADDRESS => self.timer.read_tima(),
            TMA_ADDRESS => self.timer.read_tma(),
            TAC_ADDRESS => self.timer.read_tac(),
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
            INTERRUPT_FLAG_ADDRESS => self.interrupt_flag = value,
            JOYPAD_ADDRESS => self.joypad.write(value),
            DIV_ADDRESS => {
                if self.timer.write_div() {
                    self.request_interrupt(Interrupt::Timer);
                }
            }
            TIMA_ADDRESS => self.timer.write_tima(value),
            TMA_ADDRESS => self.timer.write_tma(value),
            TAC_ADDRESS => {
                if self.timer.write_tac(value) {
                    self.request_interrupt(Interrupt::Timer);
                }
            }
            _ => {}
        }
    }

    pub fn press_button(&mut self, button: Button) {
        self.joypad.press(button);
    }

    pub fn release_button(&mut self, button: Button) {
        self.joypad.release(button);
    }

    pub fn tick(&mut self) {
        if self.timer.tick() {
            self.request_interrupt(Interrupt::Timer);
        }
    }

    pub fn request_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_flag |= 1 << interrupt.bit();
    }

    pub fn enable_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_enable |= 1 << interrupt.bit();
    }

    pub fn pending_interrupts(&self) -> u8 {
        self.interrupt_flag & self.interrupt_enable & 0x1F
    }

    pub fn next_pending_interrupt(&self) -> Option<Interrupt> {
        let pending = self.pending_interrupts();

        let priority_order = [
            Interrupt::VBlank,
            Interrupt::LcdStat,
            Interrupt::Timer,
            Interrupt::Serial,
            Interrupt::Joypad,
        ];

        priority_order
            .into_iter()
            .find(|interrupt| pending & (1 << interrupt.bit()) != 0)
    }

    pub fn clear_interrupt(&mut self, interrupt: Interrupt) {
        self.interrupt_flag &= !(1 << interrupt.bit());
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

    #[test]
    fn routes_joypad_register() {
        let mut bus = make_test_bus();

        bus.press_button(Button::A);

        // bit 5 = 0 → button group selected
        bus.write8(0xFF00, 0b1101_1111);

        assert_eq!(bus.read8(0xFF00) & 0x0F, 0b1110);

        bus.release_button(Button::A);

        assert_eq!(bus.read8(0xFF00) & 0x0F, 0b1111);
    }

    #[test]
    fn routes_to_timer_registers() {
        let mut bus = make_test_bus();

        bus.write8(TIMA_ADDRESS, 0x44);
        bus.write8(TMA_ADDRESS, 0x66);
        bus.write8(TAC_ADDRESS, 0b101);

        assert_eq!(bus.read8(TIMA_ADDRESS), 0x44);
        assert_eq!(bus.read8(TMA_ADDRESS), 0x66);
        assert_eq!(bus.read8(TAC_ADDRESS) & 0b111, 0b101);
    }

    #[test]
    fn bus_ticks_timer() {
        let mut bus = make_test_bus();

        bus.write8(0xFF07, 0b101);

        for _ in 0..16 {
            bus.tick();
        }

        assert_eq!(bus.read8(0xFF05), 1);
    }

    #[test]
    fn interrupt_timer_changes_flag() {
        let mut bus = make_test_bus();

        bus.request_interrupt(Interrupt::Timer);

        assert_eq!(bus.read8(INTERRUPT_FLAG_ADDRESS) & (1 << 2), 1 << 2);
    }

    #[test]
    fn interrupt_must_be_requested_and_enabled_to_be_pending() {
        let mut bus = make_test_bus();

        bus.request_interrupt(Interrupt::Timer);

        // IF Timer = 1, mais IE Timer = 0
        assert_eq!(bus.pending_interrupts(), 0);

        bus.write8(INTERRUPT_ENABLE_ADDRESS, 1 << 2);

        // IF Timer = 1 et IE Timer = 1
        assert_eq!(bus.pending_interrupts(), 1 << 2);
    }

    #[test]
    fn timer_interrupt_requested() {
        let mut bus = make_test_bus();

        bus.write8(TIMA_ADDRESS, 0xFF);
        bus.write8(TMA_ADDRESS, 0x42);
        bus.write8(TAC_ADDRESS, 0b101);

        for _ in 0..16 {
            bus.tick();
        }

        assert_eq!(bus.read8(TIMA_ADDRESS), 0x42);
        assert_eq!(bus.read8(INTERRUPT_FLAG_ADDRESS) & (1 << 2), 1 << 2);
    }
}
