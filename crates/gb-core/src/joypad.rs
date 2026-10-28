const LOW_NIBBLE_MASK: u8 = 0x0F;

pub enum Button {
    A,
    B,
    Select,
    Start,
    Right,
    Left,
    Up,
    Down,
}

pub struct Joypad {
    buttons: u8,
    directions: u8,

    select_buttons: bool,
    select_directions: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            buttons: 0u8,
            directions: 0u8,
            select_buttons: false,
            select_directions: false,
        }
    }

    pub fn write(&mut self, value: u8) {
        let directions_selected = value & (1 << 4) == 0;
        let buttons_selected = value & (1 << 5) == 0;

        self.select_directions = directions_selected;
        self.select_buttons = buttons_selected;
    }

    // Attention : active-low selection
    pub fn read(&self) -> u8 {
        let mut base: u8 = 0xFF;
        let mut low_nibble = LOW_NIBBLE_MASK;

        if self.select_directions {
            base &= !(1 << 4);
            low_nibble &= !self.directions;
        }

        if self.select_buttons {
            base &= !(1 << 5);
            low_nibble &= !self.buttons;
        }

        base &= 0xF0;
        base |= low_nibble;

        base
    }

    pub fn press(&mut self, button: Button) {
        match button {
            Button::A => {
                self.buttons |= 1;
            }

            Button::B => {
                self.buttons |= 1 << 1;
            }

            Button::Select => {
                self.buttons |= 1 << 2;
            }

            Button::Start => {
                self.buttons |= 1 << 3;
            }

            Button::Right => {
                self.directions |= 1;
            }

            Button::Left => {
                self.directions |= 1 << 1;
            }

            Button::Up => {
                self.directions |= 1 << 2;
            }

            Button::Down => {
                self.directions |= 1 << 3;
            }
        }
    }

    pub fn release(&mut self, button: Button) {
        match button {
            Button::A => {
                self.buttons &= !(1);
            }

            Button::B => {
                self.buttons &= !(1 << 1);
            }

            Button::Select => {
                self.buttons &= !(1 << 2);
            }

            Button::Start => {
                self.buttons &= !(1 << 3);
            }

            Button::Right => {
                self.directions &= !(1);
            }

            Button::Left => {
                self.directions &= !(1 << 1);
            }

            Button::Up => {
                self.directions &= !(1 << 2);
            }

            Button::Down => {
                self.directions &= !(1 << 3);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_when_buttons_are_selected() {
        let mut joypad = Joypad::new();

        joypad.press(Button::A);

        // bit 5 = 0 → buttons selected
        joypad.write(0b1101_1111);

        assert_eq!(joypad.read() & 0x0F, 0b1110);
    }

    #[test]
    fn reads_left_when_directions_are_selected() {
        let mut joypad = Joypad::new();

        joypad.press(Button::Left);

        // bit 4 = 0 → directions selected
        joypad.write(0b1110_1111);

        assert_eq!(joypad.read() & 0x0F, 0b1101);
    }

    #[test]
    fn released_button_is_not_reported_as_pressed() {
        let mut joypad = Joypad::new();

        joypad.write(0b1101_1111);

        joypad.press(Button::A);
        assert_eq!(joypad.read() & 0x01, 0);

        joypad.release(Button::A);
        assert_eq!(joypad.read() & 0x01, 1);
    }

    #[test]
    fn combines_buttons_and_directions_when_both_are_selected() {
        let mut joypad = Joypad::new();

        joypad.press(Button::A);
        joypad.press(Button::Left);

        // bits 4 et 5 à 0
        joypad.write(0b1100_1111);

        assert_eq!(joypad.read() & 0x0F, 0b1100);
    }
}
