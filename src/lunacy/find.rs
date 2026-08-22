/*

Search window for Lunacy
@copyright 2026 Sean Kasun

*/

use eframe::egui::{self, Id};

use crate::lunacy::{Context, Message};

#[derive(Default)]
pub struct Find {
    val: String,
}

impl Find {
    pub fn prompt(&mut self, ui: &egui::Ui, ctx: Context) -> bool {
        let mut resp = false;
        egui::Modal::new(Id::new("find")).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.val).request_focus();
                ui.horizontal(|ui| {
                    if ui.button("Search").clicked() {
                        ctx.post(Message::Find(self.val.clone()));
                        resp = true;
                    }
                    if ui.button("Cancel").clicked() {
                        resp = true;
                    }
                });
            });
        });
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            resp = true;
        }
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            ctx.post(Message::Find(self.val.clone()));
            resp = true;
        }
        resp
    }
}
