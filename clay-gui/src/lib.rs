mod motion;

use std::{
    time::{Instant, Duration},
    thread,
};
use sdl2::{
    self,
    Sdl, VideoSubsystem,
    render::{WindowCanvas, TextureAccess},
    pixels::PixelFormatEnum,
    event::Event,
    keyboard::Keycode,
};
use vecmat::vec::*;
use clay_core::{Context, Screen};
use motion::Motion;

#[allow(dead_code)]
pub struct Window {
    context: Sdl,
    video: VideoSubsystem,
    size: (usize, usize),
    canvas: WindowCanvas,
}

impl Window {
    pub fn new(size: (usize, usize)) -> clay_core::Result<Self> {
        let context = sdl2::init()?;
        let video = context.video()?;
     
        let window = video.window("Clay", size.0 as u32, size.1 as u32)
        .position_centered()/*.resizable()*/.build()
        .map_err(|e| e.to_string())?;
     
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        Ok(Self { context, video, size, canvas })
    }

    pub fn start<F>(&mut self, context: &Context, mut render: F) -> clay_core::Result<()>
    where F: FnMut(&mut Screen, Vec3<f64>) -> clay_core::Result<()> {
        let mut screen = Screen::new(context, self.size).map_err(|e| e.to_string())?;

        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator.create_texture(
            PixelFormatEnum::RGBA32,
            TextureAccess::Streaming,
            self.size.0 as u32,
            self.size.1 as u32,
        )
        .map_err(|e| e.to_string())?;

        let mut motion = Motion::new();

        let instant = Instant::now();
        let mut now = instant.elapsed();

        let mut event_pump = self.context.event_pump()?;
        'main: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,
                    other => {
                        motion.handle(&other);
                    },
                }
            }

            render(&mut screen, motion.pos)?;
            let data = screen.read()?;

            texture.update(None, &data, 4*(screen.dims().0 as usize))
            .map_err(|e| e.to_string())?;

            //self.canvas.clear();
            self.canvas.copy(&texture, None, None)?;
            self.canvas.present();

            thread::sleep(Duration::from_millis(20));
            
            let new_now = instant.elapsed();
            let dt = new_now - now;
            motion.step(dt);
            now = new_now;
        }

        Ok(())
    }
} 
