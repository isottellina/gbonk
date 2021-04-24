pub mod gameboy;
pub mod cartridge;
pub mod cpu;
pub mod bus;
pub mod joypad;
pub mod ppu;
pub mod apu;

pub use gameboy::Gameboy;

pub enum GBEvent {
	Quit
}

pub trait Platform {
	fn present_buffer(&mut self, buffer: &mut [u8]);
	fn process_events(&mut self) -> Option<GBEvent>;
}