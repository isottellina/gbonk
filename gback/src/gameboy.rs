use crate::bus::Bus;
use crate::cpu::CPU;

#[derive(Default, Debug)]
pub struct Gameboy {
    cpu: CPU,
    bus: Bus,
}

impl Gameboy {
    pub fn new() -> Gameboy {
        Default::default()
    }

    pub fn load_bios(&mut self, file: std::fs::File) {
        self.bus.load_bios(file);
    }

    pub fn load_rom(&mut self, file: std::fs::File) {
        self.bus.load_rom(file);
    }

    pub fn run_frame(&mut self) {
        loop {
            self.cpu.run_instruction(&mut self.bus);
            println!("{:?}", self.cpu);
        }
    }
}