use std::io::Result;

mod mos6502;
use mos6502::cpu::CPU;

pub mod rom;
use rom::Rom;

fn main() -> Result<()> {
    let mut ram = [0; 8192];
    let mut rom = Rom::from_file(String::from("official_only.nes"))?;
    let mut cpu = CPU::new();

    let mut apu = [0; 32]; // TODO
    let mut ppu = [0; 8];  // TODO

    cpu.mount_mapper(0x0000..0x0800, &mut ram);
    cpu.mount_mapper(0x2000..0x2008, &mut ppu);
    cpu.mount_mapper(0x4000..0x4018, &mut apu);

    cpu.mount_mapper(0xC000..0x10000, rom.program());

    cpu.exec(None);

    Ok(())
}
