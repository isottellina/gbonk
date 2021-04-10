use std::io::Read;
use crate::cartridge::Cartridge;
use crate::ppu::PPU;
use crate::apu::APU;

pub struct Bus {
    bios: [u8; 0x100],
    cart: Cartridge,
    ppu: PPU,
    apu: APU,
    hram: [u8; 0x80],

    cycles_to_spend: u32,
    bios_enable: bool,
}

impl Bus {
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
            0..=0x7FFF => self.cart.read_u8(addr),
            0xFF40..=0xFF4B => self.ppu.read_io_register(addr),
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            _ => unimplemented!("This address has not been implemented yet. {:04x}", addr)
        }
    }

    pub fn write_u8(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => self.ppu.write_vram_u8(addr, value),
            0xFF10..=0xFF26 => self.apu.write_io_register(addr, value),
            0xFF40..=0xFF4B => self.ppu.write_io_register(addr, value),
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = value,
            _ => unimplemented!("Write to unmapped adress ({:04x}, {:02x})", addr, value),
        }
    }

    pub fn spend(&mut self) {
        let t_state = self.cycles_to_spend << 2;

        self.ppu.spend(t_state);
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            cart: Default::default(),
            ppu: Default::default(),
            apu: Default::default(),
            bios: [0; 0x100],
            hram: [0; 0x80],

            cycles_to_spend: 0,
            bios_enable: true
        }
    }
}

impl std::fmt::Debug for Bus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bus")
    }
}