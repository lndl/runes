use std::convert::TryFrom;
use std::fmt;

use mos6502;
use mos6502::instruction::Instruction;

pub struct Program {
    instructions: Vec<Box<Instruction>>
}

impl Program {
    pub fn instructions(&self) -> &Vec<Box<Instruction>> {
        &self.instructions
    }

    pub fn size(&self) -> u32 {
        self.instructions.len() as u32
    }

    pub fn fetch(&self, i: u32) -> &Box<Instruction> {
        &self.instructions[i as usize]
    }
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Program listing:\n");
        write!(f, "----------------\n");
        for instruction in self.instructions() {
            write!(f, "{:?}\n", instruction);
        }
        write!(f, "\n")
    }
}

impl TryFrom<Vec<u8>> for Program {
    type Error = &'static str;

    fn try_from(program_bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let mut instructions = Vec::<Box<Instruction>>::new();
        let mut i = 0;
        let program_size = program_bytes.len();

        while i < program_size {
            // Try to build the instruction
            let instruction = mos6502::instruction::try_build(&program_bytes[i..program_size]);
            // Advance in processing
            i = i + instruction.bytesize() as usize;
            // Push the instruction
            instructions.push(instruction);
        };

        Ok(Program { instructions: instructions })
    }
}
