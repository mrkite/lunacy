/*

Base window widget
@copyright 2026 Sean Kasun

*/

use crate::{
    screen::{Color, Point, Rect, Screen, Size},
    widgets::Widget,
};

const TL: char = '┌';
const BL: char = '└';
const VERT: char = '│';
const TR: char = '┐';
const BR: char = '┘';
const HORZ: char = '─';

#[derive(Default)]
pub struct Window {
    bounds: Rect,
    desired_size: Size,
    visible: bool,
}

impl Widget for Window {
    fn min_size(&self) -> Size {
        self.desired_size.clone()
    }

    fn layout(&mut self, constraint: &Rect) {
        constraint.clone_into(&mut self.bounds);
        // center
        self.bounds.t += self.bounds.h() / 2 - self.desired_size.h / 2;
        self.bounds.l += self.bounds.w() / 2 - self.desired_size.w / 2;
        self.bounds.b = self.bounds.t + self.desired_size.h;
        self.bounds.r = self.bounds.l + self.desired_size.w;
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        let range = clip.intersect(&self.bounds.offset(origin));
        for y in range.t..range.b {
            for x in range.l..range.r {
                let ch = if x == self.bounds.l + origin.x {
                    if y == self.bounds.t + origin.y {
                        TL
                    } else if y == self.bounds.b + origin.y - 1 {
                        BL
                    } else {
                        VERT
                    }
                } else if x == self.bounds.r + origin.x - 1 {
                    if y == self.bounds.t {
                        TR
                    } else if y == self.bounds.b + origin.y - 1 {
                        BR
                    } else {
                        VERT
                    }
                } else if y == self.bounds.t + origin.y || y == self.bounds.b + origin.y - 1 {
                    HORZ
                } else {
                    ' '
                };
                s.draw(x, y, ch, Self::FG, Self::BG);
            }
        }
    }
}

impl Window {
    pub const FG: Color = Some(0x00_00_01);
    pub const BG: Color = Some(0x45_74_7a);

    pub fn new(w: i32, h: i32) -> Self {
        Self {
            desired_size: Size { w, h },
            ..Default::default()
        }
    }

    pub const fn get_bounds(&self) -> Rect {
        self.bounds.inset(1, 1)
    }
}
impl Windowable for Window {
    fn visible(&self) -> bool {
        self.visible
    }

    fn show(&mut self) {
        self.visible = true;
    }

    fn hide(&mut self) {
        self.visible = false;
    }
}

#[macro_export]
macro_rules! window_wrapper {
    ($structname: ident) => {
        impl $crate::widgets::Windowable for $structname {
            fn visible(&self) -> bool {
                self.window.visible()
            }
            fn show(&mut self) {
                self.window.show();
            }
            fn hide(&mut self) {
                self.window.hide();
            }
        }
    };
}

pub trait Windowable {
    fn visible(&self) -> bool;
    fn show(&mut self);
    fn hide(&mut self);
}
