/*

Plugin system for Lunacy
Right now it's all staticly included, but if rust ever
gets a standard ABI or better auto-registry, we'll change this.
@copyright 2026 Sean Kasun

*/

mod acorn;
mod amiga;
mod amstrad;
mod appleii;
mod ascii;
mod atari;
mod c64;
mod dos;
mod hex;
mod mac;
mod tandy;
mod vic20;
mod zxspectrum;

use crate::lunacy::File;
use crate::{
    lunacy::Context,
    plugins::{ascii::ASCII, hex::Hex},
    widgets::Widget,
};
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub trait Plugin {
    fn get_name(&self) -> &str;
    fn supports(&self, file: &File) -> Option<i16>;
    fn create(&self, file: &File, ctx: Context) -> Result<Box<dyn Widget>, &'static str>;
}

lazy_static! {
    static ref PLUGINS: Arc<Mutex<Vec<Box<dyn Plugin + Send>>>> = Arc::new(Mutex::new(Vec::new()));
    static ref CHARSETS: Arc<Mutex<HashMap<String, &'static [char; 256]>>> =
        Arc::new(Mutex::new(HashMap::new()));
    static ref APIS: Arc<Mutex<HashMap<String, &'static [u8]>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub fn register_plugin(plugin: impl Plugin + 'static + Send) {
    let mut registry = PLUGINS.lock().unwrap();
    registry.push(Box::new(plugin));
}

pub fn register_charset(name: &str, charset: &'static [char; 256]) {
    let mut registry = CHARSETS.lock().unwrap();
    registry.insert(name.to_string(), charset);
}

pub fn foreach_plugin<F>(mut f: F)
where
    F: FnMut(&dyn Plugin),
{
    let registry = PLUGINS.lock().unwrap();
    for h in registry.iter() {
        f(h.as_ref());
    }
}

pub fn get_charset(name: &str) -> &'static [char; 256] {
    let registry = CHARSETS.lock().unwrap();
    if let Some(set) = registry.get(name) {
        set
    } else {
        &ASCII
    }
}

pub fn register_plugins() {
    register_plugin(Hex {});
    acorn::register_plugins();
    amiga::register_plugins();
    amstrad::register_plugins();
    appleii::register_plugins();
    atari::register_plugins();
    c64::register_plugins();
    dos::register_plugins();
    mac::register_plugins();
    tandy::register_plugins();
    vic20::register_plugins();
    zxspectrum::register_plugins();
}
