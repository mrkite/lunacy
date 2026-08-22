/*

C64 plugins
@copyright 2026 Sean Kasun

*/

use crate::plugins::{register_charset, register_plugin};

mod bas;
mod d64;
mod petscii;

pub fn register_plugins() {
    register_plugin(d64::D64 {});
    register_plugin(bas::Bas {});
    register_charset("petscii", &petscii::PETSCII);
}
