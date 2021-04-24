use crate::bus::Bus;
use crate::cpu::CPU;
use crate::{Platform, GBEvent};

#[derive(Default, Debug)]
pub struct Gameboy {
    cpu: CPU,
    bus: Bus,
    pub running: bool,
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

    pub fn run_frame(&mut self, platform: &mut dyn Platform) {
        while !self.bus.is_frame_done() {
            self.cpu.step(&mut self.bus);
            self.bus.spend();
        }

        platform.present_buffer(&mut self.bus.frame_buffer());
        self.bus.ack_frame_done();

        while let Some(event) = platform.process_events() {
            match event {
                GBEvent::Quit => self.running = false,
            }
        }
    }
}