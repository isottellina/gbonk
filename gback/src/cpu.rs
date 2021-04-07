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

    fn read_u8(&mut self, bus: &mut Bus, addr: u16) -> u8 {
        bus.read_u8(addr)
    }

    fn read_u16(&mut self, bus: &mut Bus, addr: u16) -> u16 {
        bus.read_u8(addr) as u16 |
        (bus.read_u8(addr + 1) as u16) << 8
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

    fn xor_u8(&mut self, value: u8) {
        self.a ^= value;

        self.zero = self.a == 0;
        self.sub = false;
        self.halfcarry = false;
        self.carry = false;
    }

    pub fn run_instruction(&mut self, bus: &mut Bus) {
        let instr = self.next_u8(bus);

        match instr {
            0x21 => { let value = self.next_u16(bus); self.set_hl(value); }
            0x31 => { self.sp = self.next_u16(bus); }
            0xaf => { self.xor_u8(self.a); }
            _ => unimplemented!("Instruction not implemented ({:02x})", instr)
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