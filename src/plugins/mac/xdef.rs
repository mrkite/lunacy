/*

Mac : xDEF disassembly

*/

use crate::{
    cpus::State68k,
    lunacy::{Context, Handle},
    platforms::Mac,
    plugins::Plugin,
    widgets::Widget,
};

pub struct Xdef {}

impl Plugin for Xdef {
    fn get_name(&self) -> &'static str {
        "Mac/xDEF"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        match handle.get_mime() {
            "mac/res/CDEF" | "mac/res/WDEF" => Some(90),
            _ => None,
        }
    }

    fn create(&self, handle: &Handle, _ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let mut mac = Mac::new(handle.get_name());
        mac.add_section("", 0x10000, handle.read(0, handle.len()));
        mac.add_entry(State68k::new(0x10000))?;
        mac.disassemble();
        Ok(Box::new(mac))
    }
}
