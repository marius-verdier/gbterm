use crate::{bus::Bus, cartridge::Cartridge, cpu::Cpu};

pub struct GameBoy {
    cpu: Cpu,
    bus: Bus,
}

impl GameBoy {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cpu: Cpu::new_dmg(),
            bus: Bus::new(cartridge),
        }
    }

    pub fn step(&mut self) -> u8 {
        self.cpu.step(&mut self.bus)
    }

    pub fn get_bus(&self) -> &Bus {
        &self.bus
    }

    pub fn serial_output(&self) -> &str {
        self.bus.serial_output()
    }
}
