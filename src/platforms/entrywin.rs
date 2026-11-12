/*

Window to add entry with options
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{self, Address},
    lunacy::Context,
    screen::{Color, Point, Rect, Screen},
    widgets::{Checkbox, Entry, KeyMap, Label, Widget, Window, Windowable},
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::cmp;

const ERROR_COLOR: Color = Some(0x88_33_33);

pub struct EntryWin<ADDR: Address> {
    window: Window,
    address: Entry,
    labels: Vec<Label>,
    checkboxes: Vec<Checkbox>,
    error: Label,
    focus: usize,
    submitted: Option<(ADDR, u32)>,
}

impl<ADDR: Address> Windowable for EntryWin<ADDR> {
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

impl<ADDR: Address<Error = &'static str>> Widget for EntryWin<ADDR> {
    fn layout(&mut self, constraint: &Rect) {
        self.window.layout(constraint);
        let mut lw = 0;
        for label in &self.labels {
            lw = cmp::max(lw, label.min_size().w);
        }
        let mut cl = self.window.get_bounds();
        for label in &mut self.labels {
            cl.b = cl.t + 1;
            cl.l = self.window.get_bounds().l;
            cl.r = cl.l + lw;
            label.layout(&cl);
            cl.t = cl.b;
        }
        cl.t = self.window.get_bounds().t;
        cl.b = cl.t + 1;
        cl.l = cl.r + 1;
        cl.r = self.window.get_bounds().r;
        self.address.layout(&cl);

        for check in &mut self.checkboxes {
            cl.t = cl.b;
            cl.b = cl.t + 1;
            check.layout(&cl);
        }
        cl.t = cl.b;
        cl.l = self.window.get_bounds().l;
        cl.b = self.window.get_bounds().b;
        self.error.layout(&cl);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.window.draw(s, origin, clip);
        let range = clip.intersect(&self.window.get_bounds().offset(origin));
        for label in &self.labels {
            label.draw(s, origin, &range);
        }
        self.address.draw(s, origin, &range);
        for check in &self.checkboxes {
            check.draw(s, origin, &range);
        }
        self.error.draw(s, origin, &range);
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if self.focus == 0 && self.address.key(code, modifiers, ctx) {
            return true;
        }
        if self.focus > 0
            && let Some(check) = self.checkboxes.get_mut(self.focus - 1)
            && check.key(code, modifiers, ctx)
        {
            return true;
        }
        match code {
            KeyCode::Tab => {
                if self.focus == 0 {
                    self.address.focus(false, ctx);
                } else {
                    self.checkboxes[self.focus - 1].focus(false, ctx);
                }
                self.focus += 1;
                if self.focus > self.checkboxes.len() {
                    self.focus = 0;
                }
                if self.focus == 0 {
                    self.address.focus(true, ctx);
                } else {
                    self.checkboxes[self.focus - 1].focus(true, ctx);
                }
                true
            }
            KeyCode::BackTab => {
                if self.focus == 0 {
                    self.address.focus(false, ctx);
                } else {
                    self.checkboxes[self.focus - 1].focus(false, ctx);
                }
                self.focus = self.focus.wrapping_sub(1);
                if self.focus > self.checkboxes.len() {
                    self.focus = self.checkboxes.len();
                }
                if self.focus == 0 {
                    self.address.focus(true, ctx);
                } else {
                    self.checkboxes[self.focus - 1].focus(true, ctx);
                }
                true
            }
            KeyCode::Enter => {
                match ADDR::try_from(self.address.get().to_string()) {
                    Ok(addr) => {
                        self.hide();
                        self.submitted = Some((addr, self.to_flags()));
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
        if self.address.mouse(pt, is_down, ctx) {
            return true;
        }
        for check in &mut self.checkboxes {
            if check.mouse(pt, is_down, ctx) {
                return true;
            }
        }
        false
    }

    fn get_key_map(&self) -> KeyMap {
        if self.focus == 0 {
            self.address.get_key_map()
        } else {
            self.checkboxes
                .get(self.focus - 1)
                .map_or_else(KeyMap::default, Widget::get_key_map)
        }
    }
}

impl<ADDR: Address> Default for EntryWin<ADDR> {
    fn default() -> Self {
        let labels = vec![
            Label::new("Address:"),
            Label::new("Emulation:"),
            Label::new("8-bit M"),
            Label::new("8-bit X"),
        ];
        let address = Entry::default().with_focus();
        let checkboxes = vec![
            Checkbox::default(),
            Checkbox::default(),
            Checkbox::default(),
        ];
        Self {
            window: Window::new(60, 6),
            address,
            labels,
            checkboxes,
            error: Label::new("").with_style(ERROR_COLOR, None),
            focus: 0,
            submitted: None,
        }
    }
}

impl<ADDR: Address> EntryWin<ADDR> {
    fn to_flags(&self) -> u32 {
        let mut flags = 0;
        if self.checkboxes[0].get() {
            flags |= cpus::flags::EMU;
        }
        if self.checkboxes[1].get() {
            flags |= cpus::flags::M8;
        }
        if self.checkboxes[2].get() {
            flags |= cpus::flags::X8;
        }
        flags
    }

    pub fn set(&mut self, addr: Option<ADDR>) {
        if let Some(addr) = addr {
            self.address.set(&addr.to_string());
        } else {
            self.address.set("");
        }
        self.error.set("");
    }

    pub const fn take(&mut self) -> Option<(ADDR, u32)> {
        self.submitted.take()
    }
}
