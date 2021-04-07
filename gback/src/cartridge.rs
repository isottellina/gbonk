use std::io::Read;

#[derive(Default)]
pub struct Cartridge {
    rom: Vec<u8>,
}

impl Cartridge {
    pub fn from_file(file: std::fs::File) -> Cartridge {
        let rom = Self::get_vec_from_file(file);

        Cartridge {
            rom
        }
    }

    pub fn load_file(&mut self, file: std::fs::File) {
        self.rom = Self::get_vec_from_file(file);
    }

    fn get_vec_from_file(mut file: std::fs::File) -> Vec<u8> {
        let mut rom = vec![];
        file.read_to_end(&mut rom).unwrap();

        if rom[0x147] != 0 {
            unimplemented!("The emulator supports only ROM-ONLY games for now.");
        }

        rom
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        self.rom[addr as usize]
    }
}