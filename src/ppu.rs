use mos6502::memory_map::memory_map::{Memorable};
use rom::Rom;

#[derive(Debug)]
enum SpriteColor {
    Color0,
    Color1,
    Color2,
    Color3
}

#[derive(Debug)]
pub struct Sprite {
    rows: Vec<Vec<SpriteColor>>
}

impl Sprite {
    pub fn from_chr_chunk(chunk: &[u8]) -> Self {
        let channel1 = &chunk[0..8];
        let channel2 = &chunk[8..16];

        let mut s = Sprite { rows: Default::default() };

        for (r, (a, b)) in channel1.iter().zip(channel2.iter()).enumerate() {
            s.rows.push(Vec::new());
            for i in 0..8 {
                let b1 = if (a & (128 >> i)) == 0 { 0 } else { 1 };
                let b2 = if (b & (128 >> i)) == 0 { 0 } else { 1 };

                let color = match (b1, b2) {
                    (0, 0) => SpriteColor::Color0,
                    (0, 1) => SpriteColor::Color1,
                    (1, 0) => SpriteColor::Color2,
                    (1, 1) => SpriteColor::Color3,
                    _      => panic!("cant happen!")
                };

                s.rows[r].push(color);
            }
        };

        s
    }
}

// RGB notation
pub struct Color(u8, u8, u8);

// Consts
pub const SCANLINE_LIMIT: u16 = 340;
pub const VBLANK_SCANLINE: i16 = 241;
pub const LAST_SCANLINE: u16 = 261;
pub const PRE_SCANLINE: i16 = -1;

pub enum PpuStepEval {
    Normal,
    NMI
}

/* PPU Structure */
pub struct Ppu {
    screen: [u8; 184320],

    control: u8,
    mask: u8,
    status: u8,
    oamaddress: u8,
    oamdata: u8,
    scroll: u16,
    address: u16,
    data: u8,

    is_scroll_low: bool,
    is_address_low: bool,

    scanline: i16,
    cycles: u16,
}

impl Memorable for Ppu {
    fn read(&self, address: usize) -> u8 {
        match address % 8 {
            0 => self.control,
            1 => self.mask,
            2 => self.status,
            3 => self.oamaddress,
            4 => self.oamdata,
            5 => (self.scroll & 0x00ff) as u8,
            6 => (self.address & 0x00ff) as u8,
            7 => self.data,
            _ => panic!("lalala")
        }
    }

    fn slice(&self, _from: usize) -> &[u8] {
        panic!("Slice operation for PPU has no sense!");
    }

    fn write(&mut self, address: usize, value: u8) -> u8 {
        match address % 8 {
            0 => self.control    = value,
            1 => self.mask       = value,
            2 => panic!("Can't write PPUSTATUS!"),
            3 => self.oamaddress = value,
            4 => self.oamdata    = value,
            5 => {
                if self.is_scroll_low {
                    self.scroll = value as u16;
                } else {
                    self.scroll = (self.scroll << 8) | value as u16;
                }
                self.is_scroll_low = !self.is_scroll_low;
            },
            6 => {
                if self.is_address_low {
                    self.address = value as u16;
                } else {
                    self.address = (self.address << 8) | value as u16;
                }
                self.is_address_low = !self.is_address_low;
            },
            7 => self.data       = value,
            _ => panic!("lalala")
        };
        value
    }
}

impl Ppu {
    fn std_palette(&self, color: &SpriteColor) -> Color {
        match color {
            SpriteColor::Color0 => Color(255,255,255),
            SpriteColor::Color1 => Color(0,255,0),
            SpriteColor::Color2 => Color(0,0,255),
            SpriteColor::Color3 => Color(255,0,0)
        }
    }

    pub fn new() -> Self {
        Ppu {
            screen: [0; 184320],

            control: 0,
            mask: 0,
            status: 0,
            oamaddress: 0,
            oamdata: 0,
            scroll: 0,
            address: 0,
            data: 0,

            is_scroll_low: true,
            is_address_low: true,

            scanline: 240,
            cycles: 0,
        }
    }

    pub fn step(&mut self) -> PpuStepEval {
        if self.cycles > SCANLINE_LIMIT {
            self.cycles = 0;
            self.scanline += 1;
        }

        if self.scanline == PRE_SCANLINE && self.cycles == 1 {
            self.set_vblank(false);
        }

        if self.scanline == VBLANK_SCANLINE && self.cycles == 1 {
            self.set_vblank(true);
            if self.control & 0x80 == 1 {
                return PpuStepEval::NMI
            }
	}

        if self.scanline == LAST_SCANLINE as i16 {
            self.scanline = -1;
        }

        self.cycles += 1;

        PpuStepEval::Normal
    }

    fn set_vblank(&mut self, on: bool) {
        if on {
            self.status |= 0x80;
        } else {
            self.status &= 0x7F;
        }
    }

    pub fn screen(&mut self) -> &mut [u8; 184320] {
        &mut self.screen
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, color: Color) {
        self.screen[(y * 256 + x) * 3 + 0] = color.0;
        self.screen[(y * 256 + x) * 3 + 1] = color.1;
        self.screen[(y * 256 + x) * 3 + 2] = color.2;
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &Sprite) {
        for (i, line) in sprite.rows.iter().enumerate() {
            for (j, color) in line.iter().enumerate() {
                self.draw_pixel(x + j, y + i, self.std_palette(color));
            }
        }
    }

    pub fn draw_spritesheet(&mut self, rom: &Rom) {
        let sprite_list: Vec<Sprite> = rom.chrrom_info().chunks(16).map( |chunk| {
            Sprite::from_chr_chunk(chunk)
        }).collect();

        let mut x = 0;
        let mut y = 0;
        for (i, sprite) in sprite_list.iter().enumerate() {
            if i > 900 {
                println!("WARNING!: Exceed max quantity of sprites. Total: {:?}", sprite_list.len());
                break;
            }
            self.draw_sprite(x, y, sprite);
            if (i + 1) % 32 != 0 {
                x += 8;
            } else {
                y += 8;
                x = 0;
            }
        }
    }
}

