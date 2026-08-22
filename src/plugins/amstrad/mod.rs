/*

Amstrad plugins
@copyright 2026 Sean Kasun

*/

use crate::plugins::register_plugin;

mod dsk;

pub fn register_plugins() {
    register_plugin(dsk::Dsk {});
}
