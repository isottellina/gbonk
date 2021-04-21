pub mod gameboy;
pub mod cartridge;
pub mod cpu;
pub mod bus;
pub mod ppu;
pub mod apu;

pub use gameboy::Gameboy;

pub trait Platform {
	fn present_buffer(&mut self, buffer: &[u32]);
	fn process_events(&mut self);
}