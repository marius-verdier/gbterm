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
        let cycles = self.cpu.step(&mut self.bus);

        for _ in 0..cycles {
            self.bus.tick();
        }

        cycles
    }

    pub fn get_bus(&self) -> &Bus {
        &self.bus
    }

    pub fn serial_output(&self) -> &str {
        self.bus.serial_output()
    }
}
