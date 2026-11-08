/*

65816 Addressing
@copyright 2026 Sean Kasun

*/

use crate::cpus::Address;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::Display,
    ops::{Add, AddAssign, Sub},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Address65816 {
    Normal(u32),
    B(u16),
    D(u8),
}

impl Default for Address65816 {
    fn default() -> Self {
        Self::Normal(0)
    }
}

impl From<Address65816> for u32 {
    fn from(value: Address65816) -> Self {
        match value {
            Address65816::Normal(v) => v,
            Address65816::B(v) => Self::from(v),
            Address65816::D(v) => Self::from(v),
        }
    }
}

impl From<u32> for Address65816 {
    fn from(value: u32) -> Self {
        Self::Normal(value)
    }
}

impl From<u16> for Address65816 {
    fn from(value: u16) -> Self {
        Self::Normal(u32::from(value))
    }
}

impl From<u8> for Address65816 {
    fn from(value: u8) -> Self {
        Self::Normal(u32::from(value))
    }
}

impl PartialOrd for Address65816 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Address65816 {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::Normal(v) => match other {
                Self::Normal(ov) => v.cmp(ov),
                _ => Ordering::Less,
            },
            Self::B(v) => match other {
                Self::B(ov) => v.cmp(ov),
                _ => Ordering::Less,
            },
            Self::D(v) => match other {
                Self::D(ov) => v.cmp(ov),
                _ => Ordering::Less,
            },
        }
    }
}

impl Add for Address65816 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::Normal(v) => match rhs {
                Self::Normal(ov) => Self::Normal(v + ov),
                Self::B(ov) => Self::Normal(v + u32::from(ov)),
                Self::D(ov) => Self::Normal(v + u32::from(ov)),
            },
            Self::B(v) => match rhs {
                Self::Normal(ov) => Self::Normal(u32::from(v) + ov),
                Self::B(ov) => Self::B(v + ov),
                Self::D(ov) => Self::B(v + u16::from(ov)),
            },
            Self::D(v) => match rhs {
                Self::Normal(ov) => Self::Normal(u32::from(v) + ov),
                Self::B(ov) => Self::B(u16::from(v) + ov),
                Self::D(ov) => Self::D(v + ov),
            },
        }
    }
}

impl Add<u32> for Address65816 {
    type Output = Self;
    fn add(self, rhs: u32) -> Self::Output {
        match self {
            Self::Normal(v) => Self::Normal(v + rhs),
            Self::B(v) => Self::B(v + (rhs & 0xffff) as u16),
            Self::D(v) => Self::D(v + (rhs & 0xff) as u8),
        }
    }
}

impl Add<usize> for Address65816 {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        self + u32::try_from(rhs).unwrap_or(u32::MAX)
    }
}

impl AddAssign for Address65816 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl AddAssign<usize> for Address65816 {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl Sub for Address65816 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Self::Normal(v) => match rhs {
                Self::Normal(ov) => Self::Normal(v - ov),
                Self::B(ov) => Self::Normal(v - u32::from(ov)),
                Self::D(ov) => Self::Normal(v - u32::from(ov)),
            },
            Self::B(v) => match rhs {
                Self::Normal(ov) => Self::Normal(u32::from(v) - ov),
                Self::B(ov) => Self::B(v - ov),
                Self::D(ov) => Self::B(v - u16::from(ov)),
            },
            Self::D(v) => match rhs {
                Self::Normal(ov) => Self::Normal(u32::from(v) - ov),
                Self::B(ov) => Self::B(u16::from(v) - ov),
                Self::D(ov) => Self::D(v - ov),
            },
        }
    }
}

impl Sub<usize> for Address65816 {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        match self {
            Self::Normal(v) => Self::Normal(v - u32::try_from(rhs).unwrap_or_default()),
            Self::B(v) => Self::B(v - u16::try_from(rhs).unwrap_or_default()),
            Self::D(v) => Self::D(v - u8::try_from(rhs).unwrap_or_default()),
        }
    }
}

impl Display for Address65816 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal(v) => write!(f, "{:02x}/{:04x}", v >> 16, v & 0xffff),
            Self::B(v) => write!(f, "B:{v:04x}"),
            Self::D(v) => write!(f, "D:{v:02x}"),
        }
    }
}

impl Into<String> for Address65816 {
    fn into(self) -> String {
        self.to_string()
    }
}

impl TryFrom<String> for Address65816 {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut bank: u32 = 0;
        let mut ofs: u32 = 0;
        let mut is_special = false;

        for ch in value.chars() {
            if ch.is_ascii_hexdigit() {
                let nib = if ch > '9' {
                    (ch as u8 & 0xf) + 9
                } else {
                    ch as u8 & 0xf
                };
                ofs <<= 4;
                ofs |= u32::from(nib);
            } else if ch == '/' {
                if ofs & !0xff != 0 {
                    return Err("bank is larger than 8 bits");
                }
                bank = ofs << 16;
                ofs = 0;
            } else if ch == ':' {
                is_special = true;
                bank = ofs;
                ofs = 0;
            } else {
                return Err("not a hex digit or a bank:");
            }
        }
        if is_special {
            if bank == 0xb {
                if ofs & !0xffff != 0 {
                    Err("B: larger than 16 bits")
                } else {
                    Ok(Self::B((ofs & 0xffff) as u16))
                }
            } else if bank == 0xd {
                if ofs & !0xff != 0 {
                    Err("D: larger than 8 bits")
                } else {
                    Ok(Self::D((ofs & 0xff) as u8))
                }
            } else {
                Err("Not B: or D:")
            }
        } else {
            Ok(Self::Normal(bank | ofs))
        }
    }
}

impl Address65816 {
    pub const fn k(self) -> u32 {
        match self {
            Self::Normal(v) => v & 0xff_0000,
            _ => 0,
        }
    }

    pub const fn or(&mut self, byte: u32) {
        if let Self::Normal(v) = self {
            *v |= byte;
        }
    }
}

impl Address for Address65816 {
    fn as_usize(&self) -> usize {
        match self {
            Self::Normal(v) => usize::try_from(*v).unwrap_or(usize::MAX),
            Self::B(v) => usize::from(*v),
            Self::D(v) => usize::from(*v),
        }
    }
}
