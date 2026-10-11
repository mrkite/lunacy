/*

Export As window
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Message},
    screen::{Point, Rect, Screen},
    widgets::{Entry, KeyMap, Label, Widget, Window, Windowable},
    window_wrapper,
};
use crossterm::event::{KeyCode, KeyModifiers};

pub struct Export {
    window: Window,
    label: Label,
    input: Entry,
}
window_wrapper!(Export);

impl Default for Export {
    fn default() -> Self {
        Self {
            window: Window::new(50, 3),
            label: Label::new("Export as:"),
            input: Entry::default(),
        }
    }
}

impl Widget for Export {
    fn layout(&mut self, constraint: &Rect) {
        self.window.layout(constraint);
        let bounds = self.window.get_bounds();
        let lw = self.label.min_size().w;
        let mut c = Rect::new(bounds.l, bounds.t, bounds.l + lw, bounds.t + 1);
        self.label.layout(&c);
        c.l += lw + 1;
        c.r = bounds.r;
        self.input.layout(&c);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.window.draw(s, origin, clip);
        let range = clip.intersect(&self.window.get_bounds().offset(origin));
        self.label.draw(s, origin, &range);
        self.input.draw(s, origin, &range);
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if self.input.key(code, modifiers, ctx) {
            return true;
        }
        match code {
            KeyCode::Esc => {
                self.hide();
                true
            }
            KeyCode::Enter => {
                self.hide();
                let fname = self.input.get();
                ctx.post(Message::Export(fname.to_owned()));
                true
            }
            _ => false,
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        self.input.mouse(pt, is_down, ctx)
    }

    fn get_key_map(&self) -> KeyMap {
        let mut m = KeyMap::default();
        m.insert("a↵", "export");
        m.insert("b⎋", "cancel");
        m
    }
}

impl Export {
    pub fn set(&mut self, fname: &str) {
        self.input.set(fname);
    }
}
