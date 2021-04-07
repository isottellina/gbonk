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