/*

Custom API calls

*/

use crate::{
    lunacy::{Context, File},
    platforms::fingerprints::{Fingerprint, Print},
};
use eframe::egui;
use std::{collections::HashMap, fmt};

const DEPTH: usize = 4;

pub struct Api {
    symbols: HashMap<String, Box<dyn Symbol>>,
    search: String,
    search_results: Vec<String>,
}

enum Kind {
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
            0 => Self::U8,
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
            1 => Self::AL,
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

trait Symbol {
    fn set_size(&mut self, size: u32);
    fn get_size(&self) -> u32;
    fn set(&mut self, file: &File, ofs: usize, ids: &HashMap<u32, String>) -> usize;
    fn signature(&self) -> Option<&[Sig]> {
        None
    }
    fn to_string(&self, org: u32, depth: usize) -> String;
}

#[derive(Default)]
struct Ref {
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
    fn set(&mut self, file: &File, ofs: usize, ids: &HashMap<u32, String>) -> usize {
        let mut ofs = ofs;
        let id = file.r32(ofs);
        ofs += 4;
        if id == 0 {
            self.symbol = None;
        } else {
            self.symbol = Some(ids[&id].clone());
            self.pointer = file.r32(ofs);
            ofs += 4;
            let array = file.r32(ofs);
            ofs += 4;
            self.array = if array == 0xffffffff {
                None
            } else {
                Some(array)
            };
            self.reg = file.rps(ofs, "");
            ofs = file.skipps(ofs);
        }
        ofs
    }
    fn to_string(&self, _org: u32, _depth: usize) -> String {
        let mut fmt = String::new();
        for _ in 0..self.pointer {
            fmt.push('^');
        }
        if let Some(symbol) = &self.symbol {
            fmt.push_str(symbol);
        }
        if let Some(array) = self.array {
            fmt.push('[');
            if array > 0 {
                fmt.push_str(&format!("{}", array));
            }
            fmt.push(']');
        }
        fmt
    }
}

enum Sig {
    Value(u8),
    Wild(u32),
}

#[derive(Default)]
struct Function {
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
    fn set(&mut self, file: &File, ofs: usize, ids: &HashMap<u32, String>) -> usize {
        let mut ofs = ofs;
        let num_args = file.r32(ofs);
        ofs += 4;
        for _ in 0..num_args {
            let key = file.rps(ofs, "");
            ofs = file.skipps(ofs);
            let mut r = Ref::default();
            ofs = r.set(file, ofs, ids);
            self.arguments.push((key, r));
        }
        ofs = self.return_type.set(file, ofs, ids);
        let num_sig = file.r32(ofs);
        ofs += 4;
        for _ in 0..num_sig {
            let wild = file.r8(ofs) != 0;
            ofs += 1;
            self.signature.push(if wild {
                let len = file.r32(ofs);
                ofs += 4;
                let v = Sig::Wild(len);
                ofs = file.skipps(ofs); // we don't use the name right now
                v
            } else {
                let v = Sig::Value(file.r8(ofs));
                ofs += 1;
                v
            });
        }

        ofs
    }
    fn signature(&self) -> Option<&[Sig]> {
        Some(&self.signature)
    }

    fn to_string(&self, org: u32, depth: usize) -> String {
        let mut fmt = String::new();
        fmt.push_str("(\n");
        for arg in self.arguments.iter() {
            fmt.push_str(&indent(depth));
            fmt.push_str(&format!("  {}: ", arg.0));
            fmt.push_str(&arg.1.to_string(org, depth + DEPTH));
            fmt.push_str(",\n");
        }
        if self.return_type.symbol.is_some() {
            fmt.push_str("): ");
            fmt.push_str(&self.return_type.to_string(org, depth + DEPTH));
        } else {
            fmt.push(')');
        }
        fmt
    }
}

#[derive(Default)]
struct Interrupt {
    interrupt: u8,
    signature: HashMap<Register, u32>,
    inputs: HashMap<Register, String>,
    outputs: HashMap<Register, String>,
}
impl Symbol for Interrupt {
    fn set_size(&mut self, _size: u32) {}
    fn get_size(&self) -> u32 {
        0
    }
    fn set(&mut self, file: &File, ofs: usize, _ids: &HashMap<u32, String>) -> usize {
        let mut ofs = ofs;
        self.interrupt = file.r8(ofs);
        ofs += 1;
        let num_sig = file.r32(ofs);
        ofs += 4;
        for _ in 0..num_sig {
            let key = file.r16(ofs).into();
            ofs += 2;
            let val = file.r32(ofs);
            ofs += 4;
            match key {
                Register::AX => {
                    self.signature.insert(Register::AH, val >> 8);
                    self.signature.insert(Register::AL, val & 0xff);
                }
                Register::BX => {
                    self.signature.insert(Register::BH, val >> 8);
                    self.signature.insert(Register::BL, val & 0xff);
                }
                Register::CX => {
                    self.signature.insert(Register::CH, val >> 8);
                    self.signature.insert(Register::CL, val & 0xff);
                }
                Register::DX => {
                    self.signature.insert(Register::DH, val >> 8);
                    self.signature.insert(Register::DL, val & 0xff);
                }
                reg => {
                    self.signature.insert(reg, val);
                }
            }
        }
        let num_inp = file.r32(ofs);
        ofs += 4;
        for _ in 0..num_inp {
            let key = file.r16(ofs).into();
            ofs += 2;
            let val = file.rps(ofs, "");
            ofs = file.skipps(ofs);
            self.inputs.insert(key, val);
        }
        let num_out = file.r32(ofs);
        ofs += 4;
        for _ in 0..num_out {
            let key = file.r16(ofs).into();
            ofs += 2;
            let val = file.rps(ofs, "");
            ofs = file.skipps(ofs);
            self.outputs.insert(key, val);
        }
        ofs
    }

