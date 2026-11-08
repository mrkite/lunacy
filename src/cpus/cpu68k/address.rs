/*

68k Addressing
@copyright 2026 Sean Kasun

*/

use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub},
};

use crate::cpus::Address;

#[derive(Clone, Copy, Default, Hash, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct Address68k(pub u32);

impl From<&Address68k> for u32 {
    fn from(value: &Address68k) -> Self {
        value.0
    }
}

impl Add<usize> for Address68k {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + u32::try_from(rhs).unwrap_or_default())
    }
}

impl AddAssign<usize> for Address68k {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += u32::try_from(rhs).unwrap_or_default();
    }
}

impl Sub for Address68k {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<usize> for Address68k {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - u32::try_from(rhs).unwrap_or_default())
    }
}

impl Address for Address68k {
    fn as_usize(&self) -> usize {
        usize::try_from(self.0).unwrap_or(usize::MAX)
    }
    fn parse(s: &str) -> Result<Self, &'static str> {
        let mut ofs: u32 = 0;
        for ch in s.chars() {
            if ch.is_ascii_hexdigit() {
                let nib = if ch > '9' {
                    (ch as u8 & 0xf) + 9
                } else {
                    ch as u8 & 0xf
                };
                ofs <<= 4;
                ofs |= u32::from(nib);
            } else {
                return Err("not a hex digit");
            }
        }
        Ok(Self(ofs))
    }
}

impl Display for Address68k {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:06x}", self.0)
    }
}
