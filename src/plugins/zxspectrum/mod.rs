/*

ZXSpectrum plugins
@copyright 2026 Sean Kasun

*/

use crate::plugins::{register_charset, register_plugin};

mod bas;
mod spectrumchars;
mod tap;
mod tzx;

pub fn register_plugins() {
    register_plugin(tap::Tap {});
    register_plugin(tzx::Tzx {});
    register_plugin(bas::Bas {});

    register_charset("zxspectrum", &spectrumchars::SPECTRUM);
}
