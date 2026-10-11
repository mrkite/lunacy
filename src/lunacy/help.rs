/*

Help window
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::Context,
    screen::{Point, Rect, Screen},
    widgets::{Align, KeyMap, Label, Table, Widget, Window, Windowable},
    window_wrapper,
};
use crossterm::event::{KeyCode, KeyModifiers};

pub struct Help {
    window: Window,
    header: Label,
    table: Table,
    footer: Label,
}
window_wrapper!(Help);

impl Default for Help {
    fn default() -> Self {
        Self {
            window: Window::new(40, 10),
            header: Label::new("Lunacy v2.0").with_align(Align::Center),
            table: Table::default().with_style(None, None).without_cursor(),
            footer: Label::new("© Copyright 2026 Sean Kasun").with_align(Align::Center),
        }
    }
}

impl Widget for Help {
    fn layout(&mut self, constraint: &Rect) {
        self.window.layout(constraint);
        self.layout_children();
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.window.draw(s, origin, clip);
        let range = clip.intersect(&self.window.get_bounds().offset(origin));
        self.header.draw(s, origin, &range);
        self.table.draw(s, origin, &range);
        self.footer.draw(s, origin, &range);
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if self.table.key(code, modifiers, ctx) {
            return true;
        }
        match code {
            KeyCode::Esc | KeyCode::Backspace => {
                self.hide();
                true
            }
            _ => false,
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        self.table.mouse(pt, is_down, ctx)
    }
}

impl Help {
    pub fn show_keys(&mut self, keys: &KeyMap) {
        self.table.clear();
        let mut left = keys.keys().collect::<Vec<_>>();
        left.sort();
        for key in left {
            self.table.add(&key[1..], keys[key]);
        }
        self.layout_children();
    }

    fn layout_children(&mut self) {
        let mut c = self.window.get_bounds();
        c.b = c.t + 1;
        self.header.layout(&c);
        c.t += 1;
        c.b = self.window.get_bounds().b - 1;
        self.table.layout(&c);
        c.b += 1;
        c.t = c.b - 1;
        self.footer.layout(&c);
    }
}
