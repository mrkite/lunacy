/*

Vic-20 plugins
@copyright 2026 Sean Kasun

*/

use crate::plugins::register_plugin;

mod bas;

pub fn register_plugins() {
    register_plugin(bas::Bas {});
}
