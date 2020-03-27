use std::io::Result;

mod mos6502;
use mos6502::cpu::CPU;
use std::convert::TryFrom;

pub mod rom;
use rom::Rom;

fn main() -> Result<()> {
    let mut cpu = CPU::new();
    let program_bytes = [
        0xa9, 0x01,
        0x8d, 0x00, 0x02,
        0xa9, 0x05,
        0x8d, 0x01, 0x02,
        0xa9, 0x08,
        0x8d, 0x02, 0x02,
    ].to_vec();

    let rom = Rom::from_file(String::from("nestest.nes"))?;

    /*
    println!("---------------------------------------------------");
    println!("{:?}", rom);
    println!("---------------------------------------------------");
    */

    cpu.load_program(0xc000, rom.pgrrom_info().clone());

//    println!("{:?}", cpu.memory_map());

    cpu.exec(0xc000);

    Ok(())
}
