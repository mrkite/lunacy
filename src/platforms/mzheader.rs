/*

Shows the MZ header
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::Context,
    screen::{Point, Rect, Screen},
    widgets::{KeyMap, Table, Widget, Window, Windowable},
    window_wrapper,
};
use crossterm::event::{KeyCode, KeyModifiers};

pub struct MZHeader {
    window: Window,
    table: Table,
}
window_wrapper!(MZHeader);

impl Widget for MZHeader {
    fn layout(&mut self, constraint: &Rect) {
        self.window.layout(constraint);
        self.table.layout(&self.window.get_bounds());
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.window.draw(s, origin, clip);
        let range = clip.intersect(&self.window.get_bounds().offset(origin));
        self.table.draw(s, origin, &range);
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if self.table.key(code, modifiers, ctx) {
            return true;
        }
        if code == KeyCode::Esc || code == KeyCode::Backspace {
            self.hide();
            true
        } else {
            false
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        self.table.mouse(pt, is_down, ctx)
    }

    fn get_key_map(&self) -> KeyMap {
        self.table.get_key_map()
    }
}

impl Default for MZHeader {
    fn default() -> Self {
        Self {
            window: Window::new(30, 11),
            table: Table::default().with_style(None, None).without_cursor(),
        }
    }
}

impl MZHeader {
    pub fn set(&mut self, header: Vec<(String, String)>) {
        for line in header {
            self.table.add(&line.0, &line.1);
        }
    }
}
