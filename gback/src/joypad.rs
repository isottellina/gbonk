#[derive(Default)]
pub struct Joypad {
	mode: bool,
	right: bool,
	left: bool,
	up: bool,
	down: bool,
	a: bool,
	b: bool,
	start: bool,
	select: bool,
}

impl Joypad {
	pub fn read(&self) -> u8 {
		if self.mode {
			0xE0 |
			((!self.down) as u8) << 3 |
			((!self.up) as u8) << 2 |
			((!self.left) as u8) << 1 |
			(!self.right) as u8
		} else {
			0xD0 |
			((!self.start) as u8) << 3 |
			((!self.select) as u8) << 2 |
			((!self.a) as u8) << 1 |
			(!self.b) as u8		
		}
	}

	pub fn write(&mut self, value: u8) {
		self.mode = (value & 0x20) != 0;
	}
}