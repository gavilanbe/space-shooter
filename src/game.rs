use rand::Rng;
use crossterm::{cursor, queue};
use crossterm::style::{SetForegroundColor, Color, ResetColor};
use std::io::{Write as IoWrite, stdout, BufWriter};

use crate::framebuffer::{Framebuffer, Rgb};
use crate::player::Player;
use crate::enemies::{Enemy, EnemyType};
use crate::bullets::{Bullet, BulletOwner, Explosion, PowerUp, PowerUpType};
use crate::starfield::Starfield;

pub enum GameState {
    Title,
    Playing,
    GameOver,
}

pub struct Game {
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub explosions: Vec<Explosion>,
    pub powerups: Vec<PowerUp>,
    pub starfield: Starfield,
    pub fb: Framebuffer,
    pub state: GameState,
    pub wave: u32,
    pub wave_timer: u32,
    pub enemies_spawned: u32,
    pub enemies_per_wave: u32,
    pub shoot_cooldown: u32,
    pub term_w: usize,
    pub term_h: usize,
}

impl Game {
    pub fn new(tw: usize, th: usize) -> Self {
        let fb = Framebuffer::new(tw, th);
        let pw = fb.width;
        let ph = fb.height;
        Self {
            player: Player::new((pw as f64 / 2.0) - 4.0, ph as f64 - 14.0),
            enemies: Vec::new(),
            bullets: Vec::new(),
            explosions: Vec::new(),
            powerups: Vec::new(),
            starfield: Starfield::new(pw, ph),
            fb,
            state: GameState::Title,
            wave: 1,
            wave_timer: 0,
            enemies_spawned: 0,
            enemies_per_wave: 5,
            shoot_cooldown: 0,
            term_w: tw,
            term_h: th,
        }
    }

    pub fn reset(&mut self) {
        let pw = self.fb.width;
        let ph = self.fb.height;
        self.player = Player::new((pw as f64 / 2.0) - 4.0, ph as f64 - 14.0);
        self.enemies.clear();
        self.bullets.clear();
        self.explosions.clear();
        self.powerups.clear();
        self.wave = 1;
        self.wave_timer = 0;
        self.enemies_spawned = 0;
        self.enemies_per_wave = 5;
        self.shoot_cooldown = 0;
        self.state = GameState::Playing;
    }

    pub fn resize(&mut self, tw: usize, th: usize) {
        self.term_w = tw;
        self.term_h = th;
        self.fb.resize(tw, th);
    }

    pub fn update(&mut self) {
        if !matches!(self.state, GameState::Playing) { return; }

        let pw = self.fb.width;
        let ph = self.fb.height;

        self.player.update(pw, ph);
        self.starfield.update(ph, pw);

        if self.shoot_cooldown > 0 { self.shoot_cooldown -= 1; }

        // Spawn
        self.wave_timer += 1;
        self.spawn_wave();

        // Update enemies
        for e in &mut self.enemies { e.update(pw); }

        // Enemy shooting
        let ptx = self.player.x + 4.0;
        let pty = self.player.y + 5.0;
        let mut new_b = Vec::new();
        for e in &mut self.enemies {
            if e.can_shoot() {
                e.reset_shoot_timer();
                let ex = e.x + e.width() as f64 / 2.0;
                let ey = e.y + e.height() as f64;
                match e.enemy_type {
                    EnemyType::Boss => {
                        new_b.push(Bullet::enemy_aimed(ex, ey, ptx, pty));
                        new_b.push(Bullet::enemy(ex - 5.0, ey));
                        new_b.push(Bullet::enemy(ex + 5.0, ey));
                    }
                    _ => { new_b.push(Bullet::enemy(ex, ey)); }
                }
            }
        }
        self.bullets.extend(new_b);

        for b in &mut self.bullets {
            b.update();
            if b.offscreen(pw, ph) { b.alive = false; }
        }
        for ex in &mut self.explosions { ex.update(); }
        for pu in &mut self.powerups {
            pu.update();
            if pu.y > ph as f64 { pu.alive = false; }
        }

        self.collisions();

        self.enemies.retain(|e| e.alive);
        self.bullets.retain(|b| b.alive);
        self.explosions.retain(|e| e.alive());
        self.powerups.retain(|p| p.alive);

        for e in &mut self.enemies {
            if e.y > ph as f64 + 10.0 { e.alive = false; }
        }

        if self.enemies_spawned >= self.enemies_per_wave && self.enemies.is_empty() {
            self.next_wave();
        }

        if self.player.lives <= 0 {
            self.state = GameState::GameOver;
        }
    }

