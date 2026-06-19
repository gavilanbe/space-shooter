/// Quick viewer: renders all Rayman sprites side-by-side and animates idle + run.
/// Press Q or Esc to quit.
use crate::framebuffer::Framebuffer;
use crate::rayman::*;

pub fn run_demo(tw: usize, th: usize) {
    use crossterm::{cursor, queue, terminal};
    use crossterm::style::{SetForegroundColor, Color, ResetColor};
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};
    use std::io::{Write, stdout, BufWriter};
    use std::time::{Duration, Instant};

    let mut fb = Framebuffer::new(tw, th);
    let mut frame: usize = 0;

    let idle_frames: Vec<&[&[u8]]> = vec![&IDLE_RIGHT, &IDLE_RIGHT_2];
    let run_r_frames: Vec<&[&[u8]]> = vec![&RUN_RIGHT_1, &RUN_RIGHT_2, &RUN_RIGHT_3];
    let run_l_frames: Vec<&[&[u8]]> = vec![&RUN_LEFT_1, &RUN_LEFT_2];

    loop {
        let t0 = Instant::now();

        // Input
        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return,
                        _ => {}
                    }
                }
            }
        }

        // Resize check
        if let Ok((nw, nh)) = terminal::size() {
            let (nw, nh) = (nw as usize, nh as usize);
            if nw != fb.width || nh * 2 != fb.height {
                fb.resize(nw, nh);
            }
        }

        fb.clear();

        let spacing = 18;
        let base_y = 4;
        let mut col = 3;

        // Row 1: Static poses
        // Idle right
        draw_rayman(&mut fb, col, base_y, &IDLE_RIGHT);
        col += spacing;

        // Idle left
        draw_rayman(&mut fb, col, base_y, &IDLE_LEFT);
        col += spacing;

        // Jump right
        draw_rayman(&mut fb, col, base_y, &JUMP_RIGHT);
        col += spacing;

        // Jump left
        draw_rayman(&mut fb, col, base_y, &JUMP_LEFT);
        col += spacing;

        // Fall
        draw_rayman(&mut fb, col, base_y, &FALL_RIGHT);
        col += spacing;

        // Punch
        draw_rayman(&mut fb, col, base_y, &PUNCH_RIGHT);

        // Row 2: Animated
        let row2_y = base_y + 24;
        let anim_speed = 8; // frames per sprite change

        // Animated idle
        let idle_idx = (frame / (anim_speed * 2)) % idle_frames.len();
        draw_rayman(&mut fb, 3, row2_y, idle_frames[idle_idx]);

        // Animated run right
        let run_r_idx = (frame / anim_speed) % run_r_frames.len();
        draw_rayman(&mut fb, 3 + spacing, row2_y, run_r_frames[run_r_idx]);

        // Animated run left
        let run_l_idx = (frame / anim_speed) % run_l_frames.len();
        draw_rayman(&mut fb, 3 + spacing * 2, row2_y, run_l_frames[run_l_idx]);

        fb.render();

        // Draw text labels on top
        let mut out = BufWriter::new(stdout());
        queue!(out, SetForegroundColor(Color::Rgb { r: 180, g: 180, b: 180 })).ok();

        let labels_row1 = [
            (3, "IDLE R"),
            (3 + spacing as usize, "IDLE L"),
            (3 + spacing as usize * 2, "JUMP R"),
            (3 + spacing as usize * 3, "JUMP L"),
            (3 + spacing as usize * 4, "FALL"),
            (3 + spacing as usize * 5, "PUNCH"),
        ];
        for (x, label) in &labels_row1 {
            queue!(out, cursor::MoveTo(*x as u16, (base_y / 2 - 1) as u16)).ok();
            write!(out, "{}", label).ok();
        }

        let labels_row2 = [
            (3, "BREATHING"),
            (3 + spacing as usize, "RUN ->"),
            (3 + spacing as usize * 2, "RUN <-"),
        ];
        for (x, label) in &labels_row2 {
            queue!(out, cursor::MoveTo(*x as u16, (row2_y / 2 - 1) as u16)).ok();
            write!(out, "{}", label).ok();
        }

        // Title
        queue!(out, SetForegroundColor(Color::Rgb { r: 140, g: 60, b: 180 })).ok();
        queue!(out, cursor::MoveTo(3, 0)).ok();
        write!(out, "RAYMAN SPRITE VIEWER  [Q to quit]").ok();

        queue!(out, ResetColor).ok();
        out.flush().ok();

        frame += 1;

        let elapsed = t0.elapsed();
        let target = Duration::from_millis(33);
        if elapsed < target {
            std::thread::sleep(target - elapsed);
        }
    }
}
