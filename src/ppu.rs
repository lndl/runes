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

pub struct Ppu {
    screen: [u8; 184320]
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
            screen: [0; 184320]
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

