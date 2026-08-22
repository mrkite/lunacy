/*

TRS-80 plugins
@copyright 2026 Sean Kasun

*/

use crate::plugins::register_plugin;

mod bas;
mod jvc;

pub fn register_plugins() {
    register_plugin(jvc::Jvc {});
    register_plugin(bas::Bas {});
}
