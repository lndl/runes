use std::fs::File;
use std::io::Result;
use std::io::Read;
use std::fmt;

use mos6502::memory_map::memory_map::MemMappeable;

#[derive(Debug)]
enum RomVersion {
    INes,
    Nes2,
    Unknown
}

#[derive(Debug)]
enum TVSystem {
    NTSC,
    PAL,
    Unknown
}

struct PGRROM {
    lsb_banks: u8,
    msb_banks: u8,
    data: Vec<u8>
}

impl fmt::Debug for PGRROM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"
    PGR {{
        LSB Banks: {},
        MSB Banks: {},
        Data: <first 16 bytes: {:?}..., count: {} bytes>
    }}
        "#, self.lsb_banks, self.msb_banks,
        &self.data[0..16], self.data.len())
    }
}

pub struct CHRROM {
    lsb_banks: u8,
    msb_banks: u8,
    data: Vec<u8>
}

impl fmt::Debug for CHRROM {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"
    CHR {{
        LSB Banks: {},
        MSB Banks: {},
        Data: <first 16 bytes: {:?}..., count: {} bytes>
    }}
        "#, self.lsb_banks, self.msb_banks,
        &self.data[0..16], self.data.len())
    }
}

pub struct Rom {
    format: RomVersion,
    trainer: Option<Vec<u8>>,
    pgrrom: PGRROM,
    chrrom: CHRROM,
    mapper: u8,
    flag6: u8,
    flag7: u8,
    pgrram_banks: u8,
    tv_system: TVSystem,
}

impl fmt::Debug for Rom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, r#"
ROM {{
    NES format: {:?},
    trainer: {:?},
    PGR-ROM: {:?},
    CHR-ROM: {:?},
    mapper: {},
    flag6: {:08b},
    flag7: {:08b},
    PGR-RAM banks: {:?},
    TV System: {:?}
}}
        "#, self.format, self.trainer, self.pgrrom, self.chrrom, self.mapper,
        self.flag6, self.flag7, self.pgrram_banks, self.tv_system)
    }
}

impl Rom {
    pub fn chrrom_info(&self) -> &Vec<u8> {
        &self.chrrom.data
    }

    pub fn pgrrom_info(&self) -> &Vec<u8> {
        &self.pgrrom.data
    }

    pub fn from_file(filename: String) -> Result<Rom> {
        let mut file = File::open(&filename)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let new_rom = Rom {
            format: Self::guess_version(&data),
            trainer: Self::trainer(&data),
            pgrrom: PGRROM {
                lsb_banks: Self::pgrrom_banks(&data),
                msb_banks: data[9] & 0x0F,
                data: Self::pgrrom_data(&data)
            },
            chrrom: CHRROM {
                lsb_banks: Self::chrrom_banks(&data),
                msb_banks: data[9] & 0xF0,
                data: Self::chrrom_data(&data)
            },
            mapper: Self::parse_mapper(&data),
            flag6: data[6],
            flag7: data[7],
            pgrram_banks: data[8],
            tv_system: Self::tv_system(&data),
        };

        return Ok(new_rom);
    }

    fn parse_mapper(data: &Vec<u8>) -> u8 {
        (data[7] & 0xf0) | (data[6] >> 4)
    }

    fn guess_version(data: &Vec<u8>) -> RomVersion {
        let ines = &data[0..3] == "NES".as_bytes() && data[3] == 0x1A;
        let nes2_ext = (data[7] & 0x0C) == 0x08;

        if ines {
            if nes2_ext {
                RomVersion::Nes2
            } else {
                RomVersion::INes
            }
        } else {
            RomVersion::Unknown
        }
    }

    fn has_trainer(data: &Vec<u8>) -> bool {
        (data[6] & 0x04) != 0
    }

    fn pgrrom_banks(data: &Vec<u8>) -> u8 {
        data[4]
    }

    fn pgrrom_size(data: &Vec<u8>) -> usize {
        Self::pgrrom_banks(data) as usize * 16384
    }

    fn chrrom_banks(data: &Vec<u8>) -> u8 {
        data[5]
    }

    fn chrrom_size(data: &Vec<u8>) -> usize {
        Self::chrrom_banks(data) as usize * 8192
    }

    fn trainer(data: &Vec<u8>) -> Option<Vec<u8>> {
        if Self::has_trainer(data) {
            Some(data[16..512].to_vec())
        } else {
            None
        }
    }

    fn tv_system(data: &Vec<u8>) -> TVSystem {
        match data[9] & 0x01 {
            0 => TVSystem::NTSC,
            1 => TVSystem::PAL,
            _ => TVSystem::Unknown
        }
    }

    fn pgrrom_data(data: &Vec<u8>) -> Vec<u8> {
        let mut from = 16;
        let mut to = from + Self::pgrrom_size(data);
        if Self::has_trainer(&data) {
            from += 512;
            to += 512;
        }
        data[from..to].to_vec()
    }

    fn chrrom_data(data: &Vec<u8>) -> Vec<u8> {
        let mut from = 16 + Self::pgrrom_size(data);
        let mut to = from + Self::chrrom_size(data);
        if Self::has_trainer(&data) {
            from += 512;
            to += 512;
        }
        data[from..to].to_vec()
    }

    pub fn program(&self) -> &[u8] {
        &self.pgrrom.data
    }

    pub fn cpu_address_range(&self) -> std::ops::Range<usize> {
        if self.mapper == 0 {
            0x6000..0x10000
        } else {
            panic!("ERROR: Only supporting ROMs with mapper 0! This ROM is mapper {}!", self.mapper);
        }
    }

}

// TODO: Hardcoded Mapper 0!
impl MemMappeable for Rom {
    fn read(&self, address: usize) -> u8 {
        if address < 0x2000 {
            panic!("I've got no PGRRAM (TODO)!")
        }
        else {
            let translated = address - 0x2000;
            if address >= 0x2000 && address < 0x4000 {
                self.program()[translated]
            } else {
                let translated = translated - 0x4000;
                self.program()[translated]
            }
        }
    }

    fn slice(&self, from: usize) -> &[u8] {
        if from < 0x2000 {
            panic!("I've got no PGRRAM (TODO)!")
        }
        else {
            let translated = from - 0x2000;
            if from >= 0x2000 && from < 0x4000 {
                &self.program()[translated..self.program().len() - 1]
            } else {
                let translated = translated - 0x4000;
                &self.program()[translated..self.program().len() - 1]
            }
        }
    }

    fn write(&mut self, _address: usize, value: u8) -> u8 {
        // Not going to write in Mapper 0!
        value
    }
}
