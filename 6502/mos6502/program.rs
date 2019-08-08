use mos6502;
use mos6502::instruction::Instruction;

pub struct Program {
    bytestream: Vec<u8>
}

impl Program {
    pub fn new(bytestream: Vec<u8>) -> Self {
        Program { bytestream: bytestream }
    }

    pub fn size(&self) -> u32 {
        self.bytestream.len() as u32
    }

    pub fn fetch(&self, from: u32) -> Box<Instruction> {
        mos6502::instruction::try_build(&self.bytestream[from as usize..self.size() as usize])
    }

    pub fn compile(&self) -> Vec<Box<Instruction>> {
        let mut instructions = Vec::<Box<Instruction>>::new();
        let mut i = 0;
        let program_size = self.size();

        while i < program_size {
            // Try to build the instruction
            let instruction = mos6502::instruction::try_build(&self.bytestream[i as usize..program_size as usize]);
            // Advance in processing
            i = i + instruction.bytesize() as u32;
            // Push the instruction
            instructions.push(instruction);
        };

        instructions
    }
}
