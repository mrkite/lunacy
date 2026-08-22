/*

Parser error
@copyright 2026 Sean Kasun

*/

use crate::widgets::Widget;

pub struct ErrorView {
    message: String,
}

impl Widget for ErrorView {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _ctx: crate::lunacy::Context) {
        ui.label(&self.message);
    }
}

impl ErrorView {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}
