/*

Search window for Lunacy
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Message},
    screen::{Point, Rect, Screen},
    widgets::{Entry, KeyMap, Label, Widget, Window, Windowable},
    window_wrapper,
};
use crossterm::event::{KeyCode, KeyModifiers};

pub struct Find {
    window: Window,
    label: Label,
    input: Entry,
}
window_wrapper!(Find);

impl Default for Find {
    fn default() -> Self {
        Self {
            window: Window::new(30, 3),
            label: Label::new("Find:"),
            input: Entry::default().with_focus(),
        }
    }
}

impl Widget for Find {
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
                let key = self.input.get();
                ctx.post(Message::Find(key.to_owned()));
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
        m.insert("a↵", "find");
        m.insert("b⎋", "cancel");
        m
    }
}
