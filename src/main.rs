use std::env;

/*

Main function for Lunacy
@copyright 2026 Sean Kasun

*/
use crate::plugins::register_plugins;
use eframe::egui;
use lunacy::Lunacy;

mod cpus;
mod lunacy;
mod platforms;
mod plugins;
mod widgets;

#[tokio::main]
async fn main() -> eframe::Result {
    register_plugins();

    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/lunacy.png"))
        .unwrap_or_default();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "Lunacy",
        options,
        Box::new(|cc| Ok(Box::new(Lunacy::new(cc, env::args().nth(1))))),
    )
}
