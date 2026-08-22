/*

68k macintosh plugins
@copyright 2026 Sean Kasun

*/

use crate::plugins::{register_charset, register_plugin};

mod hfs;
mod icon;
mod macroman;
mod mfs;
mod res;
mod resources;

pub fn register_plugins() {
    register_plugin(hfs::Hfs {});
    register_plugin(mfs::Mfs {});
    register_plugin(resources::Resources {});
    register_plugin(icon::Icon {});
    register_plugin(res::Res {});
    register_charset("macroman", &macroman::MACROMAN);
}
