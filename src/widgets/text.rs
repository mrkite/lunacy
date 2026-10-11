/*

Simple multi-line text view
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::Context,
    screen::{Color, Point, Rect, Screen},
    widgets::{Scroll, Widget},
};
use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Default)]
pub struct Text {
    bounds: Rect,
    scroll: Scroll,
    value: String,
    lines: Vec<String>,
    flown_width: i32,
    fg: Color,
    bg: Color,
}

impl Text {
    pub const fn with_style(mut self, fg: Color, bg: Color) -> Self {
        self.fg = fg;
        self.bg = bg;
        self.scroll.set_style(fg, bg);
        self
    }

    pub fn append(&mut self, text: &str) {
        self.value.push_str(text);
        self.flow();
        self.scroll
            .set(i32::try_from(self.lines.len()).unwrap_or(i32::MAX));
    }

    fn flow(&mut self) {
        self.flown_width = self.bounds.w();
        if self.flown_width != 0 {
            self.lines.clear();
            let mut line = String::default();
            let mut gap = String::default();
            let mut word = String::default();
            let mut x = 0;

            for ch in self.value.chars() {
                if ch.is_whitespace() {
                    if !word.is_empty() {
                        line.push_str(&gap);
                        line.push_str(&word);
                        gap.clear();
                        word.clear();
                    }
                    if ch == '\n' {
                        self.lines.push(line.clone());
                        line.clear();
                        x = 0;
                    } else if ch == '\t' {
                        let tab = 8 - (x & 7);
                        for _ in 0..tab {
                            gap.push(' ');
                        }
                        x += tab;
                    } else {
                        gap.push(' ');
                        x += 1;
                    }
                } else {
                    if x == self.bounds.w() {
                        if line.is_empty() {
                            // no spaces yet
                            line.push_str(&gap);
                            line.push_str(&word);
                            self.lines.push(line.clone());
                            word.clear();
                        } else {
                            self.lines.push(line.clone());
                        }
                        gap.clear();
                        line.clear();
                        x = i32::try_from(word.chars().count()).unwrap_or(i32::MAX);
                    }
                    word.push(ch);
                    x += 1;
                }
            }
            line.push_str(&gap);
            line.push_str(&word);
            self.lines.push(line.clone());
        }
        self.scroll.set_range(
            0,
            i32::try_from(self.lines.len().saturating_sub(1)).unwrap_or(i32::MAX),
            self.bounds.h(),
        );
    }
}

impl Widget for Text {
    fn layout(&mut self, constraint: &Rect) {
        let without_scroll = constraint.remove_scroll();
        if self.bounds == without_scroll {
            // only reflow if necessary
            return;
        }
        self.bounds = without_scroll;
        self.scroll.layout(&constraint.to_scroll());
        self.flow();
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.scroll.draw(s, origin, clip);
        let range = clip.intersect(&self.bounds.offset(origin));
        let top = self.scroll.get();

        for y in range.t..range.b {
            let row = (y - self.bounds.t).saturating_add(top);
            if let Some(line) = self.lines.get(usize::try_from(row).unwrap_or_default()) {
                let mut x = self.bounds.l;
                for ch in line.chars() {
                    if x >= range.l && x < range.r {
                        s.draw(x, y, ch, self.fg, self.bg);
                    }
                    x += 1;
                }
                while x >= range.l && x < range.r {
                    s.draw(x, y, ' ', self.fg, self.bg);
                    x += 1;
                }
            } else {
                for x in range.l..range.r {
                    s.draw(x, y, ' ', self.fg, self.bg);
                }
            }
        }
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        self.scroll.key(code, modifiers, ctx)
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        self.scroll.mouse(pt, is_down, ctx)
    }
}
