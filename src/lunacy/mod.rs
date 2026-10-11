/*

Main Lunacy application
@copyright 2026 Sean Kasun

*/

mod calculator;
mod crumbs;
mod export;
mod expparser;
mod filepicker;
mod find;
mod handle;
mod help;
mod jump;
mod status;
mod view;

use crate::lunacy::calculator::Calculator;
use crate::lunacy::crumbs::Crumbs;
use crate::lunacy::export::Export;
use crate::lunacy::find::Find;
use crate::lunacy::help::Help;
use crate::lunacy::jump::Jump;
use crate::lunacy::status::Status;
use crate::screen::{self, Point, Rect};
use crate::widgets::{Widget, Windowable};
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind};
pub use filepicker::FilePicker;
use futures::StreamExt;
pub use handle::Handle;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tokio::time::sleep;

pub mod gestalt {
    pub const EXPORT: u8 = 1;
    pub const JUMP: u8 = 2;
    pub const FIND: u8 = 4;
    pub const FIND_NEXT: u8 = 8;
    pub const FIND_PREV: u8 = 0x10;
    pub const FOCUS: u8 = 0x20;
}

#[derive(Default, PartialEq, Eq)]
pub enum Message {
    #[default]
    None,
    NotFound,
    Open(Handle),
    Jump(usize),
    Status(String),
    Find(String),
    Export(String),
    Refresh,
    Resize(u16, u16),
    Error(String),
    Quit,
}

#[derive(Clone)]
pub struct Context {
    tx: Sender<Message>,
}

impl Context {
    pub fn post(&self, message: Message) {
        _ = self.tx.send(message);
    }
}

pub struct Lunacy {
    bounds: Rect,
    crumbs: Crumbs,
    status: Status,
    calculator: Calculator,
    jump: Jump,
    find: Find,
    help: Help,
    export: Export,
    tx: Sender<Message>,
    rx: Receiver<Message>,
}

impl Lunacy {
    pub fn new(filename: &str) -> std::io::Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        _ = tx.send(Message::Open(Handle::new_from_disk(filename)?));
        Ok(Self {
            bounds: Rect::default(),
            crumbs: Crumbs::default(),
            status: Status::default(),
            calculator: Calculator::default(),
            jump: Jump::default(),
            find: Find::default(),
            help: Help::default(),
            export: Export::default(),
            tx,
            rx,
        })
    }

    pub async fn run(&mut self) {
        let mut screen = screen::init().expect("failed to load terminal library");
        let mut uistream = EventStream::new();
        let mut clip = screen.get_bounds().clone();
        self.layout(&clip);
        let mut refresh = true;
        let ctx = Context {
            tx: self.tx.clone(),
        };
        loop {
            if refresh {
                refresh = false;
                self.update_status();
                self.draw(&mut screen, &Point::new(0, 0), &clip);
                screen.flip();
            }
            tokio::select! {
                message = self.check_messages() => {
                    match message {
                        Message::Quit => break,
                        Message::Refresh => screen.refresh(),
                        Message::Resize(w, h) => {
                            screen.resize(w, h);
                            clip = screen.get_bounds().clone();
                            self.layout(&clip);
                        }
                        m => self.handle_message(m, &ctx),
                    }
                    refresh = true;
                }
                event = uistream.next() => {
                    match event {
                        Some(Ok(event)) => if self.handle_ui(&event, &ctx) {
                            refresh = true;
                        },
                        Some(Err(e)) => eprintln!("error: {e}"),
                        None => break,
                    }
                }
            }
        }
    }

    pub fn shutdown() {
        screen::restore();
    }

    fn handle_ui(&mut self, event: &Event, ctx: &Context) -> bool {
        match event {
            Event::Key(event) => {
                if event.kind == KeyEventKind::Press {
                    self.status.clear();
                    self.key(event.code, event.modifiers, ctx);
                    true
                } else {
                    false
                }
            }
            Event::Mouse(event) => match event.kind {
                MouseEventKind::ScrollDown => self.key(KeyCode::PageDown, event.modifiers, ctx),
                MouseEventKind::ScrollUp => self.key(KeyCode::PageUp, event.modifiers, ctx),
                MouseEventKind::ScrollLeft => self.key(KeyCode::BackTab, event.modifiers, ctx),
                MouseEventKind::ScrollRight => self.key(KeyCode::Tab, event.modifiers, ctx),
                MouseEventKind::Down(_) | MouseEventKind::Drag(_) => self.mouse(
                    &Point::new(i32::from(event.column), i32::from(event.row)),
                    true,
                    ctx,
                ),
                MouseEventKind::Up(_) => self.mouse(
                    &Point::new(i32::from(event.column), i32::from(event.row)),
                    false,
                    ctx,
                ),
                MouseEventKind::Moved => false,
            },
            Event::Resize(w, h) => {
                ctx.post(Message::Resize(*w, *h));
                true
            }
            _ => false,
        }
    }

    fn update_status(&mut self) {
        let mut keys = self.crumbs.get_key_map();
        let flags = self.crumbs.gestalt();
        if flags & gestalt::JUMP != 0 {
            keys.insert("xg", "goto");
        }
        if flags & gestalt::FIND != 0 {
            keys.insert("x/", "find");
        }
        if flags & gestalt::FIND_NEXT != 0 {
            keys.insert("xn", "find next");
        }
        if flags & gestalt::FIND_PREV != 0 {
            keys.insert("xp", "find prev");
        }
        if flags & gestalt::EXPORT != 0 {
            keys.insert("ye", "export");
        }
        keys.insert("z?", "help");
        // windows take focus and replace all keybinds
        if self.jump.visible() {
            keys = self.jump.get_key_map();
        }
        if self.find.visible() {
            keys = self.find.get_key_map();
        }
        if self.export.visible() {
            keys = self.export.get_key_map();
        }
        self.status.show_keys(&keys);
        if !self.help.visible() {
            self.help.show_keys(&keys);
        }
    }

    fn handle_message(&mut self, m: Message, ctx: &Context) {
        match m {
            Message::Open(file) => self.crumbs.push(file, ctx),
            Message::Jump(to) => self.crumbs.jump(to, ctx),
            Message::Find(val) => self.crumbs.find(&val, ctx),
            Message::Export(file) => {
                if let Err(err) = self.try_export(&file) {
                    ctx.post(Message::Error(err.to_string()));
                } else {
                    ctx.post(Message::Status("exported.".to_string()));
                }
            }
            Message::NotFound => self.status.show_error("not found"),
            Message::Error(msg) => self.status.show_error(&msg),
            Message::Status(msg) => self.status.show_message(&msg),
            _ => self.status.show_error("unhandled message"),
        }
    }

    fn try_export(&self, fname: &str) -> Result<(), std::io::Error> {
        let mut out = std::fs::File::create(fname)?;
        self.crumbs.export(&mut out)
    }

    async fn check_messages(&self) -> Message {
        loop {
            if let Ok(message) = self.rx.try_recv() {
                return message;
            }
            sleep(Duration::from_secs(1)).await; // yield
            //            yield_now().await;
        }
    }
}

