use crate::bus::Bus;

#[derive(Default)]
pub struct CPU {
	pc: u16,
	sp: u16,

	a: u8,
	b: u8,
	c: u8,
	d: u8,
	e: u8,
	h: u8,
	l: u8,

	zero: bool,
	sub: bool,
	halfcarry: bool,
	carry: bool,
}

impl CPU {
	fn f(&self) -> u8 {
		((self.zero as u8) << 3) |
		((self.sub as u8) << 2) |
		((self.halfcarry as u8) << 1) |
		(self.carry as u8)
	}


	// 16-bit register manipulation
	fn af(&self) -> u16 {
		((self.a as u16) << 8) | self.f() as u16
	}

	fn bc(&self) -> u16 {
		((self.b as u16) << 8) | self.c as u16
	}

	fn de(&self) -> u16 {
		((self.d as u16) << 8) | self.e as u16
	}    
	
	fn hl(&self) -> u16 {
		((self.h as u16) << 8) | self.l as u16
	}

	fn set_pc(&mut self, bus: &mut Bus, value: u16) {
		bus.delay(1);
		self.pc = value;
	}

	fn set_af(&mut self, value: u16) {
		self.a = (value >> 8) as u8;
		self.zero = (value & 0x08) != 0;
		self.sub = (value & 0x04) != 0;
		self.halfcarry = (value & 0x02) != 0;
		self.carry = (value & 0x01) != 0;
	}

	fn set_bc(&mut self, value: u16) {
		self.b = (value >> 8) as u8;
		self.c = value as u8;
	}

	fn set_de(&mut self, value: u16) {
		self.d = (value >> 8) as u8;
		self.e = value as u8;
	}

	fn set_hl(&mut self, value: u16) {
		self.h = (value >> 8) as u8;
		self.l = value as u8;
	}

	// Memory-related functions
	fn read_u8(&mut self, bus: &mut Bus, addr: u16) -> u8 {
		bus.delay(1);
		bus.read_u8(addr)
	}

	fn read_u16(&mut self, bus: &mut Bus, addr: u16) -> u16 {
		self.read_u8(bus, addr) as u16 |
		(self.read_u8(bus, addr + 1) as u16) << 8
	}

	fn write_u8(&self, bus: &mut Bus, addr: u16, value: u8) {
		bus.delay(1);
		bus.write_u8(addr, value);
	}

	fn next_u8(&mut self, bus: &mut Bus) -> u8 {
		let value = self.read_u8(bus, self.pc);
		self.pc += 1;

		value
	}

	fn next_u16(&mut self, bus: &mut Bus) -> u16 {
		let value = self.read_u16(bus, self.pc);
		self.pc += 2;

		value
	}

	fn pop(&mut self, bus: &mut Bus) -> u16 {
		let low_byte = self.read_u8(bus, self.sp) as u16;
		self.sp += 1;
		let high_byte = self.read_u8(bus, self.sp) as u16;
		self.sp += 1;

		(high_byte << 8) | low_byte
	}

	fn push(&mut self, bus: &mut Bus, value: u16) {
		self.sp -= 1;
		self.write_u8(bus, self.sp, (value >> 8) as u8);
		self.sp -= 1;
		self.write_u8(bus, self.sp, value as u8);
	}

	fn write_hl(&self, bus: &mut Bus, value: u8) {
		let hl = self.hl();
		
		self.write_u8(bus, hl, value);
	}

	// Control functions
	fn jr(&mut self, bus: &mut Bus) {
		let value = self.next_u8(bus) as i8 as i16;

		let pc = self.pc as i16;
		self.set_pc(bus, pc.wrapping_add(value) as u16);
	}

	fn jr_cond(&mut self, bus: &mut Bus, cond: bool) {
		let value = self.next_u8(bus) as i8 as i16;

		if cond {
			let pc = self.pc as i16;
			self.set_pc(bus, pc.wrapping_add(value) as u16);
		}
	}

	fn call(&mut self, bus: &mut Bus) {
		let addr = self.next_u16(bus);
		self.push(bus, self.pc);
		self.set_pc(bus, addr);
	}

	fn ret(&mut self, bus: &mut Bus) {
		let pc = self.pop(bus);
		println!("{:04x}", pc);
		self.set_pc(bus, pc);
	}

	// Arithmetic functions
	fn cp_u8(&mut self, value: u8) {
		self.zero = self.a == value;
		self.sub = true;
		self.halfcarry = (self.a & 0xf) < (value & 0xf);
		self.carry = self.a < value;
	}

	fn inc_u8(&mut self, value: u8) -> u8 {
		self.zero = value == 0xff;
		self.sub = false;
		self.halfcarry = (value & 0xf) == 0xf;

		value.wrapping_add(1)
	}

