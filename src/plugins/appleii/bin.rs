/*

Apple II : Binary executable
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::State6502,
    lunacy::{Context, Handle},
    platforms::IIe,
    plugins::Plugin,
    widgets::Widget,
};

pub struct Bin {}

impl Plugin for Bin {
    fn get_name(&self) -> &'static str {
        "Apple/Bin"
    }
    fn supports(&self, handle: &Handle) -> Option<i16> {
        let mime = handle.get_mime();
        if mime == "dos33/4" { Some(90) } else { None }
    }
    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let mut iie = IIe::new(handle.get_name());
        let dest = handle.r16(0);
        let len = usize::from(handle.r16(2));
        iie.add_section("main", u32::from(dest), handle.read(4, len));
        iie.add_entry(State6502::new(dest))?;
        iie.disassemble();
        iie.jump(dest.into(), ctx);
        Ok(Box::new(iie))
    }
}
