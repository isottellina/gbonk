use std::io::Read;
use crate::cartridge::Cartridge;

pub struct Bus {
    bios: [u8; 0x100],
    cart: Cartridge,
}

impl Bus {
    pub fn load_bios(&mut self, mut file: std::fs::File) {
        file.read_exact(&mut self.bios).unwrap();
    }

    pub fn load_rom(&mut self, file: std::fs::File) {
        self.cart.load_file(file);
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            cart: Default::default(),
            bios: [0; 0x100]
        }
    }
}

impl std::fmt::Debug for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bus")
    }
}