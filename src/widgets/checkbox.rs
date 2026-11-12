/*

Checkbox widget
@copyright 2026 Sean Kasun

*/

use crossterm::event::{KeyCode, KeyModifiers};

use crate::{
    lunacy::{Context, gestalt},
    screen::{Color, Point, Rect, Screen},
    widgets::Widget,
};

const FG: Color = Some(0xca_e6_c5);
const BG: Color = Some(0x2c_48_4c);

#[derive(Default)]
pub struct Checkbox {
    bounds: Rect,
    checked: bool,
    focused: bool,
}

impl Widget for Checkbox {
    fn layout(&mut self, constraint: &Rect) {
        constraint.clone_into(&mut self.bounds);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        let range = clip.intersect(&self.bounds.offset(origin));

        let x = self.bounds.l + origin.x;
        let y = self.bounds.t + origin.y;
        if range.inside(x, y) {
            if self.checked {
                s.draw(x, y, '☑', FG, BG);
            } else {
                s.draw(x, y, '☐', FG, BG);
            }
        }
        if self.focused
            && let Ok(x) = u16::try_from(self.bounds.l + origin.x)
            && let Ok(y) = u16::try_from(self.bounds.t + origin.y)
        {
            s.show_cursor(x, y);
        }
    }

    fn key(&mut self, code: KeyCode, _modifiers: KeyModifiers, _ctx: &Context) -> bool {
        match code {
            KeyCode::Char(' ') | KeyCode::Char('x') => {
                self.checked = !self.checked;
                true
            }
            _ => false,
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, _ctx: &Context) -> bool {
        if is_down && self.bounds.inside(pt.x, pt.y) {
            self.focused = true;
            self.checked = !self.checked;
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

impl Checkbox {
    pub fn get(&self) -> bool {
        self.checked
    }
}
