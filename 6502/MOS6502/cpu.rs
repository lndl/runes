use std::convert::TryFrom;
use std::fmt;

use mos6502::program::Program;

pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    sr: u8,
    sp: u8,
    pc: u16,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            sr: 0,
            sp: 0xFF,
            pc: 0
        }
    }

    pub fn exec(&self, program_bytes: Vec<u8>) -> Result<(), &str> {
        let program = Program::try_from(program_bytes)?;
        Ok(())
    }
}

impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CPU State: <a: {}, b: {}, y: {}, sr: {}, sp: {}, pc: {}>\n",
               self.a, self.x, self.y, self.sr, self.sp, self.pc)
    }
}
