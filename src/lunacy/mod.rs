/*

Main Lunacy application
@copyright 2026 Sean Kasun

*/
mod crumbs;
mod expparser;
mod file;
mod find;
mod jump;
mod view;

use crate::lunacy::jump::Jump;
use crate::lunacy::{crumbs::Crumbs, find::Find};
use eframe::egui;
pub use expparser::ExpParser;
pub use file::File;
use std::{
    path::Path,
    sync::mpsc::{Receiver, Sender},
};

const RECENTS_KEY: &str = "recents";

#[derive(Clone)]
pub struct Context {
    tx: Sender<Message>,
    ctx: egui::Context,
}
impl Context {
    pub fn post(&self, message: Message) {
        _ = self.tx.send(message);
        self.ctx.request_repaint();
    }
}

pub mod gestalt {
    pub const EXPORT: u8 = 1;
    pub const JUMP: u8 = 2;
    pub const FIND: u8 = 4;
    pub const FIND_NEXT: u8 = 8;
    pub const FIND_PREV: u8 = 0x10;
    pub const MENU: u8 = 0x20;
}

#[derive(Default, PartialEq)]
pub enum Message {
    #[default]
    None,
    Open(File),
    AddRecent(String),
    Jump(usize),
    Find(String),
    FindNext,
    FindPrev,
    Save(String),
    Error(String),
}

pub struct Lunacy {
    crumbs: Crumbs,
    recents: Vec<String>,
    jump: Option<Jump>,
    find: Option<Find>,
    cmdline: Option<String>,
    tx: Sender<Message>,
    rx: Receiver<Message>,
}

impl eframe::App for Lunacy {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = Context {
            tx: self.tx.clone(),
            ctx: ui.ctx().clone(),
        };
        if let Some(file) = self.cmdline.take() {
            self.open_recent(&file, ctx.clone());
        }
        if let Ok(message) = self.rx.try_recv() {
            self.handle_message(message, ctx.clone());
        }

        self.menu_bar(ui, ctx.clone());

        egui::Panel::bottom("status").show(ui, |ui| self.crumbs.status(ui));

        egui::CentralPanel::default().show(ui, |ui| {
            self.crumbs.ui(ui, ctx.clone());
        });

        self.jump.take_if(|jump| jump.prompt(ui, ctx.clone()));
        self.find.take_if(|find| find.prompt(ui, ctx.clone()));
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, RECENTS_KEY, &self.recents);
    }
}