    fn to_string(&self, _org: u32, depth: usize) -> String {
        let mut fmt = String::new();
        fmt.push_str(&format!("int ${:02x} {{\n", self.interrupt));
        for a in self.signature.iter() {
            fmt.push_str(&format!("{}  {} = ${:x}\n", indent(depth), a.0, a.1));
        }
        for a in self.inputs.iter() {
            fmt.push_str(&format!("{}  {} = {}\n", indent(depth), a.0, a.1));
        }
        fmt.push_str(&format!("{} ---- output ----\n", indent(depth)));
        for a in self.outputs.iter() {
            fmt.push_str(&format!("{}  {} = {}\n", indent(depth), a.0, a.1));
        }
        fmt.push('}');
        fmt
    }
}

#[derive(Default)]
struct Intrinsic {
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
    fn set(&mut self, file: &File, ofs: usize, _ids: &HashMap<u32, String>) -> usize {
        self.typ = file.r8(ofs).into();
        ofs + 1
    }
    fn to_string(&self, _org: u32, _depth: usize) -> String {
        match self.typ {
            Type::U8 => "u8",
            Type::U16 => "u16",
            Type::U32 => "u32",
            Type::S8 => "i8",
            Type::S16 => "i16",
            Type::S32 => "i32",
        }
        .to_owned()
    }
}

#[derive(Default)]
struct Enum {
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
    fn set(&mut self, file: &File, ofs: usize, ids: &HashMap<u32, String>) -> usize {
        let mut ofs = ofs;
        self.typ = ids[&file.r32(ofs)].clone();
        ofs += 4;
        let num = file.r32(ofs);
        ofs += 4;
        for _ in 0..num {
            let val = file.r32(ofs);
            ofs += 4;
            let entry = file.rps(ofs, "");
            ofs = file.skipps(ofs);
            self.entries.insert(entry, val);
        }

        ofs
    }
    fn to_string(&self, _org: u32, depth: usize) -> String {
        let mut fmt = String::new();
        fmt.push_str(&format!("enum : {} {{\n", self.typ));
        let mut sorted = vec![];
        for entry in self.entries.iter() {
            sorted.push((entry.0, entry.1));
        }
        sorted.sort_unstable_by(|a, b| a.1.cmp(b.1));
        for entry in sorted.iter() {
            fmt.push_str(&format!(
                "{}  {} = ${:x},\n",
                indent(depth),
                entry.0,
                entry.1
            ));
        }
        fmt.push('}');
        fmt
    }
}

#[derive(Default)]
struct Struct {
    is_union: bool,
    size: u32,
    fields: Vec<(String, Box<dyn Symbol>)>,
}
impl Struct {
    fn new(is_union: bool) -> Self {
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
    fn set(&mut self, file: &File, ofs: usize, ids: &HashMap<u32, String>) -> usize {
        let mut ofs = ofs;
        let num_fields = file.r32(ofs);
        ofs += 4;
        for _ in 0..num_fields {
            let key = file.rps(ofs, "");
            ofs = file.skipps(ofs);
            let kind = file.r8(ofs);
            ofs += 1;
            let size = file.r32(ofs);
            ofs += 4;
            let mut sym: Box<dyn Symbol> = match kind.into() {
                Kind::Ref => Box::new(Ref::default()),
                Kind::Struct => Box::new(Struct::new(false)),
                Kind::Union => Box::new(Struct::new(true)),
                Kind::Intrinsic => Box::new(Intrinsic::default()),
                Kind::Enum => Box::new(Enum::default()),
                Kind::Alias => Box::new(Ref::default()),
                _ => panic!("invalid struct member"),
            };
            sym.set_size(size);
            ofs = sym.set(file, ofs, ids);
            self.fields.push((key, sym));
        }
        ofs
    }

    fn to_string(&self, org: u32, depth: usize) -> String {
        let mut org = org;
        let mut fmt = String::new();
        if self.is_union {
            fmt.push_str(&format!("union {{ // ${:x} bytes\n", self.size));
            for f in self.fields.iter() {
                fmt.push_str(&format!("{}  {}: ", indent(depth), f.0));
                fmt.push_str(&f.1.to_string(org, depth + DEPTH));
                fmt.push_str(&format!(" // ${:x}\n", org));
            }
            fmt.push_str(&indent(depth));
            fmt.push('}');
        } else {
            fmt.push_str(&format!("struct {{ // ${:x} bytes\n}}", self.size));
            for f in self.fields.iter() {
                fmt.push_str(&format!("{}  {}: ", indent(depth), f.0));
                fmt.push_str(&f.1.to_string(org, depth + DEPTH));
                fmt.push_str(&format!(" // ${:x}\n", org));
                org += f.1.get_size();
            }
            fmt.push_str(&indent(depth));
            fmt.push('}');
        }
        fmt
    }
}

impl Api {
    pub fn new(data: &[u8]) -> Self {
        let file = File::new_from_array(data, "", "");
        let num_symbols = file.r32(0);
        let mut ofs = 4;
        let mut ids = HashMap::new();
        let mut symbols = HashMap::new();

        for _ in 0..num_symbols {
            let id = file.r32(ofs);
            ofs += 4;
            let kind = file.r8(ofs).into();
            ofs += 1;
            let name = file.rps(ofs, "");
            ofs = file.skipps(ofs);
            ids.insert(id, name.clone());
            let s: Box<dyn Symbol> = match kind {
                Kind::Intrinsic => Box::new(Intrinsic::default()),
                Kind::Enum => Box::new(Enum::default()),
                Kind::Alias => Box::new(Ref::default()),
                Kind::Struct => Box::new(Struct::new(false)),
                Kind::Union => Box::new(Struct::new(true)),
                Kind::Function => Box::new(Function::default()),
                Kind::Interrupt => Box::new(Interrupt::default()),
                _ => panic!("Unknown type"),
            };
            symbols.insert(name, s);
        }
        for _ in 0..num_symbols {
            let id = file.r32(ofs);
            ofs += 4;
            if let Some(s) = symbols.get_mut(&ids[&id]) {
                s.set_size(file.r32(ofs));
                ofs += 4;
                ofs = s.set(&file, ofs, &ids);
            }
        }
        Self {
            symbols,
            search: String::new(),
            search_results: vec![],
        }
    }

