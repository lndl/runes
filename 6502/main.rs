use std::io::Result;

mod mos6502;
use mos6502::cpu::CPU;

pub mod rom;
use rom::Rom;

fn main() -> Result<()> {
    let mut ram = [0; 8192];
    let mut rom = Rom::from_file(String::from("nestest.nes"))?;
    let mut cpu = CPU::new();

    let mut apu = [0; 32]; // TODO

    cpu.mount_mapper(0x0000..0x2000, &mut ram);
    cpu.mount_mapper(0x4000..0x4020, &mut apu);
    cpu.mount_mapper(0xC000..0xFFFF, rom.program());

    cpu.exec(0xc000);

    Ok(())
}
