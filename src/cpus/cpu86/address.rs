/*

x86 Addressing
@copyright 2026 Sean Kasun

*/

use std::{
    cmp,
    fmt::Display,
    hash::Hash,
    ops::{Add, AddAssign, Sub},
};

use serde::{Deserialize, Serialize};

use crate::cpus::Address;

#[derive(Clone, Copy, Default, Eq, Serialize, Deserialize)]
pub struct Address86 {
    pub seg: u16,
    pub ofs: u16,
}

impl From<&Address86> for u32 {
    fn from(value: &Address86) -> Self {
        (Self::from(value.seg) << 4) + Self::from(value.ofs)
    }
}

impl From<Address86> for u32 {
    fn from(value: Address86) -> Self {
        (Self::from(value.seg) << 4) + Self::from(value.ofs)
    }
}

impl From<u32> for Address86 {
    fn from(value: u32) -> Self {
        let seg = u16::try_from(value >> 4).unwrap_or(u16::MAX);
        Self {
            seg,
            ofs: u16::try_from(value - (u32::from(seg) << 4)).unwrap_or(u16::MAX),
        }
    }
}

impl PartialOrd for Address86 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Address86 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        u32::from(self).cmp(&other.into())
    }
}

impl PartialEq for Address86 {
    fn eq(&self, other: &Self) -> bool {
        u32::from(self).eq(&u32::from(other))
    }
}

impl Hash for Address86 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.into());
    }
}

impl Add<usize> for Address86 {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        let sum = u32::from(&self) + u32::try_from(rhs).unwrap_or(u32::MAX);
        Self {
            seg: self.seg,
            ofs: u16::try_from(sum - (u32::from(self.seg) << 4)).unwrap_or(u16::MAX),
        }
    }
}

impl AddAssign<usize> for Address86 {
    fn add_assign(&mut self, rhs: usize) {
        self.ofs += u16::try_from(rhs).unwrap_or_default();
    }
}

impl Sub for Address86 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let sum = u32::from(&self) - u32::from(&rhs);
        let minseg = cmp::min(self.seg, rhs.seg);
        Self {
            seg: minseg,
            ofs: u16::try_from(sum - (u32::from(minseg) << 4)).unwrap_or(u16::MAX),
        }
    }
}

impl Sub<usize> for Address86 {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        let sum = u32::from(&self) - u32::try_from(rhs).unwrap_or(u32::MAX);
        Self {
            seg: self.seg,
            ofs: u16::try_from(sum - (u32::from(self.seg) << 4)).unwrap_or(u16::MAX),
        }
    }
}

impl Address for Address86 {
    fn as_usize(&self) -> usize {
        usize::try_from(u32::from(self)).unwrap_or(usize::MAX)
    }
    fn parse(s: &str) -> Result<Self, &'static str> {
        let mut seg: u16 = 0;
        let mut ofs: u16 = 0;
        for ch in s.chars() {
            if ch.is_ascii_hexdigit() {
                let nib = if ch > '9' {
                    (ch as u8 & 0xf) + 9
                } else {
                    ch as u8 & 0xf
                };
                ofs <<= 4;
                ofs |= u16::from(nib);
            } else if ch == ':' {
                seg = ofs;
                ofs = 0;
            } else {
                return Err("not a hex or ':'");
            }
        }
        Ok(Self { seg, ofs })
    }
}

impl Display for Address86 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04x}:{:04x}", self.seg, self.ofs)
    }
}

impl Address86 {
    pub fn relative(self, seg: u16) -> String {
        format!(
            "{:04x}:{:04x}",
            seg,
            (u32::from(self.seg) << 4) + u32::from(self.ofs) - (u32::from(seg) << 4)
        )
    }
}