    pub fn fingerprints(&self) -> Fingerprint {
        let mut root = Fingerprint {
            next: HashMap::new(),
            name: None,
        };
        for s in self.symbols.iter() {
            let mut cur = &mut root;
            if let Some(prints) = s.1.signature() {
                for sig in prints.iter() {
                    match sig {
                        Sig::Value(byte) => {
                            let child = cur.next.entry(Print::Byte(*byte)).or_default();
                            cur = child;
                        }
                        Sig::Wild(len) => {
                            for _ in 0..*len {
                                let child = cur.next.entry(Print::Wild).or_default();
                                cur = child;
                            }
                        }
                    }
                }
            }
            cur.name = Some(s.0.to_owned());
        }

        root
    }

    pub fn show(&mut self, ui: &mut egui::Ui, _ctx: Context) -> bool {
        let mut open = true;
        egui::Window::new("API")
            .open(&mut open)
            .resizable(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    let prev_search = self.search.clone();
                    ui.text_edit_singleline(&mut self.search).request_focus();
                    if !self.search.is_empty() && self.search != prev_search {
                        self.search_results = self.do_search();
                    }
                });
                ui.separator();
                // show results
                egui::ScrollArea::vertical()
                    .id_salt("search_results")
                    .max_width(f32::INFINITY)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for result in self.search_results.iter() {
                                ui.label(result);
                            }
                        })
                    });
            });
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            open = false;
        }

        !open
    }

    fn do_search(&self) -> Vec<String> {
        let key = self.search.to_ascii_lowercase();
        let mut results = vec![];
        for s in self.symbols.iter() {
            if s.0.to_ascii_lowercase().contains(&key) {
                results.push(format!("{}: {}", s.0, s.1.to_string(0, 0)))
            }
        }
        results
    }
}

fn indent(depth: usize) -> String {
    format!("{:^width$}", "", width = depth)
}
