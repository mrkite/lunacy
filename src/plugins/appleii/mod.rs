/*

Apple II plugin registration
@copyright 2026 Sean Kasun

*/
mod bas;
mod bin;
mod dos;
mod iiechars;
mod int;
mod omf;
mod prodos;
mod prodos_files;
mod sys;

use crate::plugins::{register_charset, register_plugin};

pub fn register_plugins() {
    register_plugin(bas::Bas {});
    register_plugin(bin::Bin {});
    register_plugin(dos::Dos {});
    register_plugin(int::Int {});
    register_plugin(omf::Omf {});
    register_plugin(prodos::Prodos {});
    register_plugin(sys::Sys {});

    register_charset("iie", &iiechars::IIE);
}
