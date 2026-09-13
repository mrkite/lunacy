/*

Mac : xDEF disassembly

*/

use crate::{cpus::State68k, platforms::Mac, plugins::Plugin};

pub struct Xdef {}

impl Plugin for Xdef {
    fn get_name(&self) -> &str {
        "Mac/xDEF"
    }

    fn supports(&self, file: &crate::lunacy::File) -> Option<i16> {
        match file.get_mime() {
            "mac/res/CDEF" | "mac/res/WDEF" => Some(90),
            _ => None,
        }
    }

    fn create(
        &self,
        file: &crate::lunacy::File,
        _ctx: crate::lunacy::Context,
    ) -> Result<Box<dyn crate::widgets::Widget>, &'static str> {
        let mut mac = Mac::new(&format!("xdef_{}", file.get_name()));
        mac.add_section("", 0x10000, file.read(0, file.len()));
        mac.add_entry(State68k { addr: 0x10000 })?;
        Ok(Box::new(mac))
    }
}
