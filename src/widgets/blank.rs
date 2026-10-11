/*

Blank placeholder widget
@copyright 2026 Sean Kasun

*/

use crate::{
    screen::{Point, Rect, Screen},
    widgets::Widget,
};

#[derive(Default)]
pub struct Blank {}

impl Widget for Blank {
    fn layout(&mut self, _constraint: &Rect) {}
    fn draw(&self, _s: &mut Screen, _origin: &Point, _clip: &Rect) {}
}
