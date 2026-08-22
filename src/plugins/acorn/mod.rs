/*

Acorn and BBC Micro plugins
@copyright 2026 Sean Kasun

*/
mod adfs;
mod bas;
mod uef;

use crate::plugins::register_plugin;

pub fn register_plugins() {
    register_plugin(adfs::Adfs {});
    register_plugin(bas::Bas {});
    register_plugin(uef::Uef {});
}
