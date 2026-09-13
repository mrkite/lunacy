/*

DOS plugin registration
@copyright 2026 Sean Kasun

*/

mod fat;
mod mbr;
mod mz;
mod pcchars;

use crate::plugins::{register_charset, register_plugin};

pub fn register_plugins() {
    register_plugin(mz::Mz {});
    register_plugin(fat::Fat {});
    register_plugin(mbr::Mbr {});

    register_charset("ibmpc", &pcchars::PC);
}
