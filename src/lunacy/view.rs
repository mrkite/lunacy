/*

File view for Lunacy
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle},
    plugins::foreach_plugin,
    screen::{Color, Point, Rect, Screen},
    widgets::{KeyMap, Label, Widget},
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::io::{Error, ErrorKind};

const ERROR: Color = Some(0xc0_60_60);
const BG: Color = None;

struct OpenFile {
    name: String,
    widget: Box<dyn Widget>,
}

pub struct View {
    bounds: Rect,
    supported: Vec<OpenFile>,
    selected: Option<usize>,
    handle: Handle,
}

impl View {
    pub fn new(handle: Handle, ctx: &Context) -> Self {
        let mut supported = vec![];
        let mut selected = None;
        let mut maxval = 0;
        foreach_plugin(|plugin| {
            if let Some(val) = plugin.supports(&handle) {
                if val > maxval {
                    maxval = val;
                    selected = Some(supported.len());
                }
                supported.push(OpenFile {
                    name: plugin.get_name().to_string(),
                    widget: match plugin.create(&handle, ctx) {
                        Ok(widget) => widget,
                        Err(err) => Box::new(Label::new(err).with_style(ERROR, BG)),
                    },
                });
            }
        });
        Self {
            bounds: Rect::default(),
            supported,
            selected,
            handle,
        }
    }

    pub fn get_name(&self) -> &str {
        self.handle.get_name()
    }

    pub fn get_plugin(&self) -> &str {
        if let Some(sel) = self.selected.and_then(|v| self.supported.get(v)) {
            &sel.name
        } else {
            "---"
        }
    }

    fn layout_child(&mut self) {
        if let Some(sel) = self.selected.and_then(|v| self.supported.get_mut(v)) {
            sel.widget.layout(&self.bounds);
        }
    }
}

impl Widget for View {
    fn layout(&mut self, constraint: &Rect) {
        constraint.clone_into(&mut self.bounds);
        self.layout_child();
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        let range = clip.intersect(&self.bounds.offset(origin));
        if let Some(sel) = self.selected.and_then(|v| self.supported.get(v)) {
            sel.widget.draw(s, origin, &range);
        }
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if let Some(sel) = self.selected.and_then(|v| self.supported.get_mut(v)) {
            if sel.widget.key(code, modifiers, ctx) {
                return true;
            }
            match code {
                KeyCode::Char('[') => {
                    if let Some(selected) = self.selected {
                        self.selected = Some(if selected == 0 {
                            self.supported.len().saturating_sub(1)
                        } else {
                            selected - 1
                        });
                    }
                    self.layout_child();
                    true
                }
                KeyCode::Char(']') => {
                    if let Some(selected) = self.selected {
                        self.selected = Some(selected + 1);
                        if self.selected == Some(self.supported.len()) {
                            self.selected = Some(0);
                        }
                    }
                    self.layout_child();
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        if let Some(sel) = self.selected.and_then(|v| self.supported.get_mut(v))
            && sel.widget.mouse(pt, is_down, ctx)
        {
            return true;
            // TODO: check if mouse clicked on dropdown
        }
        false
    }

    fn gestalt(&self) -> u8 {
        self.selected
            .and_then(|v| self.supported.get(v))
            .map_or(0, |sel| sel.widget.gestalt())
    }

    fn jump(&mut self, to: usize, ctx: &Context) {
        if let Some(sel) = self.selected.and_then(|v| self.supported.get_mut(v)) {
            sel.widget.jump(to, ctx);
        }
    }

    fn find(&mut self, val: &str, ctx: &Context) {
        if let Some(sel) = self.selected.and_then(|v| self.supported.get_mut(v)) {
            sel.widget.find(val, ctx);
        }
    }

    fn find_next(&mut self, ctx: &Context) {
        if let Some(sel) = self.selected.and_then(|v| self.supported.get_mut(v)) {
            sel.widget.find_next(ctx);
        }
    }

    fn find_prev(&mut self, ctx: &Context) {
        if let Some(sel) = self.selected.and_then(|v| self.supported.get_mut(v)) {
            sel.widget.find_prev(ctx);
        }
    }

    fn export(&self, out: &mut std::fs::File) -> Result<(), std::io::Error> {
        self.selected
            .and_then(|v| self.supported.get(v))
            .map_or_else(
                || Err(Error::from(ErrorKind::Unsupported)),
                |sel| sel.widget.export(out),
            )
    }

    fn get_key_map(&self) -> KeyMap {
        let mut keys = self
            .selected
            .and_then(|v| self.supported.get(v))
            .map_or_else(KeyMap::default, |sel| sel.widget.get_key_map());
        keys.insert("w[|]", "cycle view");
        keys
    }
}
