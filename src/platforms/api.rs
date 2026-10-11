/*

Custom API calls
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle},
    platforms::{
        definition::Definition,
        fingerprints::Fingerprint,
        symbols::{Enum, Function, Interrupt, Intrinsic, Kind, Ref, Sig, Struct, Symbol},
    },
    screen::{Point, Rect, Screen},
    widgets::{Align, Entry, KeyMap, Label, Table, Widget, Window, Windowable},
    window_wrapper,
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::{cmp, collections::HashMap};

pub struct Api {
    window: Window,
    symbols: HashMap<String, Box<dyn Symbol>>,
    labels: Vec<Label>,
    inputs: Vec<Entry>,
    focused: usize,
    results: Table,
    detail: Definition,
    has_interrupts: bool,
}
window_wrapper!(Api);

impl Widget for Api {
    fn layout(&mut self, constraint: &Rect) {
        self.detail.layout(constraint);
        self.window.layout(constraint);
        let bounds = self.window.get_bounds();

        let mut col1_width = 0;
        let mut col2_width = 0;
        for (idx, label) in self.labels.iter().enumerate() {
            let w = label.min_size().w;
            if (idx & 1) == 0 {
                col1_width = cmp::max(col1_width, w);
            } else {
                col2_width = cmp::max(col2_width, w);
            }
        }
        let mut line = bounds.clone();
        line.b = line.t + 1;
        let mid = bounds.w() / 2;
        for (idx, (label, entry)) in self.labels.iter_mut().zip(&mut self.inputs).enumerate() {
            line.l = bounds.l;
            line.r = line.l + col1_width;

            if self.has_interrupts && (idx & 1) == 1 {
                // right
                line.l += mid + 1;
                line.r = line.l + col2_width;
            }
            label.layout(&line);
            line.l = line.r + 1;
            line.r = bounds.r;
            if self.has_interrupts && (idx & 1) == 0 {
                line.r = bounds.l + mid;
            }
            entry.layout(&line);
            if !self.has_interrupts || (idx & 1) == 1 {
                line.t += 1;
                line.b += 1;
            }
        }
        let mut res_c = bounds;
        res_c.t = line.t;
        self.results.layout(&res_c);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        if self.detail.visible() {
            self.detail.draw(s, origin, clip);
            return;
        }
        self.window.draw(s, origin, clip);
        let range = clip.intersect(&self.window.get_bounds().offset(origin));
        for label in &self.labels {
            label.draw(s, origin, &range);
        }
        for input in &self.inputs {
            input.draw(s, origin, &range);
        }
        self.results.draw(s, origin, &range);
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if self.detail.visible() {
            return self.detail.key(code, modifiers, ctx);
        }
        if self.inputs[self.focused].key(code, modifiers, ctx) {
            self.search();
            true
        } else if self.results.key(code, modifiers, ctx) {
            true
        } else {
            match code {
                KeyCode::Tab => {
                    self.inputs[self.focused].focus(false, ctx);
                    self.focused += 1;
                    if self.focused >= self.inputs.len() {
                        self.focused = 0;
                    }
                    self.inputs[self.focused].focus(true, ctx);
                    true
                }
                KeyCode::BackTab => {
                    self.inputs[self.focused].focus(false, ctx);
                    self.focused = self.focused.wrapping_sub(1);
                    if self.focused >= self.inputs.len() {
                        self.focused = self.inputs.len() - 1;
                    }
                    self.inputs[self.focused].focus(true, ctx);
                    true
                }
                KeyCode::Enter => {
                    if let Some(sel) = self.results.selected()
                        && let Some(sym) = self.symbols.get(sel)
                    {
                        self.detail.show();
                        self.detail.set(sym.render(sel, 0), ctx);
                    }
                    true
                }
                KeyCode::Esc => {
                    self.hide();
                    true
                }
                _ => false,
            }
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        if self.detail.visible() {
            return self.detail.mouse(pt, is_down, ctx);
        }
        if self.inputs[self.focused].mouse(pt, is_down, ctx) {
            return true;
        }
        self.results.mouse(pt, is_down, ctx)
    }

    fn get_key_map(&self) -> KeyMap {
        if self.detail.visible() {
            return self.detail.get_key_map();
        }
        let mut m = KeyMap::default();
        m.insert("b↓↑", "scroll");
        m.insert("a⏎", "select");
        m
    }
}

impl Api {
    pub fn new(data: &[u8]) -> Self {
        let handle = Handle::new_from_array(data, "", "");
        let num_symbols = handle.r32(0);
        let mut ofs = 4;
        let mut ids = HashMap::new();
        let mut symbols = HashMap::new();
        let mut has_interrupts = false;

        for _ in 0..num_symbols {
            let id = handle.r32(ofs);
            ofs += 4;
            let kind = handle.r8(ofs).into();
            ofs += 1;
            let name = handle.rps(ofs, "");
            ofs = handle.skipps(ofs);
            ids.insert(id, name.clone());
            let s: Box<dyn Symbol> = match kind {
                Kind::Intrinsic => Box::new(Intrinsic::default()),
                Kind::Enum => Box::new(Enum::default()),
                Kind::Alias => Box::new(Ref::default()),
                Kind::Struct => Box::new(Struct::new(false)),
                Kind::Union => Box::new(Struct::new(true)),
                Kind::Function => Box::new(Function::default()),
                Kind::Interrupt => {
                    has_interrupts = true;
                    Box::new(Interrupt::default())
                }
                _ => panic!("Unknown type"),
            };
            symbols.insert(name, s);
        }
        for _ in 0..num_symbols {
            let id = handle.r32(ofs);
            ofs += 4;
            if let Some(s) = symbols.get_mut(&ids[&id]) {
                s.set_size(handle.r32(ofs));
                ofs += 4;
                ofs = s.set(&handle, ofs, &ids);
            }
        }
        let mut labels = vec![];
        let mut inputs = vec![];
        labels.push(Label::new("Search:").with_align(Align::Right));
        inputs.push(Entry::default().with_focus());
        if has_interrupts {
            labels.push(Label::new("Interrupt:").with_align(Align::Right));
            inputs.push(Entry::default());
            labels.push(Label::new("AH:").with_align(Align::Right));
            inputs.push(Entry::default());
            labels.push(Label::new("AL:").with_align(Align::Right));
            inputs.push(Entry::default());
            labels.push(Label::new("DH:").with_align(Align::Right));
            inputs.push(Entry::default());
            labels.push(Label::new("DL:").with_align(Align::Right));
            inputs.push(Entry::default());
        }

        Self {
            window: Window::new(60, 12),
            symbols,
            labels,
            inputs,
            focused: 0,
            results: Table::default().with_style(Window::FG, Window::BG),
            detail: Definition::default(),
            has_interrupts,
        }
    }

    pub fn fingerprints(&self) -> Fingerprint {
        let mut root = Fingerprint::default();
        for s in &self.symbols {
            let mut cur = &mut root;
            if let Some(prints) = s.1.signature() {
                for sig in prints {
                    match sig {
                        Sig::Value(byte) => {
                            let child = cur.next_entry(*byte).or_default();
                            cur = child;
                        }
                        Sig::Wild(len) => {
                            for _ in 0..*len {
                                let child = cur.next_wild().or_default();
                                cur = child;
                            }
                        }
                    }
                }
            }
            cur.name = Some(s.0.to_owned());
        }

        root
    }

    fn search(&mut self) {
        self.results.clear();

        // are we searching for interrupts?
        if let Some(intr) = self.inputs.get(1)
            && let Some(int_num) = parse_hex(intr.get())
        {
            let ah = parse_hex(self.inputs[2].get());
            let al = parse_hex(self.inputs[3].get());
            let dh = parse_hex(self.inputs[4].get());
            let dl = parse_hex(self.inputs[5].get());
            for (k, v) in &self.symbols {
                if v.matches(int_num, ah, al, dh, dl) {
                    self.results.add(k, &v.to_string());
                }
            }
            self.results.sort();
        } else {
            let key = self.inputs[0].get().to_ascii_lowercase();
            if key.len() > 2 {
                for (k, v) in &self.symbols {
                    if k.to_ascii_lowercase().contains(&key) {
                        self.results.add(k, &v.to_string());
                    }
                }
                self.results.sort();
            }
        }
    }
}

fn parse_hex(s: &str) -> Option<u8> {
    let mut byte = None;
    for ch in s.chars() {
        if ch.is_ascii_hexdigit() {
            let nib = if ch > '9' {
                (ch as u8 & 0xf) + 9
            } else {
                ch as u8 & 0xf
            };
            if let Some(b) = byte {
                byte = Some((b << 4) | nib);
            } else {
                byte = Some(nib);
            }
        } else if !ch.is_whitespace() {
            return None;
        }
    }
    byte
}
