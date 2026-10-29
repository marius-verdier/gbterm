pub struct Timer {
    divider: u16,

    tima: u8,
    tma: u8,
    tac: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            divider: 0u16,
            tima: 0u8,
            tma: 0u8,
            tac: 0u8,
        }
    }

    pub fn tick(&mut self) {
        let old_timer = self.timer_signal();

        self.divider = self.divider.wrapping_add(1);

        let new_timer = self.timer_signal();

        if old_timer && !new_timer {
            self.tima = self.tima.wrapping_add(1);
        }
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

    pub fn write_div(&mut self) {
        let old_timer = self.timer_signal();

        self.divider = 0;

        let new_timer = self.timer_signal();

        if old_timer && !new_timer {
            self.tima = self.tima.wrapping_add(1);
        }
    }

    pub fn read_tima(&self) -> u8 {
        self.tima
    }
    pub fn write_tima(&mut self, value: u8) {
        self.tima = value
    }

    pub fn read_tma(&self) -> u8 {
        self.tma
    }
    pub fn write_tma(&mut self, value: u8) {
        self.tma = value
    }

    pub fn read_tac(&self) -> u8 {
        self.tac
    }
    pub fn write_tac(&mut self, value: u8) {
        let old_timer = self.timer_signal();
        self.tac = value & 0b111;
        let new_timer = self.timer_signal();

        if old_timer && !new_timer {
            self.tima = self.tima.wrapping_add(1);
        }
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
}
