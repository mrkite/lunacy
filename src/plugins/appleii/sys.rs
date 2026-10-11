/*

Apple II : Sys executable

*/

use crate::{
    cpus::State6502,
    lunacy::{Context, Handle},
    platforms::IIe,
    plugins::Plugin,
    widgets::Widget,
};

pub struct Sys {}

impl Plugin for Sys {
    fn get_name(&self) -> &'static str {
        "Apple/Sys"
    }
    fn supports(&self, handle: &Handle) -> Option<i16> {
        let mime = handle.get_mime();
        if mime.starts_with("prodos/ff:") {
            Some(90)
        } else {
            None
        }
    }
    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let mut iie = IIe::new(handle.get_name());
        iie.add_section("main", 0x2000, handle.read(0, handle.len()));
        iie.add_entry(State6502::new(0x2000))?;
        iie.disassemble();
        iie.jump(0x2000, ctx);
        Ok(Box::new(iie))
    }
}
