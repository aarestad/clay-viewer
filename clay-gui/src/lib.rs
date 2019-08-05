mod motion;

use std::{
    time::{Duration, Instant},
};
use sdl2::{
    self,
    Sdl, VideoSubsystem,
    render::{WindowCanvas, TextureAccess},
    pixels::PixelFormatEnum,
    event::Event,
    keyboard::Keycode,
};
use vecmat::{
    vec::*,
    mat::*,
};
use clay_core::{Context, Screen};
use motion::Motion;

#[allow(dead_code)]
pub struct Window {
    context: Sdl,
    video: VideoSubsystem,
    size: (usize, usize),
    canvas: WindowCanvas,
    capture: bool,
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

        let mut self_ = Self {
            context, video,
            size, canvas,
            capture: false,
        };

        self_.toggle_capture();

        Ok(self_)
    }

    fn toggle_capture(&mut self) {
        self.capture = !self.capture;
        self.context.mouse().set_relative_mouse_mode(self.capture);
    }

    pub fn start<F>(&mut self, context: &Context, mut render: F) -> clay_core::Result<()>
    where F: FnMut(&mut Screen, Vec3<f64>, Mat3<f64>) -> clay_core::Result<()> {
        let mut screen = Screen::new(context, self.size).map_err(|e| e.to_string())?;

        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator.create_texture(
            PixelFormatEnum::RGB24,
            TextureAccess::Streaming,
            self.size.0 as u32,
            self.size.1 as u32,
        )
        .map_err(|e| e.to_string())?;

        let mut motion = Motion::new();
        let mut drop_mouse = true;
        let instant = Instant::now();
        let mut prev = instant.elapsed();
        let mut fps = -1.0;
        let mut printed = instant.elapsed();

        let mut event_pump = self.context.event_pump()?;
        'main: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} => break 'main,
                    Event::KeyDown { keycode: Some(key), .. } => match key {
                        Keycode::Escape => break 'main,
                        Keycode::Tab => {
                            self.toggle_capture();
                            if self.capture {
                                drop_mouse = true;
                            }
                        },
                        _ => (),
                    },
                    _ => (),
                }
                motion.handle_keys(&event);
            }
            let rms = event_pump.relative_mouse_state();
            if self.capture {
                if !drop_mouse {
                    motion.handle_mouse(&rms);
                } else {
                    drop_mouse = false;
                }
            } else if event_pump.mouse_state().left() {
                motion.handle_mouse(&rms);
            }

            if motion.updated || motion.key_mask != 0 {
                screen.clear()?;
                motion.updated = false;
            }
            render(&mut screen, motion.pos, motion.map())?;
            let mut n_passes = 1;
            while instant.elapsed() - prev < Duration::from_millis(20) {
                render(&mut screen, motion.pos, motion.map())?;
                n_passes += 1;
            }
            let data = screen.read()?;

            texture.update(None, &data, 3*(screen.dims().0 as usize))
            .map_err(|e| e.to_string())?;

            //self.canvas.clear();
            self.canvas.copy(&texture, None, None)?;
            self.canvas.present();

            //thread::sleep(Duration::from_millis(20));
            
            let now = instant.elapsed();

            let dt = now - prev;
            motion.step(dt);
            prev = now;

            let cfps = (n_passes as f64)*1e6/(dt.as_micros() as f64);
            if fps < 0.0 {
                fps = cfps;
            } else {
                fps = 0.95*fps + 0.05*cfps;
            }
            if (now - printed).as_secs() > 0 {
                println!("FPS: {:.2}", fps);
                printed = now;
            }
        }

        Ok(())
    }
} 
