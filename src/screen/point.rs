/*

Simple point class
@copyright 2026 Sean Kasun

*/

#[derive(Default, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
