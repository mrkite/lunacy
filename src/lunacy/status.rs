/*

Statusbar
@copyright 2026 Sean Kasun

*/

use crate::{
    screen::{Color, Point, Rect, Screen},
    widgets::{KeyMap, Widget},
};

const FG: Color = Some(0x00_00_01);
const BG: Color = Some(0x5d_57_6b);
const KEY: Color = Some(0xd4_96_a7);
const ERROR: Color = Some(0xe0_80_80);

#[derive(Default)]
pub struct Status {
    bounds: Rect,
    keys: KeyMap,
    error: Option<String>,
    message: Option<String>,
}

impl Widget for Status {
    fn layout(&mut self, constraint: &Rect) {
        self.bounds = constraint.clone();
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        let range = clip.intersect(&self.bounds.offset(origin));
        if range.t >= range.b {
            return;
        }
        let mut x = range.l - (self.bounds.l + origin.x);
        if x >= range.l && x < range.r {
            s.draw(x, range.t, ' ', FG, BG);
        }
        x += 1;
        if let Some(err) = self.error.as_ref() {
            for ch in err.chars() {
                if x >= range.l && x < range.r {
                    s.draw(x, range.t, ch, ERROR, BG);
                }
                x += 1;
            }
        } else if let Some(msg) = self.message.as_ref() {
            for ch in msg.chars() {
                if x >= range.l && x < range.r {
                    s.draw(x, range.t, ch, KEY, BG);
                }
                x += 1;
            }
        }
        s.draw(x, range.t, ' ', KEY, BG);
        x += 1; // pad

        // find available space
        let mut start = range.r;
        for key in &self.keys {
            let w = i32::try_from(
                key.0
                    .chars()
                    .count()
                    .saturating_add(key.1.chars().count())
                    .saturating_add(1),
            )
            .unwrap_or_default();
            if start - w >= x {
                start -= w;
            } else {
                break;
            }
        }

        while x < start {
            s.draw(x, range.t, ' ', FG, BG);
            x += 1;
        }

        for key in &self.keys {
            // strip off first character, it's only used for sorting
            for ch in key.0.chars().skip(1) {
                if x >= range.l && x < range.r {
                    if ch == '|' {
                        s.draw(x, range.t, ch, FG, BG);
                    } else {
                        s.draw(x, range.t, ch, KEY, BG);
                    }
                }
                x += 1;
            }
            if x >= range.l && x < range.r {
                s.draw(x, range.t, ':', FG, BG);
            }
            x += 1;
            for ch in key.1.chars() {
                if x >= range.l && x < range.r {
                    s.draw(x, range.t, ch, FG, BG);
                }
                x += 1;
            }
            if x >= range.l && x < range.r {
                s.draw(x, range.t, ' ', FG, BG);
            }
            x += 1;
        }
        while x < range.r {
            s.draw(x, range.t, ' ', FG, BG);
            x += 1;
        }
    }
}

impl Status {
    pub fn clear(&mut self) {
        self.error = None;
    }

    pub fn show_keys(&mut self, keys: &KeyMap) {
        keys.clone_into(&mut self.keys);
    }

    pub fn show_error(&mut self, err: &str) {
        self.error = Some(err.to_owned());
    }

    pub fn show_message(&mut self, msg: &str) {
        self.message = Some(msg.to_owned());
    }
}