impl Lunacy {
    pub fn new(cc: &eframe::CreationContext, cmdline: Option<String>) -> Self {
        let recents = if let Some(storage) = cc.storage {
            eframe::get_value(storage, RECENTS_KEY).unwrap_or_default()
        } else {
            vec![]
        };
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            crumbs: Crumbs::default(),
            recents,
            jump: None,
            find: None,
            cmdline,
            tx,
            rx,
        }
    }

    fn handle_message(&mut self, m: Message, ctx: Context) {
        match m {
            Message::Open(file) => self.crumbs.push(file, ctx),
            Message::AddRecent(name) => {
                if let Some(index) = self.recents.iter().position(|value| value.eq(&name)) {
                    self.recents.remove(index);
                }
                self.recents.push(name);
                self.recents.rotate_right(1); // move to front
                if self.recents.len() > 9 {
                    self.recents.resize(9, "".into());
                }
            }
            Message::Jump(to) => {
                self.crumbs.jump(to, ctx);
            }
            Message::Find(val) => {
                self.crumbs.find(&val, ctx);
            }
            Message::FindNext => {
                self.crumbs.find_next(ctx);
            }
            Message::FindPrev => {
                self.crumbs.find_prev(ctx);
            }
            Message::Save(file) => {
                if let Err(msg) = self.try_save(&file, ctx.clone()) {
                    ctx.post(Message::Error(msg.to_string()));
                }
            }
            Message::Error(msg) => {
                eprintln!("Error: {}", msg);
            }
            Message::None => {}
        }
    }

    fn pick_file(&self, ctx: Context) {
        let task = rfd::AsyncFileDialog::new().pick_file();

        tokio::spawn(async move {
            if let Some(file) = task.await {
                let fullname = file.path().to_str().unwrap().to_string();
                ctx.post(Message::AddRecent(fullname));
                ctx.post(Message::Open(load_file(file).await));
            }
        });
    }

    fn open_recent(&self, name: &str, ctx: Context) {
        let filename = name.to_owned();
        tokio::spawn(async move {
            match File::new_from_disk(&filename) {
                Ok(file) => {
                    ctx.post(Message::AddRecent(filename));
                    ctx.post(Message::Open(file));
                }
                Err(err) => ctx.post(Message::Error(err.to_string())),
            }
        });
    }

    fn save_file(&self, ctx: Context, fname: &str) {
        let task = rfd::AsyncFileDialog::new().set_file_name(fname).save_file();

        tokio::spawn(async move {
            if let Some(file) = task.await {
                let fullname = file.path().to_str().unwrap().to_string();
                ctx.post(Message::Save(fullname));
            }
        });
    }

    fn try_save(&self, fname: &str, ctx: Context) -> Result<(), std::io::Error> {
        let mut out = std::fs::File::create(fname)?;
        self.crumbs.save(&mut out, ctx)
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui, ctx: Context) {
        let open_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::O);
        let save_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S);
        let close_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::W);
        let quit_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Q);
        let jump_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::G);
        let find_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::F);
        let find_next_shortcut =
            egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::N);
        let find_prev_shortcut =
            egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::P);

        let keys = vec![
            egui::Key::Num1,
            egui::Key::Num2,
            egui::Key::Num3,
            egui::Key::Num4,
            egui::Key::Num5,
            egui::Key::Num6,
            egui::Key::Num7,
            egui::Key::Num8,
            egui::Key::Num9,
        ];
        let recents: Vec<(String, egui::KeyboardShortcut)> = self
            .recents
            .iter()
            .zip(keys)
            .map(|(recent, key)| {
                (
                    recent.clone(),
                    egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, key),
                )
            })
            .collect();

        if ui.input_mut(|i| i.consume_shortcut(&open_shortcut)) {
            self.pick_file(ctx.clone());
        }
        if ui.input_mut(|i| i.consume_shortcut(&save_shortcut))
            && self.crumbs.gestalt() & gestalt::EXPORT != 0
        {
            self.save_file(ctx.clone(), self.crumbs.get_name());
        }
        if ui.input_mut(|i| i.consume_shortcut(&close_shortcut)) && self.crumbs.can_close() {
            self.crumbs.close();
        }
        if ui.input_mut(|i| i.consume_shortcut(&quit_shortcut)) {
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if ui.input_mut(|i| i.consume_shortcut(&jump_shortcut))
            && self.crumbs.gestalt() & gestalt::JUMP != 0
        {
            self.jump = Some(Jump::default());
        }
        if ui.input_mut(|i| i.consume_shortcut(&find_shortcut))
            && self.crumbs.gestalt() & gestalt::FIND != 0
        {
            self.find = Some(Find::default());
        }
        if ui.input_mut(|i| i.consume_shortcut(&find_next_shortcut))
            && self.crumbs.gestalt() & gestalt::FIND_NEXT != 0
        {
            ctx.post(Message::FindNext);
        }
        if ui.input_mut(|i| i.consume_shortcut(&find_prev_shortcut))
            && self.crumbs.gestalt() & gestalt::FIND_PREV != 0
        {
            ctx.post(Message::FindPrev);
        }
        for recent in recents.iter() {
            if ui.input_mut(|i| i.consume_shortcut(&recent.1)) {
                self.open_recent(&recent.0, ctx.clone());
            }
        }

        egui::Panel::top("menubar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add(
                            egui::Button::new("Open...")
                                .shortcut_text(ui.ctx().format_shortcut(&open_shortcut)),
                        )
                        .clicked()
                    {
                        self.pick_file(ctx.clone());
                    }
                    ui.menu_button("Recents", |ui| {
                        for recent in recents.iter() {
                            let path = Path::new(&recent.0).file_name().unwrap();
                            if ui
                                .add(
                                    egui::Button::new(path.to_str().unwrap())
                                        .shortcut_text(ui.ctx().format_shortcut(&recent.1)),
                                )
                                .clicked()
                            {
                                self.open_recent(&recent.0, ctx.clone());
                            }
                        }
                    });
                    if ui
                        .add_enabled(
                            self.crumbs.can_close(),
                            egui::Button::new("Close")
                                .shortcut_text(ui.ctx().format_shortcut(&close_shortcut)),
                        )
                        .clicked()
                    {
                        self.crumbs.close();
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            self.crumbs.gestalt() & gestalt::EXPORT != 0,
                            egui::Button::new("Save...")
                                .shortcut_text(ui.ctx().format_shortcut(&save_shortcut)),
                        )
                        .clicked()
                    {
                        self.save_file(ctx.clone(), self.crumbs.get_name());
                    }
                    ui.separator();
                    if ui
                        .add(
                            egui::Button::new("Quit")
                                .shortcut_text(ui.ctx().format_shortcut(&quit_shortcut)),
                        )
                        .clicked()
                    {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui
                        .add_enabled(
                            self.crumbs.gestalt() & gestalt::JUMP != 0,
                            egui::Button::new("Jump...")
                                .shortcut_text(ui.ctx().format_shortcut(&jump_shortcut)),
                        )
                        .clicked()
                    {
                        self.jump = Some(Jump::default());
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            self.crumbs.gestalt() & gestalt::FIND != 0,
                            egui::Button::new("Find...")
                                .shortcut_text(ui.ctx().format_shortcut(&find_shortcut)),
                        )
                        .clicked()
                    {
                        self.find = Some(Find::default());
                    }
                    if ui
                        .add_enabled(
                            self.crumbs.gestalt() & gestalt::FIND_NEXT != 0,
                            egui::Button::new("Find Next")
                                .shortcut_text(ui.ctx().format_shortcut(&find_next_shortcut)),
                        )
                        .clicked()
                    {
                        ctx.post(Message::FindNext);
                    }
                    if ui
                        .add_enabled(
                            self.crumbs.gestalt() & gestalt::FIND_PREV != 0,
                            egui::Button::new("Find Prev")
                                .shortcut_text(ui.ctx().format_shortcut(&find_prev_shortcut)),
                        )
                        .clicked()
                    {
                        ctx.post(Message::FindPrev);
                    }
                });
                if self.crumbs.gestalt() & gestalt::MENU != 0 {
                    self.crumbs.menu(ui, ctx);
                }
            });
        });
    }
}

async fn load_file(handle: rfd::FileHandle) -> File {
    let data = handle.read().await;
    File::new_from_array(&data, &handle.file_name(), "")
}
