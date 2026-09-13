/*

Apple II : Binary executable
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::State6502,
    lunacy::{Context, File},
    platforms::IIe,
    plugins::Plugin,
    widgets::Widget,
};

pub struct Bin {}

impl Plugin for Bin {
    fn get_name(&self) -> &str {
        "Apple/Bin"
    }
    fn supports(&self, file: &File) -> Option<i16> {
        let mime = file.get_mime();
        if mime == "dos33/4" { Some(90) } else { None }
    }
    fn create(&self, file: &File, _ctx: Context) -> Result<Box<dyn Widget>, &'static str> {
        let mut iie = IIe::new(&format!("bin_{}", file.get_name()));
        let dest = file.r16(0);
        let len = usize::from(file.r16(2));
        iie.add_section("main", u32::from(dest), file.read(4, len));
        iie.add_entry(State6502 { addr: dest })?;
        Ok(Box::new(iie))
    }
}
