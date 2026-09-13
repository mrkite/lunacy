/*

Master widget definition
@copyright 2026 Sean Kasun

*/

mod basic;
mod disassembly;
mod error;
mod hex;
mod image;
mod tab;
mod treegrid;

use crate::lunacy::Context;
use eframe::egui;

pub use basic::Basic;
pub use disassembly::Disassembly;
pub use error::ErrorView;
pub use hex::HexView;
pub use image::ImageView;
pub use tab::Tab;
pub use tab::TabResponse;
pub use treegrid::TreeGrid;
pub use treegrid::TreeRow;

pub trait Widget {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: Context);

    fn status(&self, _ui: &mut egui::Ui) {}

    fn gestalt(&self) -> u8 {
        0
    }

    fn jump(&mut self, _to: usize, _ctx: Context) {}

    fn find(&mut self, _val: &str, _ctx: Context) {}

    fn find_next(&mut self, _ctx: Context) {}

    fn find_prev(&mut self, _ctx: Context) {}

    fn menu(&mut self, _ui: &mut egui::Ui, _ctx: Context) {}

    fn save(&self, _out: &mut std::fs::File, _ctx: Context) -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "unsupported",
        ))
    }
}
