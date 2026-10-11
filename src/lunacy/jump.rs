/*

Jump to address window for Lunacy
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Message, expparser::ExpParser},
    screen::{Color, Point, Rect, Screen},
    widgets::{Entry, KeyMap, Label, Widget, Window, Windowable},
    window_wrapper,
};
use crossterm::event::{KeyCode, KeyModifiers};

const ERROR_COLOR: Color = Some(0xaa_66_66);

pub struct Jump {
    window: Window,
    label: Label,
    input: Entry,
    error: Label,
    parser: ExpParser,
}
window_wrapper!(Jump);

impl Default for Jump {
    fn default() -> Self {
        Self {
            window: Window::new(30, 4),
            label: Label::new("Go to:"),
            input: Entry::default().with_focus(),
            error: Label::new("").with_style(ERROR_COLOR, None),
            parser: ExpParser::default(),
        }
    }
}

impl Widget for Jump {
    fn layout(&mut self, constraint: &Rect) {
        self.window.layout(constraint);
        let bounds = self.window.get_bounds();
        let lw = self.label.min_size().w;
        let mut c = Rect::new(bounds.l, bounds.t, bounds.l + lw, bounds.t + 1);
        self.label.layout(&c);
        c.l += lw + 1;
        c.r = bounds.r;
        self.input.layout(&c);
        c.l = bounds.l;
        c.t += 1;
        c.b = bounds.b;
        self.error.layout(&c);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.window.draw(s, origin, clip);
        let range = clip.intersect(&self.window.get_bounds().offset(origin));
        self.label.draw(s, origin, &range);
        self.input.draw(s, origin, &range);
        self.error.draw(s, origin, &range);
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
                let msg = self.input.get();
                if msg.is_empty() {
                    return true;
                }
                match self.parser.calculate(msg) {
                    Ok(val) => {
                        self.input.clear();
                        self.hide();
                        ctx.post(Message::Jump(
                            usize::try_from(val.cast_unsigned()).unwrap_or_default(),
                        ));
                    }
                    Err(msg) => {
                        self.error.set(msg);
                    }
                }
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
        m.insert("a↵", "go");
        m.insert("b⎋", "cancel");
        m
    }
}
