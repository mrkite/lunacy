/*

TUI file picker
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::Context,
    screen::{Point, Rect, Screen},
    widgets::{Entry, KeyMap, Label, Table, Widget, Window, Windowable},
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::{fs, path::PathBuf};

pub struct FilePicker {
    window: Window,
    files: Table,
    label: Label,
    entry: Entry,
    focus: bool,
    folder: PathBuf,
    picked: Option<PathBuf>,
}

impl Default for FilePicker {
    fn default() -> Self {
        let mut files = Table::default();
        let folder = "";
        show_files(&mut files, folder);

        Self {
            window: Window::new(60, 13),
            files,
            label: Label::new("Filename:"),
            entry: Entry::default().with_focus(),
            focus: true,
            folder: PathBuf::from(folder),
            picked: None,
        }
    }
}

impl Widget for FilePicker {
    fn layout(&mut self, constraint: &Rect) {
        self.window.layout(constraint);
        let bounds = self.window.get_bounds();
        let mut c = Rect::new(bounds.l, bounds.t, bounds.r, bounds.b - 1);
        self.files.layout(&c);
        let lw = self.label.min_size().w;
        c.t = c.b;
        c.b += 1;
        c.r = c.l + lw;
        self.label.layout(&c);
        c.l += lw + 1;
        c.r = bounds.r;
        self.entry.layout(&c);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.window.draw(s, origin, clip);
        let range = clip.intersect(&self.window.get_bounds().offset(origin));
        self.files.draw(s, origin, &range);
        self.label.draw(s, origin, &range);
        self.entry.draw(s, origin, &range);
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if self.focus && self.entry.key(code, modifiers, ctx) {
            return true;
        }
        if self.files.key(code, modifiers, ctx) {
            self.entry.focus(false, ctx);
            self.focus = false;
            return true;
        }
        match code {
            KeyCode::Tab | KeyCode::BackTab => {
                if self.focus {
                    self.entry.focus(false, ctx);
                }
                self.focus = !self.focus;
                if self.focus {
                    self.entry.focus(true, ctx);
                }
                true
            }
            KeyCode::Backspace => {
                if self.folder.pop()
                    && let Some(file) = self.folder.to_str()
                {
                    show_files(&mut self.files, file);
                } else {
                    self.hide();
                }
                true
            }
            KeyCode::Esc => {
                self.hide();
                true
            }
            KeyCode::Enter => {
                if self.focus {
                    self.folder.push(self.entry.get());
                } else if let Some(file) = self.files.selected() {
                    self.folder.push(file);
                }
                if self.folder.is_dir()
                    && let Some(file) = self.folder.to_str()
                {
                    show_files(&mut self.files, file);
                } else {
                    self.picked = Some(self.folder.clone());
                    self.folder.pop();
                    self.hide();
                }
                true
            }
            _ => false,
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        if self.entry.mouse(pt, is_down, ctx) {
            return true;
        }
        self.files.mouse(pt, is_down, ctx)
    }

    fn get_key_map(&self) -> KeyMap {
        let mut m = KeyMap::default();
        if self.focus {
            m.insert("a↵", "open");
        } else {
            m.insert("a↵|␣", "open");
        }
        m.insert("b⎋", "cancel");
        m
    }
}

impl Windowable for FilePicker {
    fn visible(&self) -> bool {
        self.window.visible()
    }
    fn show(&mut self) {
        self.picked = None;
        self.window.show();
    }
    fn hide(&mut self) {
        self.window.hide();
    }
}

impl FilePicker {
    pub fn set(&mut self, filename: &str) {
        self.entry.set(filename);
    }
    pub const fn get(&self) -> Option<&PathBuf> {
        self.picked.as_ref()
    }
}

fn show_files(table: &mut Table, path: &str) {
    table.clear();
    let path = if path.is_empty() { "." } else { path };
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry
                && let Some(name) = entry.file_name().to_str()
                && (entry.path().is_dir()
                    || entry.path().extension().is_some_and(|e| e == "lunacy"))
            {
                let filetype = if entry.path().is_dir() { "dir" } else { "" };
                table.add(name, filetype);
            }
        }
    }
    table.sort();
}
