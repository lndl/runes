use std::io::Result;

mod mos6502;
use mos6502::cpu::CPU;
use mos6502::memory_map::memory_map::MemMappeable;

pub mod rom;
use rom::Rom;

struct Ram {
    ram: [u8; 8192]
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            ram: [0; 8192]
        }
    }
}

impl MemMappeable for Ram {
    fn read(&self, address: usize) -> u8 {
        self.ram[address]
    }

    fn slice(&self, from: usize) -> &[u8] {
        &self.ram[from..self.ram.len() - 1]
    }

    fn write(&mut self, address: usize, value: u8) -> u8 {
        self.ram[address as usize] = value;
        value
    }
}

fn main() -> Result<()> {
    let ram = Ram::new();
    // FIXME: Change with proper PPU device!
    let ppu = Ram::new();
    // FIXME: Change with proper APU device!
    let apu = Ram::new();

    let rom = Rom::from_file(String::from("nestest.nes"))?;

    let mut cpu = CPU::new();

    // Reference: https://wiki.nesdev.com/w/index.php/CPU_memory_map
    cpu.mount_in_bus(0x0000..0x0800, ram);
    cpu.mount_in_bus(0x2000..0x2008, ppu);
    cpu.mount_in_bus(0x4000..0x4018, apu);
    //cpu.mount_in_bus(0x4018..0x4020, ...); // Only documental
    cpu.mount_in_bus(rom.cpu_address_range(), rom);

    cpu.exec(Some(0xC000));

    Ok(())
}
