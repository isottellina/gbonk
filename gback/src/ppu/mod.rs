mod render;

pub struct PPU {
	vram: [u8; 0x2000],
	pub buffer: Box<[u32; (160 * 144)]>,
	mode: PPUMode,
	frame_done: bool,
	clock: u32,
	ly: u8,

	// LCDC
	enable: bool,
	window_map: bool,
	window_enable: bool,
	tile_data: bool,
	bg_map: bool,
	obj_size: bool,
	obj_enable: bool,
	bg_window_enable: bool,

	scy: u8,
	scx: u8,
	bgp: [DmgColor; 4],
}

impl PPU {
	pub fn is_frame_done(&self) -> bool { self.frame_done }
	pub fn ack_frame_done(&mut self) { self.frame_done = false; }

	pub fn spend(&mut self, cycles: u32) {
		if !self.enable {
			return;
		}

		self.clock += cycles;
		match self.mode {
			PPUMode::HBlank => {
				if self.clock >= 456 {
					self.render_line(self.ly);
					self.increment_line();
					self.clock -= 456;

					if self.ly == 144 {
						self.mode = PPUMode::VBlank;
					} else {
						self.mode = PPUMode::ReadingOAM;
					}
				}
			},
			PPUMode::VBlank => {
				if self.clock >= 456 {
					self.increment_line();
					self.clock -= 456;

					if self.ly == 0 {
						self.frame_done = true;
						self.mode = PPUMode::ReadingOAM;
					}
				}
			},
			PPUMode::ReadingOAM => {
				if self.clock >= 80 {
					self.mode = PPUMode::Drawing;
				}
			},
			PPUMode::Drawing => {
				if self.clock >= 310 {
					self.mode = PPUMode::HBlank;
				}
			}
		}

	}

	fn increment_line(&mut self) {
		self.ly = (self.ly + 1) % 155;
	}

	pub fn write_vram_u8(&mut self, addr: u16, value: u8) {
		assert!(addr >= 0x8000);

		self.vram[(addr as usize) - 0x8000] = value;
	}

	pub fn read_io_register(&self, addr: u16) -> u8 {
		match addr {
			0xff42 => self.scy,
			0xff43 => self.scx,
			0xff44 => self.ly,
			_ => unimplemented!(
				"PPU I/O register unimplemented : {:04x}",
				addr
			),
		}
	}

	pub fn write_io_register(&mut self, addr: u16, value: u8) {
		match addr {
			0xff40 => {
				self.enable = (value & 0x80) != 0;
				self.window_map = (value & 0x40) != 0;
				self.window_enable = (value & 0x20) != 0;
				self.tile_data = (value & 0x10) != 0;
				self.bg_map = (value & 0x08) != 0;
				self.obj_size = (value & 0x04) != 0;
				self.obj_enable = (value & 0x02) != 0;
				self.bg_window_enable = (value & 0x01) != 0;
			},
			0xff42 => self.scy = value,
			0xff43 => self.scx = value,
			0xff47 => self.bgp.set_register(value),
			_ => unimplemented!(
				"PPU I/O register unimplemented : {:04x}, {:02x}",
				addr,
				value
			),
		}
	}
}

impl Default for PPU {
	fn default() -> PPU {
		PPU {
			vram: [0; 0x2000],
			buffer: Box::new([0; (160 * 144)]),
			frame_done: false,
			clock: 0,
			mode: PPUMode::ReadingOAM,
			ly: 0,

			// LCDC
			enable: false,
			window_map: false,
			window_enable: false,
			tile_data: false,
			bg_map: false,
			obj_size: false,
			obj_enable: false,
			bg_window_enable: false,

			scy: 0,
			scx: 0,
			bgp: [DmgColor::White; 4],
		}
	}
}

enum PPUMode {
	HBlank = 0,
	VBlank = 1,
	ReadingOAM = 2,
	Drawing = 3
}

trait Palette {
	fn get_register(&self) -> u8;
	fn set_register(&mut self, value: u8);
}

#[derive(Copy, Clone, Debug)]
enum DmgColor {
	White = 0,
	LightGrey = 1,
	DarkGrey = 2,
	Black = 3,
}

impl DmgColor {
    fn as_real(self) -> [u8; 4] {
        match self {
            DmgColor::White => [0, 224, 248, 208],
            DmgColor::LightGrey => [0, 136, 192, 112],
            DmgColor::DarkGrey => [0, 52, 104, 86],
            DmgColor::Black => [0, 8, 24, 32],
        }
    }
}

impl From<u8> for DmgColor {
	fn from(value: u8) -> DmgColor {
		match value {
			0 => DmgColor::White,
			1 => DmgColor::LightGrey,
			2 => DmgColor::DarkGrey,
			3 => DmgColor::Black,
			_ => unreachable!(),
		}
	}
}

impl Palette for [DmgColor; 4] {
	fn get_register(&self) -> u8 {
		((self[3] as u8) << 6) | ((self[2] as u8) << 4) | ((self[1] as u8) << 2) | (self[0] as u8)
	}

	fn set_register(&mut self, value: u8) {
		self[0] = DmgColor::from(value & 0x03);
		self[1] = DmgColor::from((value & 0x0c) >> 2);
		self[2] = DmgColor::from((value & 0x30) >> 4);
		self[3] = DmgColor::from((value & 0xc0) >> 6);
	}
}
