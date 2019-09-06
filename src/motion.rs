use std::{
    time::Duration,
    f64::consts::PI,

};
use sdl2::{
    event::Event,
    mouse::{MouseState, RelativeMouseState},
    keyboard::Keycode,
};
use nalgebra::{Vector3, Rotation3};
use crate::{WindowState, EventHandler};


fn clamp(x: f64, a: f64, b: f64) -> f64 {
    if x < a {
        a
    } else if x > b {
        b
    } else {
        x
    }
}

const KEYS: [Keycode; 12] = [
    Keycode::W,
    Keycode::Up,
    Keycode::A,
    Keycode::Left,
    Keycode::S,
    Keycode::Down,
    Keycode::D,
    Keycode::Right,
    Keycode::Space,
    Keycode::LShift,
    Keycode::Q,
    Keycode::E,
];

fn key_idx(key: Keycode) -> Option<usize> {
    KEYS.iter().position(|k| *k == key)
}

fn key_dir(key: Keycode) -> (Vector3<f64>, Vector3<f64>) {
    match key {
        Keycode::W | Keycode::Up => (Vector3::new(0.0, 0.0, -1.0), Vector3::zeros()),
        Keycode::A | Keycode::Left => (Vector3::new(-1.0, 0.0, 0.0), Vector3::zeros()),
        Keycode::S | Keycode::Down => (Vector3::new(0.0, 0.0, 1.0), Vector3::zeros()),
        Keycode::D | Keycode::Right => (Vector3::new(1.0, 0.0, 0.0), Vector3::zeros()),
        Keycode::Space => (Vector3::zeros(), Vector3::new(0.0, 0.0, 1.0)),
        Keycode::LShift => (Vector3::zeros(), Vector3::new(0.0, 0.0, -1.0)),
        _ => (Vector3::zeros(), Vector3::zeros()),
    }
}

pub struct Motion {
    pub updated: bool,
    pub key_mask: usize,

    pub pos: Vector3<f64>,

    // Euler angles
    pub phi: f64,
    pub theta: f64,

    pub speed: f64,
    pub sens: f64,
}

impl Motion {
    pub fn new(pos: Vector3<f64>, ori: Rotation3<f64>) -> Self {
        let (theta, _, phi) = ori.euler_angles();
        Self {
            updated: false, key_mask: 0,
            pos, phi, theta,
            speed: 1.0, sens: 4e-3,
        }
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed;
    }
    pub fn set_sensitivity(&mut self, sens: f64) {
        self.sens = sens;
    }

    fn handle_keys(&mut self, event: &Event) {
        match event {
            Event::KeyDown { keycode: Some(key), .. } => if let Some(i) = key_idx(*key) {
                self.key_mask |= 1 << i;
                self.updated = true;
            },
            Event::KeyUp { keycode: Some(key), .. } => if let Some(i) = key_idx(*key) {
                self.key_mask &= !(1 << i);
                self.updated = true;
            },
            _ => (),
        }
    }

    fn handle_mouse(&mut self, mouse: &RelativeMouseState) {
        if mouse.x() != 0 || mouse.y() != 0 {
            self.phi -= self.sens*(mouse.x() as f64);
            let t = (self.phi/(2.0*PI)).floor() as i32;
            if t != 0 {
                self.phi -= 2.0*PI*(t as f64);
            }

            self.theta -= self.sens*(mouse.y() as f64);
            if self.theta < 0.0 {
                self.theta = 0.0;
            } else if self.theta > PI {
                self.theta = PI;
            }
            self.updated = true;
        }
    }

    pub fn pos(&self) -> Vector3<f64> {
        self.pos.clone()
    }
    pub fn ori(&self) -> Rotation3<f64> {
        Rotation3::from_euler_angles(self.theta, 0.0, self.phi)
    }

    pub fn was_updated(&self) -> bool {
        self.updated || self.key_mask != 0
    }

    pub fn step(&mut self, dt: Duration) {
        let (mut dir, mut idir) = (Vector3::zeros(), Vector3::zeros());
        for (i, k) in KEYS.iter().enumerate() {
            if ((self.key_mask >> i) & 1) != 0 {
                let (dv, di) = key_dir(*k);
                dir += dv;
                idir += di;
            }
        }
        dir = dir.map(|x| clamp(x, -1.0, 1.0));
        if dir.norm() > 1e-4 {
            dir = dir.normalize();
        }
        dir = self.ori()*dir;
        self.pos += 1e-6*(dt.as_micros() as f64)*self.speed*(dir + idir);

        self.updated = false;
    }
}

impl EventHandler for Motion {
    fn handle_keys(&mut self, _state: &WindowState, event: &Event) -> clay_core::Result<()> {
        self.handle_keys(event);
        Ok(())
    }
    fn handle_mouse(
        &mut self, state: &WindowState,
        ms: &MouseState, rms: &RelativeMouseState,
    ) -> clay_core::Result<()> {
        if state.capture {
            self.handle_mouse(&rms);
        } else if ms.left() {
            self.handle_mouse(&rms);
        }
        Ok(())
    }
}
