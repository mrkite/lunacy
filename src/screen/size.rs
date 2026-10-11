/*

Simple size class
@copyright 2026 Sean Kasun

*/

#[derive(Default, Clone)]
pub struct Size {
    pub w: i32,
    pub h: i32,
}

impl Size {
    pub const fn new(w: i32, h: i32) -> Self {
        Self { w, h }
    }
}