    fn spawn_wave(&mut self) {
        if self.enemies_spawned >= self.enemies_per_wave { return; }
        let interval = match self.wave { 1..=3 => 30, 4..=6 => 25, _ => 18 };
        if self.wave_timer % interval != 0 { return; }

        let mut rng = rand::thread_rng();
        let pw = self.fb.width;
        let x = rng.gen_range(2.0..(pw as f64 - 22.0).max(4.0));

        let etype = if self.wave % 5 == 0 && self.enemies_spawned == 0 {
            EnemyType::Boss
        } else {
            let r: f64 = rng.gen();
            if self.wave <= 2 {
                if r < 0.7 { EnemyType::Scout } else { EnemyType::Fighter }
            } else if self.wave <= 5 {
                if r < 0.4 { EnemyType::Scout }
                else if r < 0.75 { EnemyType::Fighter }
                else { EnemyType::Bomber }
            } else {
                if r < 0.3 { EnemyType::Scout }
                else if r < 0.6 { EnemyType::Fighter }
                else { EnemyType::Bomber }
            }
        };

        self.enemies.push(Enemy::new(x, -12.0, etype));
        self.enemies_spawned += 1;
    }

    fn next_wave(&mut self) {
        self.wave += 1;
        self.wave_timer = 0;
        self.enemies_spawned = 0;
        self.enemies_per_wave = if self.wave % 5 == 0 { 1 } else { 5 + self.wave * 2 };
    }

    pub fn player_shoot(&mut self) {
        if self.shoot_cooldown > 0 { return; }
        self.shoot_cooldown = match self.player.power_level { 1 => 7, 2 => 5, _ => 4 };

        let px = self.player.x + 4.0;
        let py = self.player.y;

        match self.player.power_level {
            1 => { self.bullets.push(Bullet::player(px, py - 1.0)); }
            2 => {
                self.bullets.push(Bullet::player(px - 1.0, py - 1.0));
                self.bullets.push(Bullet::player(px + 1.0, py - 1.0));
            }
            _ => {
                self.bullets.push(Bullet::player(px, py - 1.0));
                self.bullets.push(Bullet::player_angled(px - 2.0, py, -0.4));
                self.bullets.push(Bullet::player_angled(px + 2.0, py, 0.4));
            }
        }
    }

    fn collisions(&mut self) {
        let mut rng = rand::thread_rng();

        // Player bullets vs enemies
        for b in &mut self.bullets {
            if !b.alive || b.owner != BulletOwner::Player { continue; }
            for e in &mut self.enemies {
                if !e.alive { continue; }
                let (ex, ey, ew, eh) = e.hitbox();
                if b.x >= ex && b.x <= ex + ew && b.y >= ey && b.y <= ey + eh {
                    b.alive = false;
                    e.hp -= b.power;
                    if e.hp <= 0 {
                        e.alive = false;
                        self.player.score += e.score_value();
                        let cx = e.x + e.width() as f64 / 2.0;
                        let cy = e.y + e.height() as f64 / 2.0;
                        let big = matches!(e.enemy_type, EnemyType::Boss | EnemyType::Bomber);
                        self.explosions.push(Explosion::new(cx, cy, big));
                        if rng.gen::<f64>() < 0.15 {
                            let kind = if rng.gen::<f64>() < 0.5 { PowerUpType::WeaponUp }
                                else if rng.gen::<f64>() < 0.5 { PowerUpType::Shield }
                                else { PowerUpType::ExtraLife };
                            self.powerups.push(PowerUp::new(cx, cy, kind));
                        }
                    }
                    break;
                }
            }
        }

        // Enemy bullets vs player
        let (px, py, pw, ph) = self.player.hitbox();
        for b in &mut self.bullets {
            if !b.alive || b.owner != BulletOwner::Enemy { continue; }
            if b.x >= px && b.x <= px + pw && b.y >= py && b.y <= py + ph {
                b.alive = false;
                self.player.hit();
                self.explosions.push(Explosion::new(b.x, b.y, false));
            }
        }

        // Enemy ram vs player
        if !self.player.is_invincible() {
            for e in &mut self.enemies {
                if !e.alive { continue; }
                let (ex, ey, ew, eh) = e.hitbox();
                if px < ex + ew && px + pw > ex && py < ey + eh && py + ph > ey {
                    self.player.hit();
                    e.hp -= 3;
                    if e.hp <= 0 {
                        e.alive = false;
                        let cx = e.x + e.width() as f64 / 2.0;
                        let cy = e.y + e.height() as f64 / 2.0;
                        self.explosions.push(Explosion::new(cx, cy, false));
                        self.player.score += e.score_value() / 2;
                    }
                    break;
                }
            }
        }

        // Powerups vs player
        for pu in &mut self.powerups {
            if !pu.alive { continue; }
            if pu.x >= px && pu.x <= px + pw && pu.y >= py && pu.y <= py + ph {
                pu.alive = false;
                match pu.kind {
                    PowerUpType::WeaponUp => {
                        if self.player.power_level < 3 { self.player.power_level += 1; }
                        self.player.score += 50;
                    }
                    PowerUpType::Shield => {
                        self.player.invincible_timer = 90;
                        self.player.score += 50;
                    }
                    PowerUpType::ExtraLife => {
                        self.player.lives += 1;
                        self.player.score += 50;
                    }
                }
            }
        }
    }

