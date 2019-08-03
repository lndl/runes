use std::convert::TryFrom;
use std::fmt;

use mos6502::cpu::CPU;
use mos6502::cpu::Flag;

pub trait Instruction : fmt::Debug {
    fn exec(&self, cpu: &mut CPU) -> ();
    fn bytesize(&self) -> u8;
}

#[derive(Debug)]
struct LDAInmediate { x: u8 }
#[derive(Debug)]
struct STAAbsolute { abs_l: u8, abs_h: u8 }
#[derive(Debug)]
struct BRK {}

pub fn try_build(chunk: &[u8]) -> Box<Instruction> {
    let bytecode = chunk[0];

    match bytecode {
        0x00 => Box::new(BRK {}),
        0xa9 => Box::new(LDAInmediate { x: chunk[1] }),
        0x8d => Box::new(STAAbsolute { abs_l: chunk[1], abs_h: chunk[2] }),
        _    => Box::new(BRK {})
    }
}

impl Instruction for LDAInmediate {
    fn exec(&self, cpu: &mut CPU) -> () {
        cpu.set_acc(self.x);
    }

    fn bytesize(&self) -> u8 {
        2
    }
}

impl Instruction for STAAbsolute {
    fn exec(&self, cpu: &mut CPU) -> () {
        let address = ((self.abs_h as u16) << 8) | self.abs_l as u16;
        println!("Store current accumulator value {} in memory address: {:04x}", cpu.acc(), address);
    }

    fn bytesize(&self) -> u8 {
        3
    }
}

impl Instruction for BRK {
    fn exec(&self, cpu: &mut CPU) -> () {
        cpu.set_flag(Flag::Interrupt);
    }

    fn bytesize(&self) -> u8 {
        1
    }
}
