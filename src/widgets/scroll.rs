/*

Simple vertical scrollbar
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::Context,
    screen::{Color, Point, Rect, Screen},
    widgets::{KeyMap, Widget},
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::cmp;

const SCROLL_THUMB: char = '║';
const SCROLL_BAR: char = '│';
const SCROLL_FG: Color = Some(0x2b_9f_bf);
const SCROLL_BG: Color = None;

pub struct Scroll {
    bounds: Rect,
    val: i32,
    max: i32,
    min: i32,
    fg: Color,
    bg: Color,
    bucket: i32,
}

impl Default for Scroll {
    fn default() -> Self {
        Self {
            bounds: Rect::default(),
            fg: SCROLL_FG,
            bg: SCROLL_BG,
            val: 0,
            min: 0,
            max: 0,
            bucket: 1,
        }
    }
}

impl Scroll {
    pub const fn with_style(mut self, fg: Color, bg: Color) -> Self {
        self.set_style(fg, bg);
        self
    }

    pub const fn set_style(&mut self, fg: Color, bg: Color) {
        self.fg = fg;
        self.bg = bg;
    }

    pub const fn get(&self) -> i32 {
        self.val
    }

    pub fn set(&mut self, val: i32) {
        self.val = val.clamp(self.min, self.max);
    }

    pub fn set_range(&mut self, min: i32, max: i32, bucket: i32) {
        self.min = min;
        self.max = cmp::max(min, max.saturating_sub(bucket));
        self.bucket = bucket;
    }
}

impl Widget for Scroll {
    fn layout(&mut self, constraint: &Rect) {
        constraint.clone_into(&mut self.bounds);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        let range = clip.intersect(&self.bounds.offset(origin));
        for y in range.t..range.b {
            let x = self.bounds.l + origin.x;
            if range.inside(x, y) {
                let scale = self.max + self.bucket - self.min;
                let top = (self.val - self.min) * self.bounds.h() / scale;
                let btm = cmp::max(
                    (self.val + self.bucket - self.min) * self.bounds.h() / scale,
                    top + 1,
                );
                let rel_y = y - (self.bounds.t + origin.y);
                let ch = if (top..btm).contains(&rel_y) {
                    SCROLL_THUMB
                } else {
                    SCROLL_BAR
                };
                s.draw(x, y, ch, self.fg, self.bg);
            }
        }
    }

    fn key(&mut self, code: KeyCode, _modifiers: KeyModifiers, _ctx: &Context) -> bool {
        match code {
            KeyCode::Up | KeyCode::Char('k') => self.val = self.val.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => self.val = self.val.saturating_add(1),
            KeyCode::PageUp | KeyCode::Char('K') => self.val = self.val.saturating_sub(self.bucket),
            KeyCode::PageDown | KeyCode::Char('J') => {
                self.val = self.val.saturating_add(self.bucket);
            }
            KeyCode::Home => self.val = 0,
            KeyCode::End => self.val = self.max,
            _ => return false,
        }
        self.set(self.val);
        true
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, _ctx: &Context) -> bool {
        if is_down && self.bounds.inside(pt.x, pt.y) {
            let line = pt.y - self.bounds.t;
            let val = line * (self.max + self.bucket - self.min) / self.bounds.h();
            self.set(val);
            true
        } else {
            false
        }
    }

    fn get_key_map(&self) -> KeyMap {
        let mut m = KeyMap::default();
        m.insert("p↓↑|jk", "scroll");
        m
    }
}

impl Rect {
    pub const fn to_scroll(&self) -> Self {
        Self::new(self.r - 1, self.t, self.r, self.b)
    }

    pub const fn remove_scroll(&self) -> Self {
        Self::new(self.l, self.t, self.r - 1, self.b)
    }
}
