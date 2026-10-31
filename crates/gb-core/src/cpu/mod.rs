mod registers;

use registers::Registers;

use crate::bus::Bus;

pub struct Cpu {
    registers: Registers,
    ime: bool,
    halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            ime: false,
            halted: false,
        }
    }

    pub fn new_dmg() -> Self {
        let mut register = Registers::new();

        register.set_af(0x01B0);
        register.set_bc(0x0013);
        register.set_de(0x00D8);
        register.set_hl(0x014D);
        register.sp = 0xFFFE;
        register.pc = 0x0100;

        Self {
            registers: register,
            ime: false,
            halted: false,
        }
    }

    fn fetch_word(&mut self, bus: &Bus) -> u16 {
        let low = self.fetch_byte(bus);
        let high = self.fetch_byte(bus);

        (high as u16) << 8 | (low as u16)
    }

    pub fn step(&mut self, bus: &mut Bus) -> u8 {

        let opcode = self.fetch_byte(bus);

        match opcode {
            0x00 => 4, // NOP
            0x01 => {
                // LD BC,d16
                let value = self.fetch_word(bus);
                self.registers.set_bc(value);
                12
            }

            0x11 => {
                // LD DE,d16
                let value = self.fetch_word(bus);
                self.registers.set_de(value);
                12
            }

            0x21 => {
                // LD HL,d16
                let value = self.fetch_word(bus);
                self.registers.set_hl(value);
                12
            }

            0x31 => {
                // LD SP,d16
                let value = self.fetch_word(bus);
                self.registers.sp = value;
                12
            }

            0x06 => {
                // LD B,d8
                let value = self.fetch_byte(bus);
                self.registers.b = value;
                8
            }

            0x0E => {
                // LD C,d8
                let value = self.fetch_byte(bus);
                self.registers.c = value;
                8
            }

            0x16 => {
                // LD D,d8
                let value = self.fetch_byte(bus);
                self.registers.d = value;
                8
            }

            0x1E => {
                // LD E,d8
                let value = self.fetch_byte(bus);
                self.registers.e = value;
                8
            }

            0x26 => {
                // LD H,d8
                let value = self.fetch_byte(bus);
                self.registers.h = value;
                8
            }

            0x2E => {
                // LD L,d8
                let value = self.fetch_byte(bus);
                self.registers.l = value;
                8
            }

            0x3E => {
                // LD A,d8
                let value = self.fetch_byte(bus);
                self.registers.a = value;
                8
            }

            0x40..=0x7F => {
                // LD R, R
                if opcode == 0x76 {
                    todo!("HALT not implemented")
                }

                let destination = (opcode >> 3) & 0b111;
                let source = opcode & 0b111;

                if source == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    self.write_register_from_code(destination, value);

                    return 8;
                }

                if destination == 6 {
                    let value = self.read_register_from_code(source);
                    let address = self.registers.hl();

                    bus.write8(address, value);

                    return 8;
                }

                let value = self.read_register_from_code(source);
                self.write_register_from_code(destination, value);

                4
            }

            _ => todo!("Opcode {opcode:02X} not implemented"),
        }
    }

    fn fetch_byte(&mut self, bus: &Bus) -> u8 {
        let read_opcode = bus.read8(self.registers.pc);

        self.registers.pc = self.registers.pc.wrapping_add(1);

        read_opcode
    }

    fn read_register_from_code(&self, code: u8) -> u8 {
        match code {
            0 => self.registers.b,
            1 => self.registers.c,
            2 => self.registers.d,
            3 => self.registers.e,
            4 => self.registers.h,
            5 => self.registers.l,
            6 => unreachable!("register code 6 represents HL"),
            7 => self.registers.a,
            _ => unreachable!("register code does not match"),
        }
    }
    fn write_register_from_code(&mut self, code: u8, value: u8) {
        match code {
            0 => self.registers.b = value,
            1 => self.registers.c = value,
            2 => self.registers.d = value,
            3 => self.registers.e = value,
            4 => self.registers.h = value,
            5 => self.registers.l = value,
            6 => unreachable!("register code 6 represents HL"),
            7 => self.registers.a = value,
            _ => unreachable!("register code does not match"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bus::test_utils::make_test_bus, cartridge::Cartridge};

    #[test]
    fn test_opcode_fetch_and_pc_increment() {
        let mut cpu = Cpu::new();
        let bus = make_test_bus();

        cpu.registers.pc = 0x4000;
        assert_eq!(cpu.fetch_byte(&bus), 0x01);
        assert_eq!(cpu.registers.pc, 0x4001);
    }

    #[test]
    fn executes_nop() {
        let mut cpu = Cpu::new_dmg();
        let mut bus = make_test_bus();

        let cycles = cpu.step(&mut bus);

        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.pc, 0x0101);
    }

    #[test]
    fn executes_all_ld_rr_d16() {
        let mut rom = crate::cartridge::test_utils::make_test_rom();

        rom[0x0100] = 0x01;
        rom[0x0101] = 0x34;
        rom[0x0102] = 0x12;

        rom[0x0103] = 0x11;
        rom[0x0104] = 0x78;
        rom[0x0105] = 0x56;

        rom[0x0106] = 0x21;
        rom[0x0107] = 0xBC;
        rom[0x0108] = 0x9A;

        rom[0x0109] = 0x31;
        rom[0x010A] = 0xFE;
        rom[0x010B] = 0xFF;

        let cartridge = Cartridge::new(rom).unwrap();
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::new_dmg();

        assert_eq!(cpu.step(&mut bus), 12);
        assert_eq!(cpu.registers.bc(), 0x1234);

        assert_eq!(cpu.step(&mut bus), 12);
        assert_eq!(cpu.registers.de(), 0x5678);

        assert_eq!(cpu.step(&mut bus), 12);
        assert_eq!(cpu.registers.hl(), 0x9ABC);

        assert_eq!(cpu.step(&mut bus), 12);
        assert_eq!(cpu.registers.sp, 0xFFFE);

        assert_eq!(cpu.registers.pc, 0x010C);
    }

    #[test]
    fn executes_all_ld_r_d8() {
        let mut rom = crate::cartridge::test_utils::make_test_rom();

        rom[0x0100] = 0x06;
        rom[0x0101] = 0x11;

        rom[0x0102] = 0x0E;
        rom[0x0103] = 0x22;

        rom[0x0104] = 0x16;
        rom[0x0105] = 0x33;

        rom[0x0106] = 0x1E;
        rom[0x0107] = 0x44;

        rom[0x0108] = 0x26;
        rom[0x0109] = 0x55;

        rom[0x010A] = 0x2E;
        rom[0x010B] = 0x66;

        rom[0x010C] = 0x3E;
        rom[0x010D] = 0x77;

        let cartridge = Cartridge::new(rom).unwrap();
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::new_dmg();

        assert_eq!(cpu.step(&mut bus), 8);
        assert_eq!(cpu.registers.b, 0x11);

        assert_eq!(cpu.step(&mut bus), 8);
        assert_eq!(cpu.registers.c, 0x22);

        assert_eq!(cpu.step(&mut bus), 8);
        assert_eq!(cpu.registers.d, 0x33);

        assert_eq!(cpu.step(&mut bus), 8);
        assert_eq!(cpu.registers.e, 0x44);

        assert_eq!(cpu.step(&mut bus), 8);
        assert_eq!(cpu.registers.h, 0x55);

        assert_eq!(cpu.step(&mut bus), 8);
        assert_eq!(cpu.registers.l, 0x66);

        assert_eq!(cpu.step(&mut bus), 8);
        assert_eq!(cpu.registers.a, 0x77);
    }
}
