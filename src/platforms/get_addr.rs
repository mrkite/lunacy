/*

Get Address for Disassembly
@copyright 2026 Sean Kasun

*/

use crate::lunacy::ExpParser;
use eframe::egui::{self, Color32, Id};

const ERROR_COLOR: Color32 = Color32::from_rgb(0xaa, 0x66, 0x66);

#[derive(Default)]
pub struct GetAddress {
    to: String,
    error: Option<String>,
    parser: ExpParser,
}

impl GetAddress {
    pub fn prompt(&mut self, ui: &egui::Ui) -> Option<(bool, i32)> {
        let mut resp = None;
        egui::Modal::new(Id::new("get_addr")).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label("Address:");
                ui.text_edit_singleline(&mut self.to).request_focus();
                if let Some(err) = &self.error {
                    ui.colored_label(ERROR_COLOR, err);
                }
                ui.horizontal(|ui| {
                    if ui.button("Disassemble").clicked() {
                        resp = self.calculate();
                    }
                    if ui.button("Cancel").clicked() {
                        resp = Some((false, 0));
                    }
                });
            });
        });
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            resp = Some((false, 0));
        }
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            resp = self.calculate();
        }
        resp
    }

    fn calculate(&mut self) -> Option<(bool, i32)> {
        match self.parser.calculate(&self.to) {
            Ok(val) => {
                self.to.clear();
                Some((true, val))
            }
            Err(msg) => {
                self.error = Some(msg.to_string());
                None
            }
        }
    }
}
