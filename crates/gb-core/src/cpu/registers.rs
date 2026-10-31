const CARRY_FLAG: u8 = 1 << 4;
const HALF_CARRY_FLAG: u8 = 1 << 5;
const SUBTRACT_FLAG: u8 = 1 << 6;
const ZERO_FLAG: u8 = 1 << 7;

pub struct Registers {
    pub a: u8,
    pub f: u8,

    pub b: u8,
    pub c: u8,

    pub d: u8,
    pub e: u8,

    pub h: u8,
    pub l: u8,

    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | self.f as u16
    }
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    pub fn set_af(&mut self, value: u16) {
        let upper = (value >> 8) as u8;
        let lower = (value as u8) & 0xF0;

        self.a = upper;
        self.f = lower;
    }

    pub fn set_bc(&mut self, value: u16) {
        let upper = (value >> 8) as u8;
        let lower = value as u8;

        self.b = upper;
        self.c = lower;
    }

    pub fn set_de(&mut self, value: u16) {
        let upper = (value >> 8) as u8;
        let lower = value as u8;

        self.d = upper;
        self.e = lower;
    }

    pub fn set_hl(&mut self, value: u16) {
        let upper = (value >> 8) as u8;
        let lower = value as u8;

        self.h = upper;
        self.l = lower;
    }

    pub fn zero_flag(&self) -> bool {
        self.f & ZERO_FLAG != 0
    }
    pub fn subtract_flag(&self) -> bool {
        self.f & SUBTRACT_FLAG != 0
    }
    pub fn half_carry_flag(&self) -> bool {
        self.f & HALF_CARRY_FLAG != 0
    }
    pub fn carry_flag(&self) -> bool {
        self.f & CARRY_FLAG != 0
    }

    fn set_flag(&mut self, mask: u8, value: bool) {
        if value {
            self.f |= mask;
        } else {
            self.f &= !mask;
        }
    }

    pub fn set_zero_flag(&mut self, value: bool) {
        self.set_flag(ZERO_FLAG, value);
    }
    pub fn set_subtract_flag(&mut self, value: bool) {
        self.set_flag(SUBTRACT_FLAG, value);
    }
    pub fn set_half_carry_flag(&mut self, value: bool) {
        self.set_flag(HALF_CARRY_FLAG, value);
    }
    pub fn set_carry_flag(&mut self, value: bool) {
        self.set_flag(CARRY_FLAG, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn getter_concat() {
        let mut register = Registers::new();

        register.a = 0x66;
        register.f = 0x70;

        register.b = 0x66;
        register.c = 0x99;

        register.d = 0x00;
        register.e = 0x77;

        register.h = 0x99;
        register.l = 0x33;

        assert_eq!(register.af(), 0x6670);
        assert_eq!(register.bc(), 0x6699);
        assert_eq!(register.de(), 0x0077);
        assert_eq!(register.hl(), 0x9933);
    }

    #[test]
    fn sets_bc_register_pair() {
        let mut registers = Registers::new();

        registers.set_bc(0x1234);

        assert_eq!(registers.b, 0x12);
        assert_eq!(registers.c, 0x34);
        assert_eq!(registers.bc(), 0x1234);
    }

    #[test]
    fn set_af_keeps_lower_flag_bits_zero() {
        let mut registers = Registers::new();

        registers.set_af(0x12FF);

        assert_eq!(registers.a, 0x12);
        assert_eq!(registers.f, 0xF0);
        assert_eq!(registers.af(), 0x12F0);
    }

    #[test]
    fn reads_flags() {
        let mut registers = Registers::new();

        registers.f = 0b1011_0000;

        assert!(registers.zero_flag());
        assert!(!registers.subtract_flag());
        assert!(registers.half_carry_flag());
        assert!(registers.carry_flag());
    }

    #[test]
    fn setting_zero_flag_preserves_other_flags() {
        let mut registers = Registers::new();

        registers.f = 0b0011_0000; // H + C

        registers.set_zero_flag(true);

        assert_eq!(registers.f, 0b1011_0000);

        registers.set_zero_flag(false);

        assert_eq!(registers.f, 0b0011_0000);
    }
}
