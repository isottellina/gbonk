use std::io::Read;
use crate::cartridge::Cartridge;

pub struct Bus {
    bios: [u8; 0x100],
    cart: Cartridge,

    bios_enable: bool,
}

impl Bus {
    pub fn load_bios(&mut self, mut file: std::fs::File) {
        file.read_exact(&mut self.bios).unwrap();
    }

    pub fn load_rom(&mut self, file: std::fs::File) {
        self.cart.load_file(file);
    }

    pub fn read_u8(&mut self, addr: u16) -> u8 {
        match addr {
            0..=0xFF if self.bios_enable => self.bios[addr as usize],
            0..=0x7FFF => self.cart.read_u8(addr),
            _ => unimplemented!("This address has not been implemented yet.")
        }
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            cart: Default::default(),
            bios: [0; 0x100],
            bios_enable: true
        }
    }
}

impl std::fmt::Debug for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bus")
    }
}