/*

Main function for Lunacy
@copyright 2026 Sean Kasun

*/
use crate::plugins::register_plugins;
use lunacy::Lunacy;
use std::{env, process::exit};

mod cpus;
mod lunacy;
mod platforms;
mod plugins;
mod screen;
mod widgets;

#[tokio::main]
async fn main() {
    if let Some(filename) = env::args().nth(1) {
        register_plugins();
        let mut lunacy = Lunacy::new(filename.as_str()).unwrap_or_else(|err| {
            eprintln!("failed to open {filename}: {err}");
            exit(-1);
        });
        lunacy.run().await;
        Lunacy::shutdown();
    } else {
        eprintln!("Usage: lunacy <filename>");
    }
}
