use crossterm::{cursor, queue};
use crossterm::style::{SetForegroundColor, SetBackgroundColor, Color, ResetColor};
use std::io::{Write, stdout, BufWriter};

/// RGB color
#[derive(Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }
    pub const BLACK: Rgb = Rgb::new(0, 0, 0);

}

/// Framebuffer with half-block rendering.
/// Each terminal cell = 2 vertical pixels using '▀' (upper half block).
/// Foreground color = top pixel, Background color = bottom pixel.
pub struct Framebuffer {
    pub width: usize,   // pixels wide = terminal columns
    pub height: usize,  // pixels tall = terminal rows * 2
    pub pixels: Vec<Rgb>,
}

impl Framebuffer {
    pub fn new(term_w: usize, term_h: usize) -> Self {
        let width = term_w;
        let height = term_h * 2; // double vertical resolution
        Self {
            width,
            height,
            pixels: vec![Rgb::BLACK; width * height],
        }
    }

    pub fn resize(&mut self, term_w: usize, term_h: usize) {
        self.width = term_w;
        self.height = term_h * 2;
        self.pixels = vec![Rgb::BLACK; self.width * self.height];
    }

    pub fn clear(&mut self) {
        for p in &mut self.pixels {
            *p = Rgb::BLACK;
        }
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, color: Rgb) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.pixels[y as usize * self.width + x as usize] = color;
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Rgb {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x]
        } else {
            Rgb::BLACK
        }
    }

    /// Draw a pixel-art sprite from a color-indexed grid.
    /// `sprite` is rows of pixel data, each pixel is an index into `palette`.
    /// Index 0 = transparent.
    pub fn draw_sprite(
        &mut self,
        x: i32,
        y: i32,
        sprite: &[&[u8]],
        palette: &[Rgb],
    ) {
        for (row, line) in sprite.iter().enumerate() {
            for (col, &idx) in line.iter().enumerate() {
                if idx == 0 { continue; } // transparent
                if let Some(&color) = palette.get(idx as usize) {
                    self.set(x + col as i32, y + row as i32, color);
                }
            }
        }
    }

    /// Render framebuffer to terminal using half-block characters.
    pub fn render(&self) {
        let mut out = BufWriter::with_capacity(self.width * self.height * 20, stdout());
        queue!(out, cursor::MoveTo(0, 0)).ok();

        let term_h = self.height / 2;
        let mut last_fg = Rgb::BLACK;
        let mut last_bg = Rgb::BLACK;
        let mut first = true;

        for row in 0..term_h {
            let y_top = row * 2;
            let y_bot = row * 2 + 1;

            for col in 0..self.width {
                let top = self.get(col, y_top);
                let bot = self.get(col, y_bot);

                if first || top != last_fg || bot != last_bg {
                    queue!(out, SetForegroundColor(Color::Rgb { r: top.r, g: top.g, b: top.b })).ok();
                    queue!(out, SetBackgroundColor(Color::Rgb { r: bot.r, g: bot.g, b: bot.b })).ok();
                    last_fg = top;
                    last_bg = bot;
                    first = false;
                }
                write!(out, "▀").ok();
            }
        }

        queue!(out, ResetColor).ok();
        out.flush().ok();
    }
}
