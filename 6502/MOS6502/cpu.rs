use std::fmt;

use mos6502::program::Program;

pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    sr: u8,
    sp: u8,
    pc: u32,
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

    pub fn exec(&mut self, program: Program) -> Result<(), &str> {
        let mut i = 0;
        while i < program.size() {
            let instruction = program.fetch(i);
            instruction.exec(self);
            self.pc += instruction.bytesize() as u32;
            i += 1;
        }
        Ok(())
    }

    pub fn acc(&self) -> u8 {
        self.a
    }

    pub fn set_acc(&mut self, val: u8) {
        self.a = val;
    }
}

impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CPU State: <a: {}, b: {}, y: {}, sr: {}, sp: {}, pc: {:04x}>\n",
               self.a, self.x, self.y, self.sr, self.sp, self.pc)
    }
}