impl Widget for Lunacy {
    fn layout(&mut self, constraint: &Rect) {
        let mut statusrect = constraint.clone();
        statusrect.t = statusrect.b - 1;
        self.status.layout(&statusrect);
        self.bounds = constraint.clone();
        self.bounds.b -= 1;
        self.crumbs.layout(&self.bounds);
        self.calculator.layout(&self.bounds);
        self.help.layout(&self.bounds);
        self.jump.layout(&self.bounds);
        self.find.layout(&self.bounds);
        self.export.layout(&self.bounds);
    }

    fn draw(&self, s: &mut screen::Screen, origin: &Point, clip: &Rect) {
        self.crumbs.draw(s, origin, clip);
        self.status.draw(s, origin, clip);
        if self.calculator.visible() {
            self.calculator.draw(s, origin, clip);
        }
        if self.jump.visible() {
            self.jump.draw(s, origin, clip);
        }
        if self.find.visible() {
            self.find.draw(s, origin, clip);
        }
        if self.export.visible() {
            self.export.draw(s, origin, clip);
        }
        if self.help.visible() {
            self.help.draw(s, origin, clip);
        }
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        // global ctl-c handler
        if code == KeyCode::Char('c') && modifiers == KeyModifiers::CONTROL {
            ctx.post(Message::Quit);
        }
        if code == KeyCode::Char('l') && modifiers == KeyModifiers::CONTROL {
            ctx.post(Message::Refresh);
        }
        if self.help.visible() {
            return self.help.key(code, modifiers, ctx);
        }
        if self.calculator.visible() {
            return self.calculator.key(code, modifiers, ctx);
        }
        if self.jump.visible() {
            return self.jump.key(code, modifiers, ctx);
        }
        if self.find.visible() {
            return self.find.key(code, modifiers, ctx);
        }
        if self.export.visible() {
            return self.export.key(code, modifiers, ctx);
        }
        if !self.crumbs.key(code, modifiers, ctx) {
            match code {
                KeyCode::Char('?') => self.help.show(),
                KeyCode::Char('$') => self.calculator.show(),
                KeyCode::Char('g') => {
                    if self.crumbs.gestalt() & gestalt::JUMP != 0 {
                        self.jump.show();
                    }
                }
                KeyCode::Char('/') => {
                    if self.crumbs.gestalt() & gestalt::FIND != 0 {
                        self.find.show();
                    }
                }
                KeyCode::Char('n') => {
                    if self.crumbs.gestalt() & gestalt::FIND_NEXT != 0 {
                        self.crumbs.find_next(ctx);
                    }
                }
                KeyCode::Char('p') => {
                    if self.crumbs.gestalt() & gestalt::FIND_PREV != 0 {
                        self.crumbs.find_prev(ctx);
                    }
                }
                KeyCode::Char('e') => {
                    if self.crumbs.gestalt() & gestalt::EXPORT != 0 {
                        self.export.set(self.crumbs.get_name());
                        self.export.show();
                    }
                }
                _ => return false,
            }
        }
        true
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        if self.help.visible() {
            return self.help.mouse(pt, is_down, ctx);
        }
        if self.calculator.visible() {
            return self.calculator.mouse(pt, is_down, ctx);
        }
        if self.jump.visible() {
            return self.jump.mouse(pt, is_down, ctx);
        }
        if self.find.visible() {
            return self.find.mouse(pt, is_down, ctx);
        }
        if self.export.visible() {
            return self.export.mouse(pt, is_down, ctx);
        }
        self.crumbs.mouse(pt, is_down, ctx)
    }
}
