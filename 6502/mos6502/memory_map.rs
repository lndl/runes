pub struct MemoryMap {
    mm: [u8; 65535]
}

impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap {

            mm: [255; 65535]
        }
    }

    pub fn is_overflowing(&self, index: u16) -> bool {
        index as usize >= self.mm.len()
    }

    pub fn copy<S>(&mut self, from: u16, stream: S) where S : IntoIterator<Item=u8> {
        let mut index = from - 1;
        for byte in stream.into_iter() {
            self.mm[index as usize] = byte;
            index += 1;
        }
    }

    pub fn write(&mut self, address: u16, value: u8) -> u8 {
        self.mm[address as usize] = value;
        value
    }

    pub fn read(&self, address: u16) -> u8 {
        self.mm[address as usize]
    }

    pub fn from(&self, from_address: u16) -> &[u8] {
        &self.mm[(from_address - 1) as usize..self.mm.len()]
    }
}