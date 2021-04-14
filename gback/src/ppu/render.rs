use crate::ppu::PPU;

impl PPU {
	pub fn render_line(&mut self, line: u8) {
		for x in 0..=159 {
			let real_x = self.scx.wrapping_add(x) as usize;
			let real_y = self.scy.wrapping_add(line) as usize;

			let tile = if self.bg_map {
				self.vram[0x1C00 + (real_x >> 3) + ((real_y & 0xf8) << 2)]
			} else {
				self.vram[0x1800 + (real_x >> 3) + ((real_y & 0xf8) << 2)]
			};

			let tile_color = self.get_pixel_from_tile(tile, real_x as u8, real_y as u8);
			let real_color = self.bgp[tile_color as usize].as_real();

			self.buffer[(line as usize * 160) + x as usize] = u32::from_be_bytes(real_color);
		};
	}

	fn get_pixel_from_tile(&self, tile: u8, x: u8, y: u8) -> u8 {
		let tile_x = x & 0x7;
		let tile_y = y & 0x7;

		let offset = if self.tile_data {
			((tile as usize) << 4) + ((tile_y as usize) << 1)
		} else {
			let tile = tile as i8;
			(0x1000 + ((tile as isize) << 4) + ((y as isize) << 1)) as usize
		};

		let (data1, data2) = (
			self.vram[offset],
			self.vram[offset + 1]
		);

		(data1 >> (7 - tile_x) & 1) | (((data2 >> (7 - tile_x)) & 1) << 1)
	}
}