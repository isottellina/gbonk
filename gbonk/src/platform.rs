use gback::Platform;
use minifb::{Window, WindowOptions};

pub struct MiniFBPlatform {
	window: Window,
}

impl MiniFBPlatform {
	pub fn new() -> MiniFBPlatform {
		let mut window_options: WindowOptions = Default::default();
		window_options.scale = minifb::Scale::X4;
		window_options.scale_mode = minifb::ScaleMode::AspectRatioStretch;

		let window = Window::new(
			"GBonk",
			160,
			144,
			window_options
		).expect("Unable to initialize window.");

		MiniFBPlatform {
			window
		}
	}
}

impl Platform for MiniFBPlatform {
	fn present_buffer(&mut self, buffer: &[u32]) {
		self.window.update_with_buffer(
			buffer,	160, 144
		).unwrap();
	}
}