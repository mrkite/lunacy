/*

General BASIC viewer
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Message, gestalt},
    screen::{Color, Point, Rect, Screen},
    widgets::{KeyMap, Scroll, Widget},
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::io::Write;

const FG: Color = Some(0xba_9c_7e);
const BG: Color = Some(0x00_00_01);

struct Line {
    number: i32,
    text: String,
}

#[derive(Default)]
pub struct Basic {
    bounds: Rect,
    scroll: Scroll,
    lines: Vec<Line>,
    wrapped: Vec<Line>,
    flown_width: i32,
    cursor: usize,
    search: Option<String>,
}

impl Basic {
    pub fn add(&mut self, line_num: i32, line: &str) {
        self.lines.push(Line {
            number: line_num,
            text: format!("{line_num:-8} {line}"),
        });
    }

    fn scroll_to_cursor(&mut self) {
        if !self.lines.is_empty() {
            self.cursor = self.cursor.clamp(0, self.wrapped.len().saturating_sub(1));
            if let Ok(cursor) = i32::try_from(self.cursor) {
                if cursor < self.scroll.get() {
                    self.scroll.set(cursor);
                }
                let page = self.bounds.h().saturating_sub(1);
                if cursor >= self.scroll.get() + page {
                    self.scroll.set(cursor.saturating_sub(page));
                }
            }
        }
    }

    fn flow(&mut self) {
        self.flown_width = self.bounds.w();
        if self.flown_width != 0 {
            self.wrapped.clear();
            let flown = self.flown_width;

            for line in &self.lines {
                let mut wrap = String::default();
                let mut x = 0;
                for ch in line.text.chars() {
                    wrap.push(ch);
                    x += 1;
                    if x == flown {
                        self.wrapped.push(Line {
                            number: line.number,
                            text: wrap.clone(),
                        });
                        wrap.clear();
                        x = 0;
                    }
                }
                self.wrapped.push(Line {
                    number: line.number,
                    text: wrap,
                });
            }
        }
        self.scroll.set_range(
            0,
            i32::try_from(self.wrapped.len()).unwrap_or(i32::MAX),
            self.bounds.h(),
        );
    }

    fn status(&self, ctx: &Context) {
        if let Some(line) = self.lines.get(self.cursor) {
            ctx.post(Message::Status(format!("Line {}", line.number)));
        }
    }
}

impl Widget for Basic {
    fn layout(&mut self, constraint: &Rect) {
        let without_scroll = constraint.remove_scroll();
        if self.bounds == without_scroll {
            // only reflow if we need to
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
            let row = (y - self.bounds.t) + top;
            if let Ok(idx) = usize::try_from(row)
                && let Some(line) = self.wrapped.get(idx)
            {
                let (fg, bg) = if idx == self.cursor {
                    (BG, FG)
                } else {
                    (FG, BG)
                };
                let mut x = self.bounds.l;
                for ch in line.text.chars() {
                    if x >= range.l && x < range.r {
                        s.draw(x, y, ch, fg, bg);
                    }
                    x += 1;
                }
                while x >= range.l && x < range.r {
                    s.draw(x, y, ' ', fg, bg);
                    x += 1;
                }
            } else {
                for x in range.l..range.r {
                    s.draw(x, y, ' ', FG, BG);
                }
            }
        }
    }

    fn key(&mut self, code: KeyCode, _modifiers: KeyModifiers, ctx: &Context) -> bool {
        let page = isize::try_from(self.bounds.h().saturating_sub(1)).unwrap_or(1);

        match code {
            KeyCode::Up | KeyCode::Char('k') => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => self.cursor = self.cursor.saturating_add(1),
            KeyCode::PageUp | KeyCode::Char('K') => {
                self.cursor = self.cursor.saturating_sub_signed(page);
            }
            KeyCode::PageDown | KeyCode::Char('J') => {
                self.cursor = self.cursor.saturating_add_signed(page);
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.wrapped.len(),
            _ => return false,
        }
        self.scroll_to_cursor();
        self.status(ctx);
        true
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        if is_down && self.bounds.inside(pt.x, pt.y) {
            if let Ok(cursor) = usize::try_from(self.scroll.get() + pt.y - self.bounds.t) {
                self.cursor = cursor;
                self.scroll_to_cursor();
                self.status(ctx);
            }
            true
        } else {
            self.scroll.mouse(pt, is_down, ctx)
        }
    }

    fn gestalt(&self) -> u8 {
        let mut mask = gestalt::JUMP | gestalt::FIND | gestalt::EXPORT;
        if self.search.is_some() {
            mask |= gestalt::FIND_NEXT | gestalt::FIND_PREV;
        }
        mask
    }

    fn jump(&mut self, to: usize, ctx: &Context) {
        let addr = u32::try_from(to).unwrap_or_default().cast_signed();
        self.cursor = 0;
        for (idx, line) in self.wrapped.iter().enumerate() {
            self.cursor = idx;
            if line.number >= addr {
                break;
            }
        }
        self.scroll_to_cursor();
        self.status(ctx);
    }

    fn find(&mut self, key: &str, ctx: &Context) {
        self.search = Some(key.to_string());
        self.find_next(ctx);
    }

    fn find_next(&mut self, ctx: &Context) {
        if let Some(search) = self.search.as_ref() {
            for cursor in self.cursor.saturating_add(1)..self.wrapped.len() {
                if let Some(row) = self.wrapped.get(cursor)
                    && row.text.contains(search)
                {
                    self.cursor = cursor;
                    self.scroll_to_cursor();
                    self.status(ctx);
                    return;
                }
            }
        }
        ctx.post(Message::NotFound);
    }

    fn find_prev(&mut self, ctx: &Context) {
        if let Some(search) = self.search.as_ref() {
            for cursor in (0..self.cursor.saturating_sub(1)).rev() {
                if let Some(row) = self.lines.get(cursor)
                    && row.text.contains(search)
                {
                    self.cursor = cursor;
                    self.scroll_to_cursor();
                    self.status(ctx);
                    return;
                }
            }
        }
        ctx.post(Message::NotFound);
    }

    fn export(&self, out: &mut std::fs::File) -> Result<(), std::io::Error> {
        for line in &self.lines {
            writeln!(out, "{}", line.text)?;
        }
        Ok(())
    }

    fn get_key_map(&self) -> KeyMap {
        let mut m = KeyMap::default();
        m.insert("a↓↑|jk", "scroll");
        m
    }
}
