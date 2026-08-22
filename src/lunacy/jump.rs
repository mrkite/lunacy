/*

Jump to address window for Lunacy
@copyright 2026 Sean Kasun

*/

use crate::lunacy::{Context, Message, expparser::ExpParser};
use eframe::egui::{self, Color32, Id};

const ERROR_COLOR: Color32 = Color32::from_rgb(0xaa, 0x66, 0x66);

#[derive(Default)]
pub struct Jump {
    to: String,
    error: Option<String>,
    parser: ExpParser,
}

impl Jump {
    pub fn prompt(&mut self, ui: &egui::Ui, ctx: Context) -> bool {
        let mut resp = false;
        egui::Modal::new(Id::new("jump")).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label("Go to:");
                ui.text_edit_singleline(&mut self.to).request_focus();
                if let Some(err) = &self.error {
                    ui.colored_label(ERROR_COLOR, err);
                }
                ui.horizontal(|ui| {
                    if ui.button("Jump").clicked() {
                        resp = self.calculate(ctx.clone());
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
            resp = self.calculate(ctx);
        }
        resp
    }

    fn calculate(&mut self, ctx: Context) -> bool {
        match self.parser.calculate(&self.to) {
            Ok(val) => {
                self.to.clear();
                ctx.post(Message::Jump(val as usize));
                true
            }
            Err(msg) => {
                self.error = Some(msg.to_string());
                false
            }
        }
    }
}
