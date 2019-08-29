mod motion;

#[macro_use]
extern crate rental; 

use std::{
    time::{Duration, Instant},
};
use sdl2::{
    self,
    Sdl, EventPump,
    render::{WindowCanvas, TextureAccess},
    pixels::PixelFormatEnum,
    event::Event,
    keyboard::Keycode,
};
use nalgebra::{Vector3, Rotation3};
use clay_core::buffer::Image;

use motion::Motion;


rental! { mod rent {
    use sdl2::{
        video::{WindowContext},
        render::{TextureCreator, Texture},
    };

    #[rental_mut]
    pub struct RentTexture {
        creator: Box<TextureCreator<WindowContext>>,
        texture: Texture<'creator>,
    }
}}
use rent::{RentTexture};


pub struct Window {
    context: Sdl,
    size: (usize, usize),
    canvas: WindowCanvas,

    texture: Option<RentTexture>,
    event_pump: Option<EventPump>,
    motion: Motion,
    state: State,
}

struct State {
    capture: bool,
    drop_mouse: bool,
    // time measurement
    instant: Instant,
    prev: Duration,
}

impl Window {
    pub fn new(size: (usize, usize)) -> clay_core::Result<Self> {
        let context = sdl2::init()?;
        let video = context.video()?;
     
        let window = video.window("Clay", size.0 as u32, size.1 as u32)
        .position_centered()/*.resizable()*/.build()
        .map_err(|e| e.to_string())?;
     
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        context.mouse().set_relative_mouse_mode(true);

        let texture_creator = canvas.texture_creator();
        let texture = Some(RentTexture::try_new_or_drop(
            Box::new(texture_creator),
            |tc| {
                tc.create_texture(
                    PixelFormatEnum::RGB24,
                    TextureAccess::Streaming,
                    size.0 as u32,
                    size.1 as u32,
                ).map_err(|e| e.to_string())
            }
        )?);

        let event_pump = Some(context.event_pump()?);

        let mut self_ = Self {
            context, size, canvas,
            texture, event_pump,
            motion: Motion::new(),
            state: Self::new_state()?,
        };

        self_.toggle_capture();

        Ok(self_)
    }

    fn new_state() -> clay_core::Result<State> {
        let instant = Instant::now();
        Ok(State {
            capture: false,
            drop_mouse: true,
            instant,
            prev: instant.elapsed(),
        })
    }

    fn toggle_capture(&mut self) {
        self.state.capture = !self.state.capture;
        self.context.mouse().set_relative_mouse_mode(self.state.capture);
    }


    pub fn poll(&mut self) -> clay_core::Result<bool> {
        self.motion.updated = false;

        let now = self.state.instant.elapsed();
        self.motion.step(now - self.state.prev);
        self.state.prev = now;

        let mut quit = false;
        let mut event_pump = self.event_pump.take().unwrap();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => { quit = true; },
                Event::KeyDown { keycode: Some(key), .. } => match key {
                    Keycode::Escape => { quit = true; },
                    Keycode::Tab => {
                        self.toggle_capture();
                        if self.state.capture {
                            self.state.drop_mouse = true;
                        }
                    },
                    _ => (),
                },
                _ => (),
            }
            self.motion.handle_keys(&event);
        }

        let rms = event_pump.relative_mouse_state();
        if self.state.capture {
            if !self.state.drop_mouse {
                self.motion.handle_mouse(&rms);
            } else {
                self.state.drop_mouse = false;
            }
        } else if event_pump.mouse_state().left() {
            self.motion.handle_mouse(&rms);
        }

        assert!(self.event_pump.replace(event_pump).is_none());
        Ok(quit)
    }

    pub fn was_updated(&self) -> bool {
        self.motion.updated || self.motion.key_mask != 0
    }

    pub fn view_params(&self) -> (Vector3<f64>, Rotation3<f64>) {
        (self.motion.pos, self.motion.map())
    }

    pub fn size(&self) -> (usize, usize) {
        self.size
    }

    pub fn draw(&mut self, image: &Image) -> clay_core::Result<()> {
        let mut texture = self.texture.take().unwrap();

        let res = image.read()
        .and_then(|data| {
            texture.rent_mut(|texture| {
                texture.update(None, &data, 3*(image.dims().0 as usize))
            }).map_err(|e| clay_core::Error::from(e.to_string()))
        })
        .and_then(|()| {
            //self.canvas.clear();
            texture.rent(|texture| {
                self.canvas.copy(texture, None, None)
                .map_err(|e| clay_core::Error::from(e))
            })
            .map(|()| self.canvas.present())
        });

        assert!(self.texture.replace(texture).is_none());

        res
    }
} 
