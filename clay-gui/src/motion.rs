use std::{
    time::Duration,
    f64::consts::PI,
};
use sdl2::{
    event::Event,
    mouse::RelativeMouseState,
    keyboard::Keycode,
};
use vecmat::{
    vec::*,
    mat::*,
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

fn key_dir(key: Keycode) -> (Vec3<f64>, Vec3<f64>) {
    match key {
        Keycode::W | Keycode::Up => (Vec3::from(0.0, 0.0, -1.0), Vec3::zero()),
        Keycode::A | Keycode::Left => (Vec3::from(-1.0, 0.0, 0.0), Vec3::zero()),
        Keycode::S | Keycode::Down => (Vec3::from(0.0, 0.0, 1.0), Vec3::zero()),
        Keycode::D | Keycode::Right => (Vec3::from(1.0, 0.0, 0.0), Vec3::zero()),
        Keycode::Space => (Vec3::zero(), Vec3::from(0.0, 0.0, 1.0)),
        Keycode::LShift => (Vec3::zero(), Vec3::from(0.0, 0.0, -1.0)),
        _ => (Vec3::zero(), Vec3::zero()),
    }
}

pub struct Motion {
    key_mask: usize,
    pub pos: Vec3<f64>,
    pub phi: f64,
    pub theta: f64,
    pub speed: f64,
    pub sens: f64,
}

impl Motion {
    pub fn new() -> Self {
        Self {
            key_mask: 0,
            pos: Vec3::zero(), phi: 0.0, theta: 0.5*PI,
            speed: 1.0, sens: 4e-3,
        }
    }

    pub fn handle_keys(&mut self, event: &Event) {
        match event {
            Event::KeyDown { keycode: Some(key), .. } => if let Some(i) = key_idx(*key) {
                self.key_mask |= 1 << i;
            },
            Event::KeyUp { keycode: Some(key), .. } => if let Some(i) = key_idx(*key) {
                self.key_mask &= !(1 << i);
            },
            _ => (),
        }
    }

    pub fn handle_mouse(&mut self, mouse: &RelativeMouseState) {
        println!("{}, {}", mouse.x(), mouse.y());

        self.phi += self.sens*(mouse.x() as f64);
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
    }

    pub fn map_theta(&self) -> Mat3<f64> {
        let mut mat = Mat3::one();
        mat[(1, 1)] = self.theta.cos();
        mat[(1, 2)] = -self.theta.sin();
        mat[(2, 1)] = self.theta.sin();
        mat[(2, 2)] = self.theta.cos();
        mat
    }
    pub fn map_phi(&self) -> Mat3<f64> {
        let mut mat = Mat3::one();
        mat[(0, 0)] = self.phi.cos();
        mat[(0, 1)] = self.phi.sin();
        mat[(1, 0)] = -self.phi.sin();
        mat[(1, 1)] = self.phi.cos();
        mat
    }

    pub fn map(&self) -> Mat3<f64> {
        self.map_theta().dot(self.map_phi())
    }

    pub fn step(&mut self, dt: Duration) {
        let (mut dir, mut idir) = (Vec3::zero(), Vec3::zero());
        for (i, k) in KEYS.iter().enumerate() {
            if ((self.key_mask >> i) & 1) != 0 {
                let (dv, di) = key_dir(*k);
                dir += dv;
                idir += di;
            }
        }
        dir = dir.map(|x| clamp(x, -1.0, 1.0));
        if dir.sqrlen() > 1e6 {
            dir = dir.normalize();
        }
        dir = dir.dot(self.map());
        self.pos += 1e-6*(dt.as_micros() as f64)*self.speed*(dir + idir);
    }
}
