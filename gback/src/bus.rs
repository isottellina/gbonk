use std::io::Read;
use crate::cartridge::Cartridge;
use crate::ppu::PPU;
use crate::apu::APU;
use crate::joypad::Joypad;

pub struct Bus {
	bios: [u8; 0x100],
	cart: Cartridge,
	ppu: PPU,
	apu: APU,
	joypad: Joypad,
	wram: [u8; 0x2000],
	hram: [u8; 0x80],

	cycles_to_spend: u32,
	bios_enable: bool,

	// Interruptions
	enable_vblank_irq: bool,
	enable_stat_irq: bool,

	// DMA
	dma_ongoing: bool,
	dma_src: u16,
	dma_dst: u16,
}

impl Bus {
	pub fn is_frame_done(&self) -> bool { self.ppu.is_frame_done() }
	pub fn ack_frame_done(&mut self) { self.ppu.ack_frame_done(); }
	pub fn frame_buffer(&self) -> [u8; 160 * 144 * 4] { *self.ppu.buffer }

	pub fn has_irq(&self) -> Option<u16> {
		if self.ppu.has_vblank_irq() && self.enable_vblank_irq {
			Some(0x40)
		} else if self.ppu.has_stat_irq() && self.enable_stat_irq {
			Some(0x48)
		} else {
			None
		}
	}

	pub fn ack_irq(&mut self) {
		if self.ppu.has_vblank_irq() && self.enable_vblank_irq {
			self.ppu.ack_vblank_irq()
		} else if self.ppu.has_stat_irq() && self.enable_stat_irq {
			self.ppu.ack_stat_irq()
		}
	}

	pub fn load_bios(&mut self, mut file: std::fs::File) {
		file.read_exact(&mut self.bios).unwrap();
	}

	pub fn load_rom(&mut self, file: std::fs::File) {
		self.cart.load_file(file);
	}

	pub fn delay(&mut self, cycles: u32) {
		self.cycles_to_spend += cycles;
	}

	pub fn read_u8(&mut self, addr: u16) -> u8 {
		match addr {
			0..=0xFF if self.bios_enable => self.bios[addr as usize],
			0..=0x7FFF => self.cart.read_rom_u8(addr),
			0xC000..=0xDFFF => self.wram[(addr & 0x1FFF) as usize],
			0xE000..=0xFDFF => self.wram[(addr & 0x1FFF) as usize],
			0xFE00..=0xFE9F => self.ppu.read_oam_u8(addr),
			0xFEA0..=0xFEFF => 0xFF,
			0xFF00 => self.joypad.read(),
			0xFF40..=0xFF4B => self.ppu.read_io_register(addr),
			0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
			0xFFFF => {
				(self.enable_vblank_irq as u8) |
				((self.enable_stat_irq as u8) << 1)
			}
			_ => unimplemented!("This address has not been implemented yet. {:04x}", addr)
		}
	}

	pub fn write_u8(&mut self, addr: u16, value: u8) {
		match addr {
			0x0000..=0x7FFF => self.cart.write_rom_u8(addr, value),
			0x8000..=0x9FFF => self.ppu.write_vram_u8(addr, value),
			0xA000..=0xBFFF => self.cart.write_ram_u8(addr, value),
			0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize] = value,
			0xFE00..=0xFE9F => self.ppu.write_oam_u8(addr, value),
			0xFEA0..=0xFEFF => { },
			0xFF00 => self.joypad.write(value),
			0xFF01 => {},
			0xFF02 => {},
			// TODO: Timer registers
			0xFF04..=0xFF07 => { },
			0xFF0F => {
				self.ppu.set_vblank_irq((value & 0x01) != 0);
				self.ppu.set_stat_irq((value & 0x02) != 0);
			}
			0xFF10..=0xFF26 => self.apu.write_io_register(addr, value),
			0xFF40..=0xFF45 => self.ppu.write_io_register(addr, value),
			0xFF46 => {
				// DMA
				self.dma_ongoing = true;
				self.dma_src = (value as u16) << 8;
				self.dma_dst = 0xFE00;
			}
			0xFF47..=0xFF4B => self.ppu.write_io_register(addr, value),
			0xFF50 => self.bios_enable = false,
			0xFF7F => {},
			0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = value,
			0xFFFF => {
				self.enable_vblank_irq = (value & 0x01) != 0;
				self.enable_stat_irq = (value & 0x02) != 0;
			}
			_ => unimplemented!("Write to unmapped adress ({:04x}, {:02x})", addr, value),
		}
	}

	pub fn spend(&mut self) {
		let t_state = self.cycles_to_spend << 2;

		self.ppu.spend(t_state);

		// DMA
		if self.dma_ongoing {
			for _ in 0..t_state {
				let value = self.read_u8(self.dma_src);
				self.ppu.write_oam_u8(self.dma_dst, value);

				self.dma_src += 1;
				self.dma_dst += 1;

				if self.dma_dst == 0xfea0 {
					self.dma_ongoing = false;
					break;
				}
			}
		}

		self.cycles_to_spend = 0;
	}
}

impl Default for Bus {
	fn default() -> Self {
		Self {
			cart: Default::default(),
			ppu: Default::default(),
			apu: Default::default(),
			joypad: Default::default(),
			bios: [0; 0x100],
			hram: [0; 0x80],
			wram: [0; 0x2000],

			cycles_to_spend: 0,
			bios_enable: true,

			enable_vblank_irq: false,
			enable_stat_irq: false,

			// DMA
			dma_ongoing: false,
			dma_src: 0,
			dma_dst: 0xFE00,
		}
	}
}

impl std::fmt::Debug for Bus {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Bus")
	}
}