use std::thread;

use sdl2::{
    self,
    Sdl, VideoSubsystem,
    render::{WindowCanvas, TextureAccess},
    pixels::PixelFormatEnum,
    event::Event,
    keyboard::Keycode,
};

use std::time::Duration;

use clay_core::{Worker};

#[allow(dead_code)]
pub struct Window {
    context: Sdl,
    video: VideoSubsystem,
    size: (usize, usize),
    canvas: WindowCanvas,
}

impl Window {
    pub fn new(size: (usize, usize)) -> Result<Self, String> {
        let context = sdl2::init()?;
        let video = context.video()?;
     
        let window = video.window("Clay", size.0 as u32, size.1 as u32)
        .position_centered()/*.resizable()*/.build()
        .map_err(|e| e.to_string())?;
     
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        Ok(Self { context, video, size, canvas })
    }

    pub fn start(&mut self, worker: &mut Worker) -> Result<(), String> {
        let mut screen = worker.create_screen(self.size).map_err(|e| e.to_string())?;

        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator.create_texture(
            PixelFormatEnum::RGBA32,
            TextureAccess::Streaming,
            self.size.0 as u32,
            self.size.1 as u32,
        )
        .map_err(|e| e.to_string())?;

        let mut event_pump = self.context.event_pump()?;
        'main: loop {
            self.canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'main;
                    },
                    _ => {}
                }
            }

            worker.render(&mut screen).map_err(|e| e.to_string())?;
            let data = screen.read().map_err(|e| e.to_string())?;
            texture.update(None, &data, 4*(screen.size().0 as usize)).map_err(|e| e.to_string())?;
            self.canvas.copy(&texture, None, None)?;

            self.canvas.present();
            thread::sleep(Duration::from_millis(20));
        }

        Ok(())
    }
} 

