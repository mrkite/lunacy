/*

Simple Key Value table
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Message, gestalt},
    screen::{Color, Point, Rect, Screen},
    widgets::{KeyMap, Scroll, Widget},
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::cmp;

const FG: Color = Some(0xba_9c_7e);
const BG: Color = None;

pub struct Table {
    bounds: Rect,
    scroll: Scroll,
    lines: Vec<(String, String)>,
    wrapped: Vec<(String, String)>,
    cursor: i32,
    search: Option<String>,
    need_reflow: bool,
    has_cursor: bool,
    fg: Color,
    bg: Color,
}

impl Default for Table {
    fn default() -> Self {
        Self {
            bounds: Rect::default(),
            scroll: Scroll::default(),
            lines: vec![],
            wrapped: vec![],
            cursor: 0,
            search: None,
            need_reflow: true,
            has_cursor: true,
            fg: FG,
            bg: BG,
        }
    }
}

impl Table {
    pub const fn with_style(mut self, fg: Color, bg: Color) -> Self {
        self.fg = fg;
        self.bg = bg;
        self.scroll.set_style(fg, bg);
        self
    }

    pub const fn without_cursor(mut self) -> Self {
        self.has_cursor = false;
        self
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.need_reflow = true;
    }

    pub fn add(&mut self, key: &str, value: &str) {
        // expand tabs
        let mut expanded = String::new();
        for ch in key.chars() {
            match ch {
                '\t' => {
                    let tab = (expanded.chars().count() + 8) & !7;
                    for _ in 0..tab {
                        expanded.push(' ');
                    }
                }
                ch => expanded.push(ch),
            }
        }

        self.lines.push((expanded, value.to_owned()));
        self.need_reflow = true;
    }

    pub fn sort(&mut self) {
        self.lines.sort();
        self.flow();
    }

    fn scroll_to_cursor(&mut self) {
        if !self.wrapped.is_empty() {
            self.cursor = self.cursor.clamp(
                0,
                i32::try_from(self.wrapped.len().saturating_sub(1)).unwrap_or(i32::MAX),
            );
            if self.cursor < self.scroll.get() {
                self.scroll.set(self.cursor);
            }
            let page = self.bounds.h().saturating_sub(1);
            if self.cursor >= self.scroll.get() + page {
                self.scroll.set(self.cursor.saturating_sub(page));
            }
        }
    }

    fn flow(&mut self) {
        self.need_reflow = false;
        let flown = self.bounds.w();
        if flown != 0 {
            self.wrapped.clear();

            // first calculate key column width
            let mut key_col = 0;
            for (key, _) in &self.lines {
                key_col = cmp::max(
                    key_col,
                    i32::try_from(key.chars().count()).unwrap_or(i32::MAX),
                );
            }
            key_col += 1;
            if flown > key_col {
                // if we're too narrow, show nothing
                for (key, val) in &self.lines {
                    let mut wrap = String::default();
                    let mut x = 0;
                    for ch in key.chars() {
                        wrap.push(ch);
                        x += 1;
                    }
                    while x < key_col {
                        wrap.push(' ');
                        x += 1;
                    }
                    for ch in val.chars() {
                        wrap.push(ch);
                        x += 1;
                        if x == flown {
                            self.wrapped.push((key.clone(), wrap.clone()));
                            wrap.clear();
                            x = 0;
                            while x < key_col {
                                wrap.push(' ');
                                x += 1;
                            }
                        }
                    }
                    self.wrapped.push((key.clone(), wrap));
                }
            }
        }
        self.scroll.set_range(
            0,
            i32::try_from(self.wrapped.len()).unwrap_or(i32::MAX),
            self.bounds.h(),
        );
    }

    pub fn selected(&self) -> Option<&str> {
        if let Ok(idx) = usize::try_from(self.cursor)
            && let Some(line) = self.wrapped.get(idx)
        {
            Some(&line.0)
        } else {
            None
        }
    }
}

impl Widget for Table {
    fn layout(&mut self, constraint: &Rect) {
        let without_scroll = constraint.remove_scroll();
        if self.bounds == without_scroll && !self.need_reflow {
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
                let (fg, bg) = if row == self.cursor {
                    (
                        if self.bg.is_none() {
                            Some(0x00_00_01)
                        } else {
                            self.bg
                        },
                        self.fg,
                    )
                } else {
                    (self.fg, self.bg)
                };
                let mut x = self.bounds.l;
                for ch in line.1.chars() {
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
                    s.draw(x, y, ' ', self.fg, self.bg);
                }
            }
        }
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if self.has_cursor {
            let page = self.bounds.h().saturating_sub(1);
            match code {
                KeyCode::Up | KeyCode::Char('k') => self.cursor = self.cursor.saturating_sub(1),
                KeyCode::Down | KeyCode::Char('j') => self.cursor = self.cursor.saturating_add(1),
                KeyCode::PageUp | KeyCode::Char('K') => {
                    self.cursor = self.cursor.saturating_sub(page);
                }
                KeyCode::PageDown | KeyCode::Char('J') => {
                    self.cursor = self.cursor.saturating_add(page);
                }
                KeyCode::Home => self.cursor = 0,
                KeyCode::End => self.cursor = i32::try_from(self.wrapped.len()).unwrap_or(i32::MAX),
                _ => return false,
            }
            self.scroll_to_cursor();
            true
        } else {
            self.scroll.key(code, modifiers, ctx)
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        if self.has_cursor && is_down && self.bounds.inside(pt.x, pt.y) {
            self.cursor = self.scroll.get() + pt.y - self.bounds.t;
            self.scroll_to_cursor();
            true
        } else {
            self.scroll.mouse(pt, is_down, ctx)
        }
    }

    fn gestalt(&self) -> u8 {
        if self.has_cursor {
            let mut mask = gestalt::FIND;
            if self.search.is_some() {
                mask |= gestalt::FIND_NEXT | gestalt::FIND_PREV;
            }
            mask
        } else {
            self.scroll.gestalt()
        }
    }

    fn find(&mut self, val: &str, ctx: &Context) {
        self.search = Some(val.to_string());
        self.find_next(ctx);
    }

    fn find_next(&mut self, ctx: &Context) {
        if let Some(search) = self.search.as_ref() {
            for cursor in self.cursor + 1..i32::try_from(self.wrapped.len()).unwrap_or(i32::MAX) {
                if let Ok(idx) = usize::try_from(cursor)
                    && let Some(row) = self.wrapped.get(idx)
                    && row.1.contains(search)
                {
                    self.cursor = cursor;
                    self.scroll_to_cursor();
                    return;
                }
            }
        }
        ctx.post(Message::NotFound);
    }

    fn find_prev(&mut self, ctx: &Context) {
        if let Some(search) = self.search.as_ref() {
            for cursor in (0..self.cursor - 1).rev() {
                if let Ok(idx) = usize::try_from(cursor)
                    && let Some(row) = self.wrapped.get(idx)
                    && row.1.contains(search)
                {
                    self.cursor = cursor;
                    self.scroll_to_cursor();
                    return;
                }
            }
        }
        ctx.post(Message::NotFound);
    }

    fn get_key_map(&self) -> KeyMap {
        let mut m = KeyMap::default();
        m.insert("a↓↑|jk", "scroll");
        m
    }
}
