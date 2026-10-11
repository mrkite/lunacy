/*

Shows a symbol's defition
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::Context,
    screen::{Point, Rect, Screen},
    widgets::{KeyMap, TreeGrid, TreeRow, Widget, Window, Windowable},
    window_wrapper,
};
use crossterm::event::{KeyCode, KeyModifiers};

pub struct Definition {
    window: Window,
    tree: TreeGrid,
}
window_wrapper!(Definition);

impl Widget for Definition {
    fn layout(&mut self, constraint: &Rect) {
        self.window.layout(constraint);
        self.tree.layout(&self.window.get_bounds());
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.window.draw(s, origin, clip);
        let range = clip.intersect(&self.window.get_bounds().offset(origin));
        self.tree.draw(s, origin, &range);
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if self.tree.key(code, modifiers, ctx) {
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
        self.tree.mouse(pt, is_down, ctx)
    }

    fn get_key_map(&self) -> KeyMap {
        self.tree.get_key_map()
    }
}

impl Default for Definition {
    fn default() -> Self {
        Self {
            window: Window::new(60, 11),
            tree: TreeGrid::new(&["Key", "Type", "Ofs"]),
        }
    }
}

impl Definition {
    pub fn set(&mut self, row: TreeRow, ctx: &Context) {
        self.tree.clear();
        self.tree.add(row, ctx);
        self.tree.layout(&self.window.get_bounds());
    }
}
