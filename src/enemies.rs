use rand::Rng;
use crate::framebuffer::{Framebuffer, Rgb};
use crate::sprites;

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyType {
    Scout,
    Fighter,
    Bomber,
    Boss,
}

pub struct Enemy {
    pub x: f64,
    pub y: f64,
    pub vy: f64,
    pub enemy_type: EnemyType,
    pub hp: i32,
    pub anim_frame: usize,
    pub anim_timer: u32,
    pub alive: bool,
    pub shoot_timer: u32,
    pub phase: f64,
}

impl Enemy {
    pub fn new(x: f64, y: f64, enemy_type: EnemyType) -> Self {
        let (hp, vy) = match enemy_type {
            EnemyType::Scout => (1, 1.0),
            EnemyType::Fighter => (2, 0.5),
            EnemyType::Bomber => (5, 0.25),
            EnemyType::Boss => (40, 0.12),
        };
        let mut rng = rand::thread_rng();
        Self {
            x, y, vy, enemy_type, hp,
            anim_frame: 0, anim_timer: 0,
            alive: true,
            shoot_timer: rng.gen_range(20..60),
            phase: rng.gen_range(0.0..std::f64::consts::TAU),
        }
    }

    pub fn update(&mut self, screen_w: usize) {
        if !self.alive { return; }
        self.y += self.vy;

        match self.enemy_type {
            EnemyType::Scout => {
                self.phase += 0.1;
                self.x += self.phase.sin() * 1.2;
            }
            EnemyType::Fighter => {
                self.phase += 0.05;
                self.x += self.phase.sin() * 0.7;
            }
            EnemyType::Bomber => {
                self.phase += 0.03;
                self.x += self.phase.sin() * 0.4;
            }
            EnemyType::Boss => {
                self.phase += 0.015;
                self.x = (screen_w as f64 / 2.0) - (self.width() as f64 / 2.0)
                    + self.phase.sin() * (screen_w as f64 * 0.25);
                if self.y > 4.0 { self.vy = 0.0; self.y = 4.0; }
            }
        }

        if self.x < 0.0 { self.x = 0.0; }
        let max = screen_w as f64 - self.width() as f64;
        if self.x > max { self.x = max; }

        self.anim_timer += 1;
        if self.anim_timer >= 6 {
            self.anim_timer = 0;
            self.anim_frame = (self.anim_frame + 1) % 2;
        }

        if self.shoot_timer > 0 { self.shoot_timer -= 1; }
    }

    pub fn can_shoot(&self) -> bool {
        matches!(self.enemy_type, EnemyType::Fighter | EnemyType::Boss)
            && self.shoot_timer == 0 && self.alive
    }

    pub fn reset_shoot_timer(&mut self) {
        self.shoot_timer = match self.enemy_type {
            EnemyType::Fighter => 50,
            EnemyType::Boss => 18,
            _ => 999,
        };
    }

    pub fn width(&self) -> usize {
        match self.enemy_type {
            EnemyType::Scout => 7,
            EnemyType::Fighter => 9,
            EnemyType::Bomber => 11,
            EnemyType::Boss => 21,
        }
    }

    pub fn height(&self) -> usize {
        match self.enemy_type {
            EnemyType::Scout => 7,
            EnemyType::Fighter => 8,
            EnemyType::Bomber => 9,
            EnemyType::Boss => 12,
        }
    }

    pub fn hitbox(&self) -> (f64, f64, f64, f64) {
        match self.enemy_type {
            EnemyType::Scout => (self.x + 1.0, self.y + 1.0, 5.0, 5.0),
            EnemyType::Fighter => (self.x + 1.0, self.y + 1.0, 7.0, 6.0),
            EnemyType::Bomber => (self.x + 1.0, self.y + 1.0, 9.0, 7.0),
            EnemyType::Boss => (self.x + 3.0, self.y + 2.0, 15.0, 8.0),
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer) {
        if !self.alive { return; }
        let (sprite, palette): (&[&[u8]], &[Rgb]) = match self.enemy_type {
            EnemyType::Scout => {
                if self.anim_frame == 0 {
                    (&sprites::SCOUT_SPRITE, &sprites::SCOUT_PALETTE)
                } else {
                    (&sprites::SCOUT_SPRITE2, &sprites::SCOUT_PALETTE)
                }
            }
            EnemyType::Fighter => {
                if self.anim_frame == 0 {
                    (&sprites::FIGHTER_SPRITE, &sprites::FIGHTER_PALETTE)
                } else {
                    (&sprites::FIGHTER_SPRITE2, &sprites::FIGHTER_PALETTE)
                }
            }
            EnemyType::Bomber => {
                if self.anim_frame == 0 {
                    (&sprites::BOMBER_SPRITE, &sprites::BOMBER_PALETTE)
                } else {
                    (&sprites::BOMBER_SPRITE2, &sprites::BOMBER_PALETTE)
                }
            }
            EnemyType::Boss => {
                if self.anim_frame == 0 {
                    (&sprites::BOSS_SPRITE, &sprites::BOSS_PALETTE)
                } else {
                    (&sprites::BOSS_SPRITE2, &sprites::BOSS_PALETTE)
                }
            }
        };
        fb.draw_sprite(self.x.round() as i32, self.y.round() as i32, sprite, palette);
    }

    pub fn score_value(&self) -> u64 {
        match self.enemy_type {
            EnemyType::Scout => 100,
            EnemyType::Fighter => 250,
            EnemyType::Bomber => 500,
            EnemyType::Boss => 5000,
        }
    }
}
