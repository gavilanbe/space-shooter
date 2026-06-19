use rand::Rng;
use crate::framebuffer::{Framebuffer, Rgb};

pub struct Star {
    pub x: f64,
    pub y: f64,
    pub speed: f64,
    pub color: Rgb,
}

pub struct Starfield {
    pub stars: Vec<Star>,
}

impl Starfield {
    pub fn new(w: usize, h: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut stars = Vec::new();

        // Far layer: dim, slow
        for _ in 0..40 {
            let b = rng.gen_range(30..60) as u8;
            stars.push(Star {
                x: rng.gen_range(0.0..w as f64),
                y: rng.gen_range(0.0..h as f64),
                speed: rng.gen_range(0.1..0.25),
                color: Rgb::new(b, b, (b as u16 + 20).min(255) as u8),
            });
        }
        // Mid layer
        for _ in 0..25 {
            let b = rng.gen_range(80..140) as u8;
            stars.push(Star {
                x: rng.gen_range(0.0..w as f64),
                y: rng.gen_range(0.0..h as f64),
                speed: rng.gen_range(0.3..0.6),
                color: Rgb::new(b, b, (b as u16 + 40).min(255) as u8),
            });
        }
        // Near layer: bright, fast
        for _ in 0..12 {
            let b = rng.gen_range(180..255) as u8;
            stars.push(Star {
                x: rng.gen_range(0.0..w as f64),
                y: rng.gen_range(0.0..h as f64),
                speed: rng.gen_range(0.7..1.2),
                color: Rgb::new(b, b, 255),
            });
        }

        Self { stars }
    }

    pub fn update(&mut self, h: usize, w: usize) {
        let mut rng = rand::thread_rng();
        for star in &mut self.stars {
            star.y += star.speed;
            if star.y >= h as f64 {
                star.y = 0.0;
                star.x = rng.gen_range(0.0..w as f64);
            }
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer) {
        for star in &self.stars {
            fb.set(star.x.round() as i32, star.y.round() as i32, star.color);
        }
    }
}
