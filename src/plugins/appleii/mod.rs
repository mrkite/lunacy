/*

Apple II plugin registration
@copyright 2026 Sean Kasun

*/
mod bas;
mod dos;
mod iieaddresses;
mod iiechars;
mod int;
mod pnt;
mod prodos;
mod prodos_files;

use crate::plugins::{register_charset, register_plugin};

pub fn register_plugins() {
    register_plugin(bas::Bas {});
    register_plugin(dos::Dos {});
    register_plugin(int::Int {});
    register_plugin(pnt::Pnt {});
    register_plugin(prodos::Prodos {});

    register_charset("iie", &iiechars::IIE);
}
