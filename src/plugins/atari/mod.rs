/*

Atari ST plugins
@copyright 2026 Sean Kasun

*/

use crate::plugins::{register_charset, register_plugin};

mod atascii;
mod atr;
mod bas;

pub fn register_plugins() {
    register_plugin(atr::Atr {});
    register_plugin(bas::Bas {});
    register_charset("atascii", &atascii::ATASCII);
}
