use std::convert::TryFrom;
use std::fmt;

use mos6502::cpu::CPU;

pub trait Instruction : fmt::Debug {
    fn exec(&self, cpu: &mut CPU) -> ();
    fn bytesize(&self) -> u8;
}

#[derive(Debug)]
struct LDAInmediate { x: u8 }
#[derive(Debug)]
struct STAAbsolute { abs: u8, y: u8 }
#[derive(Debug)]
struct BRK {}

pub fn try_build(chunk: &[u8]) -> Box<Instruction> {
    let bytecode = chunk[0];

    match bytecode {
        0xa9 => Box::new(LDAInmediate { x: chunk[1] }),
        0x8d => Box::new(STAAbsolute { abs: chunk[1], y: chunk[2] }),
        _    => Box::new(BRK {})
    }
}

impl Instruction for LDAInmediate {
    fn exec(&self, cpu: &mut CPU) -> () {
        ();
    }

    fn bytesize(&self) -> u8 {
        2
    }
}

impl Instruction for STAAbsolute {
    fn exec(&self, cpu: &mut CPU) -> () {
        ();
    }

    fn bytesize(&self) -> u8 {
        3
    }
}

impl Instruction for BRK {
    fn exec(&self, cpu: &mut CPU) -> () {
        ();
    }

    fn bytesize(&self) -> u8 {
        1
    }
}
