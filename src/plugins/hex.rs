/*

Default binary plugin
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle},
    plugins::Plugin,
    widgets::{self, Widget},
};

pub struct Hex {}

impl Plugin for Hex {
    fn get_name(&self) -> &'static str {
        "Hex"
    }

    fn supports(&self, _file: &Handle) -> Option<i16> {
        // we support every file.  Choose a # in the middle so it won't be the default though
        Some(50)
    }

    fn create(&self, handle: &Handle, _ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        Ok(Box::new(widgets::HexView::new(handle)))
    }
}
