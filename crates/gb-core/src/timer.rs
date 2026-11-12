pub struct Timer {
    divider: u16,

    tima: u8,
    tma: u8,
    tac: u8,

    reload_delay: Option<u8>,
    reloaded_this_tick: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            divider: 0u16,

            tima: 0u8,
            tma: 0u8,
            tac: 0u8,

            reload_delay: None,
            reloaded_this_tick: false,
        }
    }

    fn increment_tima(&mut self) {
        if self.tima == 0xFF {
            self.tima = 0x00;
            self.reload_delay = Some(4);
        } else {
            self.tima = self.tima.wrapping_add(1);
        }
    }

    pub fn tick(&mut self) -> bool {
        self.reloaded_this_tick = false;
        let mut request_interrupt = false;

        if let Some(delay) = self.reload_delay {
            if delay == 1 {
                self.tima = self.tma;
                self.reload_delay = None;
                self.reloaded_this_tick = true;
                request_interrupt = true;
            } else {
                self.reload_delay = Some(delay - 1);
            }
        }

        let old_signal = self.timer_signal();
        self.divider = self.divider.wrapping_add(1);
        let new_signal = self.timer_signal();

        if old_signal && !new_signal {
            self.increment_tima();
        }

        request_interrupt
    }

    fn timer_signal(&self) -> bool {
        let tima_active_from_tac = self.tac & (1 << 2) != 0;

        if !tima_active_from_tac {
            return false;
        }

        let target_bit: usize = match self.tac & 0b11 {
            0b00 => 9,
            0b01 => 3,
            0b10 => 5,
            0b11 => 7,
            _ => unreachable!(),
        };

        self.divider & (1 << target_bit) != 0
    }
    pub fn read_div(&self) -> u8 {
        (self.divider >> 8) as u8
    }

    pub fn write_div(&mut self) -> bool {
        let old_timer = self.timer_signal();

        self.divider = 0;

        let new_timer = self.timer_signal();

        if old_timer && !new_timer {
            self.increment_tima();
        }

        false
    }

    pub fn read_tima(&self) -> u8 {
        self.tima
    }
    pub fn write_tima(&mut self, value: u8) {
        if self.reloaded_this_tick {
            return;
        }

        self.tima = value;
        self.reload_delay = None;
    }

    pub fn read_tma(&self) -> u8 {
        self.tma
    }
    pub fn write_tma(&mut self, value: u8) {
        self.tma = value;

        if self.reloaded_this_tick {
            self.tima = value;
        }
    }

    pub fn read_tac(&self) -> u8 {
        self.tac
    }
    pub fn write_tac(&mut self, value: u8) -> bool {
        let old_timer = self.timer_signal();
        self.tac = value & 0b111;
        let new_timer = self.timer_signal();

        if old_timer && !new_timer {
            self.increment_tima();
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_precision_on_ticks() {
        let mut timer = Timer::new();

        for _ in 0..255 {
            timer.tick();
        }

        assert_eq!(timer.read_div(), 0);

        timer.tick();

        assert_eq!(timer.read_div(), 1);
    }

    #[test]
    fn divider_write_resets_ticks() {
        let mut timer = Timer::new();

        for _ in 0..257 {
            timer.tick();
        }

        assert_eq!(timer.read_div(), 1);

        timer.write_div();

        assert_eq!(timer.read_div(), 0);
    }
    #[test]
    fn tima_increments_on_selected_falling_edge() {
        let mut timer = Timer::new();

        timer.write_tac(0b101);

        for _ in 0..15 {
            timer.tick();
        }

        assert_eq!(timer.read_tima(), 0);

        timer.tick();

        assert_eq!(timer.read_tima(), 1);
    }

    #[test]
    fn tima_uses_selected_frequency() {
        let cases = [(0b100, 1024), (0b101, 16), (0b110, 64), (0b111, 256)];

        for (tac, ticks) in cases {
            let mut timer = Timer::new();

            timer.write_tac(tac);

            for _ in 0..ticks - 1 {
                timer.tick();
            }

            assert_eq!(timer.read_tima(), 0);

            timer.tick();

            assert_eq!(timer.read_tima(), 1);
        }
    }

    #[test]
    fn resetting_div_can_increment_tima() {
        let mut timer = Timer::new();

        timer.write_tac(0b101);

        for _ in 0..8 {
            timer.tick();
        }

        assert_eq!(timer.read_tima(), 0);

        timer.write_div();

        assert_eq!(timer.read_tima(), 1);
    }

    #[test]
    fn changing_tac_can_increment_tima() {
        let mut timer = Timer::new();

        timer.write_tac(0b101);

        for _ in 0..8 {
            timer.tick();
        }

        timer.write_tac(0);

        assert_eq!(timer.read_tima(), 1);
    }

    #[test]
    fn normal_tima_increment_does_not_request_interrupt() {
        let mut timer = Timer::new();

        timer.write_tac(0b101);

        let mut interrupt_requested = false;

        for _ in 0..16 {
            interrupt_requested |= timer.tick();
        }

        assert_eq!(timer.read_tima(), 1);
        assert!(!interrupt_requested);
    }

    #[test]
    fn tima_overflow_reloads_after_four_t_cycles() {
        let mut timer = Timer::new();

        timer.write_tima(0xFF);
        timer.write_tma(0x42);
        timer.write_tac(0b101);

        // Advance to just before the first falling edge.
        for _ in 0..15 {
            assert!(!timer.tick());
        }

        assert_eq!(timer.read_tima(), 0xFF);

        // Falling edge: TIMA overflows to 0x00.
        assert!(!timer.tick());
        assert_eq!(timer.read_tima(), 0x00);

        // Reload is still pending for three more T-cycles.
        for _ in 0..3 {
            assert!(!timer.tick());
            assert_eq!(timer.read_tima(), 0x00);
        }

        // Fourth T-cycle after overflow:
        // TIMA reloads from TMA and the interrupt is requested.
        assert!(timer.tick());
        assert_eq!(timer.read_tima(), 0x42);
    }

    #[test]
    fn writing_tma_during_reload_delay_changes_reload_value() {
        let mut timer = Timer::new();

        timer.write_tima(0xFF);
        timer.write_tma(0x42);
        timer.write_tac(0b101);

        // Reach the falling edge that causes TIMA to overflow.
        for _ in 0..16 {
            assert!(!timer.tick());
        }

        assert_eq!(timer.read_tima(), 0x00);

        // Move one T-cycle into the reload window.
        assert!(!timer.tick());

        // Changing TMA does not cancel the pending reload.
        timer.write_tma(0x99);

        assert_eq!(timer.read_tma(), 0x99);
        assert_eq!(timer.read_tima(), 0x00);

        // Two more T-cycles: reload is still pending.
        assert!(!timer.tick());
        assert!(!timer.tick());

        assert_eq!(timer.read_tima(), 0x00);

        // Fourth T-cycle after the overflow:
        // TIMA reloads using the NEW TMA value.
        assert!(timer.tick());

        assert_eq!(timer.read_tima(), 0x99);
    }
}
