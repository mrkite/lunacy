/*

68k disassembler
@copyright 2026 Sean Kasun

*/

use std::collections::HashMap;

use crate::{
    cpus::{self, Cpu, CpuLine, CpuState, InstType, Instruction, cpu68k::opcodes::OPCODES},
    lunacy::File,
};

mod opcodes;

#[derive(Clone)]
pub struct State68k {
    pub addr: u32,
}

impl CpuState for State68k {
    fn jump(&mut self, addr: u32) {
        self.addr = addr;
    }
    fn address(&self) -> u32 {
        self.addr
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Default, PartialEq)]
enum Opr {
    #[default]
    None,
    Reg,
    RegPost,
    RegPre,
    RegInd,
    RegIndDisp,
    RegIndex,
    Abs,
    Imm,
    Flags,
    RegList,
}

#[derive(Default)]
struct Operand {
    oper_type: Opr,
    reg: u8,
    index: u8,
    width: u8,
    constant: u32,
}

#[derive(Default)]
pub struct Operands68k {
    width: Option<u8>,
    operands: Vec<Operand>,
}

impl cpus::Operand for Operands68k {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Cpu68k {
    opcodes: HashMap<u8, usize>,
}

impl Default for Cpu68k {
    fn default() -> Self {
        let mut opcodes = HashMap::new();
        let mut prev = 0xff;
        for (idx, opcode) in OPCODES.iter().enumerate() {
            let bit = (opcode.b >> 12) as u8;
            if bit != prev {
                opcodes.insert(bit, idx);
                prev = bit;
            }
        }
        opcodes.insert(0xa, OPCODES.len());
        opcodes.insert(0xf, OPCODES.len());
        opcodes.insert(0x10, OPCODES.len());
        Self { opcodes }
    }
}

impl Cpu for Cpu68k {
    fn decode(
        &self,
        file: &File,
        offset: usize,
        state: &mut dyn CpuState,
    ) -> Result<Instruction, &'static str> {
        let state = if let Some(state) = state.as_any().downcast_mut::<State68k>() {
            state
        } else {
            return Err("68k disassembler called without a 68k state");
        };
        let start_offset = offset;
        let mut offset = offset;
        let mut saved_constant = None;
        let opcode = file.r16be(offset);
        offset += 2;
        let group = (opcode >> 12) as u8;
        for op in OPCODES
            .iter()
            .take(self.opcodes[&(group + 1)])
            .skip(self.opcodes[&group])
        {
            if opcode & op.m == op.b {
                let mut found = true;
                let mut sub = Box::new(Operands68k::default());
                if op.a.starts_with('.') {
                    sub.width = Some(((opcode >> 6) & 3) as u8);
                }
                let mut it = op.a.chars();
                while let Some(ch) = it.next() {
                    let mut op = Operand::default();
                    match ch {
                        '.' => {
                            it.next();
                        }
                        '%' => {
                            saved_constant = Some(file.r16be(offset));
                            offset += 2;
                            it.next();
                        }
                        'a' => {
                            let ch = it.next().unwrap();
                            match ch {
                                'w' => {
                                    op.oper_type = Opr::RegIndDisp;
                                    op.width = 0;
                                    op.constant = u32::from(file.r16be(offset));
                                    offset += 2;
                                    op.reg = (opcode & 7) as u8;
                                }
                                'x' => {
                                    op.oper_type = Opr::Reg;
                                    op.reg = ((opcode >> 9) & 7) as u8;
                                }
                                _ => {
                                    op.oper_type = Opr::Reg;
                                    op.reg = (opcode & 7) as u8;
                                }
                            }
                            op.reg += 8;
                        }
                        'd' => {
                            let ch = it.next().unwrap();
                            op.oper_type = Opr::Reg;
                            op.reg = match ch {
                                'x' => opcode >> 9,
                                _ => opcode,
                            } as u8
                                & 7;
                        }
                        '+' => {
                            let ch = it.next().unwrap();
                            op.oper_type = Opr::RegPost;
                            op.reg = match ch {
                                'x' => opcode >> 9,
                                _ => opcode,
                            } as u8
                                & 7;
                            op.reg += 8;
                        }
                        '-' => {
                            let ch = it.next().unwrap();
                            op.oper_type = Opr::RegPre;
                            op.reg = match ch {
                                'x' => opcode >> 9,
                                _ => opcode,
                            } as u8
                                & 7;
                            op.reg += 8;
                        }
                        'i' => {
                            let ch = it.next().unwrap();
                            op.oper_type = Opr::Imm;
                            op.constant = match ch {
                                'v' => u32::from(opcode & 7),
                                '4' => u32::from(opcode & 0xf),
                                '8' => (opcode & 0xff) as i8 as i32 as u32,
                                'q' => {
                                    if (opcode & 0xe00) == 0 {
                                        8
                                    } else {
                                        u32::from((opcode >> 9) & 7)
                                    }
                                }
                                'b' => {
                                    let cons = (file.r16be(offset) & 0xff) as i8 as i32 as u32;
                                    offset += 2;
                                    cons
                                }
                                'w' => {
                                    let cons = file.r16be(offset) as i16 as i32 as u32;
                                    offset += 2;
                                    cons
                                }
                                'm' => {
                                    if let Some(w) = sub.width {
                                        match w {
                                            0 => {
                                                let cons =
                                                    (file.r16be(offset) & 0xff) as i8 as i32 as u32;
                                                offset += 2;
                                                cons
                                            }
                                            1 => {
                                                let cons = file.r16be(offset) as i16 as i32 as u32;
                                                offset += 2;
                                                cons
                                            }
                                            2 => {
                                                let cons = file.r32be(offset);
                                                offset += 4;
                                                cons
                                            }
                                            _ => 0,
                                        }
                                    } else {
                                        0
                                    }
                                }
                                _ => panic!("missing immediate variant"),
                            };
                        }
                        'e' => {
                            let ch = it.next().unwrap();
                            if !check_ea((opcode >> 3) as u8 & 7, opcode as u8 & 7) {
                                found = false
                            } else {
                                let size = if ch == 'a'
                                    && let Some(w) = sub.width
                                {
                                    ['b', 'w', 'l', 'x'][usize::from(w)]
                                } else {
                                    ch
                                };
                                parse_ea(
                                    &mut op,
                                    (opcode >> 3) as u8 & 7,
                                    opcode as u8 & 7,
                                    size,
                                    file,
                                    &mut offset,
                                    state.addr,
                                );
                            }
                        }
                        'E' => {
                            let ch = it.next().unwrap();
                            if !check_ea((opcode >> 6) as u8 & 7, (opcode >> 9) as u8 & 7) {
                                found = false
                            } else {
                                let size = if ch == 'a'
                                    && let Some(w) = sub.width
                                {
                                    ['b', 'w', 'l', 'x'][usize::from(w)]
                                } else {
                                    ch
                                };
                                parse_ea(
                                    &mut op,
                                    (opcode >> 6) as u8 & 7,
                                    (opcode >> 9) as u8 & 7,
                                    size,
                                    file,
                                    &mut offset,
                                    state.addr,
                                );
                            }
                        }
                        'l' => {
                            it.next();
                            op.oper_type = Opr::RegList;
                            let mut list = saved_constant.unwrap_or({
                                let list = file.r16be(offset);
                                offset += 2;
                                list
                            });
                            if opcode & 0x38 == 0x20 {
                                let tmp = list;
                                list = 0;
                                for i in 0..16 {
                                    if tmp & (0x8000 >> i) != 0 {
                                        list |= 1 << i;
                                    }
                                }
                            }
                            op.constant = u32::from(list);
                        }
                        '~' => {
                            let ch = it.next().unwrap();
                            op.oper_type = Opr::Abs;
                            let mut rel = 0;
                            if ch == '~' {
                                rel = (opcode & 0xff) as i8 as i32;
                            }
                            if rel == 0 {
                                rel = file.r16be(offset) as i16 as i32;
                                offset += 2;
                            }
                            op.constant = state.addr.wrapping_add_signed(rel) + 2;
                        }
                        '\'' => {
                            let ch = it.next().unwrap();
                            op.oper_type = Opr::Flags;
                            op.constant = if ch == 'c' { 0 } else { 1 };
                        }
                        _ => found = false,
                    }
                    sub.operands.push(op);
                }

                if found {
                    let length = (offset - start_offset) as u32;
                    state.addr += length;
                    let raw = file.read(start_offset, length as usize).to_vec();

                    return Ok(Instruction {
                        name: op.n.to_string(),
                        inst_type: op.i,
                        length,
                        raw,
                        operands: sub,
                    });
                }
            }
        }
        if (opcode & 0xf000) == 0xa000 {
            let length = (offset - start_offset) as u32;
            state.addr += length;
            let raw = file.read(start_offset, length as usize).to_vec();
            let mut sub = Box::new(Operands68k::default());
            let op = Operand {
                oper_type: Opr::Abs,
                constant: u32::from(opcode),
                ..Default::default()
            };
            sub.operands.push(op);
            return Ok(Instruction {
                name: "db".to_owned(),
                inst_type: InstType::Normal,
                length,
                raw,
                operands: sub,
            });
        }
        Err("opcode not found")
    }

