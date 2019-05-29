use std::io::Result;

pub mod rom;
use rom::Rom;

#[derive(Debug)]
struct Config {
    nes_file: String
}

fn parse_config() -> Config {
    let config = Config { nes_file: String::from("sm3.nes") };

    return config;
}

fn main() -> Result<()> {
    println!("Rust NES Emulator");
    let config = parse_config();

    println!("Configuration is: {:#?}", config);
    let rom = Rom::from_file(config.nes_file)?;

    println!("Rom is:\n {:#?}", rom);

    Ok(())
}
