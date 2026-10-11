/*

Window to add label
@copyright 2026 Sean Kasun

*/

use std::cmp;

use crate::{
    lunacy::Context,
    screen::{Point, Rect, Screen},
    widgets::{Entry, KeyMap, Label, Widget, Window, Windowable},
    window_wrapper,
};
use crossterm::event::{KeyCode, KeyModifiers};

pub struct LabelWin {
    window: Window,
    entries: Vec<Entry>,
    labels: Vec<Label>,
    focus: usize,
    submitted: Option<(String, String)>,
}
window_wrapper!(LabelWin);

impl Widget for LabelWin {
    fn layout(&mut self, constraint: &Rect) {
        self.window.layout(constraint);
        let mut lw = 0;
        for label in &self.labels {
            lw = cmp::max(lw, label.min_size().w);
        }
        let mut cl = self.window.get_bounds().clone();
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
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.window.draw(s, origin, clip);
        let range = clip.intersect(&self.window.get_bounds().offset(origin));
        for (label, entry) in self.labels.iter().zip(&self.entries) {
            label.draw(s, origin, &range);
            entry.draw(s, origin, &range);
        }
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
                self.hide();
                self.submitted = Some((
                    self.entries[0].get().to_string(),
                    self.entries[1].get().to_string(),
                ));
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
        if let Some(entry) = self.entries.get(self.focus) {
            entry.get_key_map()
        } else {
            KeyMap::default()
        }
    }
}

impl Default for LabelWin {
    fn default() -> Self {
        let labels = vec![Label::new("Address:"), Label::new("Label:")];
        let entries = vec![Entry::default(), Entry::default().with_focus()];
        Self {
            window: Window::new(60, 4),
            labels,
            entries,
            focus: 1,
            submitted: None,
        }
    }
}

impl LabelWin {
    pub fn set(&mut self, addr: &str) {
        self.entries[0].set(addr);
        self.entries[1].set("");
    }
    pub fn take(&mut self) -> Option<(String, String)> {
        self.submitted.take()
    }
}
