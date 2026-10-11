/*

Base widget definition
@copyright 2026 Sean Kasun

*/

mod basic;
mod blank;
mod disassembly;
mod entry;
mod hex;
mod label;
mod link;
mod scroll;
mod table;
mod text;
mod treegrid;
mod treerow;
mod window;

use crate::lunacy::Context;
use crate::screen::Point;
use crate::screen::Rect;
use crate::screen::Screen;
use crate::screen::Size;
pub use basic::Basic;
pub use blank::Blank;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
pub use disassembly::Disassembly;
pub use entry::Entry;
pub use hex::HexView;
pub use label::Align;
pub use label::Label;
pub use link::Link;
pub use scroll::Scroll;
use std::collections::BTreeMap;
pub use table::Table;
pub use text::Text;
pub use treegrid::TreeGrid;
pub use treerow::TreeRow;
pub use window::Window;
pub use window::Windowable;

pub type KeyMap = BTreeMap<&'static str, &'static str>;

#[allow(unused_variables)]
pub trait Widget {
    fn min_size(&self) -> Size {
        Size::new(1, 1)
    }
    fn layout(&mut self, constraint: &Rect);
    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect);
    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        false
    }
    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        false
    }
    fn gestalt(&self) -> u8 {
        0
    }
    fn jump(&mut self, to: usize, ctx: &Context) {}
    fn find(&mut self, val: &str, ctx: &Context) {}
    fn find_next(&mut self, ctx: &Context) {}
    fn find_prev(&mut self, ctx: &Context) {}
    fn export(&self, out: &mut std::fs::File) -> Result<(), std::io::Error> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "unsupported",
        ))
    }
    fn focus(&mut self, focus: bool, ctx: &Context) {}
    fn get_key_map(&self) -> KeyMap {
        KeyMap::default()
    }
}
