/*

Default binary plugin
@copyright 2026 Sean Kasun

*/
use crate::{lunacy::Context, plugins::Plugin, widgets};

pub struct Hex {}

impl Plugin for Hex {
    fn get_name(&self) -> &str {
        "Hex"
    }

    fn supports(&self, _file: &super::File) -> Option<i16> {
        // we support every file.  Choose a # in the middle so it won't be the default though
        Some(50)
    }

    fn create(
        &self,
        file: &super::File,
        _ctx: Context,
    ) -> Result<Box<dyn widgets::Widget>, &'static str> {
        Ok(Box::new(widgets::HexView::new(file)))
    }
}
