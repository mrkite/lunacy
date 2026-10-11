/*

Simple rectangle class
@copyright 2026 Sean Kasun

*/

use crate::screen::point::Point;
use std::cmp;

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Rect {
    pub t: i32,
    pub l: i32,
    pub b: i32,
    pub r: i32,
}

impl Rect {
    pub const fn new(l: i32, t: i32, r: i32, b: i32) -> Self {
        Self { t, l, b, r }
    }

    pub const EMPTY: Self = Self {
        t: 0,
        l: 0,
        b: 0,
        r: 0,
    };

    pub const fn w(&self) -> i32 {
        self.r - self.l
    }

    pub const fn h(&self) -> i32 {
        self.b - self.t
    }

    pub const fn inside(&self, x: i32, y: i32) -> bool {
        x >= self.l && x < self.r && y >= self.t && y < self.b
    }

    pub fn intersect(&self, with: &Self) -> Self {
        Self::new(
            cmp::max(self.l, with.l),
            cmp::max(self.t, with.t),
            cmp::min(self.r, with.r),
            cmp::min(self.b, with.b),
        )
    }

    pub const fn inset(&self, dx: i32, dy: i32) -> Self {
        Self::new(self.l + dx, self.t + dy, self.r - dx, self.b - dy)
    }

    pub const fn offset(&self, by: &Point) -> Self {
        Self::new(self.l + by.x, self.t + by.y, self.r + by.x, self.b + by.y)
    }

    pub const fn clear(&mut self) {
        self.l = 0;
        self.t = 0;
        self.r = 0;
        self.b = 0;
    }
}
