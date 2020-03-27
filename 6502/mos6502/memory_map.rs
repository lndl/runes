pub mod MemoryMap {
    use std::fmt;
    use std::ops::Range;

    const MM_SIZE: usize = 65536;

    pub const STKRANGE : Range<usize> = 0x0100..0x0200;
    pub const INTVECADR : u16 = 0xfffe;

    pub struct MemoryMap {
        mm: [u8; MM_SIZE]
    }

    impl MemoryMap {
        pub fn new() -> MemoryMap {
            MemoryMap {
                mm: [0xEA; MM_SIZE] // Fill mem with NOPs
            }
        }

        pub fn is_overflowing(&self, index: usize) -> bool {
            index  >= self.mm.len()
        }

        pub fn copy<S>(&mut self, from: usize, stream: S) where S : IntoIterator<Item=u8> {
            let mut index = from;
            for byte in stream.into_iter() {
                if self.is_overflowing(index) {
                    println!("MemoryMap#copy has overflowed!");
                    break;
                }
                self.mm[index as usize] = byte;
                index += 1;
            }
        }

        pub fn register(&self, key: &str, range: Range<u16>) {

        }

        pub fn portion_for(&mut self, key: &str) -> &mut [u8] {
            &mut self.mm[STKRANGE]
        }

        pub fn write(&mut self, address: u16, value: u8) -> u8 {
            self.mm[address as usize] = value;
            value
        }

        pub fn read(&self, address: u16) -> u8 {
            self.mm[address as usize]
        }

        pub fn from(&self, from_address: u16) -> &[u8] {
            &self.mm[from_address as usize..self.mm.len()]
        }
    }

    impl fmt::Debug for MemoryMap {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Memory Dump:\n")?;
            write!(f, "---------------------------------------------------------\n");
            for line in 0..(MM_SIZE / 16) {
                let line_no = line * 16;
                let line_bytes : &Vec<std::string::String> = &self.mm[line_no..(line_no + 16)].to_vec()
                    .into_iter().map(|x| format!("{:02x}", x)).collect();


                write!(f, "{:04x}: {}\n", line_no, line_bytes.join(" "));
            }
            write!(f, "\n")?;
            write!(f, "---------------------------------------------------------\n")
        }
    }
}