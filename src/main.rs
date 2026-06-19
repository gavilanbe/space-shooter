mod audio;
mod bullets;
mod enemies;
mod framebuffer;
mod game;
mod player;
mod rayman;
mod rayman_demo;
mod sprites;
mod starfield;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::stdout;
use std::time::{Duration, Instant};

use audio::Audio;
use game::{Game, GameState};

const MOVE_SPEED: f64 = 1.8;

fn main() {
    let mut stdout = stdout();

    let args: Vec<String> = std::env::args().collect();
    let rayman_mode = args.iter().any(|a| a == "--rayman");

    terminal::enable_raw_mode().expect("raw mode");
    stdout.execute(terminal::EnterAlternateScreen).expect("alt screen");
    stdout.execute(cursor::Hide).expect("hide cursor");
    stdout.execute(terminal::Clear(ClearType::All)).ok();

    let (tw, th) = terminal::size().unwrap_or((80, 24));

    if rayman_mode {
        rayman_demo::run_demo(tw as usize, th as usize);
        cleanup();
        return;
    }

    let _audio = Audio::new();
    let mut game = Game::new(tw as usize, th as usize);
    game.render();

    let frame_duration = Duration::from_millis(33);

    let mut up = false;
    let mut down = false;
    let mut left = false;
    let mut right = false;
    let mut shooting = false;

    loop {
        let t0 = Instant::now();

        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            cleanup(); return;
                        }
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            if matches!(game.state, GameState::GameOver) { game.reset(); }
                        }
                        KeyCode::Char(' ') => match game.state {
                            GameState::Title => { game.state = GameState::Playing; }
                            GameState::Playing => { shooting = true; }
                            _ => {}
                        },
                        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => up = true,
                        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => down = true,
                        KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => left = true,
                        KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => right = true,
                        _ => {}
                    }
                } else if key.kind == KeyEventKind::Release {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => up = false,
                        KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => down = false,
                        KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => left = false,
                        KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => right = false,
                        KeyCode::Char(' ') => shooting = false,
                        _ => {}
                    }
                }
            }
        }

        if matches!(game.state, GameState::Playing) {
            game.player.moving_left = left;
            game.player.moving_right = right;

            if up { game.player.y -= MOVE_SPEED; }
            if down { game.player.y += MOVE_SPEED; }
            if left { game.player.x -= MOVE_SPEED; }
            if right { game.player.x += MOVE_SPEED; }

            if shooting { game.player_shoot(); }
        }

        if let Ok((nw, nh)) = terminal::size() {
            let (nw, nh) = (nw as usize, nh as usize);
            if nw != game.term_w || nh != game.term_h {
                game.resize(nw, nh);
            }
        }

        game.update();
        game.render();

        let elapsed = t0.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}

fn cleanup() {
    let mut stdout = stdout();
    terminal::disable_raw_mode().ok();
    stdout.execute(terminal::LeaveAlternateScreen).ok();
    stdout.execute(cursor::Show).ok();
}
