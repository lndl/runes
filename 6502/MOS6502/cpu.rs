use std::fmt;
use std::collections::HashSet;

use mos6502::program::Program;
use mos6502::instruction::Executable;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Flag {
    Carry,
    Zero,
    Interrupt,
    Decimal,
    Break,
    Overflow,
    Negative,
}

pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u32,

    flags: HashSet<Flag>
}

impl CPU {
    pub fn new() -> Self {
        let mut flags: HashSet<Flag> = HashSet::new();
        flags.insert(Flag::Decimal);

        CPU {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFF,
            pc: 0,
            flags: flags
        }
    }

    pub fn exec(&mut self, program: Program) -> Result<(), &str> {
        while self.pc < program.size() {
            let instruction = program.fetch(self.pc);
            instruction.exec(self);
            self.pc += instruction.bytesize() as u32;
        }
        Ok(())
    }

    pub fn acc(&self) -> u8                  { self.a }
    pub fn set_acc(&mut self, val: u8)       { self.a = val; }
    pub fn set_flag(&mut self, flag: Flag)   { self.flags.insert(flag); }
    pub fn unset_flag(&mut self, flag: Flag) { self.flags.remove(&flag); }
    pub fn flags(&self) -> &HashSet<Flag>    { &self.flags }
}

impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CPU State: <a: {}, b: {}, y: {}, flags: {:?}, sp: {}, pc: {:04x}>\n",
               self.a, self.x, self.y, self.flags(), self.sp, self.pc)
    }
}
