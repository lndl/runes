use mos6502;
use mos6502::instruction::Instruction;
use mos6502::instruction::Executable;

type DecompiledProgram = Vec<u8>;
type CompiledProgram   = Vec<Instruction>;

pub struct Program {
    bytestream: DecompiledProgram
}

impl Program {
    pub fn new(bytestream: DecompiledProgram) -> Self {
        Program { bytestream: bytestream }
    }

    pub fn size(&self) -> u32 {
        self.bytestream.len() as u32
    }

    pub fn fetch(&self, from: u32) -> Instruction {
        mos6502::instruction::try_build(&self.bytestream[from as usize..self.size() as usize])
    }

    pub fn compile(&self) -> CompiledProgram {
        let mut instructions = Vec::<Instruction>::new();
        let mut i = 0;
        let program_size = self.size();

        while i < program_size {
            // Try to build the instruction
            let instruction = mos6502::instruction::try_build(&self.bytestream[i as usize..program_size as usize]);
            println!("{:?}", instruction);
            // Advance in processing
            i = i + instruction.bytesize() as u32;
            // Push the instruction
            instructions.push(instruction);
        };

        instructions
    }
}
