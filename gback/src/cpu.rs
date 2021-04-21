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

	ime: bool,
}

impl CPU {
	fn f(&self) -> u8 {
		((self.zero as u8) << 7) |
		((self.sub as u8) << 6) |
		((self.halfcarry as u8) << 5) |
		((self.carry as u8) << 4)
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
		self.zero = (value & 0x80) != 0;
		self.sub = (value & 0x40) != 0;
		self.halfcarry = (value & 0x20) != 0;
		self.carry = (value & 0x10) != 0;
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
	fn read_u8(&self, bus: &mut Bus, addr: u16) -> u8 {
		bus.delay(1);
		bus.read_u8(addr)
	}

	fn read_u16(&self, bus: &mut Bus, addr: u16) -> u16 {
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

	fn read_hl(&self, bus: &mut Bus) -> u8 {
		let hl = self.hl();
		self.read_u8(bus, hl)
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

	fn jp_cond(&mut self, bus: &mut Bus, cond: bool) {
		let addr = self.next_u16(bus);

		if cond {
			self.set_pc(bus, addr);
		}
	}

	fn call(&mut self, bus: &mut Bus) {
		let addr = self.next_u16(bus);
		self.push(bus, self.pc);
		self.set_pc(bus, addr);
	}

	fn call_cond(&mut self, bus: &mut Bus, cond: bool) {
		let addr = self.next_u16(bus);

		if cond {
			self.push(bus, self.pc);
			self.set_pc(bus, addr);
		}
	}

	fn rst(&mut self, bus: &mut Bus, addr: u16) {
		self.push(bus, self.pc);
		self.set_pc(bus, addr);
	}

	fn ret(&mut self, bus: &mut Bus) {
		let pc = self.pop(bus);
		self.set_pc(bus, pc);
	}

	fn ret_cond(&mut self, bus: &mut Bus, cond: bool) {
		bus.delay(1);

		if cond {
			let pc = self.pop(bus);
			self.set_pc(bus, pc);
		}
	}

	// Arithmetic functions
	fn add_u8(&mut self, value: u8) {
		let a = self.a as u16;
		let v = value as u16;

		let res = a + v;

		self.zero = (res & 0xff) == 0;
		self.sub = false;
		self.halfcarry = (a ^ v ^ res) & 0x10 == 0x10;
		self.carry = (res & 0x100) == 0x100;

		self.a = res as u8;
	}

	fn add_u16(&mut self, bus: &mut Bus, value: u16) {
		let a = self.hl() as u32;
		let v = value as u32;

		let res = a + v;

		self.sub = false;
		self.halfcarry = (a ^ v ^ res) & 0x1000 == 0x1000;
		self.carry = (res & 0x10000) == 0x10000;

		self.set_hl(res as u16);
		bus.delay(1);
	}

	fn adc_u8(&mut self, value: u8) {
		let carry = self.carry as u8;
		let res = (self.a as u16) + (value as u16) + carry as u16;

		self.zero = (res & 0xff) == 0;
		self.sub = false;
		self.halfcarry = (((self.a & 0xf) + (value & 0xf) + carry) & 0x10) == 0x10;
		self.carry = (res & 0x100) == 0x100;

		self.a = res as u8;
	}

	fn sub_u8(&mut self, value: u8) {
		self.zero = self.a == value;
		self.sub = true;
		self.halfcarry = (self.a & 0xf) < (value & 0xf);
		self.carry = self.a < value;

		self.a = self.a.wrapping_sub(value);
	}

	fn sbc_u8(&mut self, value: u8) {
		let carry = self.carry as u8;
		self.zero = self.a == value.wrapping_add(carry);
		self.sub = true;
		self.halfcarry = (self.a & 0xf) < (value.wrapping_add(carry) & 0xf);
		self.carry = self.a < value.wrapping_add(1);

		self.a = self.a.wrapping_sub(value).wrapping_sub(carry);
	}

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

	fn dec_u16(&self, bus: &mut Bus, value: u16) -> u16 {
		bus.delay(1);

		value.wrapping_sub(1)
	}

	// Logic functions
	fn xor_u8(&mut self, value: u8) {
		self.a ^= value;

		self.zero = self.a == 0;
		self.sub = false;
		self.halfcarry = false;
		self.carry = false;
	}

	fn and_u8(&mut self, value: u8) {
		self.a &= value;

		self.zero = self.a == 0;
		self.sub = false;
		self.halfcarry = true;
		self.carry = false;
	}

	fn or_u8(&mut self, value: u8) {
		self.a |= value;

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

	fn rr_u8(&mut self, value: u8) -> u8 {
		let carry = self.carry as u8;
		self.carry = (value & 0x01) != 0;

		let new_value = (value >> 1) | (carry << 7);

		self.zero = new_value == 0;
		self.sub = false;
		self.halfcarry = false;

		new_value
	}

	fn swap_u8(&mut self, value: u8) -> u8 {
		(value >> 4) | ((value & 0x0f) << 4)
	}

	fn srl_u8(&mut self, value: u8) -> u8 {
		let new_value = value >> 1;

		self.zero = new_value == 0;
		self.sub = false;
		self.halfcarry = false;
		self.carry = (value & 1) != 0;

		new_value
	}

	fn res_u8(&mut self, bit: u8, value: u8) -> u8 {
		value & !(1 << bit)
	}

	fn bit_u8(&mut self, bit: u8, value: u8) {
		self.zero = (value & (1 << bit)) == 0;
		self.sub = false;
		self.halfcarry = true;
	}

	fn run_instruction(&mut self, bus: &mut Bus) {
		let instr = self.next_u8(bus);

		match instr {
			0x00 => {}
			0x01 => { let bc = self.next_u16(bus); self.set_bc(bc); }
			0x03 => { let bc = self.inc_u16(bus, self.bc()); self.set_bc(bc); }
			0x04 => { self.b = self.inc_u8(self.b); }
			0x05 => { self.b = self.dec_u8(self.b); }
			0x06 => { self.b = self.next_u8(bus); }
			0x0b => { let bc = self.dec_u16(bus, self.bc()); self.set_bc(bc); }
			0x0c => { self.c = self.inc_u8(self.c); }
			0x0d => { self.c = self.dec_u8(self.c); }
			0x0e => { self.c = self.next_u8(bus); }
			0x11 => { let de = self.next_u16(bus); self.set_de(de); }
			0x12 => { self.write_u8(bus, self.de(), self.a); }
			0x13 => { let de = self.inc_u16(bus, self.de()); self.set_de(de); }
			0x14 => { self.d = self.inc_u8(self.d); }
			0x15 => { self.d = self.dec_u8(self.d); }
			0x16 => { self.d = self.next_u8(bus); }
			0x17 => { self.a = self.rl_u8(self.a); self.zero = false; }
			0x18 => { self.jr(bus); }
			0x19 => { self.add_u16(bus, self.de()); }
			0x1d => { self.e = self.dec_u8(self.e); }
			0x1c => { self.e = self.inc_u8(self.e); }
			0x1a => { let de = self.de(); self.a = self.read_u8(bus, de); }
			0x1e => { self.e = self.next_u8(bus); }
			0x1f => { self.a = self.rr_u8(self.a); self.zero = false; }
			0x20 => { let c = !self.zero; self.jr_cond(bus, c); }
			0x21 => { let value = self.next_u16(bus); self.set_hl(value); }
			0x22 => {
				let hl = self.hl();

				self.write_hl(bus, self.a);
				self.set_hl(hl.wrapping_add(1));
			}
			0x23 => { let hl = self.inc_u16(bus, self.hl()); self.set_hl(hl); }
			0x24 => { self.h = self.inc_u8(self.h); }
			0x25 => { self.h = self.dec_u8(self.h); }
			0x26 => { self.h = self.next_u8(bus); }
			0x28 => { let c = self.zero; self.jr_cond(bus, c); }
			0x29 => { self.add_u16(bus, self.hl()); }
			0x2a => {
				let hl = self.hl();

				self.a = self.read_hl(bus);
				self.set_hl(hl.wrapping_add(1));
			}
			0x2c => { self.l = self.inc_u8(self.l); }
			0x2d => { self.l = self.dec_u8(self.l); }
			0x2e => { self.l = self.next_u8(bus); }
			0x2f => { 
				self.a ^= 0xFF;
				self.sub = true;
				self.halfcarry = true;
			}
			0x30 => { self.jr_cond(bus, !self.carry); }
			0x31 => { self.sp = self.next_u16(bus); }
			0x32 => {
				let hl = self.hl();

				self.write_hl(bus, self.a);
				self.set_hl(hl.wrapping_sub(1));
			}
			0x34 => {
				let value = self.inc_u8(self.read_hl(bus));
				self.write_hl(bus, value);
			}
			0x35 => {
				let value = self.dec_u8(self.read_hl(bus));
				self.write_hl(bus, value);
			}
			0x36 => { let value = self.next_u8(bus); self.write_hl(bus, value); }
			0x3c => { self.a = self.inc_u8(self.a); }
			0x3d => { self.a = self.dec_u8(self.a); }
			0x3e => { self.a = self.next_u8(bus); }
			0x40 => { self.b = self.b; }
			0x41 => { self.b = self.c; }
			0x42 => { self.b = self.d; }
			0x43 => { self.b = self.e; }
			0x44 => { self.b = self.h; }
			0x45 => { self.b = self.l; }
			0x46 => { self.b = self.read_hl(bus); }
			0x47 => { self.b = self.a; }
			0x48 => { self.c = self.b; }
			0x49 => { self.c = self.c; }
			0x4a => { self.c = self.d; }
			0x4b => { self.c = self.e; }
			0x4c => { self.c = self.h; }
			0x4d => { self.c = self.l; }
			0x4e => { self.c = self.read_hl(bus); }
			0x4f => { self.c = self.a; }
			0x50 => { self.d = self.b; }
			0x51 => { self.d = self.c; }
			0x52 => { self.d = self.d; }
			0x53 => { self.d = self.e; }
			0x54 => { self.d = self.h; }
			0x55 => { self.d = self.l; }
			0x56 => { self.d = self.read_hl(bus); }
			0x57 => { self.d = self.a; }
			0x58 => { self.e = self.b; }
			0x59 => { self.e = self.c; }
			0x5a => { self.e = self.d; }
			0x5b => { self.e = self.e; }
			0x5c => { self.e = self.h; }
			0x5d => { self.e = self.l; }
			0x5e => { self.e = self.read_hl(bus); }
			0x5f => { self.e = self.a; }
			0x60 => { self.h = self.b; }
			0x61 => { self.h = self.c; }
			0x62 => { self.h = self.d; }
			0x63 => { self.h = self.e; }
			0x64 => { self.h = self.h; }
			0x65 => { self.h = self.l; }
			0x66 => { self.h = self.read_hl(bus); }
			0x67 => { self.h = self.a; }
			0x68 => { self.l = self.b; }
			0x69 => { self.l = self.c; }
			0x6a => { self.l = self.d; }
			0x6b => { self.l = self.e; }
			0x6c => { self.l = self.h; }
			0x6d => { self.l = self.l; }
			0x6e => { self.l = self.read_hl(bus); }
			0x6f => { self.l = self.a; }
			0x70 => { self.write_hl(bus, self.b); }
			0x71 => { self.write_hl(bus, self.c); }
			0x72 => { self.write_hl(bus, self.d); }
			0x73 => { self.write_hl(bus, self.e); }
			0x74 => { self.write_hl(bus, self.h); }
			0x75 => { self.write_hl(bus, self.l); }
			0x77 => { self.write_hl(bus, self.a); }
			0x78 => { self.a = self.b; }
			0x79 => { self.a = self.c; }
			0x7a => { self.a = self.d; }
			0x7b => { self.a = self.e; }
			0x7c => { self.a = self.h; }
			0x7d => { self.a = self.l; }
			0x7e => { self.a = self.read_hl(bus); }
			0x7f => { self.a = self.a; }
			0x86 => { let value = self.read_hl(bus); self.add_u8(value); }
			0x87 => { self.add_u8(self.a); }
			0x90 => { self.sub_u8(self.b); }
			0xa1 => { self.and_u8(self.c); }
			0xa7 => { self.and_u8(self.a); }
			0xa9 => { self.xor_u8(self.c); }
			0xaa => { self.xor_u8(self.d); }
			0xab => { self.xor_u8(self.e); }
			0xac => { self.xor_u8(self.h); }
			0xad => { self.xor_u8(self.l); }
			0xae => { self.xor_u8(self.read_hl(bus)); }
			0xaf => { self.xor_u8(self.a); }
			0xb0 => { self.or_u8(self.b); }
			0xb1 => { self.or_u8(self.c); }
			0xb6 => { self.or_u8(self.read_hl(bus)); }
			0xb7 => { self.or_u8(self.a); }
			0xbe => { let value = self.read_hl(bus); self.cp_u8(value); }
			0xc0 => { self.ret_cond(bus, !self.zero); }
			0xc1 => { let bc = self.pop(bus); self.set_bc(bc); }
			0xc3 => { let pc = self.next_u16(bus); self.set_pc(bus, pc); }
			0xc4 => { self.call_cond(bus, !self.zero); }
			0xc5 => { let bc = self.bc(); self.push(bus, bc); }
			0xc6 => { let value = self.next_u8(bus); self.add_u8(value); }
			0xc8 => { self.ret_cond(bus, self.zero);  }
			0xc9 => { self.ret(bus); }
			0xca => { self.jp_cond(bus, self.zero); }
			0xcb => { self.run_cb_instruction(bus); }
			0xce => { let value = self.next_u8(bus); self.adc_u8(value); }
			0xcd => { self.call(bus); }
			0xd0 => { self.ret_cond(bus, !self.carry); }
			0xd1 => { let de = self.pop(bus); self.set_de(de); }
			0xd5 => { self.push(bus, self.de()) }
			0xd6 => { let value = self.next_u8(bus); self.sub_u8(value); }
			0xd8 => { self.ret_cond(bus, self.carry); }
			0xd9 => { self.ret(bus); self.ime = true; }
			0xde => { let value = self.next_u8(bus); self.sbc_u8(value); }
			0xe0 => { 
				let addr = 0xFF00 + self.next_u8(bus) as u16;
				self.write_u8(bus, addr, self.a);
			}
			0xe1 => { let value = self.pop(bus); self.set_hl(value); }
			0xe2 => { self.write_u8(bus, 0xFF00 + (self.c as u16), self.a); }
			0xe5 => { self.push(bus, self.hl()); }
			0xe6 => { let value = self.next_u8(bus); self.and_u8(value); }
			0xe9 => { self.set_pc(bus, self.hl()); }
			0xea => { let addr = self.next_u16(bus); self.write_u8(bus, addr, self.a); }
			0xee => { let value = self.next_u8(bus); self.xor_u8(value); }
			0xef => { self.rst(bus, 0x28) }
			0xf0 => { 
				let addr = 0xFF00 + self.next_u8(bus) as u16;
				self.a = self.read_u8(bus, addr);
			}
			0xf1 => { let af = self.pop(bus); self.set_af(af); }
			0xf3 => { self.ime = false; }
			0xf5 => { self.push(bus, self.af()); }
			0xfa => { let addr = self.next_u16(bus); self.a = self.read_u8(bus, addr); }
			0xfb => { self.ime = true; }
			0xfe => { let value = self.next_u8(bus); self.cp_u8(value); }
			0xff => { self.rst(bus, 0x38) }
			_ => unimplemented!("Instruction not implemented ({:02x})", instr)
		}
	}

	fn run_cb_instruction(&mut self, bus: &mut Bus) {
		let instr = self.next_u8(bus);

		match instr {
			0x11 => { self.c = self.rl_u8(self.c); }
			0x19 => { self.c = self.rr_u8(self.c); }
			0x1a => { self.d = self.rr_u8(self.d); }
			0x1b => { self.e = self.rr_u8(self.e); }
			0x1c => { self.h = self.rr_u8(self.h); }
			0x1d => { self.l = self.rr_u8(self.l); }
			0x1f => { self.a = self.rr_u8(self.a); }
			0x37 => { self.a = self.swap_u8(self.a); }
			0x38 => { self.b = self.srl_u8(self.b); }
			0x7c => { self.bit_u8(7, self.h); }
			0x87 => { self.a = self.res_u8(0, self.a); }
			_ => unimplemented!("CB-prefixed instruction not implemented cb{:02x}", instr),
		}
	}

	pub fn step(&mut self, bus: &mut Bus) {
		self.run_instruction(bus);

		if let Some(it) = bus.has_irq() {
			if self.ime {
				self.push(bus, self.pc);
				self.set_pc(bus, it);

				self.ime = false;
				bus.ack_irq();
			}
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