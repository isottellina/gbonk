mod render;

pub struct PPU {
	vram: [u8; 0x2000],
	oam: [u8; 0xA0],
	pub buffer: Box<[u32; (160 * 144)]>,
	mode: PPUMode,
	frame_done: bool,
	clock: u32,
	ly: u8,
	wx: u8,
	wy: u8,

	// LCDC
	enable: bool,
	window_map: bool,
	window_enable: bool,
	tile_data: bool,
	bg_map: bool,
	obj_size: bool,
	obj_enable: bool,
	bg_window_enable: bool,

	// STAT
	lyc: u8,
	coincidence_irq: bool,
	mode2_irq: bool,
	mode1_irq: bool,
	mode0_irq: bool,

	scy: u8,
	scx: u8,
	bgp: [DmgColor; 4],
	obp0: [DmgColor; 4],
	obp1: [DmgColor; 4],

	// Interruptions
	vblank_irq: bool,
	stat_irq: bool,
}

impl PPU {
	pub fn is_frame_done(&self) -> bool { self.frame_done }
	pub fn ack_frame_done(&mut self) { self.frame_done = false; }

	pub fn has_vblank_irq(&self) -> bool { self.vblank_irq }
	pub fn ack_vblank_irq(&mut self) { self.vblank_irq = false; }
	pub fn set_vblank_irq(&mut self, value: bool) { self.vblank_irq = value; }

	pub fn has_stat_irq(&self) -> bool { self.stat_irq }
	pub fn ack_stat_irq(&mut self) { self.stat_irq = false; }
	pub fn set_stat_irq(&mut self, value: bool) { self.stat_irq = value; }

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
						
						if self.mode1_irq {
							self.stat_irq = true;
						}
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

						if self.mode2_irq {
							self.stat_irq = true;
						}
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

					if self.mode0_irq {
						self.stat_irq = true;
					}
				}
			}
		}

	}

	fn increment_line(&mut self) {
		self.ly = (self.ly + 1) % 155;

		if self.coincidence_irq && self.ly == self.lyc {
			self.stat_irq = true;
		}
	}

	pub fn write_vram_u8(&mut self, addr: u16, value: u8) {
		assert!(addr >= 0x8000);

		self.vram[(addr as usize) - 0x8000] = value;
	}

	pub fn write_oam_u8(&mut self, addr: u16, value: u8) {
		assert!(addr >= 0xFE00);

		self.oam[(addr as usize) - 0xFE00] = value;
	}

	pub fn read_oam_u8(&mut self, addr: u16) -> u8 {
		self.oam[(addr as usize) - 0xFE00]
	}

	pub fn read_io_register(&self, addr: u16) -> u8 {
		match addr {
			0xff40 => (self.enable as u8) << 7 |
				(self.window_map as u8) << 6 |
				(self.window_enable as u8) << 5 |
				(self.tile_data as u8) << 4 |
				(self.bg_map as u8) << 3 |
				(self.obj_size as u8) << 2 |
				(self.obj_enable as u8) << 1 |
				(self.bg_window_enable as u8),
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
			0xff41 => {
				self.mode0_irq = (value & 0x08) != 0;
				self.mode1_irq = (value & 0x10) != 0;
				self.mode2_irq = (value & 0x20) != 0;
				self.coincidence_irq = (value & 0x40) != 0;
			},
			0xff42 => self.scy = value,
			0xff43 => self.scx = value,
			0xff45 => self.lyc = value,
			0xff47 => self.bgp.set_register(value),
			0xff48 => self.obp0.set_register(value),
			0xff49 => self.obp1.set_register(value),
			0xff4a => self.wy = value,
			0xff4b => self.wx = value,
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
			oam: [0; 0xA0],
			buffer: Box::new([0; (160 * 144)]),
			frame_done: false,
			clock: 0,
			mode: PPUMode::ReadingOAM,
			ly: 0,
			wx: 0,
			wy: 0,

			// LCDC
			enable: false,
			window_map: false,
			window_enable: false,
			tile_data: false,
			bg_map: false,
			obj_size: false,
			obj_enable: false,
			bg_window_enable: false,

			// STAT
			lyc: 0,
			coincidence_irq: false,
			mode2_irq: false,
			mode1_irq: false,
			mode0_irq: false,

			scy: 0,
			scx: 0,
			bgp: [DmgColor::White; 4],
			obp0: [DmgColor::White; 4],
			obp1: [DmgColor::White; 4],
			vblank_irq: false,
			stat_irq: false
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
