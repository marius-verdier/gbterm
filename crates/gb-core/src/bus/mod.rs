mod constants;

#[cfg(test)]
pub(crate) mod test_utils;

use crate::{
    cartridge::Cartridge,
    joypad::{Button, Joypad},
    serial::Serial,
    timer::Timer,
};
use constants::*;

pub struct Bus {
    cartridge: Cartridge,
    joypad: Joypad,
    timer: Timer,
    serial: Serial,

    vram: [u8; 0x2000],
    wram: [u8; 0x2000],
    hram: [u8; 0x7F],

    oam: [u8; 0xA0],

    dma: u8,
    dma_active: bool,
    dma_source: u16,
    dma_index: u16,
    dma_ticks: u8,

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
            serial: Serial::new(),

            vram: [0u8; 0x2000],
            wram: [0u8; 0x2000],
            hram: [0u8; 0x7F],

            oam: [0u8; 0xA0],

            dma: 0u8,
            dma_active: false,
            dma_source: 0u16,
            dma_index: 0u16,
            dma_ticks: 0u8,

            interrupt_enable: 0u8,
            interrupt_flag: 0u8,
        }
    }

    fn start_dma(&mut self, value: u8) {
        self.dma = value;

        self.dma_source = (value as u16) << 8;
        self.dma_index = 0;
        self.dma_ticks = 0;
        self.dma_active = true;
    }

    pub fn dma_active(&self) -> bool {
        self.dma_active
    }

    fn tick_dma(&mut self) {
        if !self.dma_active {
            return;
        }

        self.dma_ticks += 1;

        if self.dma_ticks < 4 {
            return;
        }

        self.dma_ticks = 0;

        let source_address = self.dma_source + self.dma_index;
        let value = self.read8(source_address);

        self.oam[self.dma_index as usize] = value;

        self.dma_index += 1;

        if self.dma_index == 0xA0 {
            self.dma_active = false;
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
            SERIAL_DATA_ADDRESS => self.serial.read_data(),
            SERIAL_CONTROL_ADDRESS => self.serial.read_control(),
            DMA_ADDRESS => self.dma,
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
            SERIAL_DATA_ADDRESS => self.serial.write_data(value),
            SERIAL_CONTROL_ADDRESS => {
                if self.serial.write_control(value) {
                    self.request_interrupt(Interrupt::Serial);
                }
            }
            DMA_ADDRESS => self.start_dma(value),
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

        self.tick_dma();
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

    pub fn serial_output(&self) -> &str {
        self.serial.output()
    }

    pub fn cpu_read8(&self, address: u16) -> u8 {
        if self.dma_active && !(0xFF80..=0xFFFE).contains(&address) {
            return 0xFF;
        }

        self.read8(address)
    }

    pub fn cpu_write8(&mut self, address: u16, value: u8) {
        if self.dma_active && !(0xFF80..=0xFFFE).contains(&address) {
            return;
        }

        self.write8(address, value)
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
    fn timer_interrupt_requested_after_delayed_reload() {
        let mut bus = make_test_bus();

        bus.write8(0xFF05, 0xFF); // TIMA
        bus.write8(0xFF06, 0x42); // TMA
        bus.write8(0xFF07, 0b101); // Enable timer, divider bit 3

        // Reach the falling edge that causes the overflow.
        for _ in 0..16 {
            bus.tick();
        }

        // TIMA has overflowed but has not reloaded yet.
        assert_eq!(bus.read8(0xFF05), 0x00);
        assert_eq!(bus.read8(0xFF0F) & (1 << 2), 0);

        // Wait for the four-cycle reload delay.
        for _ in 0..4 {
            bus.tick();
        }

        // TIMA reloads from TMA and requests the timer interrupt.
        assert_eq!(bus.read8(0xFF05), 0x42);
        assert_ne!(bus.read8(0xFF0F) & (1 << 2), 0);
    }

    #[test]
    fn dma_completes_after_640_t_cycles() {
        let mut bus = make_test_bus();

        for offset in 0..0xA0u16 {
            bus.write8(0xC000 + offset, offset as u8);
        }

        bus.write8(0xFF46, 0xC0);

        for _ in 0..639 {
            bus.tick();
        }

        assert!(bus.dma_active);

        bus.tick();

        assert!(!bus.dma_active);

        for offset in 0..0xA0u16 {
            assert_eq!(bus.read8(0xFE00 + offset), offset as u8,);
        }
    }

    #[test]
    fn cpu_cannot_read_wram_during_dma() {
        let mut bus = make_test_bus();

        bus.write8(0xC000, 0x42);

        assert_eq!(bus.cpu_read8(0xC000), 0x42);

        bus.write8(0xFF46, 0xC0);

        assert_eq!(bus.cpu_read8(0xC000), 0xFF);
    }

    #[test]
    fn cpu_cannot_write_wram_during_dma() {
        let mut bus = make_test_bus();

        bus.write8(0xC100, 0x11);

        bus.write8(0xFF46, 0xC0);

        bus.cpu_write8(0xC100, 0x99);

        // Raw/internal read: verify that the CPU write was ignored.
        assert_eq!(bus.read8(0xC100), 0x11);
    }

    #[test]
    fn cpu_can_access_hram_during_dma() {
        let mut bus = make_test_bus();

        bus.write8(0xFF46, 0xC0);

        bus.cpu_write8(0xFF80, 0x42);

        assert_eq!(bus.cpu_read8(0xFF80), 0x42);
    }
}
