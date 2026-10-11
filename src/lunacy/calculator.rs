/*

Simple calculator
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, expparser::ExpParser},
    screen::{Point, Rect, Screen},
    widgets::{Entry, KeyMap, Text, Widget, Window, Windowable},
    window_wrapper,
};
use crossterm::event::{KeyCode, KeyModifiers};

pub struct Calculator {
    window: Window,
    input: Entry,
    results: Text,
    parser: ExpParser,
}
window_wrapper!(Calculator);

impl Default for Calculator {
    fn default() -> Self {
        Self {
            window: Window::new(40, 10),
            input: Entry::default().with_focus(),
            results: Text::default().with_style(None, None),
            parser: ExpParser::default(),
        }
    }
}

impl Widget for Calculator {
    fn layout(&mut self, constraint: &Rect) {
        self.window.layout(constraint);
        let bounds = self.window.get_bounds();
        let mut inp_c = bounds.clone();
        inp_c.b = inp_c.t + 1;
        self.input.layout(&inp_c);
        let mut res_c = bounds;
        res_c.t += 1;
        self.results.layout(&res_c);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.window.draw(s, origin, clip);
        let range = clip.intersect(&self.window.get_bounds().offset(origin));
        self.input.draw(s, origin, &range);
        self.results.draw(s, origin, &range);
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if self.input.key(code, modifiers, ctx) {
            return true;
        }
        if self.results.key(code, modifiers, ctx) {
            return true;
        }
        match code {
            KeyCode::Esc => {
                self.hide();
                true
            }
            KeyCode::Enter => {
                let msg = self.input.get();
                match self.parser.calculate(msg) {
                    Ok(val) => {
                        let fmt = format!("{msg} = {val} (${val:x})\n");
                        self.input.clear();
                        self.results.append(&fmt);
                    }
                    Err(msg) => self.results.append(&format!("{msg}\n")),
                }
                true
            }
            _ => false,
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        self.input.mouse(pt, is_down, ctx) || self.results.mouse(pt, is_down, ctx)
    }

    fn get_key_map(&self) -> KeyMap {
        let mut m = self.input.get_key_map();
        m.insert("a↵", "calculate");
        m.insert("b⎋", "close");
        m
    }
}
