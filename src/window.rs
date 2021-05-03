use clay_core::buffer::Image;
use sdl2::{
    self,
    event::Event,
    keyboard::{Keycode, Scancode},
    mouse::{MouseState, RelativeMouseState},
    pixels::PixelFormatEnum,
    render::{TextureAccess, WindowCanvas},
    EventPump, Sdl,
};
use std::time::{Duration, Instant};

use clay_utils::save_screenshot;

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
use rent::RentTexture;

pub struct Window {
    context: Sdl,
    size: (usize, usize),
    canvas: WindowCanvas,

    texture: Option<RentTexture>,
    event_pump: Option<EventPump>,
    state: WindowState,
}

pub struct WindowState {
    pub lock: bool,
    pub capture: bool,
    pub drop_mouse: bool,
    // time measurement
    instant: Instant,
    pub previous: Duration,
    pub current: Duration,
    screenshot: Option<bool>,
}

pub trait EventHandler {
    fn handle_keys(&mut self, state: &WindowState, event: &Event) -> clay_core::Result<()>;
    fn handle_mouse(
        &mut self,
        state: &WindowState,
        ms: &MouseState,
        rms: &RelativeMouseState,
    ) -> clay_core::Result<()>;
}

struct DummyHandler();
impl EventHandler for DummyHandler {
    fn handle_keys(&mut self, _state: &WindowState, _event: &Event) -> clay_core::Result<()> {
        Ok(())
    }
    fn handle_mouse(
        &mut self,
        _state: &WindowState,
        _ms: &MouseState,
        _rms: &RelativeMouseState,
    ) -> clay_core::Result<()> {
        Ok(())
    }
}

impl WindowState {
    fn new() -> Self {
        let instant = Instant::now();
        let time = instant.elapsed();
        Self {
            lock: false,
            capture: true,
            drop_mouse: true,
            instant,
            previous: time,
            current: time,
            screenshot: None,
        }
    }

    fn step_frame(&mut self) {
        self.previous = self.current;
        self.current = self.instant.elapsed();
    }

    pub fn frame_duration(&self) -> Duration {
        self.current - self.previous
    }
}

impl Window {
    pub fn new(size: (usize, usize)) -> clay_core::Result<Self> {
        let context = sdl2::init()?;
        let video = context.video()?;

        let window = video
            .window("Clay", size.0 as u32, size.1 as u32)
            .position_centered() /*.resizable()*/
            .build()
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
                )
                .map_err(|e| e.to_string())
            },
        )?);

        let event_pump = Some(context.event_pump()?);

        let mut self_ = Self {
            context,
            size,
            canvas,
            texture,
            event_pump,
            state: WindowState::new(),
        };

        self_.set_capture_mode(false);
        self_.unlock();

        Ok(self_)
    }

    pub fn set_capture_mode(&mut self, state: bool) {
        self.state.capture = state;
        self.context.mouse().set_relative_mouse_mode(state);
    }

    pub fn lock(&mut self) {
        self.state.lock = true;
        self.context.mouse().set_relative_mouse_mode(false);
        self.canvas.window_mut().set_title("Clay [LOCKED]").unwrap();
    }

    pub fn unlock(&mut self) {
        self.state.lock = false;
        self.context
            .mouse()
            .set_relative_mouse_mode(self.state.capture);
        self.canvas.window_mut().set_title("Clay").unwrap();
    }

    pub fn locked(&self) -> bool {
        self.state.lock
    }

    fn poll_inner(
        &mut self,
        handler: &mut dyn EventHandler,
        event_pump: &mut EventPump,
    ) -> clay_core::Result<bool> {
        'event_loop: loop {
            let event = match event_pump.poll_event() {
                Some(evt) => evt,
                None => break 'event_loop,
            };
            let kbs = event_pump.keyboard_state();
            let shift = kbs.is_scancode_pressed(Scancode::LShift)
                || kbs.is_scancode_pressed(Scancode::RShift);

            match event {
                Event::Quit { .. } => {
                    return Ok(true);
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Escape => {
                        return Ok(true);
                    }
                    Keycode::Tab => {
                        if !self.locked() {
                            self.set_capture_mode(!self.state.capture);
                            if self.state.capture {
                                self.state.drop_mouse = true;
                            }
                        }
                    }
                    Keycode::P => {
                        self.state.screenshot = Some(shift);
                    }
                    Keycode::L => {
                        if !shift {
                            self.lock();
                        } else {
                            self.unlock();
                            if self.state.capture {
                                self.state.drop_mouse = true;
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            }

            if !self.locked() {
                handler.handle_keys(&self.state, &event)?;
            }
        }

        if !self.locked() {
            if !self.state.drop_mouse {
                handler.handle_mouse(
                    &self.state,
                    &event_pump.mouse_state(),
                    &event_pump.relative_mouse_state(),
                )?;
            } else {
                event_pump.relative_mouse_state();
                self.state.drop_mouse = false;
            }
        }

        Ok(false)
    }

    pub fn poll_with_handler(&mut self, handler: &mut dyn EventHandler) -> clay_core::Result<bool> {
        let mut event_pump = self.event_pump.take().unwrap();
        let res = self.poll_inner(handler, &mut event_pump);
        assert!(self.event_pump.replace(event_pump).is_none());
        res
    }

    pub fn poll(&mut self) -> clay_core::Result<bool> {
        self.poll_with_handler(&mut DummyHandler())
    }

    pub fn state(&self) -> &WindowState {
        &self.state
    }

    pub fn step_frame(&mut self) -> Duration {
        self.state.step_frame();
        self.state.frame_duration()
    }

    pub fn size(&self) -> (usize, usize) {
        self.size
    }

    pub fn draw(&mut self, img: &Image) -> clay_core::Result<()> {
        let mut texture = self.texture.take().unwrap();

        if let Some(ll) = self.state.screenshot {
            println!("saving screenshot ...");
            match save_screenshot(img, ll) {
                Ok(f) => println!("... saved to '{}'", f),
                Err(e) => eprintln!("error saving screenshot: {}", e),
            }
            self.state.screenshot = None;
        }

        let res = img
            .read()
            .and_then(|data| {
                texture
                    .rent_mut(|texture| texture.update(None, &data, 3 * img.dims().0))
                    .map_err(|e| clay_core::Error::from(e.to_string()))
            })
            .and_then(|()| {
                //self.canvas.clear();
                texture
                    .rent(|texture| {
                        self.canvas
                            .copy(texture, None, None)
                            .map_err(|e| clay_core::Error::from(e))
                    })
                    .map(|()| self.canvas.present())
            });

        assert!(self.texture.replace(texture).is_none());

        res
    }
}
