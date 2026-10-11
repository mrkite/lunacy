/*

Symbols used by API
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::Handle,
    widgets::{Blank, Label, TreeRow},
};
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(PartialEq, Eq)]
pub enum Kind {
    Intrinsic,
    Enum,
    Alias,
    Struct,
    Union,
    Ref,
    Function,
    Interrupt,
    Invalid,
}
impl From<u8> for Kind {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Intrinsic,
            1 => Self::Enum,
            2 => Self::Alias,
            3 => Self::Struct,
            4 => Self::Union,
            5 => Self::Ref,
            6 => Self::Function,
            7 => Self::Interrupt,
            _ => Self::Invalid,
        }
    }
}

#[derive(Default)]
enum Type {
    #[default]
    U8,
    U16,
    U32,
    S8,
    S16,
    S32,
}
impl From<u8> for Type {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::U16,
            2 => Self::U32,
            3 => Self::S8,
            4 => Self::S16,
            5 => Self::S32,
            _ => Self::U8,
        }
    }
}

#[derive(Default, PartialEq, Eq, Hash)]
enum Register {
    #[default]
    AL,
    AH,
    AX,
    BL,
    BH,
    BX,
    CL,
    CH,
    CX,
    DL,
    DH,
    DX,
    BP,
    SI,
    DI,
    CS,
    DS,
    ES,
    SS,
}
impl From<u16> for Register {
    fn from(value: u16) -> Self {
        match value {
            2 => Self::AH,
            3 => Self::AX,
            4 => Self::BL,
            8 => Self::BH,
            0xc => Self::BX,
            0x10 => Self::CL,
            0x20 => Self::CH,
            0x30 => Self::CX,
            0x40 => Self::DL,
            0x80 => Self::DH,
            0xc0 => Self::DX,
            0x100 => Self::BP,
            0x200 => Self::SI,
            0x400 => Self::DI,
            0x800 => Self::CS,
            0x1000 => Self::DS,
            0x2000 => Self::ES,
            0x4000 => Self::SS,
            _ => Self::AL,
        }
    }
}
impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::AL => write!(f, "al"),
            Self::AH => write!(f, "ah"),
            Self::AX => write!(f, "ax"),
            Self::BL => write!(f, "bl"),
            Self::BH => write!(f, "bh"),
            Self::BX => write!(f, "bx"),
            Self::CL => write!(f, "cl"),
            Self::CH => write!(f, "ch"),
            Self::CX => write!(f, "cx"),
            Self::DL => write!(f, "dl"),
            Self::DH => write!(f, "dh"),
            Self::DX => write!(f, "dx"),
            Self::BP => write!(f, "bp"),
            Self::SI => write!(f, "si"),
            Self::DI => write!(f, "di"),
            Self::CS => write!(f, "cs"),
            Self::DS => write!(f, "ds"),
            Self::ES => write!(f, "es"),
            Self::SS => write!(f, "ss"),
        }
    }
}

pub trait Symbol: Send + Display {
    fn set_size(&mut self, size: u32);
    fn get_size(&self) -> u32;
    fn set(&mut self, handle: &Handle, ofs: usize, ids: &HashMap<u32, String>) -> usize;
    fn signature(&self) -> Option<&[Sig]> {
        None
    }
    fn render(&self, key: &str, org: u32) -> TreeRow;
    fn matches(
        &self,
        _intnum: u8,
        _ah: Option<u8>,
        _al: Option<u8>,
        _dh: Option<u8>,
        _dl: Option<u8>,
    ) -> bool {
        false
    }
}

#[derive(Default)]
pub struct Ref {
    size: u32,
    symbol: Option<String>,
    pointer: u32,
    array: Option<u32>,
    reg: String,
}
impl Symbol for Ref {
    fn set_size(&mut self, size: u32) {
        self.size = size;
    }
    fn get_size(&self) -> u32 {
        self.size
    }
    fn set(&mut self, handle: &Handle, ofs: usize, ids: &HashMap<u32, String>) -> usize {
        let mut ofs = ofs;
        let id = handle.r32(ofs);
        ofs += 4;
        if id == 0 {
            self.symbol = None;
        } else {
            self.symbol = Some(ids[&id].clone());
            self.pointer = handle.r32(ofs);
            ofs += 4;
            let array = handle.r32(ofs);
            ofs += 4;
            self.array = if array == 0xffff_ffff {
                None
            } else {
                Some(array)
            };
            self.reg = handle.rps(ofs, "");
            ofs = handle.skipps(ofs);
        }
        ofs
    }
    fn render(&self, key: &str, org: u32) -> TreeRow {
        let mut row = TreeRow::default();
        row.push_column(Label::new(key));
        row.push_column(Label::new(&self.to_string()));
        row.push_column(Label::new(&format!("${org:x}")));
        row
    }
}
impl Display for Ref {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.pointer {
            let _ = write!(f, "^");
        }
        if let Some(symbol) = &self.symbol {
            let _ = write!(f, "{symbol}");
        }
        if let Some(array) = self.array {
            let _ = write!(f, "[");
            if array > 0 {
                let _ = write!(f, "{array}");
            }
            let _ = write!(f, "]");
        }
        Ok(())
    }
}

pub enum Sig {
    Value(u8),
    Wild(u32),
}

#[derive(Default)]
pub struct Function {
    size: u32,
    arguments: Vec<(String, Ref)>,
    return_type: Ref,
    signature: Vec<Sig>,
}
impl Symbol for Function {
    fn set_size(&mut self, size: u32) {
        self.size = size;
    }
    fn get_size(&self) -> u32 {
        self.size
    }
    fn set(&mut self, handle: &Handle, ofs: usize, ids: &HashMap<u32, String>) -> usize {
        let mut ofs = ofs;
        let num_args = handle.r32(ofs);
        ofs += 4;
        for _ in 0..num_args {
            let key = handle.rps(ofs, "");
            ofs = handle.skipps(ofs);
            let mut r = Ref::default();
            ofs = r.set(handle, ofs, ids);
            self.arguments.push((key, r));
        }
        ofs = self.return_type.set(handle, ofs, ids);
        let num_sig = handle.r32(ofs);
        ofs += 4;
        for _ in 0..num_sig {
            let wild = handle.r8(ofs) != 0;
            ofs += 1;
            self.signature.push(if wild {
                let len = handle.r32(ofs);
                ofs += 4;
                let v = Sig::Wild(len);
                ofs = handle.skipps(ofs); // we don't use the name right now
                v
            } else {
                let v = Sig::Value(handle.r8(ofs));
                ofs += 1;
                v
            });
        }

        ofs
    }
    fn signature(&self) -> Option<&[Sig]> {
        Some(&self.signature)
    }

    fn render(&self, key: &str, _org: u32) -> TreeRow {
        let mut row = TreeRow::default();
        row.push_column(Label::new(key));
        row.push_column(Label::new(&self.to_string()));
        row.push_column(Blank::default());
        for arg in &self.arguments {
            let mut child = TreeRow::default();
            child.push_column(Label::new(&arg.0));
            child.push_column(Label::new(&arg.1.to_string()));
            child.push_column(Blank::default());
            row.push_child(child);
        }
        if self.return_type.symbol.is_some() {
            let mut child = TreeRow::default();
            child.push_column(Label::new("<return>"));
            child.push_column(Label::new(&self.return_type.to_string()));
            child.push_column(Blank::default());
            row.push_child(child);
        }
        row
    }
}
impl Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "function")
    }
}

#[derive(Default)]
pub struct Interrupt {
    value: u8,
    signature: HashMap<Register, u8>,
    inputs: HashMap<Register, String>,
    outputs: HashMap<Register, String>,
}
impl Symbol for Interrupt {
    fn set_size(&mut self, _size: u32) {}
    fn get_size(&self) -> u32 {
        0
    }
    fn set(&mut self, handle: &Handle, ofs: usize, _ids: &HashMap<u32, String>) -> usize {
        let mut ofs = ofs;
        self.value = handle.r8(ofs);
        ofs += 1;
        let num_sig = handle.r32(ofs);
        ofs += 4;
        for _ in 0..num_sig {
            let key = handle.r16(ofs).into();
            ofs += 2;
            let val = handle.r32(ofs);
            ofs += 4;
            match key {
                Register::AX => {
                    self.signature.insert(
                        Register::AH,
                        u8::try_from((val >> 8) & 0xff).unwrap_or_default(),
                    );
                    self.signature
                        .insert(Register::AL, u8::try_from(val & 0xff).unwrap_or_default());
                }
                Register::BX => {
                    self.signature.insert(
                        Register::BH,
                        u8::try_from((val >> 8) & 0xff).unwrap_or_default(),
                    );
                    self.signature
                        .insert(Register::BL, u8::try_from(val & 0xff).unwrap_or_default());
                }
                Register::CX => {
                    self.signature.insert(
                        Register::CH,
                        u8::try_from((val >> 8) & 0xff).unwrap_or_default(),
                    );
                    self.signature
                        .insert(Register::CL, u8::try_from(val & 0xff).unwrap_or_default());
                }
                Register::DX => {
                    self.signature.insert(
                        Register::DH,
                        u8::try_from((val >> 8) & 0xff).unwrap_or_default(),
                    );
                    self.signature
                        .insert(Register::DL, u8::try_from(val & 0xff).unwrap_or_default());
                }
                reg => {
                    self.signature.insert(reg, (val & 0xff) as u8);
                }
            }
        }
        let num_inp = handle.r32(ofs);
        ofs += 4;
        for _ in 0..num_inp {
            let key = handle.r16(ofs).into();
            ofs += 2;
            let val = handle.rps(ofs, "");
            ofs = handle.skipps(ofs);
            self.inputs.insert(key, val);
        }
        let num_out = handle.r32(ofs);
        ofs += 4;
        for _ in 0..num_out {
            let key = handle.r16(ofs).into();
            ofs += 2;
            let val = handle.rps(ofs, "");
            ofs = handle.skipps(ofs);
            self.outputs.insert(key, val);
        }
        ofs
    }

    fn render(&self, key: &str, _org: u32) -> TreeRow {
        let mut row = TreeRow::default();
        row.push_column(Label::new(key));
        row.push_column(Label::new(&format!("int ${:02x}", self.value)));
        row.push_column(Blank::default());
        for a in &self.signature {
            let mut child = TreeRow::default();
            child.push_column(Label::new(&a.0.to_string()));
            child.push_column(Label::new(&format!("${:x}", a.1)));
            child.push_column(Blank::default());
            row.push_child(child);
        }
        let mut inputs = TreeRow::default();
        inputs.push_column(Label::new("inputs"));
        inputs.push_column(Blank::default());
        inputs.push_column(Blank::default());
        for a in &self.inputs {
            let mut child = TreeRow::default();
            child.push_column(Label::new(&a.0.to_string()));
            child.push_column(Label::new(a.1));
            child.push_column(Blank::default());
            inputs.push_child(child);
        }
        row.push_child(inputs);
        let mut outputs = TreeRow::default();
        outputs.push_column(Label::new("outputs"));
        outputs.push_column(Blank::default());
        outputs.push_column(Blank::default());
        for a in &self.outputs {
            let mut child = TreeRow::default();
            child.push_column(Label::new(&a.0.to_string()));
            child.push_column(Label::new(a.1));
            child.push_column(Blank::default());
            outputs.push_child(child);
        }
        row.push_child(outputs);
        row
    }

    fn matches(
        &self,
        intnum: u8,
        ah: Option<u8>,
        al: Option<u8>,
        dh: Option<u8>,
        dl: Option<u8>,
    ) -> bool {
        if self.value == intnum {
            !(ah.is_some_and(|v| Some(&v) != self.signature.get(&Register::AH))
                || al.is_some_and(|v| Some(&v) != self.signature.get(&Register::AL))
                || dh.is_some_and(|v| Some(&v) != self.signature.get(&Register::DH))
                || dl.is_some_and(|v| Some(&v) != self.signature.get(&Register::DL)))
        } else {
            false
        }
    }
}
impl Display for Interrupt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "int ${:02x}", self.value)
    }
}

#[derive(Default)]
pub struct Intrinsic {
    size: u32,
    typ: Type,
}
impl Symbol for Intrinsic {
    fn set_size(&mut self, size: u32) {
        self.size = size;
    }
    fn get_size(&self) -> u32 {
        self.size
    }
    fn set(&mut self, handle: &Handle, ofs: usize, _ids: &HashMap<u32, String>) -> usize {
        self.typ = handle.r8(ofs).into();
        ofs + 1
    }
    fn render(&self, key: &str, org: u32) -> TreeRow {
        let mut row = TreeRow::default();
        row.push_column(Label::new(key));
        row.push_column(Label::new(&self.to_string()));
        row.push_column(Label::new(&format!("{org:x}")));
        row
    }
}
impl Display for Intrinsic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.typ {
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::S8 => write!(f, "i8"),
            Type::S16 => write!(f, "i16"),
            Type::S32 => write!(f, "i32"),
        }
    }
}

#[derive(Default)]
pub struct Enum {
    size: u32,
    typ: String,
    entries: HashMap<String, u32>,
}
impl Symbol for Enum {
    fn set_size(&mut self, size: u32) {
        self.size = size;
    }
    fn get_size(&self) -> u32 {
        self.size
    }
    fn set(&mut self, handle: &Handle, ofs: usize, ids: &HashMap<u32, String>) -> usize {
        let mut ofs = ofs;
        self.typ.clone_from(&ids[&handle.r32(ofs)]);
        ofs += 4;
        let num = handle.r32(ofs);
        ofs += 4;
        for _ in 0..num {
            let val = handle.r32(ofs);
            ofs += 4;
            let entry = handle.rps(ofs, "");
            ofs = handle.skipps(ofs);
            self.entries.insert(entry, val);
        }
        ofs
    }

    fn render(&self, key: &str, org: u32) -> TreeRow {
        let mut row = TreeRow::default();
        row.push_column(Label::new(key));
        row.push_column(Label::new(&self.to_string()));
        row.push_column(Label::new(&format!("{org:x}")));
        let mut sorted = vec![];
        for entry in &self.entries {
            sorted.push((entry.0, entry.1));
        }
        sorted.sort_unstable_by(|a, b| a.1.cmp(b.1));
        for entry in &sorted {
            let mut child = TreeRow::default();
            child.push_column(Label::new(entry.0));
            child.push_column(Label::new(&format!("${:x}", entry.1)));
            child.push_column(Blank::default());
            row.push_child(child);
        }
        row
    }
}
impl Display for Enum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "enum<{}>", self.typ)
    }
}

#[derive(Default)]
pub struct Struct {
    is_union: bool,
    size: u32,
    fields: Vec<(String, Box<dyn Symbol>)>,
}
impl Struct {
    pub fn new(is_union: bool) -> Self {
        Self {
            is_union,
            ..Default::default()
        }
    }
}
impl Symbol for Struct {
    fn set_size(&mut self, size: u32) {
        self.size = size;
    }
    fn get_size(&self) -> u32 {
        self.size
    }
    fn set(&mut self, handle: &Handle, ofs: usize, ids: &HashMap<u32, String>) -> usize {
        let mut ofs = ofs;
        let num_fields = handle.r32(ofs);
        ofs += 4;
        for _ in 0..num_fields {
            let key = handle.rps(ofs, "");
            ofs = handle.skipps(ofs);
            let kind = handle.r8(ofs);
            ofs += 1;
            let size = handle.r32(ofs);
            ofs += 4;
            let mut sym: Box<dyn Symbol> = match kind.into() {
                Kind::Struct => Box::new(Self::new(false)),
                Kind::Union => Box::new(Self::new(true)),
                Kind::Intrinsic => Box::new(Intrinsic::default()),
                Kind::Enum => Box::new(Enum::default()),
                Kind::Ref | Kind::Alias => Box::new(Ref::default()),
                _ => panic!("invalid struct member"),
            };
            sym.set_size(size);
            ofs = sym.set(handle, ofs, ids);
            self.fields.push((key, sym));
        }
        ofs
    }

    fn render(&self, key: &str, org: u32) -> TreeRow {
        let mut row = TreeRow::default();
        let mut org = org;
        row.push_column(Label::new(key));
        row.push_column(Label::new(&self.to_string()));
        row.push_column(Label::new(&format!("{org:x}")));
        if self.is_union {
            for f in &self.fields {
                row.push_child(f.1.render(&f.0, org));
            }
        } else {
            for f in &self.fields {
                row.push_child(f.1.render(&f.0, org));
                org += f.1.get_size();
            }
        }
        row
    }
}
impl Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_union {
            write!(f, "union")
        } else {
            write!(f, "struct")
        }
    }
}
