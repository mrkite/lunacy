/*

General hex view widget
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle, Message, gestalt},
    screen::{Color, Point, Rect, Screen, Size},
    widgets::{KeyMap, Scroll, Widget},
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::cmp;

const ADDRESS_COLOR: Color = Some(0x41_72_9f);
const HEX_COLOR: Color = Some(0x58_85_af);
const ASCII_COLOR: Color = Some(0xb9_90_95);
const DIVIDER_COLOR: Color = Some(0x27_44_72);
const BG: Color = Some(0x00_00_01);
const DIVIDER: char = '│';

#[derive(Default)]
pub struct HexView {
    bounds: Rect,
    scroll: Scroll,
    handle: Handle,
    cursor: usize,
    search: Option<Vec<u8>>,
}

impl Widget for HexView {
    fn min_size(&self) -> Size {
        Size::new(80, 1)
    }

    fn layout(&mut self, constraint: &Rect) {
        self.bounds = constraint.remove_scroll();
        self.scroll.layout(&constraint.to_scroll());
        self.scroll.set_range(
            0,
            i32::try_from(self.handle.len().div_ceil(16)).unwrap_or(i32::MAX),
            self.bounds.h(),
        );
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.scroll.draw(s, origin, clip);
        let range = clip.intersect(&self.bounds.offset(origin));
        let top = self.scroll.get();

        for y in range.t..range.b {
            let addr = (top + y - (self.bounds.t + origin.y)) * 16;
            for x in range.l..range.r {
                let col = x - (self.bounds.l + origin.x);
                if col < 0 {
                    s.draw(x, y, ' ', ADDRESS_COLOR, BG);
                } else if col < 8 {
                    if let Ok(idx) = usize::try_from(addr)
                        && idx < self.handle.len()
                    {
                        let ch = HEX[(idx >> (28 - col * 4)) & 0xf];
                        if idx / 16 == self.cursor / 16 {
                            s.draw(x, y, ch, BG, ADDRESS_COLOR);
                        } else {
                            s.draw(x, y, ch, ADDRESS_COLOR, BG);
                        }
                    } else {
                        s.draw(x, y, ' ', ADDRESS_COLOR, BG);
                    }
                } else if col < 11 {
                    if col == 9 {
                        s.draw(x, y, DIVIDER, DIVIDER_COLOR, BG);
                    } else {
                        s.draw(x, y, ' ', DIVIDER_COLOR, BG);
                    }
                } else if col < 60 {
                    let col = col - 11;
                    let (nib, col) = if col >= 24 {
                        ((col - 1) % 3, (col - 1) / 3)
                    } else {
                        (col % 3, col / 3)
                    };

                    if let Ok(idx) = usize::try_from(addr + col)
                        && idx < self.handle.len()
                    {
                        let ch = self.handle.r8(idx);
                        let (fg, bg) = if idx == self.cursor {
                            (BG, HEX_COLOR)
                        } else {
                            (HEX_COLOR, BG)
                        };
                        match nib {
                            0 => s.draw(x, y, HEX[usize::from(ch >> 4)], fg, bg),
                            1 => s.draw(x, y, HEX[usize::from(ch & 0xf)], fg, bg),
                            _ => s.draw(x, y, ' ', HEX_COLOR, BG),
                        }
                    } else {
                        s.draw(x, y, ' ', HEX_COLOR, BG);
                    }
                } else if col < 62 {
                    if col == 60 {
                        s.draw(x, y, DIVIDER, DIVIDER_COLOR, BG);
                    } else {
                        s.draw(x, y, ' ', DIVIDER_COLOR, BG);
                    }
                } else if col < 78 {
                    let col = col - 62;
                    if let Ok(idx) = usize::try_from(addr + col)
                        && idx < self.handle.len()
                    {
                        let ch = self.handle.r8(idx);
                        let (fg, bg) = if idx == self.cursor {
                            (BG, ASCII_COLOR)
                        } else {
                            (ASCII_COLOR, BG)
                        };
                        if (32..127).contains(&ch) {
                            s.draw(x, y, ch as char, fg, bg);
                        } else {
                            s.draw(x, y, '.', fg, bg);
                        }
                    }
                } else {
                    s.draw(x, y, ' ', ASCII_COLOR, BG);
                }
            }
        }
    }

    fn key(&mut self, code: KeyCode, _modifiers: KeyModifiers, ctx: &Context) -> bool {
        let page = isize::try_from(16 * self.bounds.h().saturating_sub(1)).unwrap_or_default();
        match code {
            KeyCode::Up | KeyCode::Char('k') => self.cursor = self.cursor.saturating_sub(16),
            KeyCode::Down | KeyCode::Char('j') => self.cursor = self.cursor.saturating_add(16),
            KeyCode::Left | KeyCode::Char('h') => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right | KeyCode::Char('l') => self.cursor = self.cursor.saturating_add(1),
            KeyCode::PageUp | KeyCode::Char('K') => {
                self.cursor = self.cursor.saturating_sub_signed(page);
            }
            KeyCode::PageDown | KeyCode::Char('J') => {
                self.cursor = self.cursor.saturating_add_signed(page);
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.handle.len(),
            _ => return false,
        }
        self.scroll_to_cursor();
        self.status(ctx);
        true
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        if self.bounds.inside(pt.x, pt.y) {
            self.cursor = usize::try_from(((pt.y - self.bounds.t) + self.scroll.get()) * 16)
                .unwrap_or_default();
            let col = pt.x - self.bounds.l;
            if (11..60).contains(&col) {
                let col = usize::try_from(col - 11).unwrap_or_default();
                if col >= 24 {
                    self.cursor = self.cursor.saturating_add(col.saturating_sub(1) / 3);
                } else {
                    self.cursor = self.cursor.saturating_add(col / 3);
                }
            } else if (62..78).contains(&col) {
                let col = usize::try_from(col - 62).unwrap_or_default();
                self.cursor = self.cursor.saturating_add(col);
            } else if col >= 78 {
                self.cursor += 15;
            }
            self.cursor = cmp::min(self.cursor, self.handle.len().saturating_sub(1));
            self.status(ctx);
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
        self.cursor = to;
        self.scroll_to_cursor();
        self.status(ctx);
    }

    fn find(&mut self, key: &str, ctx: &Context) {
        if key.is_empty() {
            self.search = None;
        } else {
            // assume key is a series of hex values
            let mut ishex = true;
            let mut state = 0;
            let mut byte = 0;
            let mut search = vec![];
            for ch in key.chars() {
                match state {
                    0 | 2 => {
                        if ch.is_ascii_hexdigit() {
                            byte = if ch > '9' {
                                (ch as u8 & 0xf) + 9
                            } else {
                                ch as u8 & 0xf
                            };
                            state = 1;
                        } else if !ch.is_whitespace() || state == 2 {
                            ishex = false;
                        }
                    }
                    1 => {
                        // second nibble of byte or '0x'
                        if ch.is_ascii_hexdigit() {
                            byte <<= 4;
                            byte |= if ch > '9' {
                                (ch as u8 & 0xf) + 9
                            } else {
                                ch as u8 & 0xf
                            };
                            search.push(byte);
                            state = 0;
                        } else if ch == 'x' && byte == 0 {
                            // 0x
                            state = 2;
                        } else if ch.is_whitespace() {
                            search.push(byte);
                            state = 0;
                        } else {
                            ishex = false;
                        }
                    }
                    _ => {}
                }
            }
            if state == 1 && ishex {
                search.push(byte);
            }
            if !ishex {
                search.clear();
                for ch in key.bytes() {
                    search.push(ch);
                }
            }
            self.search = Some(search);
            self.find_next(ctx);
        }
    }

    fn find_next(&mut self, ctx: &Context) {
        if let Some(search) = self.search.as_ref()
            && let Some(cursor) = self.handle.find(self.cursor.saturating_add(1), search)
        {
            self.cursor = cursor;
            self.scroll_to_cursor();
            self.status(ctx);
        } else {
            ctx.post(Message::NotFound);
        }
    }

    fn find_prev(&mut self, ctx: &Context) {
        if let Some(search) = self.search.as_ref()
            && let Some(cursor) = self.handle.rfind(self.cursor, search)
        {
            self.cursor = cursor;
            self.scroll_to_cursor();
            self.status(ctx);
        } else {
            ctx.post(Message::NotFound);
        }
    }

    fn export(&self, out: &mut std::fs::File) -> Result<(), std::io::Error> {
        self.handle.write(out)?;
        Ok(())
    }

    fn get_key_map(&self) -> KeyMap {
        let mut m = KeyMap::default();
        m.insert("a←↓↑→|hjkl", "move");
        m
    }
}

impl HexView {
    pub fn new(handle: &Handle) -> Self {
        Self {
            handle: handle.clone(),
            ..Default::default()
        }
    }

    fn scroll_to_cursor(&mut self) {
        self.cursor = cmp::min(self.cursor, self.handle.len().saturating_sub(1));
        let mut pos = self.scroll.get();
        if let Ok(cursor) = i32::try_from(self.cursor) {
            if cursor / 16 < pos {
                pos = cursor / 16;
            }
            if cursor / 16 > pos + self.bounds.h() - 1 {
                pos = cursor / 16 - (self.bounds.h() - 1);
            }
            self.scroll.set(pos);
        }
    }

    fn status(&self, ctx: &Context) {
        if self.cursor < self.handle.len() {
            let ch = self.handle.r8(self.cursor);
            ctx.post(Message::Status(format!(
                "${:08x} | ${:02x} ({})",
                self.cursor, ch, ch
            )));
        }
    }
}

const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
