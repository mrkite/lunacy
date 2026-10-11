/*

lightweight termcell-style class
@copyright 2026 Sean Kasun

*/

mod point;
mod rect;
mod size;

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::EnableMouseCapture,
    execute, queue,
    style::{self, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{
        self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};
pub use point::Point;
pub use rect::Rect;
pub use size::Size;
use std::io::{self, Write, stdout};

pub type Color = Option<u32>;

#[derive(Clone, PartialEq)]
struct Cell {
    c: char,
    fg: u32,
    bg: u32,
}

impl Cell {
    const EMPTY: Self = Self {
        c: ' ',
        fg: 0,
        bg: 0,
    };

    pub const fn set(&mut self, c: char, fg: Color, bg: Color) {
        self.c = c;
        if let Some(fg) = fg {
            self.fg = fg;
        }
        if let Some(bg) = bg {
            self.bg = bg;
        }
    }
}

pub struct Screen {
    bounds: Rect,
    cursor_x: Option<u16>,
    cursor_y: Option<u16>,
    buffer: Vec<Cell>,
    live: Vec<Cell>,
}

impl Screen {
    pub fn new(w: u16, h: u16) -> Self {
        let len = usize::from(w) * usize::from(h);
        Self {
            bounds: Rect::new(0, 0, i32::from(w), i32::from(h)),
            cursor_x: None,
            cursor_y: None,
            buffer: vec![Cell::EMPTY; len],
            live: vec![Cell::EMPTY; len],
        }
    }

    pub fn draw(&mut self, x: i32, y: i32, c: char, fg: Color, bg: Color) {
        let idx = self.coord_to_idx(x, y);
        self.buffer[idx].set(c, fg, bg);
    }

    fn coord_to_idx(&self, x: i32, y: i32) -> usize {
        usize::try_from(y * self.bounds.w() + x).unwrap_or_default()
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.bounds.b = i32::from(h);
        self.bounds.r = i32::from(w);
        let len = usize::from(w) * usize::from(h);
        if self.buffer.len() > len {
            self.buffer.truncate(len);
            self.live.truncate(len);
        } else {
            self.buffer.resize(len, Cell::EMPTY);
            self.live.resize(len, Cell::EMPTY);
        }
    }

    pub fn flip(&mut self) {
        let mut stdout = stdout();
        let mut delta = vec![];
        for (i, (curr, prev)) in self.buffer.iter().zip(self.live.iter()).enumerate() {
            if curr != prev {
                delta.push((i, &self.buffer[i]));
            }
        }

        let mut fg = 0;
        let mut bg = 0;
        let mut prev_x = 0;
        let mut prev_y = 0;
        let _ = queue!(stdout, MoveTo(prev_x, prev_y));
        let width = usize::try_from(self.bounds.w()).unwrap_or(usize::MAX);
        for (i, cell) in &delta {
            if let Ok(x) = u16::try_from(i % width)
                && let Ok(y) = u16::try_from(i / width)
            {
                if x != prev_x + 1 || y != prev_y {
                    let _ = queue!(stdout, MoveTo(x, y));
                }
                prev_x = x;
                prev_y = y;
                if cell.fg != fg {
                    let _ = queue!(
                        stdout,
                        SetForegroundColor(style::Color::Rgb {
                            r: ((cell.fg >> 16) & 0xff) as u8,
                            g: ((cell.fg >> 8) & 0xff) as u8,
                            b: (cell.fg & 0xff) as u8,
                        })
                    );
                }
                if cell.bg != bg {
                    let _ = queue!(
                        stdout,
                        SetBackgroundColor(style::Color::Rgb {
                            r: ((cell.bg >> 16) & 0xff) as u8,
                            g: ((cell.bg >> 8) & 0xff) as u8,
                            b: (cell.bg & 0xff) as u8,
                        })
                    );
                }
                fg = cell.fg;
                bg = cell.bg;
                let _ = queue!(stdout, Print(cell.c));
            }
            self.live[*i].set(cell.c, Some(cell.fg), Some(cell.bg));
        }
        if let (Some(x), Some(y)) = (self.cursor_x, self.cursor_y) {
            let _ = queue!(stdout, MoveTo(x, y));
        }
        let _ = stdout.flush();
    }

    pub fn refresh(&mut self) {
        self.live.fill(Cell::EMPTY);
        self.flip();
    }

    pub fn hide_cursor(&mut self) {
        let _ = execute!(stdout(), Hide);
        self.cursor_x = None;
        self.cursor_y = None;
    }

    pub fn show_cursor(&mut self, x: u16, y: u16) {
        let _ = execute!(stdout(), Show);
        self.cursor_x = Some(x);
        self.cursor_y = Some(y);
    }

    pub const fn get_bounds(&self) -> &Rect {
        &self.bounds
    }
}

pub fn init() -> io::Result<Screen> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        hook(info);
    }));
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    execute!(stdout(), EnableMouseCapture)?;
    let (w, h) = terminal::size()?;
    Ok(Screen::new(w, h))
}

pub fn restore() {
    if let Err(err) = try_restore() {
        std::eprintln!("failed to restore terminal: {err}");
    }
}

fn try_restore() -> io::Result<()> {
    let _ = disable_raw_mode();
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