    fn disassemble(
        &self,
        address: u32,
        inst: &Instruction,
        lookup: &dyn Fn(u32) -> Option<String>,
    ) -> CpuLine {
        let sub = if let Some(sub) = inst.operands.as_any().downcast_ref::<Operands68k>() {
            sub
        } else {
            return CpuLine {
                opcode: "invalid operand".to_string(),
                ..Default::default()
            };
        };
        let mut out = CpuLine {
            opcode: inst.name.clone(),
            address,
            ..Default::default()
        };
        for (idx, code) in inst.raw.iter().enumerate() {
            if out.hex.len() == 9 {
                out.hex.push('…');
                break;
            }
            if idx != 0 && idx & 1 != 1 {
                out.hex.push(' ');
            }
            out.hex.push(HEX[usize::from(code >> 4)]);
            out.hex.push(HEX[usize::from(code & 0xf)]);
        }
        while out.hex.chars().count() < 11 {
            out.hex.push(' ');
        }

        if let Some(w) = sub.width {
            out.opcode.push('.');
            out.opcode.push(['b', 'w', 'l', 'x'][usize::from(w)]);
        }
        while out.opcode.len() < 10 {
            out.opcode.push(' ');
        }

        for op in sub.operands.iter() {
            if !out.operand.is_empty() {
                out.operand.push_str(", ");
            }
            match op.oper_type {
                Opr::None => {}
                Opr::Reg => {
                    out.operand.push_str(
                        format!("{}{}", if op.reg & 8 == 8 { 'a' } else { 'd' }, op.reg & 7)
                            .as_str(),
                    );
                }
                Opr::RegPost => out.operand.push_str(
                    format!(
                        "({}{})+",
                        if op.reg & 8 == 8 { 'a' } else { 'd' },
                        op.reg & 7
                    )
                    .as_str(),
                ),
                Opr::RegPre => out.operand.push_str(
                    format!(
                        "-({}{})",
                        if op.reg & 8 == 8 { 'a' } else { 'd' },
                        op.reg & 7
                    )
                    .as_str(),
                ),
                Opr::RegInd => out.operand.push_str(
                    format!(
                        "({}{})",
                        if op.reg & 8 == 8 { 'a' } else { 'd' },
                        op.reg & 7
                    )
                    .as_str(),
                ),
                Opr::RegIndDisp => {
                    if op.reg == 13 {
                        // A5
                        // dump a5 symbol
                        out.operand.push_str(&to_hex(op.constant, lookup));
                        out.operand.push_str("(a5)");
                    } else {
                        out.operand.push_str(&to_hex(op.constant, lookup));
                        out.operand.push_str(
                            format!("({}{}", if op.reg & 8 == 8 { 'a' } else { 'd' }, op.reg & 7)
                                .as_str(),
                        );
                        if op.width != 0 {
                            out.operand.push('.');
                            out.operand.push(['x', 'w', 'l'][usize::from(op.width)]);
                        }
                        out.operand.push(')');
                    }
                }
                Opr::RegIndex => {
                    let v = (op.constant & 0xff) as i8 as i32 as u32;
                    out.operand.push_str(&to_hex(v, lookup));
                    out.operand.push_str(
                        format!(
                            "({}{}, {}{}",
                            if op.reg & 8 == 8 { 'a' } else { 'd' },
                            op.reg & 7,
                            if op.index & 8 == 8 { 'a' } else { 'd' },
                            op.index & 7
                        )
                        .as_str(),
                    );
                    if op.width != 0 {
                        out.operand.push('.');
                        out.operand.push(['x', 'w', 'l'][usize::from(op.width)]);
                    }
                    out.operand.push(')');
                }
                Opr::Abs => out.operand.push_str(&to_hex(op.constant, lookup)),
                Opr::Imm => {
                    out.operand.push('#');
                    out.operand.push_str(&to_hex(op.constant, lookup));
                }
                Opr::Flags => {
                    if op.constant == 1 {
                        out.operand.push_str("sr");
                    } else {
                        out.operand.push_str("ccr");
                    }
                }
                Opr::RegList => {
                    let list = op.constant;
                    let mut start = None;
                    let mut end = None;
                    let mut multi = false;
                    let mut off = false;
                    for i in 0..16 {
                        if (list & (1 << i)) != 0 {
                            end = Some(i & 7);
                            if start.is_none() {
                                start = Some(i & 7);
                            }
                            if i == 7 {
                                off = true;
                            }
                        } else if start.is_some() {
                            off = true;
                        }
                        if off {
                            if multi {
                                out.operand.push('/');
                            }
                            multi = true;
                            if let Some(start) = start {
                                out.operand.push_str(
                                    format!("{}{}", if i & 8 == 8 { 'a' } else { 'd' }, start)
                                        .as_str(),
                                );
                            }
                            if start != end
                                && let Some(end) = end
                            {
                                out.operand.push_str(
                                    format!("-{}{}", if i & 8 == 8 { 'a' } else { 'd' }, end)
                                        .as_str(),
                                );
                            }
                            start = None;
                            off = false;
                        }
                    }
                }
            }
        }
        out
    }

