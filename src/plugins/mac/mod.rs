/*

68k macintosh plugins
@copyright 2026 Sean Kasun

*/

use crate::plugins::{register_charset, register_plugin};

mod code;
mod hfs;
mod macroman;
mod mfs;
mod res;
mod resources;
mod resparser;
mod xdef;

pub fn register_plugins() {
    register_plugin(code::Code {});
    register_plugin(hfs::Hfs {});
    register_plugin(mfs::Mfs {});
    register_plugin(resources::Resources {});
    register_plugin(res::Res {});
    register_plugin(xdef::Xdef {});
    register_charset("macroman", &macroman::MACROMAN);
}
