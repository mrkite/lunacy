/*

Text entry widget
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, gestalt},
    screen::{Color, Point, Rect, Screen},
    widgets::Widget,
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::cmp;

const FG: Color = Some(0xca_e6_c5);
const BG: Color = Some(0x2c_48_4c);

#[derive(Default)]
pub struct Entry {
    bounds: Rect,
    scroll: i32,
    cursor: i32,
    text: String,
    focused: bool,
}

impl Widget for Entry {
    fn layout(&mut self, constraint: &Rect) {
        constraint.clone_into(&mut self.bounds);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        let range = clip.intersect(&self.bounds.offset(origin));

        let mut x = self.bounds.l + origin.x - self.scroll;
        let y = self.bounds.t + origin.y;
        for ch in self.text.chars() {
            if range.inside(x, y) {
                s.draw(x, y, ch, FG, BG);
            }
            x += 1;
        }
        while x < range.r {
            if x >= range.l {
                s.draw(x, y, ' ', FG, BG);
            }
            x += 1;
        }
        if self.focused
            && let Ok(x) =
                u16::try_from(self.bounds.l + origin.x + self.cursor.wrapping_sub(self.scroll))
            && let Ok(y) = u16::try_from(self.bounds.t + origin.y)
        {
            s.show_cursor(x, y);
        }
    }

    fn key(&mut self, code: KeyCode, _modifiers: KeyModifiers, _ctx: &Context) -> bool {
        match code {
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => {
                self.cursor = cmp::min(
                    self.cursor + 1,
                    i32::try_from(self.text.chars().count()).unwrap_or(i32::MAX),
                );
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => {
                self.cursor = i32::try_from(self.text.chars().count()).unwrap_or(i32::MAX);
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor = self.cursor.saturating_sub(1);
                    if let Ok(idx) = usize::try_from(self.cursor) {
                        self.text.remove(idx);
                    }
                }
            }
            KeyCode::Delete => {
                if let Ok(idx) = usize::try_from(self.cursor)
                    && idx < self.text.chars().count()
                {
                    self.text.remove(idx);
                }
            }
            KeyCode::Char(ch) => {
                if let Ok(idx) = usize::try_from(self.cursor) {
                    self.text.insert(idx, ch);
                    self.cursor += 1;
                }
            }
            _ => return false,
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        }
        if self.cursor >= self.scroll + self.bounds.w() {
            self.scroll = self.cursor - (self.bounds.w() - 1);
        }
        true
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, _ctx: &Context) -> bool {
        if is_down && self.bounds.inside(pt.x, pt.y) {
            self.focused = true;
            let tx = pt.x - self.bounds.l;
            self.cursor = tx.clamp(
                0,
                i32::try_from(self.text.chars().count()).unwrap_or(i32::MAX),
            );
            true
        } else {
            false
        }
    }

    fn gestalt(&self) -> u8 {
        gestalt::FOCUS
    }

    fn focus(&mut self, focus: bool, _ctx: &Context) {
        self.focused = focus;
    }
}

impl Entry {
    pub const fn with_focus(mut self) -> Self {
        self.focused = true;
        self
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }

    pub fn get(&self) -> &str {
        &self.text
    }

    pub fn set(&mut self, text: &str) {
        text.clone_into(&mut self.text);
        if let Ok(cursor) = i32::try_from(self.text.chars().count()) {
            self.cursor = cursor;
        }
    }
}
