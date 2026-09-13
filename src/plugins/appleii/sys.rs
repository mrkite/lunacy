/*

Apple II : Sys executable

*/

use crate::{
    cpus::State6502,
    lunacy::{Context, File},
    platforms::IIe,
    plugins::Plugin,
    widgets::Widget,
};

pub struct Sys {}

impl Plugin for Sys {
    fn get_name(&self) -> &str {
        "Apple/Sys"
    }
    fn supports(&self, file: &File) -> Option<i16> {
        let mime = file.get_mime();
        if mime.starts_with("prodos/ff:") {
            Some(90)
        } else {
            None
        }
    }
    fn create(&self, file: &File, _ctx: Context) -> Result<Box<dyn Widget>, &'static str> {
        let mut iie = IIe::new(&format!("sys_{}", file.get_name()));
        iie.add_section("main", 0x2000, file.read(0, file.len()));
        iie.add_entry(State6502 { addr: 0x2000 })?;
        Ok(Box::new(iie))
    }
}
