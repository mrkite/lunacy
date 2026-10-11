/*

Simple label widget
@copyright 2026 Sean Kasun

*/

use crate::{
    screen::{Color, Point, Rect, Screen, Size},
    widgets::Widget,
};

#[derive(Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Default)]
pub struct Label {
    bounds: Rect,
    value: String,
    fg: Option<u32>,
    bg: Option<u32>,
    align: Align,
}

impl Widget for Label {
    fn min_size(&self) -> Size {
        Size::new(
            i32::try_from(self.value.chars().count()).unwrap_or(i32::MAX),
            1,
        )
    }

    fn layout(&mut self, constraint: &Rect) {
        constraint.clone_into(&mut self.bounds);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        let range = clip.intersect(&self.bounds.offset(origin));
        // fill in visible background
        for y in range.t..range.b {
            for x in range.l..range.r {
                s.draw(x, y, ' ', self.fg, self.bg);
            }
        }
        // and draw text
        let mut x = self.bounds.l + origin.x;
        let y = self.bounds.t + origin.y + self.bounds.h() / 2;
        if self.align == Align::Center {
            x += (self.bounds.w() / 2)
                .wrapping_sub_unsigned(u32::try_from(self.value.len()).unwrap_or(u32::MAX) / 2);
        } else if self.align == Align::Right {
            x += self
                .bounds
                .w()
                .wrapping_sub_unsigned(u32::try_from(self.value.len()).unwrap_or(u32::MAX));
        }
        for ch in self.value.chars() {
            if range.inside(x, y) {
                s.draw(x, y, ch, self.fg, self.bg);
            }
            x += 1;
        }
    }
}

impl Label {
    pub fn new(text: &str) -> Self {
        Self {
            value: text.to_owned(),
            fg: None,
            bg: None,
            ..Default::default()
        }
    }

    pub const fn with_style(mut self, fg: Color, bg: Color) -> Self {
        self.fg = fg;
        self.bg = bg;
        self
    }

    pub const fn with_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn set(&mut self, msg: &str) {
        msg.clone_into(&mut self.value);
    }
}