    fn target(&self, inst: &Instruction) -> Option<u32> {
        if let Some(sub) = inst.operands.as_any().downcast_ref::<Operands68k>() {
            for opr in sub.operands.iter() {
                if opr.oper_type == Opr::Imm || opr.oper_type == Opr::Abs {
                    return Some(opr.constant);
                }
            }
        }
        None
    }
}

fn check_ea(mode: u8, var: u8) -> bool {
    if mode < 7 { true } else { var < 5 }
}

fn parse_ea(
    op: &mut Operand,
    mode: u8,
    var: u8,
    size: char,
    file: &File,
    offset: &mut usize,
    address: u32,
) {
    match mode {
        0 => {
            op.oper_type = Opr::Reg;
            op.reg = var;
        }
        1 => {
            op.oper_type = Opr::Reg;
            op.reg = var + 8;
        }
        2 => {
            op.oper_type = Opr::RegInd;
            op.reg = var + 8;
        }
        3 => {
            op.oper_type = Opr::RegPost;
            op.reg = var + 8;
        }
        4 => {
            op.oper_type = Opr::RegPre;
            op.reg = var + 8;
        }
        5 => {
            op.oper_type = Opr::RegIndDisp;
            op.width = 0;
            op.constant = u32::from(file.r16be(*offset));
            *offset += 2;
            op.reg = var + 8;
        }
        6 => {
            op.oper_type = Opr::RegIndex;
            let imm = file.r16be(*offset);
            *offset += 2;
            op.constant = (imm & 0xff) as i8 as i32 as u32;
            op.reg = var + 8;
            op.index = (imm >> 12) as u8 & 0xf;
            op.width = if (imm & 0x800) == 0x800 { 2 } else { 1 };
        }
        7 => match var {
            0 => {
                op.oper_type = Opr::Abs;
                op.constant = u32::from(file.r16be(*offset));
                *offset += 2;
            }
            1 => {
                op.oper_type = Opr::Abs;
                op.constant = file.r32be(*offset);
                *offset += 4;
            }
            2 => {
                op.oper_type = Opr::Abs;
                let rel = file.r16be(*offset) as i16 as i32;
                *offset += 2;
                op.constant = address.wrapping_add_signed(rel) + 2;
            }
            3 => {
                op.oper_type = Opr::RegIndDisp;
                let imm = file.r16be(*offset);
                *offset += 2;
                let rel = (imm & 0xff) as i8 as i32;
                op.constant = address.wrapping_add_signed(rel) + 2;
                op.reg = (imm >> 12) as u8 & 0xf;
                op.width = if (imm & 0x800) == 0x800 { 2 } else { 1 };
            }
            4 => {
                op.oper_type = Opr::Imm;
                op.constant = match size {
                    'b' => {
                        let c = (file.r16be(*offset) & 0xff) as i8 as i32 as u32;
                        *offset += 2;
                        c
                    }
                    'w' => {
                        let c = u32::from(file.r16be(*offset));
                        *offset += 2;
                        c
                    }
                    'l' => {
                        let c = file.r32be(*offset);
                        *offset += 4;
                        c
                    }
                    _ => 0,
                };
            }
            _ => {}
        },
        _ => {}
    }
}

fn can_fourcc(val: u32) -> bool {
    for i in 0..4 {
        let c = (val >> ((3 - i) * 8)) as u8;
        if !(b' '..=b'~').contains(&c) {
            return false;
        }
    }
    true
}

fn to_fourcc(val: u32) -> String {
    let mut s = String::new();
    s.push('\'');
    for i in 0..4 {
        let c = (val >> ((3 - i) * 8)) as u8;
        s.push(c as char);
    }
    s.push('\'');
    s
}

fn to_hex(val: u32, lookup: &dyn Fn(u32) -> Option<String>) -> String {
    // check if address lookup here
    if let Some(cmt) = lookup(val) {
        return cmt;
    }
    if can_fourcc(val) {
        to_fourcc(val)
    } else if (val & !0xff) == !0xff {
        format!("$-{:02x}", !val + 1)
    } else if (val & !0xff) == 0 {
        format!("${:02x}", val)
    } else if (val & !0xffff) == !0xffff {
        format!("$-{:04x}", !val + 1)
    } else if (val & !0xffff) == 0 {
        format!("${:04x}", val)
    } else if (val as i32) < 0 {
        format!("$-{:08x}", !val + 1)
    } else {
        format!("${:08x}", val)
    }
}

const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
