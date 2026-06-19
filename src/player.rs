use crate::framebuffer::Framebuffer;
use crate::sprites;

pub const SHIP_W: i32 = 9;
pub const SHIP_H: i32 = 11;

pub struct Player {
    pub x: f64,
    pub y: f64,
    pub lives: i32,
    pub score: u64,
    pub power_level: u8,
    pub invincible_timer: u32,
    pub thrust_frame: usize,
    pub thrust_timer: u32,
    pub moving_left: bool,
    pub moving_right: bool,
}

impl Player {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x, y,
            lives: 3,
            score: 0,
            power_level: 1,
            invincible_timer: 0,
            thrust_frame: 0,
            thrust_timer: 0,
            moving_left: false,
            moving_right: false,
        }
    }

    pub fn update(&mut self, pw: usize, ph: usize) {
        if self.x < 0.0 { self.x = 0.0; }
        if self.y < 2.0 { self.y = 2.0; }
        let max_x = pw as f64 - SHIP_W as f64;
        let max_y = ph as f64 - SHIP_H as f64;
        if self.x > max_x { self.x = max_x; }
        if self.y > max_y { self.y = max_y; }

        if self.invincible_timer > 0 {
            self.invincible_timer -= 1;
        }

        self.thrust_timer += 1;
        if self.thrust_timer >= 4 {
            self.thrust_timer = 0;
            self.thrust_frame = (self.thrust_frame + 1) % 2;
        }
    }

    pub fn is_invincible(&self) -> bool {
        self.invincible_timer > 0
    }

    pub fn hit(&mut self) {
        if self.is_invincible() { return; }
        self.lives -= 1;
        self.invincible_timer = 60;
        if self.power_level > 1 {
            self.power_level -= 1;
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer) {
        if self.is_invincible() && (self.invincible_timer % 4 >= 2) {
            return; // blink
        }

        let sprite = if self.moving_left {
            &sprites::PLAYER_SHIP_LEFT
        } else if self.moving_right {
            &sprites::PLAYER_SHIP_RIGHT
        } else if self.thrust_frame == 0 {
            &sprites::PLAYER_SHIP
        } else {
            &sprites::PLAYER_SHIP_THRUST2
        };

        let sx = self.x.round() as i32;
        let sy = self.y.round() as i32;
        for (row, line) in sprite.iter().enumerate() {
            for (col, &idx) in line.iter().enumerate() {
                if idx == 0 { continue; }
                if let Some(&color) = sprites::PLAYER_PALETTE.get(idx as usize) {
                    fb.set(sx + col as i32, sy + row as i32, color);
                }
            }
        }
    }

    pub fn hitbox(&self) -> (f64, f64, f64, f64) {
        (self.x + 2.0, self.y + 1.0, 5.0, 8.0)
    }
}
