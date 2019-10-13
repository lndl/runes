use std::convert::TryFrom;
use std::fmt;

use mos6502::cpu::CPU;
use mos6502::cpu::Flag;

pub trait Executable : fmt::Debug {
    fn exec(&self, cpu: &mut CPU) -> ();
    fn bytesize(&self) -> u8;
}

#[derive(Debug)]
pub enum Instruction {
    LDAInmediate { x: u8 },
    STAAbsolute { abs_l: u8, abs_h: u8 },
    BRK,
}

use self::Instruction::*;

pub fn try_build(chunk: &[u8]) -> Instruction {
    let bytecode = chunk[0];

    match bytecode {
        0x00 => BRK {},
        0xa9 => LDAInmediate { x: chunk[1] },
        0x8d => STAAbsolute { abs_l: chunk[1], abs_h: chunk[2] },
        _    => BRK {}
    }
}

impl Executable for Instruction {
    fn exec(&self, cpu: &mut CPU) -> () {
        match *self {
            LDAInmediate { x } => {
                cpu.set_acc(x);
            },
            STAAbsolute { abs_l, abs_h } => {
                let address = ((abs_h as u16) << 8) | abs_l as u16;
                println!("Store current accumulator value {} in memory address: {:04x}", cpu.acc(), address);
            },
            BRK => {
                cpu.set_flag(Flag::Interrupt);
            }
        }
    }

    fn bytesize(&self) -> u8 {
        match self {
            LDAInmediate { .. } => 2,
            STAAbsolute  { .. } => 3,
            BRK                 => 1
        }
    }
}
