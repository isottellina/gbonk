use gback::{Platform, GBEvent};
use sdl2::event::Event;
use sdl2::surface::Surface;

pub struct SDLPlatform {
	window: sdl2::video::Window,
	event_pump: sdl2::EventPump,
}

impl SDLPlatform {
	pub fn new() -> SDLPlatform {
		let sdl_context = sdl2::init().unwrap();
		let video_subsystem = sdl_context.video().unwrap();

		let window = video_subsystem.window("GBonk", 640, 576)
			.position_centered()
			.build()
			.unwrap();

		let event_pump = sdl_context.event_pump().unwrap();

		SDLPlatform {
			window,
			event_pump
		}
	}
}

impl Platform for SDLPlatform {
	fn present_buffer(&mut self, buffer: &mut [u8]) {
		if let Ok(mut window_surface) = self.window.surface(&self.event_pump) {
			let buffer_surface = Surface::from_data(
				buffer,
				160, 144,
				160 * 4,
				sdl2::pixels::PixelFormatEnum::RGB888,
			).unwrap();

			buffer_surface.blit_scaled(
				None,
				&mut window_surface,
				None
			).unwrap();

			window_surface.update_window().unwrap();
		}
	}

	fn process_events(&mut self) -> Option<GBEvent> {
		match self.event_pump.poll_event() {
			Some(Event::Quit {..}) => Some(GBEvent::Quit),
			None => None,
			_ => None
		}
	}
}