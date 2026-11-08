/*

6502 Addressing
@copyright 2026 Sean Kasun

*/

use crate::cpus::Address;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub},
};

#[derive(Clone, Copy, Default, Hash, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub struct Address6502(pub u16);

impl From<&Address6502> for u16 {
    fn from(value: &Address6502) -> Self {
        value.0
    }
}

impl From<u16> for Address6502 {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<u8> for Address6502 {
    fn from(value: u8) -> Self {
        Self(u16::from(value))
    }
}

impl Add<usize> for Address6502 {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + u16::try_from(rhs).unwrap_or_default())
    }
}

impl AddAssign<usize> for Address6502 {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += u16::try_from(rhs).unwrap_or_default();
    }
}

impl Sub for Address6502 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<usize> for Address6502 {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - u16::try_from(rhs).unwrap_or_default())
    }
}

impl TryFrom<String> for Address6502 {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut ofs: u16 = 0;
        for ch in value.chars() {
            if ch.is_ascii_hexdigit() {
                let nib = if ch > '9' {
                    (ch as u8 & 0xf) + 9
                } else {
                    ch as u8 & 0xf
                };
                ofs <<= 4;
                ofs |= u16::from(nib);
            } else {
                return Err("not a hex digit");
            }
        }
        Ok(Self(ofs))
    }
}

impl Address for Address6502 {
    fn as_usize(&self) -> usize {
        usize::from(self.0)
    }
}

impl Display for Address6502 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04x}", self.0)
    }
}
