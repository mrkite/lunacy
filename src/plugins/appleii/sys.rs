/*

Apple II : Sys executable

*/

use crate::{
    cpus::{Address65816, State65816, flags},
    lunacy::{Context, Handle},
    platforms::IIgs,
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
        let mut iigs = IIgs::new(handle.get_name());
        iigs.add_section(
            "main",
            Address65816::Normal(0x2000),
            handle.read(0, handle.len()),
        );
        iigs.add_entry(State65816::new(
            Address65816::Normal(0x2000),
            flags::EMU | flags::M8 | flags::X8,
        ))?;
        iigs.disassemble();
        iigs.jump(0x2000, ctx);
        Ok(Box::new(iigs))
    }
}
