use crate::framebuffer::{Framebuffer, Rgb};
use crate::sprites;

#[derive(Clone, Copy, PartialEq)]
pub enum BulletOwner {
    Player,
    Enemy,
}

pub struct Bullet {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub owner: BulletOwner,
    pub alive: bool,
    pub power: i32,
}

impl Bullet {
    pub fn player(x: f64, y: f64) -> Self {
        Self { x, y, vx: 0.0, vy: -2.0, owner: BulletOwner::Player, alive: true, power: 1 }
    }

    pub fn player_angled(x: f64, y: f64, vx: f64) -> Self {
        Self { x, y, vx, vy: -2.0, owner: BulletOwner::Player, alive: true, power: 1 }
    }

    pub fn enemy(x: f64, y: f64) -> Self {
        Self { x, y, vx: 0.0, vy: 1.2, owner: BulletOwner::Enemy, alive: true, power: 1 }
    }

    pub fn enemy_aimed(x: f64, y: f64, tx: f64, ty: f64) -> Self {
        let dx = tx - x;
        let dy = ty - y;
        let len = (dx * dx + dy * dy).sqrt().max(0.1);
        Self { x, y, vx: dx / len * 1.0, vy: dy / len * 1.0, owner: BulletOwner::Enemy, alive: true, power: 1 }
    }

    pub fn update(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
    }

    pub fn offscreen(&self, w: usize, h: usize) -> bool {
        self.y < -2.0 || self.y > h as f64 + 2.0 || self.x < -2.0 || self.x > w as f64 + 2.0
    }

    pub fn draw(&self, fb: &mut Framebuffer) {
        if !self.alive { return; }
        let x = self.x.round() as i32;
        let y = self.y.round() as i32;
        match self.owner {
            BulletOwner::Player => {
                // Bright cyan laser bolt (2 pixels tall)
                fb.set(x, y, Rgb::new(150, 230, 255));
                fb.set(x, y - 1, Rgb::new(80, 160, 255));
            }
            BulletOwner::Enemy => {
                // Red dot
                fb.set(x, y, Rgb::new(255, 80, 80));
                fb.set(x, y + 1, Rgb::new(200, 40, 40));
            }
        }
    }
}

// Explosion effect
pub struct Explosion {
    pub x: f64,
    pub y: f64,
    pub frame: usize,
    pub timer: u32,
    pub big: bool,
}

impl Explosion {
    pub fn new(x: f64, y: f64, big: bool) -> Self {
        Self { x, y, frame: 0, timer: 0, big }
    }

    pub fn update(&mut self) {
        self.timer += 1;
        let speed = if self.big { 3 } else { 2 };
        if self.timer >= speed {
            self.timer = 0;
            self.frame += 1;
        }
    }

    pub fn alive(&self) -> bool {
        self.frame < 5
    }

    pub fn draw(&self, fb: &mut Framebuffer) {
        let sprite: &[&[u8]] = match self.frame {
            0 => &sprites::EXPLOSION_1,
            1 => &sprites::EXPLOSION_2,
            2 => &sprites::EXPLOSION_3,
            3 => &sprites::EXPLOSION_4,
            _ => &sprites::EXPLOSION_5,
        };
        let sx = self.x.round() as i32 - (sprite[0].len() as i32 / 2);
        let sy = self.y.round() as i32 - (sprite.len() as i32 / 2);

        let scale = if self.big { 2 } else { 1 };
        for (row, line) in sprite.iter().enumerate() {
            for (col, &idx) in line.iter().enumerate() {
                if idx == 0 { continue; }
                if let Some(&color) = sprites::EXPLOSION_PALETTE.get(idx as usize) {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            fb.set(
                                sx + (col as i32) * scale + dx,
                                sy + (row as i32) * scale + dy,
                                color,
                            );
                        }
                    }
                }
            }
        }
    }
}

// Power-up
#[derive(Clone, Copy, PartialEq)]
pub enum PowerUpType {
    WeaponUp,
    Shield,
    ExtraLife,
}

pub struct PowerUp {
    pub x: f64,
    pub y: f64,
    pub kind: PowerUpType,
    pub alive: bool,
    pub timer: u32,
}

impl PowerUp {
    pub fn new(x: f64, y: f64, kind: PowerUpType) -> Self {
        Self { x, y, kind, alive: true, timer: 0 }
    }

    pub fn update(&mut self) {
        self.y += 0.4;
        self.timer += 1;
    }

    pub fn draw(&self, fb: &mut Framebuffer) {
        if !self.alive { return; }
        let x = self.x.round() as i32;
        let y = self.y.round() as i32;
        let pulse = ((self.timer as f32 * 0.2).sin() * 0.3 + 0.7).max(0.0);
        let color = match self.kind {
            PowerUpType::WeaponUp => Rgb::new((255.0 * pulse) as u8, (200.0 * pulse) as u8, 0),
            PowerUpType::Shield => Rgb::new(0, (200.0 * pulse) as u8, (255.0 * pulse) as u8),
            PowerUpType::ExtraLife => Rgb::new((255.0 * pulse) as u8, (80.0 * pulse) as u8, (80.0 * pulse) as u8),
        };
        // 3x3 diamond shape
        fb.set(x, y - 1, color);
        fb.set(x - 1, y, color);
        fb.set(x, y, Rgb::new(255, 255, 255));
        fb.set(x + 1, y, color);
        fb.set(x, y + 1, color);
    }
}
