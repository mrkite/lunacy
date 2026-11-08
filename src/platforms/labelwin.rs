/*

Window to add label
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::Address,
    lunacy::Context,
    screen::{Color, Point, Rect, Screen},
    widgets::{Entry, KeyMap, Label, Widget, Window, Windowable},
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::cmp;

const ERROR_COLOR: Color = Some(0x88_33_33);

pub struct LabelWin<ADDR: Address> {
    window: Window,
    entries: Vec<Entry>,
    labels: Vec<Label>,
    error: Label,
    focus: usize,
    submitted: Option<(ADDR, String)>,
}

impl<ADDR: Address> Windowable for LabelWin<ADDR> {
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

impl<ADDR: Address<Error = &'static str>> Widget for LabelWin<ADDR> {
    fn layout(&mut self, constraint: &Rect) {
        self.window.layout(constraint);
        let mut lw = 0;
        for label in &self.labels {
            lw = cmp::max(lw, label.min_size().w);
        }
        let mut cl = self.window.get_bounds();
        for (label, entry) in self.labels.iter_mut().zip(&mut self.entries) {
            cl.b = cl.t + 1;
            cl.l = self.window.get_bounds().l;
            cl.r = cl.l + lw;
            label.layout(&cl);
            cl.l = cl.r + 1;
            cl.r = self.window.get_bounds().r;
            entry.layout(&cl);
            cl.t = cl.b;
        }
        cl.l = self.window.get_bounds().l;
        cl.r = self.window.get_bounds().r;
        cl.b = self.window.get_bounds().b;
        self.error.layout(&cl);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.window.draw(s, origin, clip);
        let range = clip.intersect(&self.window.get_bounds().offset(origin));
        for (label, entry) in self.labels.iter().zip(&self.entries) {
            label.draw(s, origin, &range);
            entry.draw(s, origin, &range);
        }
        self.error.draw(s, origin, &range);
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if let Some(entry) = self.entries.get_mut(self.focus)
            && entry.key(code, modifiers, ctx)
        {
            return true;
        }
        match code {
            KeyCode::Tab => {
                self.entries[self.focus].focus(false, ctx);
                self.focus += 1;
                if self.focus >= self.entries.len() {
                    self.focus = 0;
                }
                self.entries[self.focus].focus(true, ctx);
                true
            }
            KeyCode::BackTab => {
                self.entries[self.focus].focus(false, ctx);
                self.focus = self.focus.wrapping_sub(1);
                if self.focus >= self.entries.len() {
                    self.focus = self.entries.len().saturating_sub(1);
                }
                self.entries[self.focus].focus(true, ctx);
                true
            }
            KeyCode::Enter => {
                match ADDR::try_from(self.entries[0].get().to_string()) {
                    Ok(addr) => {
                        self.hide();
                        self.submitted = Some((addr, self.entries[1].get().to_string()));
                    }
                    Err(err) => self.error.set(err),
                }
                true
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.hide();
                true
            }
            _ => false,
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        for entry in &mut self.entries {
            if entry.mouse(pt, is_down, ctx) {
                return true;
            }
        }
        false
    }

    fn get_key_map(&self) -> KeyMap {
        self.entries
            .get(self.focus)
            .map_or_else(KeyMap::default, Widget::get_key_map)
    }
}

impl<ADDR: Address> Default for LabelWin<ADDR> {
    fn default() -> Self {
        let labels = vec![Label::new("Address:"), Label::new("Label:")];
        let entries = vec![Entry::default(), Entry::default().with_focus()];
        Self {
            window: Window::new(60, 5),
            labels,
            entries,
            error: Label::new("").with_style(ERROR_COLOR, None),
            focus: 1,
            submitted: None,
        }
    }
}

impl<ADDR: Address> LabelWin<ADDR> {
    pub fn set(&mut self, addr: Option<ADDR>) {
        if let Some(addr) = addr {
            self.entries[0].set(&addr.to_string());
        } else {
            self.entries[0].set("");
        }
        self.entries[1].set("");
        self.error.set("");
    }
    pub const fn take(&mut self) -> Option<(ADDR, String)> {
        self.submitted.take()
    }
}
