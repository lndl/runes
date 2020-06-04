use sdl2::render::{Canvas, Texture, TextureAccess};
use sdl2::Sdl;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::TextureCreator;
use sdl2::video::Window;
use sdl2::video::WindowContext;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use Ppu;

/// Emulated screen width in pixels
const SCREEN_WIDTH: usize = 256;
/// Emulated screen height in pixels
const SCREEN_HEIGHT: usize = 240;
/// Screen texture size in bytes
const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 3;

//
// Screen scaling
//

#[derive(Copy, Clone)]
pub enum Scale {
    Scale1x,
    Scale2x,
    Scale3x,
}

impl Scale {
    fn factor(self) -> usize {
        match self {
            Scale::Scale1x => 1,
            Scale::Scale2x => 2,
            Scale::Scale3x => 3,
        }
    }
}

pub struct Gfx {
    pub renderer: Box<Canvas<Window>>,
    pub texture: Texture<'static>,
    pub scale: Scale,
    pub sdl: Sdl,
    _texture_creator: TextureCreator<WindowContext>,
}

impl Gfx {
    pub fn new(scale: Scale) -> Gfx {
        // FIXME: Handle SDL better

        let sdl = sdl2::init().unwrap();
        let video_subsystem = sdl.video().unwrap();

        let mut window_builder = video_subsystem.window(
            "RuNES",
            (SCREEN_WIDTH as usize * scale.factor()) as u32,
            (SCREEN_HEIGHT as usize * scale.factor()) as u32,
        );
        let window = window_builder.position_centered().build().unwrap();

        let renderer = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .unwrap();
        let texture_creator = renderer.texture_creator();
        let texture_creator_pointer = &texture_creator as *const TextureCreator<WindowContext>;
        let texture = unsafe { &*texture_creator_pointer }
        .create_texture(
            PixelFormatEnum::BGR24,
            TextureAccess::Streaming,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
        )
            .unwrap();

        Gfx {
            renderer: Box::new(renderer),
            texture,
            scale,
            sdl,
            _texture_creator: texture_creator,
        }
    }

    pub fn main_loop(&mut self, ppu: &mut Ppu) -> () {
        let mut event_pump = self.sdl.event_pump().unwrap();
        'running: loop {
            self.composite(&mut ppu.screen());
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running
                    },
                    Event::KeyDown { keycode: Some(keycode), .. } => {
                        println!("key pressed! -> {}", keycode)
                    },
                    _ => {}
                }
            }
        }
    }

    /// Copies the overlay onto the given screen and displays it to the SDL window.
    pub fn composite(&mut self, ppu_screen: &mut [u8; SCREEN_SIZE]) {
        self.blit(ppu_screen);
        self.renderer.clear();
        let _ = self.renderer.copy(&self.texture, None, None);
        self.renderer.present();
    }

    /// Updates the window texture with new screen data.
    fn blit(&mut self, ppu_screen: &[u8; SCREEN_SIZE]) {
        self.texture
            .update(None, ppu_screen, SCREEN_WIDTH * 3)
            .unwrap()
    }
}
