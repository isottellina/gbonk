extern crate clap;
extern crate minifb;
extern crate gback;

mod platform;

use clap::{App, Arg};
use std::fs::File;
use gback::Gameboy;

fn main() -> std::io::Result<()> {
    let matches = App::new("GBonk")
        .about("A simple Gameboy emulator written live.")
        .author("Isottellina")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("BOOTROM")
            .short("b")
            .long("bootrom")
            .required(true)
            .value_name("bootrom"))
        .arg(Arg::with_name("ROM")
            .required(true)
            .value_name("rom"))
        .get_matches();

    let bootrom_fn = matches.value_of("BOOTROM").unwrap();
    let rom_fn = matches.value_of("ROM").unwrap();
    let bootrom = File::open(bootrom_fn)?;
    let rom = File::open(rom_fn)?;
    
    let mut platform = platform::MiniFBPlatform::new();
    let mut gameboy = Gameboy::new();
    gameboy.load_bios(bootrom);
    gameboy.load_rom(rom);

    loop {
        gameboy.run_frame(&mut platform);
    }
}