    pub fn render(&mut self) {
        match self.state {
            GameState::Title => self.render_title(),
            GameState::Playing => self.render_game(),
            GameState::GameOver => self.render_gameover(),
        }
    }

    fn render_game(&mut self) {
        self.fb.clear();

        // Draw layers back to front
        self.starfield.draw(&mut self.fb);
        for pu in &self.powerups { pu.draw(&mut self.fb); }
        for e in &self.enemies { e.draw(&mut self.fb); }
        for b in &self.bullets { b.draw(&mut self.fb); }
        self.player.draw(&mut self.fb);
        for ex in &self.explosions { ex.draw(&mut self.fb); }

        // HUD: draw colored text on top row of pixels
        let hud = format!(
            " LIVES:{} SCORE:{:08} WAVE:{} PWR:{}",
            self.player.lives, self.player.score, self.wave, self.player.power_level
        );
        let hud_color = Rgb::new(255, 220, 50);
        for (i, _ch) in hud.chars().enumerate() {
            self.fb.set(i as i32, 0, hud_color);
            self.fb.set(i as i32, 1, Rgb::new(40, 40, 0));
        }

        // Render framebuffer with half-blocks
        self.fb.render();

        // Overlay HUD text on terminal row 0
        let mut out = BufWriter::new(stdout());
        queue!(out, cursor::MoveTo(0, 0)).ok();
        queue!(out, SetForegroundColor(Color::Rgb { r: 255, g: 220, b: 50 })).ok();
        write!(out, "{}", hud).ok();
        queue!(out, ResetColor).ok();
        out.flush().ok();
    }

    fn render_title(&mut self) {
        self.fb.clear();
        self.starfield.update(self.fb.height, self.fb.width);
        self.starfield.draw(&mut self.fb);
        self.fb.render();

        let mut out = BufWriter::new(stdout());
        let cx = self.term_w / 2;
        let cy = self.term_h / 2;

        let title = [
            " ____  ____   _    ____ _____",
            "/ ___||  _ \\ / \\  / ___| ____|",
            "\\___ \\| |_) / _ \\| |   |  _|",
            " ___) |  __/ ___ \\ |___| |___",
            "|____/|_| /_/   \\_\\____|_____|",
            "",
            " ____  _   _  ___   ___ _____",
            "/ ___|| | | |/ _ \\ / _ \\_   _|",
            "\\___ \\| |_| | | | | | | || |",
            " ___) |  _  | |_| | |_| || |",
            "|____/|_| |_|\\___/ \\___/ |_|",
        ];

        queue!(out, SetForegroundColor(Color::Rgb { r: 80, g: 180, b: 255 })).ok();
        for (i, line) in title.iter().enumerate() {
            let x = cx.saturating_sub(16);
            let y = cy.saturating_sub(8) + i;
            queue!(out, cursor::MoveTo(x as u16, y as u16)).ok();
            write!(out, "{}", line).ok();
        }

        queue!(out, SetForegroundColor(Color::White)).ok();
        let msg = "SPACE to start | WASD/Arrows: Move | SPACE: Shoot | Q: Quit";
        let x = cx.saturating_sub(msg.len() / 2);
        queue!(out, cursor::MoveTo(x as u16, (cy + 5) as u16)).ok();
        write!(out, "{}", msg).ok();

        queue!(out, ResetColor).ok();
        out.flush().ok();
    }

    fn render_gameover(&mut self) {
        // Keep last frame visible, overlay text
        let mut out = BufWriter::new(stdout());
        let cx = self.term_w / 2;
        let cy = self.term_h / 2;

        let lines = [
            "╔═══════════════════════════════╗",
            "║                               ║",
            "║        G A M E   O V E R      ║",
            "║                               ║",
            "║     Your ship was destroyed!   ║",
            "║                               ║",
            "║     Press R to retry           ║",
            "║     Press Q to quit            ║",
            "║                               ║",
            "╚═══════════════════════════════╝",
        ];

        queue!(out, SetForegroundColor(Color::Rgb { r: 255, g: 60, b: 60 })).ok();
        for (i, line) in lines.iter().enumerate() {
            let x = cx.saturating_sub(16);
            let y = cy.saturating_sub(5) + i;
            queue!(out, cursor::MoveTo(x as u16, y as u16)).ok();
            write!(out, "{}", line).ok();
        }

        queue!(out, SetForegroundColor(Color::Yellow)).ok();
        let info = format!("Score: {:08}  Wave: {}", self.player.score, self.wave);
        let x = cx.saturating_sub(info.len() / 2);
        queue!(out, cursor::MoveTo(x as u16, (cy + 6) as u16)).ok();
        write!(out, "{}", info).ok();

        queue!(out, ResetColor).ok();
        out.flush().ok();
    }
}
