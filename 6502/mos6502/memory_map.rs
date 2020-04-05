pub mod memory_map {
    use std::fmt;
    use std::ops::Range;
    use std::collections::HashMap;
    use std::cell::RefCell;

    use mos6502::instruction::Instruction;

    pub const STKBASE : u16 = 0x1ff;
    pub const INTVECADR : u16 = 0xfffe;

    pub struct MemoryMap<'a> {
        mappers: HashMap<Range<u16>, RefCell<&'a mut [u8]>>,
    }

    impl<'a> MemoryMap<'a> {
        pub fn new() -> MemoryMap<'a> {
            MemoryMap {
                mappers: HashMap::new()
            }
        }

        pub fn register_mapper(&mut self, mem_range: Range<u16>, mapper: &'a mut [u8]) {
            self.mappers.insert(mem_range, RefCell::new(mapper));
        }

        pub fn push_to_stack(&mut self, sp: u8, value: u8) -> u8 {
            self.write(STKBASE - (0xFF - sp) as u16, value);
            value
        }

        pub fn peek_from_stack(&self, sp: u8) -> u8 {
            self.read(STKBASE - (0xFF - sp) as u16)
        }

        pub fn read(&self, address: u16) -> u8 {
            self.mapper_for(address).borrow()[self.address_inside_mapper(address) as usize]
        }

        pub fn write(&mut self, address: u16, value: u8) -> u8 {
            self.mapper_for(address).borrow_mut()[self.address_inside_mapper(address) as usize] = value;
            value
        }

        pub fn fetch_instruction(&self, address: u16) -> Option<Instruction> {
            let program_sector = self.mapper_for(address).borrow();
            let chunk = &program_sector[self.address_inside_mapper(address) as usize..program_sector.len()];
            Instruction::build(chunk)
        }

        pub fn mapper_for(&self, address: u16) -> &RefCell<&'a mut [u8]> {
            for (mem_range, mapper) in &self.mappers {
                if mem_range.contains(&address) {
                    return mapper
                }
            }
            panic!("Invalid access from 0x{:04x}", address)
        }

        fn address_inside_mapper(&self, address: u16) -> u16 {
            for (mem_range, _) in &self.mappers {
                if mem_range.contains(&address) {
                    return address - mem_range.start
                }
            }
            panic!("Invalid translation from requested address 0x{:04x}", address)
        }

    }

    impl<'a> fmt::Debug for MemoryMap<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Memory Dump:\n")?;
            write!(f, "================================================================================\n")?;
            let mut mem_ranges : Vec<_> = self.mappers.keys().clone().into_iter().collect();
            mem_ranges.sort_by(|r1, r2| r1.start.cmp(&r2.start));
            for mem_range in mem_ranges  {
                let mem_range : Vec<_> = mem_range.clone().collect();
                for mem_chunk in mem_range.chunks(16) {
                    let line : Vec<std::string::String> = mem_chunk.into_iter()
                        .map(|x| format!("{:02x}", self.read(*x))).collect();
                    write!(f, "{:04x}: {}\n", mem_chunk[0], line.join(" "))?;
                }
                write!(f, "<<<<<<<<<<<<<< SKIP >>>>>>>>>>>>>>\n")?;
            }
            write!(f, "================================================================================\n")
        }
    }
}