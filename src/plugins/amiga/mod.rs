/*

Amiga plugins
@copyright 2026 Sean Kasun

*/

use crate::plugins::register_plugin;

mod adf;

pub fn register_plugins() {
    register_plugin(adf::Adf {});
}
