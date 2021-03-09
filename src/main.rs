extern crate sdl2;
extern crate mos6502;

pub mod rom;
pub mod gfx;
pub mod ppu;

use std::io::Result;

use mos6502::cpu::CPU;
use mos6502::memory_map::memory_map::{MemoryMap, Memorable};
use rom::Rom;
use ppu::Ppu;
use gfx::{Gfx, Scale};

const RAM_SIZE : usize = 2048;

struct Ram {
    ram: [u8; RAM_SIZE]
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            ram: [0; RAM_SIZE]
        }
    }
}

impl Memorable for Ram {
    /* RAM Mirrored:
    0x0000...0x07FF = Original
    0x0800...0x0FFF = Mirror 1
    0x1000...0x17FF = Mirror 2
    0x1800...0x1FFF = Mirror 3
    */

    fn read(&self, address: usize) -> u8 {
        self.ram[address % RAM_SIZE]
    }

    fn slice(&self, from: usize) -> &[u8] {
        let range = (from % RAM_SIZE)..self.ram.len() - 1;
        &self.ram[range]
    }

    fn write(&mut self, address: usize, value: u8) -> u8 {
        self.ram[address % RAM_SIZE] = value;
        value
    }
}

#[derive(Debug)]
struct Config {
    nes_file: String
}

fn parse_config() -> Config {
    let config = Config { nes_file: String::from("testroms/dk.nes") };

    return config;
}

pub struct NesDevice {
    cpu: CPU,
    ppu: std::rc::Rc<std::cell::RefCell<Ppu>>
}

impl NesDevice {
    pub fn new(rom: Rom) -> NesDevice {
        let mut memmap = MemoryMap::new();

        let ram = Ram::new();
        let ppu = Ppu::new();
        // FIXME: Change with proper APU device!
        let apu = Ram::new();

        let r1 = std::rc::Rc::new(std::cell::RefCell::new(ram));
        let p  = std::rc::Rc::new(std::cell::RefCell::new(ppu));
        let a  = std::rc::Rc::new(std::cell::RefCell::new(apu));
        let r2 = std::rc::Rc::new(std::cell::RefCell::new(rom));

        // Reference: https://wiki.nesdev.com/w/index.php/CPU_memory_map
        memmap.register_device(0x0000..0x2000, r1); // Mirrored
        memmap.register_device(0x2000..0x4000, p.clone()); // Mirrored
        memmap.register_device(0x4000..0x4018, a);
        //self.cpu.mount_reader_in_bus(0x4018..0x4020, ...); // Only documental
        memmap.register_device(r2.clone().borrow().cpu_address_range(), r2);

        let mut cpu = CPU::new(memmap);

        cpu.request_reset();

        NesDevice {
            cpu: cpu,
            ppu: p
        }
    }

    pub fn reset(&mut self) {
        self.cpu.request_reset();
    }

    pub fn step(&mut self) -> String {
        let status = self.cpu.step();
        let ppu_status = self.ppu.clone().borrow_mut().step();

        match ppu_status {
            ppu::PpuStepEval::Normal => {

            }
            ppu::PpuStepEval::NMI => {
                self.cpu.request_nmi();
            }
        }

        status
    }

    pub fn show_memmap(&self) -> String {
        self.cpu.show_memmap()
    }
}

pub fn main() -> Result<()> {
    println!("Rust NES Emulator");
    let config = parse_config();
    let rom = Rom::from_file(config.nes_file)?;

    let mut nes = NesDevice::new(rom);

    let mut gfx = Gfx::new(Scale::Scale1x);
    gfx.debug_loop(&mut nes);

    Ok(())
}
