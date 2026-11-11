mod registers;

use registers::Registers;

use crate::bus::{Bus, Interrupt};

pub struct Cpu {
    registers: Registers,
    ime: bool,
    ime_enable_delay: u8,
    halted: bool,
    halt_bug: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            ime: false,
            ime_enable_delay: 0,
            halted: false,
            halt_bug: false,
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
            ime_enable_delay: 0,
            halted: false,
            halt_bug: false,
        }
    }

    fn fetch_word(&mut self, bus: &Bus) -> u16 {
        let low = self.fetch_byte(bus);
        let high = self.fetch_byte(bus);

        (high as u16) << 8 | (low as u16)
    }

    fn push_word(&mut self, bus: &mut Bus, word: u16) {
        let high = (word >> 8) as u8;
        let low = word as u8;

        self.registers.sp = self.registers.sp.wrapping_sub(1);
        bus.write8(self.registers.sp, high);

        self.registers.sp = self.registers.sp.wrapping_sub(1);
        bus.write8(self.registers.sp, low);
    }

    fn pop_word(&mut self, bus: &Bus) -> u16 {
        let low = bus.read8(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let high = bus.read8(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);

        (high as u16) << 8 | low as u16
    }

    fn service_interrupt(&mut self, bus: &mut Bus, interrupt: Interrupt) -> u8 {
        self.ime = false;
        self.halted = false;
        self.ime_enable_delay = 0;

        bus.clear_interrupt(interrupt);

        self.push_word(bus, self.registers.pc);

        self.registers.pc = interrupt.vector();

        20
    }

    fn jr_if(&mut self, bus: &Bus, condition: bool) -> u8 {
        let offset = self.fetch_byte(bus) as i8;

        if condition {
            self.registers.pc = self.registers.pc.wrapping_add_signed(offset as i16);

            12
        } else {
            8
        }
    }

    fn inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag((value & 0x0F) == 0x0F);

        result
    }

    fn dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(true);
        self.registers.set_half_carry_flag((value & 0x0F) == 0);

        result
    }

    fn add_a(&mut self, value: u8) {
        let a = self.registers.a;
        let result = a.wrapping_add(value);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers
            .set_half_carry_flag((a & 0x0F) + (value & 0x0F) > 0x0F);
        self.registers
            .set_carry_flag((a as u16) + (value as u16) > 0xFF);

        self.registers.a = result;
    }

    fn adc_a(&mut self, value: u8) {
        let a = self.registers.a;
        let carry = u8::from(self.registers.carry_flag());

        let result = a.wrapping_add(value).wrapping_add(carry);

        self.registers.set_zero_flag(result == 0);

        self.registers.set_subtract_flag(false);

        self.registers
            .set_half_carry_flag((a & 0x0F) + (value & 0x0F) + carry > 0x0F);

        self.registers
            .set_carry_flag((a as u16) + (value as u16) + (carry as u16) > 0xFF);

        self.registers.a = result;
    }

    fn sub_a(&mut self, value: u8) {
        let a = self.registers.a;
        let result = a.wrapping_sub(value);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(true);

        self.registers
            .set_half_carry_flag((a & 0x0F) < (value & 0x0F));
        self.registers.set_carry_flag(a < value);

        self.registers.a = result;
    }

    fn sbc_a(&mut self, value: u8) {
        let a = self.registers.a;
        let carry = u8::from(self.registers.carry_flag());

        let result = a.wrapping_sub(value).wrapping_sub(carry);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(true);

        self.registers
            .set_half_carry_flag((a & 0x0F) < ((value & 0x0F) + carry));
        self.registers
            .set_carry_flag((a as u16) < (value as u16 + carry as u16));

        self.registers.a = result;
    }

    fn and_a(&mut self, value: u8) {
        self.registers.a &= value;

        self.registers.set_zero_flag(self.registers.a == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(true);
        self.registers.set_carry_flag(false);
    }

    fn xor_a(&mut self, value: u8) {
        self.registers.a ^= value;

        self.registers.set_zero_flag(self.registers.a == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(false);
    }

    fn or_a(&mut self, value: u8) {
        self.registers.a |= value;

        self.registers.set_zero_flag(self.registers.a == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(false);
    }

    fn cp_a(&mut self, value: u8) {
        let a = self.registers.a;
        let result = a.wrapping_sub(value);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(true);

        self.registers
            .set_half_carry_flag((a & 0x0F) < (value & 0x0F));
        self.registers.set_carry_flag(a < value);
    }

    fn call_if(&mut self, bus: &mut Bus, condition: bool) -> u8 {
        let address = self.fetch_word(bus);
        if condition {
            self.push_word(bus, self.registers.pc);
            self.registers.pc = address;
            24
        } else {
            12
        }
    }

    fn ret_if(&mut self, bus: &Bus, condition: bool) -> u8 {
        if condition {
            self.registers.pc = self.pop_word(bus);
            20
        } else {
            8
        }
    }

    fn rst(&mut self, bus: &mut Bus, address: u16) -> u8 {
        self.push_word(bus, self.registers.pc);
        self.registers.pc = address;

        16
    }

    fn rlca(&mut self) {
        let a = self.registers.a;
        let carry = (a & 0x80) != 0;

        self.registers.a = a.rotate_left(1);

        self.registers.set_zero_flag(false);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry);
    }

    fn rrca(&mut self) {
        let a = self.registers.a;
        let carry = (a & 0x01) != 0;

        self.registers.a = a.rotate_right(1);

        self.registers.set_zero_flag(false);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry);
    }

    fn rla(&mut self) {
        let a = self.registers.a;
        let old_carry = u8::from(self.registers.carry_flag());
        let carry = (a & 0x80) != 0;

        self.registers.a = (a << 1) | old_carry;

        self.registers.set_zero_flag(false);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry);
    }

    fn rra(&mut self) {
        let a = self.registers.a;
        let old_carry = u8::from(self.registers.carry_flag());
        let carry = (a & 0x01) != 0;

        self.registers.a = (a >> 1) | (old_carry << 7);

        self.registers.set_zero_flag(false);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry);
    }

    fn update_ime_delay(&mut self) {
        if self.ime_enable_delay == 0 {
            return;
        }

        self.ime_enable_delay -= 1;

        if self.ime_enable_delay == 0 {
            self.ime = true;
        }
    }

    fn rlc(&mut self, value: u8) -> u8 {
        let carry = (value & 0x80) != 0;
        let result = value.rotate_left(1);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry);

        result
    }

    fn rrc(&mut self, value: u8) -> u8 {
        let carry = (value & 0x01) != 0;

        let result = value.rotate_right(1);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry);

        result
    }

    fn rl(&mut self, value: u8) -> u8 {
        let old_carry = u8::from(self.registers.carry_flag());
        let carry = (value & 0x80) != 0;

        let result = (value << 1) | old_carry;

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry);

        result
    }

    fn rr(&mut self, value: u8) -> u8 {
        let old_carry = u8::from(self.registers.carry_flag());
        let carry = (value & 0x01) != 0;

        let result = (value >> 1) | (old_carry << 7);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry);

        result
    }

    fn sla(&mut self, value: u8) -> u8 {
        let carry = (value & 0x80) != 0;
        let result = value << 1;

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry);

        result
    }

    fn sra(&mut self, value: u8) -> u8 {
        let carry = (value & 0x01) != 0;
        let result = (value >> 1) | (value & 0x80);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry);

        result
    }

    fn swap(&mut self, value: u8) -> u8 {
        let result = value.rotate_left(4);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(false);

        result
    }

    fn srl(&mut self, value: u8) -> u8 {
        let carry = (value & 0x01) != 0;
        let result = value >> 1;

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(carry);

        result
    }

    fn bit(&mut self, value: u8, bit: u8) {
        let mask = 1 << bit;

        self.registers.set_zero_flag(value & mask == 0);
        self.registers.set_subtract_flag(false);
        self.registers.set_half_carry_flag(false);
    }

    fn res(value: u8, bit: u8) -> u8 {
        value & !(1 << bit)
    }

    fn set(value: u8, bit: u8) -> u8 {
        value | (1 << bit)
    }

    fn exec_cb(&mut self, bus: &mut Bus, opcode: u8) -> u8 {
        match opcode {
            0x00..=0x07 => {
                // RLC r
                let register_code = opcode & 0b111;
                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);
                    let result = self.rlc(value);

                    bus.write8(address, result);
                    16
                } else {
                    let value = self.read_register_from_code(register_code);
                    let result = self.rlc(value);

                    self.write_register_from_code(register_code, result);
                    8
                }
            }

            0x08..=0x0F => {
                // RRC r
                let register_code = opcode & 0b111;
                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);
                    let result = self.rrc(value);

                    bus.write8(address, result);
                    16
                } else {
                    let value = self.read_register_from_code(register_code);
                    let result = self.rrc(value);

                    self.write_register_from_code(register_code, result);
                    8
                }
            }

            0x10..=0x17 => {
                // RL r
                let register_code = opcode & 0b111;
                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);
                    let result = self.rl(value);

                    bus.write8(address, result);
                    16
                } else {
                    let value = self.read_register_from_code(register_code);
                    let result = self.rl(value);

                    self.write_register_from_code(register_code, result);
                    8
                }
            }

            0x18..=0x1F => {
                // RR r
                let register_code = opcode & 0b111;
                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);
                    let result = self.rr(value);

                    bus.write8(address, result);
                    16
                } else {
                    let value = self.read_register_from_code(register_code);
                    let result = self.rr(value);

                    self.write_register_from_code(register_code, result);
                    8
                }
            }

            0x20..=0x27 => {
                // SLA r
                let register_code = opcode & 0b111;
                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);
                    let result = self.sla(value);

                    bus.write8(address, result);
                    16
                } else {
                    let value = self.read_register_from_code(register_code);
                    let result = self.sla(value);

                    self.write_register_from_code(register_code, result);
                    8
                }
            }

            0x28..=0x2F => {
                // SRA r
                let register_code = opcode & 0b111;
                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);
                    let result = self.sra(value);

                    bus.write8(address, result);
                    16
                } else {
                    let value = self.read_register_from_code(register_code);
                    let result = self.sra(value);

                    self.write_register_from_code(register_code, result);
                    8
                }
            }

            0x30..=0x37 => {
                // SWAP
                let register_code = opcode & 0b111;
                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);
                    let result = self.swap(value);

                    bus.write8(address, result);
                    16
                } else {
                    let value = self.read_register_from_code(register_code);
                    let result = self.swap(value);

                    self.write_register_from_code(register_code, result);
                    8
                }
            }

            0x38..=0x3F => {
                // SRL r
                let register_code = opcode & 0b111;
                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);
                    let result = self.srl(value);

                    bus.write8(address, result);
                    16
                } else {
                    let value = self.read_register_from_code(register_code);
                    let result = self.srl(value);

                    self.write_register_from_code(register_code, result);
                    8
                }
            }

            0x40..=0x7F => {
                // BIT b,r
                let bit = (opcode >> 3) & 0b111;
                let register_code = opcode & 0b111;

                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    self.bit(value, bit);
                    12
                } else {
                    let value = self.read_register_from_code(register_code);
                    self.bit(value, bit);
                    8
                }
            }

            0x80..=0xBF => {
                // RES b,r
                let bit = (opcode >> 3) & 0b111;
                let register_code = opcode & 0b111;

                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    let result = Self::res(value, bit);

                    bus.write8(address, result);
                    16
                } else {
                    let value = self.read_register_from_code(register_code);
                    let result = Self::res(value, bit);

                    self.write_register_from_code(register_code, result);
                    8
                }
            }

            0xC0..=0xFF => {
                // SET b,r
                let bit = (opcode >> 3) & 0b111;
                let register_code = opcode & 0b111;

                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    let result = Self::set(value, bit);

                    bus.write8(address, result);
                    16
                } else {
                    let value = self.read_register_from_code(register_code);
                    let result = Self::set(value, bit);

                    self.write_register_from_code(register_code, result);
                    8
                }
            }
        }
    }

    pub fn step(&mut self, bus: &mut Bus) -> u8 {
        let pending = bus.next_pending_interrupt();

        if let Some(interrupt) = pending {
            if self.ime {
                return self.service_interrupt(bus, interrupt);
            }

            if self.halted {
                self.halted = false;
            }
        }

        if self.halted {
            return 4;
        }

        let opcode = self.fetch_byte(bus);

        let cycles = match opcode {
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

            0x76 => {
                // HALT + HALT BUG
                if !self.ime && bus.pending_interrupts() != 0 {
                    self.halt_bug = true;
                } else {
                    self.halted = true;
                }

                4
            }

            0x40..=0x7F => {
                // LD R, R
                let destination = (opcode >> 3) & 0b111;
                let source = opcode & 0b111;

                if source == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    self.write_register_from_code(destination, value);

                    8
                } else if destination == 6 {
                    let value = self.read_register_from_code(source);
                    let address = self.registers.hl();

                    bus.write8(address, value);

                    8
                } else {
                    let value = self.read_register_from_code(source);
                    self.write_register_from_code(destination, value);

                    4
                }
            }

            0xF3 => {
                // DI
                self.ime = false;
                self.ime_enable_delay = 0;

                4
            }

            0xFB => {
                // EI
                self.ime_enable_delay = 2;

                4
            }

            0xD9 => {
                // RETI
                self.registers.pc = self.pop_word(bus);
                self.ime = true;
                self.ime_enable_delay = 0;

                16
            }

            0xC3 => {
                // JP a16 -> absolute
                let address = self.fetch_word(bus);
                self.registers.pc = address;

                16
            }

            0x18 => {
                // JR r8 -> relative
                let offset = self.fetch_byte(bus) as i8;

                self.registers.pc = self.registers.pc.wrapping_add_signed(offset as i16);

                12
            }

            0x20 => self.jr_if(bus, !self.registers.zero_flag()),
            0x28 => self.jr_if(bus, self.registers.zero_flag()),
            0x30 => self.jr_if(bus, !self.registers.carry_flag()),
            0x38 => self.jr_if(bus, self.registers.carry_flag()),

            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                // INC r
                let register_code = (opcode >> 3) & 0b111;

                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    let result = self.inc(value);
                    bus.write8(address, result);

                    12
                } else {
                    let value = self.read_register_from_code(register_code);
                    let result = self.inc(value);

                    self.write_register_from_code(register_code, result);

                    4
                }
            }

            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                // DEC r
                let register_code = (opcode >> 3) & 0b111;

                if register_code == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    let result = self.dec(value);
                    bus.write8(address, result);

                    12
                } else {
                    let value = self.read_register_from_code(register_code);
                    let result = self.dec(value);

                    self.write_register_from_code(register_code, result);

                    4
                }
            }

            0x80..=0x87 => {
                // ADD A, r
                let source = opcode & 0b111;

                if source == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    self.add_a(value);

                    8
                } else {
                    let value = self.read_register_from_code(source);
                    self.add_a(value);

                    4
                }
            }

            0x88..=0x8F => {
                // ADC A, r
                let source = opcode & 0b111;

                if source == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    self.adc_a(value);

                    8
                } else {
                    let value = self.read_register_from_code(source);
                    self.adc_a(value);

                    4
                }
            }

            0x90..=0x97 => {
                // SUB A, r
                let source = opcode & 0b111;

                if source == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    self.sub_a(value);

                    8
                } else {
                    let value = self.read_register_from_code(source);
                    self.sub_a(value);

                    4
                }
            }

            0x98..=0x9F => {
                // SBC A, r
                let source = opcode & 0b111;

                if source == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    self.sbc_a(value);

                    8
                } else {
                    let value = self.read_register_from_code(source);
                    self.sbc_a(value);

                    4
                }
            }

            0xA0..=0xA7 => {
                // AND A, r
                let source = opcode & 0b111;

                if source == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    self.and_a(value);

                    8
                } else {
                    let value = self.read_register_from_code(source);
                    self.and_a(value);

                    4
                }
            }

            0xA8..=0xAF => {
                // XOR A, r
                let source = opcode & 0b111;

                if source == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    self.xor_a(value);

                    8
                } else {
                    let value = self.read_register_from_code(source);
                    self.xor_a(value);

                    4
                }
            }

            0xB0..=0xB7 => {
                // OR A, r
                let source = opcode & 0b111;

                if source == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    self.or_a(value);

                    8
                } else {
                    let value = self.read_register_from_code(source);
                    self.or_a(value);

                    4
                }
            }

            0xB8..=0xBF => {
                // CP A, r
                let source = opcode & 0b111;

                if source == 6 {
                    let address = self.registers.hl();
                    let value = bus.read8(address);

                    self.cp_a(value);

                    8
                } else {
                    let value = self.read_register_from_code(source);
                    self.cp_a(value);

                    4
                }
            }

            0xC6 => {
                // ADD A,d8
                let value = self.fetch_byte(bus);
                self.add_a(value);

                8
            }

            0xCE => {
                // ADC A,d8
                let value = self.fetch_byte(bus);
                self.adc_a(value);

                8
            }

            0xD6 => {
                // SUB A,d8
                let value = self.fetch_byte(bus);
                self.sub_a(value);

                8
            }

            0xDE => {
                // SBC A,d8
                let value = self.fetch_byte(bus);
                self.sbc_a(value);

                8
            }

            0xE6 => {
                // AND d8
                let value = self.fetch_byte(bus);
                self.and_a(value);

                8
            }

            0xEE => {
                // XOR d8
                let value = self.fetch_byte(bus);
                self.xor_a(value);

                8
            }

            0xF6 => {
                // OR d8
                let value = self.fetch_byte(bus);
                self.or_a(value);

                8
            }

            0xFE => {
                // CP d8
                let value = self.fetch_byte(bus);
                self.cp_a(value);

                8
            }

            0xC5 => {
                // PUSH BC
                self.push_word(bus, self.registers.bc());
                16
            }

            0xD5 => {
                // PUSH DE
                self.push_word(bus, self.registers.de());
                16
            }

            0xE5 => {
                // PUSH HL
                self.push_word(bus, self.registers.hl());
                16
            }

            0xF5 => {
                // PUSH AF
                self.push_word(bus, self.registers.af());
                16
            }

            0xC1 => {
                // POP BC
                let value = self.pop_word(bus);
                self.registers.set_bc(value);
                12
            }

            0xD1 => {
                // POP DE
                let value = self.pop_word(bus);
                self.registers.set_de(value);
                12
            }

            0xE1 => {
                // POP HL
                let value = self.pop_word(bus);
                self.registers.set_hl(value);
                12
            }

            0xF1 => {
                // POP AF
                let value = self.pop_word(bus);
                self.registers.set_af(value);
                12
            }

            0xCD => {
                // CALL a16
                let address = self.fetch_word(bus);
                self.push_word(bus, self.registers.pc);

                self.registers.pc = address;

                24
            }

            0xC9 => {
                self.registers.pc = self.pop_word(bus);
                16
            }

            // CALL cc,a16
            0xC4 => self.call_if(bus, !self.registers.zero_flag()),
            0xCC => self.call_if(bus, self.registers.zero_flag()),
            0xD4 => self.call_if(bus, !self.registers.carry_flag()),
            0xDC => self.call_if(bus, self.registers.carry_flag()),

            // RET cc
            0xC0 => self.ret_if(bus, !self.registers.zero_flag()),
            0xC8 => self.ret_if(bus, self.registers.zero_flag()),
            0xD0 => self.ret_if(bus, !self.registers.carry_flag()),
            0xD8 => self.ret_if(bus, self.registers.carry_flag()),

            // RST
            0xC7 => self.rst(bus, 0x00),
            0xCF => self.rst(bus, 0x08),
            0xD7 => self.rst(bus, 0x10),
            0xDF => self.rst(bus, 0x18),
            0xE7 => self.rst(bus, 0x20),
            0xEF => self.rst(bus, 0x28),
            0xF7 => self.rst(bus, 0x30),
            0xFF => self.rst(bus, 0x38),

            // RLCA
            0x07 => {
                self.rlca();
                4
            }
            // RRCA
            0x0F => {
                self.rrca();
                4
            }
            // RLA
            0x17 => {
                self.rla();
                4
            }
            // RRA
            0x1F => {
                self.rra();
                4
            }

            // CB
            0xCB => {
                let cb_opcode = self.fetch_byte(bus);
                self.exec_cb(bus, cb_opcode)
            }

            _ => todo!("Opcode {opcode:02X} not implemented"),
        };

        self.update_ime_delay();

        cycles
    }

    fn fetch_byte(&mut self, bus: &Bus) -> u8 {
        let read_opcode = bus.read8(self.registers.pc);

        if self.halt_bug {
            self.halt_bug = false;
        } else {
            self.registers.pc = self.registers.pc.wrapping_add(1);
        }

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
    use crate::{
        bus::test_utils::make_test_bus,
        cartridge::{Cartridge, test_utils::make_test_rom},
    };

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

    #[test]
    fn pushes_word_on_stack() {
        let mut cpu = Cpu::new_dmg();
        let mut bus = make_test_bus();

        assert_eq!(cpu.registers.sp, 0xFFFE);

        cpu.push_word(&mut bus, 0x1234);

        assert_eq!(cpu.registers.sp, 0xFFFC);
        assert_eq!(bus.read8(0xFFFC), 0x34);
        assert_eq!(bus.read8(0xFFFD), 0x12);
    }

    #[test]
    fn interrupt_service() {
        let mut cpu = Cpu::new_dmg();
        let mut bus = make_test_bus();

        cpu.registers.pc = 0x1234;
        cpu.ime = true;

        bus.request_interrupt(Interrupt::Timer);
        bus.enable_interrupt(Interrupt::Timer);

        let cycles = cpu.service_interrupt(&mut bus, Interrupt::Timer);

        assert_eq!(cpu.registers.pc, 0x0050);
        assert_eq!(cpu.registers.sp, 0xFFFC);
        assert_eq!(bus.read8(0xFFFC), 0x34);
        assert_eq!(bus.read8(0xFFFD), 0x12);

        assert!(!cpu.ime);
        assert_eq!(cycles, 20);
    }

    #[test]
    fn step_services_pending_interrupt_when_ime_is_enabled() {
        let mut cpu = Cpu::new_dmg();
        let mut bus = make_test_bus();

        cpu.registers.pc = 0x1234;
        cpu.ime = true;

        bus.request_interrupt(Interrupt::Timer);
        bus.enable_interrupt(Interrupt::Timer);

        let cycles = cpu.step(&mut bus);

        assert_eq!(cycles, 20);
        assert_eq!(cpu.registers.pc, 0x0050);
        assert_eq!(cpu.registers.sp, 0xFFFC);

        assert_eq!(bus.read8(0xFFFC), 0x34);
        assert_eq!(bus.read8(0xFFFD), 0x12);

        assert!(!cpu.ime);
    }

    #[test]
    fn executes_jp_a16() {
        let mut rom = make_test_rom();

        rom[0x0100] = 0xC3;
        rom[0x0101] = 0x34;
        rom[0x0102] = 0x12;

        let cartridge = Cartridge::new(rom).unwrap();
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::new_dmg();

        let cycles = cpu.step(&mut bus);

        assert_eq!(cycles, 16);
        assert_eq!(cpu.registers.pc, 0x1234)
    }

    #[test]
    fn exec_jr_offset() {
        let mut rom = crate::cartridge::test_utils::make_test_rom();

        rom[0x0100] = 0x18;
        rom[0x0101] = 0x05;
        rom[0x0107] = 0x18;
        rom[0x0108] = 0xFD;

        let cartridge = Cartridge::new(rom).unwrap();
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::new_dmg();

        assert_eq!(cpu.step(&mut bus), 12);
        assert_eq!(cpu.registers.pc, 0x0107);

        assert_eq!(cpu.step(&mut bus), 12);
        assert_eq!(cpu.registers.pc, 0x0106);
    }

    #[test]
    fn executes_inc_register_and_memory() {
        let mut rom = crate::cartridge::test_utils::make_test_rom();

        rom[0x0100] = 0x04; // INC B
        rom[0x0101] = 0x34; // INC [HL]

        let cartridge = Cartridge::new(rom).unwrap();
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::new_dmg();

        cpu.registers.b = 0x0F;
        cpu.registers.set_hl(0xC123);

        bus.write8(0xC123, 0xFF);

        cpu.registers.set_carry_flag(true);

        assert_eq!(cpu.step(&mut bus), 4);
        assert_eq!(cpu.registers.b, 0x10);
        assert!(cpu.registers.half_carry_flag());
        assert!(cpu.registers.carry_flag());

        assert_eq!(cpu.step(&mut bus), 12);
        assert_eq!(bus.read8(0xC123), 0x00);
        assert!(cpu.registers.zero_flag());
        assert!(cpu.registers.half_carry_flag());
        assert!(cpu.registers.carry_flag());
    }

    #[test]
    fn executes_dec_register_and_memory() {
        let mut rom = crate::cartridge::test_utils::make_test_rom();

        rom[0x0100] = 0x05; // DEC B
        rom[0x0101] = 0x35; // DEC [HL]

        let cartridge = Cartridge::new(rom).unwrap();
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::new_dmg();

        cpu.registers.b = 0x10;
        cpu.registers.set_hl(0xC123);

        bus.write8(0xC123, 0x01);

        cpu.registers.set_carry_flag(true);

        assert_eq!(cpu.step(&mut bus), 4);
        assert_eq!(cpu.registers.b, 0x0F);
        assert!(cpu.registers.half_carry_flag());
        assert!(cpu.registers.carry_flag());
        assert!(!cpu.registers.zero_flag());
        assert!(cpu.registers.subtract_flag());

        assert_eq!(cpu.step(&mut bus), 12);
        assert_eq!(bus.read8(0xC123), 0x00);
        assert!(cpu.registers.zero_flag());
        assert!(!cpu.registers.half_carry_flag());
        assert!(cpu.registers.carry_flag());
        assert!(cpu.registers.subtract_flag());
    }

    #[test]
    fn executes_add_adc_sub_sbc_sequence() {
        let mut rom = make_test_rom();

        rom[0x0100] = 0x80; // ADD A,B
        rom[0x0101] = 0x89; // ADC A,C
        rom[0x0102] = 0x92; // SUB D
        rom[0x0103] = 0x9B; // SBC A,E

        let cartridge = Cartridge::new(rom).unwrap();
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::new_dmg();

        cpu.registers.a = 0xFF;
        cpu.registers.b = 0x01;
        cpu.registers.c = 0x0F;
        cpu.registers.d = 0x20;
        cpu.registers.e = 0x0F;

        // 0xFF + 0x01 = 0x00, carry = 1
        assert_eq!(cpu.step(&mut bus), 4);
        assert_eq!(cpu.registers.a, 0x00);
        assert!(cpu.registers.zero_flag());
        assert!(!cpu.registers.subtract_flag());
        assert!(cpu.registers.half_carry_flag());
        assert!(cpu.registers.carry_flag());

        // 0x00 + 0x0F + carry(1) = 0x10
        assert_eq!(cpu.step(&mut bus), 4);
        assert_eq!(cpu.registers.a, 0x10);
        assert!(!cpu.registers.zero_flag());
        assert!(!cpu.registers.subtract_flag());
        assert!(cpu.registers.half_carry_flag());
        assert!(!cpu.registers.carry_flag());

        // 0x10 - 0x20 = 0xF0, borrow = 1
        assert_eq!(cpu.step(&mut bus), 4);
        assert_eq!(cpu.registers.a, 0xF0);
        assert!(!cpu.registers.zero_flag());
        assert!(cpu.registers.subtract_flag());
        assert!(!cpu.registers.half_carry_flag());
        assert!(cpu.registers.carry_flag());

        // 0xF0 - 0x0F - carry(1) = 0xE0
        assert_eq!(cpu.step(&mut bus), 4);
        assert_eq!(cpu.registers.a, 0xE0);
        assert!(!cpu.registers.zero_flag());
        assert!(cpu.registers.subtract_flag());
        assert!(cpu.registers.half_carry_flag());
        assert!(!cpu.registers.carry_flag());

        assert_eq!(cpu.registers.pc, 0x0104);
    }

    #[test]
    fn executes_and_xor_or_cp_sequence() {
        let mut rom = make_test_rom();

        rom[0x0100] = 0xA0; // AND B
        rom[0x0101] = 0xA9; // XOR C
        rom[0x0102] = 0xB2; // OR D
        rom[0x0103] = 0xBB; // CP E

        let cartridge = Cartridge::new(rom).unwrap();
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::new_dmg();

        cpu.registers.a = 0xAC;
        cpu.registers.b = 0xF0;
        cpu.registers.c = 0x0F;
        cpu.registers.d = 0x33;
        cpu.registers.e = 0xBF;

        // AC & F0 = A0
        assert_eq!(cpu.step(&mut bus), 4);
        assert_eq!(cpu.registers.a, 0xA0);
        assert!(!cpu.registers.zero_flag());
        assert!(!cpu.registers.subtract_flag());
        assert!(cpu.registers.half_carry_flag());
        assert!(!cpu.registers.carry_flag());

        // A0 ^ 0F = AF
        assert_eq!(cpu.step(&mut bus), 4);
        assert_eq!(cpu.registers.a, 0xAF);
        assert!(!cpu.registers.zero_flag());
        assert!(!cpu.registers.subtract_flag());
        assert!(!cpu.registers.half_carry_flag());
        assert!(!cpu.registers.carry_flag());

        // AF | 33 = BF
        assert_eq!(cpu.step(&mut bus), 4);
        assert_eq!(cpu.registers.a, 0xBF);
        assert!(!cpu.registers.zero_flag());
        assert!(!cpu.registers.subtract_flag());
        assert!(!cpu.registers.half_carry_flag());
        assert!(!cpu.registers.carry_flag());

        // BF - BF = 0 pour les flags,
        // mais CP ne modifie PAS A
        assert_eq!(cpu.step(&mut bus), 4);
        assert_eq!(cpu.registers.a, 0xBF);
        assert!(cpu.registers.zero_flag());
        assert!(cpu.registers.subtract_flag());
        assert!(!cpu.registers.half_carry_flag());
        assert!(!cpu.registers.carry_flag());

        assert_eq!(cpu.registers.pc, 0x0104);
    }

    #[test]
    fn executes_call_and_ret() {
        let mut rom = make_test_rom();

        // 0x0100: CALL 0x0200
        rom[0x0100] = 0xCD;
        rom[0x0101] = 0x00;
        rom[0x0102] = 0x02;

        // Adresse de retour
        rom[0x0103] = 0x00; // NOP

        // 0x0200: RET
        rom[0x0200] = 0xC9;

        let cartridge = Cartridge::new(rom).unwrap();
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::new_dmg();

        assert_eq!(cpu.registers.pc, 0x0100);
        assert_eq!(cpu.registers.sp, 0xFFFE);

        // CALL 0x0200
        assert_eq!(cpu.step(&mut bus), 24);

        assert_eq!(cpu.registers.pc, 0x0200);
        assert_eq!(cpu.registers.sp, 0xFFFC);

        // CALL doit avoir sauvegardé l'adresse de retour 0x0103
        assert_eq!(bus.read8(0xFFFC), 0x03);
        assert_eq!(bus.read8(0xFFFD), 0x01);

        // RET
        assert_eq!(cpu.step(&mut bus), 16);

        assert_eq!(cpu.registers.pc, 0x0103);
        assert_eq!(cpu.registers.sp, 0xFFFE);

        // Vérifie qu'on est réellement revenu au bon endroit :
        // 0x0103 contient NOP
        assert_eq!(cpu.step(&mut bus), 4);
        assert_eq!(cpu.registers.pc, 0x0104);
    }

    #[test]
    fn executes_cb_rlc_register_and_memory() {
        let mut rom = make_test_rom();

        // CB 00 → RLC B
        rom[0x0100] = 0xCB;
        rom[0x0101] = 0x00;

        // CB 06 → RLC [HL]
        rom[0x0102] = 0xCB;
        rom[0x0103] = 0x06;

        let cartridge = Cartridge::new(rom).unwrap();
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::new_dmg();

        cpu.registers.b = 0x81;
        cpu.registers.set_hl(0xC123);

        bus.write8(0xC123, 0x00);

        // RLC B
        // 1000_0001 → 0000_0011
        assert_eq!(cpu.step(&mut bus), 8);

        assert_eq!(cpu.registers.b, 0x03);
        assert!(cpu.registers.carry_flag());
        assert!(!cpu.registers.zero_flag());
        assert!(!cpu.registers.subtract_flag());
        assert!(!cpu.registers.half_carry_flag());

        assert_eq!(cpu.registers.pc, 0x0102);

        // RLC [HL]
        // 0000_0000 → 0000_0000
        // Z should be 1
        assert_eq!(cpu.step(&mut bus), 16);

        assert_eq!(bus.read8(0xC123), 0x00);
        assert!(!cpu.registers.carry_flag());
        assert!(cpu.registers.zero_flag());
        assert!(!cpu.registers.subtract_flag());
        assert!(!cpu.registers.half_carry_flag());

        assert_eq!(cpu.registers.pc, 0x0104);
    }

    #[test]
    fn executes_cb_shift_and_swap_operations() {
        let mut rom = make_test_rom();

        rom[0x0100] = 0xCB;
        rom[0x0101] = 0x20; // SLA B

        rom[0x0102] = 0xCB;
        rom[0x0103] = 0x29; // SRA C

        rom[0x0104] = 0xCB;
        rom[0x0105] = 0x32; // SWAP D

        rom[0x0106] = 0xCB;
        rom[0x0107] = 0x3B; // SRL E

        let cartridge = Cartridge::new(rom).unwrap();
        let mut bus = Bus::new(cartridge);
        let mut cpu = Cpu::new_dmg();

        cpu.registers.b = 0x81;
        cpu.registers.c = 0x83;
        cpu.registers.d = 0xA3;
        cpu.registers.e = 0x83;

        // SLA B
        // 1000_0001 -> 0000_0010
        assert_eq!(cpu.step(&mut bus), 8);
        assert_eq!(cpu.registers.b, 0x02);
        assert!(cpu.registers.carry_flag());
        assert!(!cpu.registers.zero_flag());
        assert!(!cpu.registers.subtract_flag());
        assert!(!cpu.registers.half_carry_flag());

        // SRA C
        // 1000_0011 -> 1100_0001
        // The most significant bit is preserved
        assert_eq!(cpu.step(&mut bus), 8);
        assert_eq!(cpu.registers.c, 0xC1);
        assert!(cpu.registers.carry_flag());
        assert!(!cpu.registers.zero_flag());
        assert!(!cpu.registers.subtract_flag());
        assert!(!cpu.registers.half_carry_flag());

        // SWAP D
        // 1010_0011 -> 0011_1010
        assert_eq!(cpu.step(&mut bus), 8);
        assert_eq!(cpu.registers.d, 0x3A);
        assert!(!cpu.registers.carry_flag());
        assert!(!cpu.registers.zero_flag());
        assert!(!cpu.registers.subtract_flag());
        assert!(!cpu.registers.half_carry_flag());

        // SRL E
        // 1000_0011 -> 0100_0001
        // Unlike SRA, the most significant bit becomes 0
        assert_eq!(cpu.step(&mut bus), 8);
        assert_eq!(cpu.registers.e, 0x41);
        assert!(cpu.registers.carry_flag());
        assert!(!cpu.registers.zero_flag());
        assert!(!cpu.registers.subtract_flag());
        assert!(!cpu.registers.half_carry_flag());

        // Four CB-prefixed instructions, 2 bytes each
        assert_eq!(cpu.registers.pc, 0x0108);
    }
}