	fn dec_u8(&mut self, value: u8) -> u8 {
		self.zero = value == 1;
		self.sub = true;
		self.halfcarry = (value & 0xf) == 0;

		value.wrapping_sub(1)
	}

	fn inc_u16(&self, bus: &mut Bus, value: u16) -> u16 {
		bus.delay(1);

		value.wrapping_add(1)
	}

	// Logic functions
	fn xor_u8(&mut self, value: u8) {
		self.a ^= value;

		self.zero = self.a == 0;
		self.sub = false;
		self.halfcarry = false;
		self.carry = false;
	}

	fn rl_u8(&mut self, value: u8) -> u8 {
		let carry = self.carry as u8;
		self.carry = (value & 0x80) != 0;

		let new_value = (value << 1) | carry;

		self.zero = new_value == 0;
		self.sub = false;
		self.halfcarry = false;

		new_value
	}

	fn bit_u8(&mut self, bit: u8, value: u8) {
		self.zero = (value & (1 << bit)) == 0;
		self.sub = false;
		self.halfcarry = true;
	}

	pub fn run_instruction(&mut self, bus: &mut Bus) {
		let instr = self.next_u8(bus);

		match instr {
			0x04 => { self.b = self.inc_u8(self.b); }
			0x05 => { self.b = self.dec_u8(self.b); }
			0x06 => { self.b = self.next_u8(bus); }
			0x0c => { self.c = self.inc_u8(self.c); }
			0x0d => { self.c = self.dec_u8(self.c); }
			0x0e => { self.c = self.next_u8(bus); }
			0x11 => { let de = self.next_u16(bus); self.set_de(de); }
			0x13 => { let de = self.inc_u16(bus, self.de()); self.set_de(de); }
			0x17 => { self.a = self.rl_u8(self.a); self.zero = false; }
			0x18 => { self.jr(bus); }
			0x1a => { let de = self.de(); self.a = self.read_u8(bus, de); }
			0x1e => { self.e = self.next_u8(bus); }
			0x20 => { let c = !self.zero; self.jr_cond(bus, c); }
			0x21 => { let value = self.next_u16(bus); self.set_hl(value); }
			0x22 => {
				let hl = self.hl();

				self.write_hl(bus, self.a);
				self.set_hl(hl.wrapping_add(1));
			}
			0x23 => { let hl = self.inc_u16(bus, self.hl()); self.set_hl(hl); }
			0x28 => { let c = self.zero; self.jr_cond(bus, c); }
			0x2e => { self.l = self.next_u8(bus); }
			0x31 => { self.sp = self.next_u16(bus); }
			0x32 => {
				let hl = self.hl();

				self.write_hl(bus, self.a);
				self.set_hl(hl.wrapping_sub(1));
			}
			0x3d => { self.a = self.dec_u8(self.a); }
			0x3e => { self.a = self.next_u8(bus); }
			0x4f => { self.c = self.a; }
			0x57 => { self.d = self.a; }
			0x67 => { self.h = self.a; }
			0x77 => { self.write_hl(bus, self.a); }
			0x7b => { self.a = self.e; }
			0xaf => { self.xor_u8(self.a); }
			0xc1 => { let bc = self.pop(bus); self.set_bc(bc); }
			0xcb => { self.run_cb_instruction(bus); }
			0xcd => { self.call(bus); }
			0xc5 => { let bc = self.bc(); self.push(bus, bc); }
			0xc9 => { self.ret(bus); }
			0xe0 => { 
				let addr = 0xFF00 + self.next_u8(bus) as u16;
				self.write_u8(bus, addr, self.a);
			}
			0xe2 => { self.write_u8(bus, 0xFF00 + (self.c as u16), self.a); }
			0xea => { let addr = self.next_u16(bus); self.write_u8(bus, addr, self.a); }
			0xf0 => { 
				let addr = 0xFF00 + self.next_u8(bus) as u16;
				self.a = self.read_u8(bus, addr);
			}
			0xf1 => { let af = self.pop(bus); self.set_af(af); }
			0xfe => { let value = self.next_u8(bus); self.cp_u8(value); }
			_ => unimplemented!("Instruction not implemented ({:02x})", instr)
		}
	}

	fn run_cb_instruction(&mut self, bus: &mut Bus) {
		let instr = self.next_u8(bus);

		match instr {
			0x11 => { self.c = self.rl_u8(self.c); }
			0x7c => { self.bit_u8(7, self.h); }
			_ => unimplemented!("CB-prefixed instruction not implemented cb{:02x}", instr),
		}
	}
}

impl std::fmt::Debug for CPU {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f,
			"CPU {{ \n\
				AF: {:04x} BC: {:04x}\n\
				DE: {:04x} HL: {:04x}\n\
				PC: {:04x} SP: {:04x}\n\
			}}",
			self.af(), self.bc(),
			self.de(), self.hl(),
			self.pc, self.sp
		)
	}
}