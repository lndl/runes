extern crate sdl2;

pub mod rom;
pub mod gfx;
pub mod ppu;
pub mod mos6502;

use std::io::Result;
use rom::Rom;
use ppu::Ppu;
use gfx::{Gfx, Scale};
use mos6502::cpu::CPU;

#[derive(Debug)]
struct Config {
    nes_file: String
}

fn parse_config() -> Config {
    let config = Config { nes_file: String::from("dk.nes") };

    return config;
}

pub fn main() -> Result<()> {
    println!("Rust NES Emulator");
    let config = parse_config();
    let rom = Rom::from_file(config.nes_file)?;

    let mut cpu = CPU::new();
    let mut ppu = Ppu::new();

    //ppu.draw_spritesheet(&rom);

    //println!("{:?}", &ppu.screen[0..100]);

    //let mut gfx = Gfx::new(Scale::Scale3x);
    //gfx.main_loop(&mut ppu);

    Ok(())
}
