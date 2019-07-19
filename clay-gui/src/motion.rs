use std::{
    time::Duration,
};
use sdl2::{
    event::Event,
    keyboard::Keycode,
};
use vecmat::{
    vec::*,
    //mat::*,
};


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

fn key_dir(key: Keycode) -> Vec3<f64> {
    match key {
        Keycode::W | Keycode::Up => Vec3::from(0.0, 0.0, -1.0),
        Keycode::A | Keycode::Left => Vec3::from(-1.0, 0.0, 0.0),
        Keycode::S | Keycode::Down => Vec3::from(0.0, 0.0, 1.0),
        Keycode::D | Keycode::Right => Vec3::from(1.0, 0.0, 0.0),
        Keycode::Space => Vec3::from(0.0, 1.0, 0.0),
        Keycode::LShift => Vec3::from(0.0, -1.0, 0.0),
        _ => Vec3::zero(),
    }
}

pub struct Motion {
    mask: usize,
    pub pos: Vec3<f64>,
    pub speed: f64,
}

impl Motion {
    pub fn new() -> Self {
        Self { mask: 0, pos: Vec3::zero(), speed: 1.0 }
    }

    pub fn handle(&mut self, event: &Event) {
        match event {
            Event::KeyDown { keycode: Some(key), .. } => if let Some(i) = key_idx(*key) {
                self.mask |= 1 << i;
            },
            Event::KeyUp { keycode: Some(key), .. } => if let Some(i) = key_idx(*key) {
                self.mask &= !(1 << i);
            }
            _ => (),
        }
    }

    pub fn step(&mut self, dt: Duration) {
        let mut dir = Vec3::zero();
        for (i, k) in KEYS.iter().enumerate() {
            if ((self.mask >> i) & 1) != 0 {
                dir += key_dir(*k);
            }
        }
        dir = dir.map(|x| clamp(x, -1.0, 1.0));
        if dir.sqrlen() > 1e6 {
            dir = dir.normalize();
        }
        self.pos += 1e-6*(dt.as_micros() as f64)*self.speed*dir;
    }
}
