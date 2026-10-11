/*

x86 disassembler
@copyright 2026 Sean Kasun

*/

use std::collections::HashMap;

use crate::{
    cpus::{
        Cpu, CpuLine, CpuState, InstType, Instruction, Operand,
        cpu86::opcodes::{OPCODES, OPCODES0F},
    },
    lunacy::Handle,
};

mod opcodes;

#[derive(Clone)]
pub struct State86 {
    pub seg: u16,
    pub ofs: u16,
}

impl State86 {
    pub fn new(addr: u32) -> Self {
        Self {
            seg: u16::try_from((addr >> 4) & 0xffff).unwrap_or_default(),
            ofs: u16::try_from(addr & 0xf).unwrap_or_default(),
        }
    }
}

impl CpuState for State86 {
    fn jump(&mut self, addr: u32) {
        self.seg = u16::try_from((addr >> 4) & 0xffff).unwrap_or_default();
        self.ofs = u16::try_from(addr & 0xf).unwrap_or_default();
    }
    fn address(&self) -> u32 {
        (u32::from(self.seg) << 4) + u32::from(self.ofs)
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Default, PartialEq)]
enum Opr {
    #[default]
    None,
    Abs,
    Imm,
    Reg,
    ExpPtr,
}

#[derive(Default)]
struct Arg {
    typ: Opr,
    base: &'static str,
    index: &'static str,
    scale: u8,
    seg: u16,
    ofs: u16,
    disp: u32,
}

impl From<&Arg> for u32 {
    fn from(val: &Arg) -> Self {
        match val.typ {
            Opr::Abs => (Self::from(val.seg) << 4) + Self::from(val.ofs),
            Opr::Imm => val.disp,
            _ => 0,
        }
    }
}

#[derive(Default)]
struct ModRM {
    set: bool,
    val: u8,
}

impl ModRM {
    pub fn reg(&mut self, handle: &Handle, offset: &mut usize) -> u8 {
        if !self.set {
            self.set = true;
            self.val = handle.r8(*offset);
            *offset += 1;
        }
        (self.val >> 3) & 7
    }
    pub fn md(&mut self, handle: &Handle, offset: &mut usize) -> u8 {
        if !self.set {
            self.set = true;
            self.val = handle.r8(*offset);
            *offset += 1;
        }
        self.val >> 6
    }
    pub fn rm(&mut self, handle: &Handle, offset: &mut usize) -> u8 {
        if !self.set {
            self.set = true;
            self.val = handle.r8(*offset);
            *offset += 1;
        }
        self.val & 7
    }
}

fn sib(arg: &mut Arg, md: u8, handle: &Handle, offset: &mut usize) {
    let sib = handle.r8(*offset);
    *offset += 1;
    if (sib & 7) == 5 && md == 0 {
        arg.disp = handle.r32(*offset);
        *offset += 4;
    } else {
        arg.base = REGNAMES[usize::from(sib & 7) + 1];
    }
    arg.scale = 1 << ((sib & 0xc0) >> 6);
    if (sib & 0x38) != 0x20 {
        arg.index = REGNAMES[usize::from((sib & 0x38) >> 3) + 1];
    }
}

impl Arg {
    pub const fn reg(name: &'static str) -> Self {
        Self {
            typ: Opr::Reg,
            base: name,
            index: "",
            scale: 0,
            seg: 0,
            ofs: 0,
            disp: 0,
        }
    }
    pub const fn seg_ofs(seg: u16, ofs: u16) -> Self {
        Self {
            typ: Opr::Abs,
            base: "",
            index: "",
            scale: 0,
            seg,
            ofs,
            disp: 0,
        }
    }
    pub const fn imm(imm: u32) -> Self {
        Self {
            typ: Opr::Imm,
            base: "",
            index: "",
            scale: 0,
            seg: 0,
            ofs: 0,
            disp: imm,
        }
    }
    pub fn modrm(md: u8, rm: u8, width: usize, handle: &Handle, offset: &mut usize) -> Self {
        if md == 3 {
            return Self::reg(REGNAMES[usize::from(rm) + REGOFS[width]]);
        }
        if width <= 2 {
            return Self::modrm16(md, rm, handle, offset);
        }
        let mut arg = Self {
            typ: Opr::ExpPtr,
            base: "",
            index: "",
            scale: 0,
            seg: 0,
            ofs: 0,
            disp: 0,
        };
        if md == 0 {
            if rm == 5 {
                arg.disp = handle.r32(*offset);
                *offset += 4;
            } else if rm == 4 {
                sib(&mut arg, md, handle, offset);
            } else {
                arg.base = REGNAMES[usize::from(rm) + 1];
            }
        } else {
            if rm == 4 {
                sib(&mut arg, md, handle, offset);
            } else {
                arg.base = REGNAMES[usize::from(rm) + 1];
            }
            if md == 1 {
                arg.disp = i32::from(handle.r8(*offset).cast_signed()).cast_unsigned();
                *offset += 1;
            } else {
                arg.disp = handle.r32(*offset);
                *offset += 4;
            }
        }
        arg
    }

    fn modrm16(md: u8, rm: u8, handle: &Handle, offset: &mut usize) -> Self {
        let mut arg = Self {
            typ: Opr::ExpPtr,
            base: "",
            index: "",
            scale: 0,
            seg: 0,
            ofs: 0,
            disp: 0,
        };
        match rm {
            0 => {
                arg.base = REGNAMES[9 + 3];
                arg.index = REGNAMES[9 + 6];
            }
            1 => {
                arg.base = REGNAMES[9 + 3];
                arg.index = REGNAMES[9 + 7];
            }
            2 => {
                arg.base = REGNAMES[9 + 5];
                arg.index = REGNAMES[9 + 6];
            }
            3 => {
                arg.base = REGNAMES[9 + 5];
                arg.index = REGNAMES[9 + 7];
            }
            4 => arg.base = REGNAMES[9 + 6],
            5 => arg.base = REGNAMES[9 + 7],
            6 => {
                if md != 0 {
                    arg.base = REGNAMES[9 + 5];
                } else {
                    arg.disp = u32::from(handle.r16(*offset));
                    *offset += 2;
                }
            }
            7 => arg.base = REGNAMES[9 + 3],
            _ => {}
        }
        if md == 1 {
            arg.disp = i32::from(handle.r8(*offset).cast_signed()).cast_unsigned();
            *offset += 1;
        } else if md == 2 {
            arg.disp = i32::from(handle.r16(*offset).cast_signed()).cast_unsigned();
            *offset += 2;
        }
        arg
    }
}

mod flags {
    pub const SEG_CS: u16 = 0x01;
    //    pub const SEG_DS: u16 = 0x02;
    pub const SEG_ES: u16 = 0x04;
    pub const SEG_SS: u16 = 0x08;
    pub const REPZ: u16 = 0x40;
    pub const REPNZ: u16 = 0x80;
    pub const BYTE: u16 = 0x100;
    pub const WORD: u16 = 0x200;
    pub const DWORD: u16 = 0x400;
    pub const FAR: u16 = 0x800;
}

#[derive(Default)]
pub struct Operands86 {
    args: Vec<Arg>,
    flags: u16,
}

impl Operand for Operands86 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct Op {
    name: &'static str,
    args: &'static str,
    i: InstType,
    children: HashMap<u8, Self>,
}

pub struct Cpu86 {
    opcodes: HashMap<u8, Op>,
}

impl Default for Cpu86 {
    fn default() -> Self {
        let mut opcodes = HashMap::new();
        for opcode in &OPCODES {
            let child = Op {
                name: opcode.n,
                args: opcode.a,
                i: opcode.i,
                children: HashMap::new(),
            };
            if opcode.i == InstType::Prefix {
                opcodes.insert(opcode.o, child);
            } else if let Some(r) = opcode.r {
                let e = opcodes.entry(opcode.o).or_insert_with(|| Op {
                    name: "",
                    args: "",
                    i: InstType::Special,
                    children: HashMap::new(),
                });
                e.children.insert(r, child);
            } else {
                opcodes.insert(opcode.o, child);
            }
        }
        let pfx = opcodes.entry(0x0f).or_insert_with(|| Op {
            name: "",
            args: "",
            i: InstType::Prefix,
            children: HashMap::new(),
        });
        for opcode in &OPCODES0F {
            let child = Op {
                name: opcode.n,
                args: opcode.a,
                i: opcode.i,
                children: HashMap::new(),
            };
            if opcode.i == InstType::Prefix {
                pfx.children.insert(opcode.o, child);
            } else if let Some(r) = opcode.r {
                let e = pfx.children.entry(opcode.o).or_insert_with(|| Op {
                    name: "",
                    args: "",
                    i: InstType::Special,
                    children: HashMap::new(),
                });
                e.children.insert(r, child);
            } else {
                pfx.children.insert(opcode.o, child);
            }
        }
        Self { opcodes }
    }
}

impl Cpu for Cpu86 {
    #[allow(clippy::too_many_lines)]
    fn decode(
        &self,
        handle: &Handle,
        offset: usize,
        state: &mut dyn CpuState,
    ) -> Result<Instruction, &'static str> {
        let Some(state) = state.as_any().downcast_mut::<State86>() else {
            return Err("x86 disassembler called without a x86 state");
        };
        let start_offset = offset;
        let mut offset = offset;
        let mut op = handle.r8(offset);
        offset += 1;
        let mut sub = Box::new(Operands86 {
            args: vec![],
            flags: 0,
        });
        let mut opcode = self.opcodes.get(&op).ok_or("Invalid opcode")?;
        let mut modrm = ModRM::default();
        let width = 2; // how does this change to 4?
        while opcode.i == InstType::Prefix || opcode.i == InstType::Special {
            if opcode.i == InstType::Prefix {
                match op {
                    0x0f => {
                        // two-byte opcode
                        op = handle.r8(offset);
                        offset += 1;
                        opcode = self
                            .opcodes
                            .get(&0xf)
                            .ok_or("Invalid opcode")?
                            .children
                            .get(&op)
                            .ok_or("Invalid 0f opcode")?;
                    }
                    0x26 => {
                        // es:
                        sub.flags |= flags::SEG_ES;
                        op = handle.r8(offset);
                        offset += 1;
                        opcode = self.opcodes.get(&op).ok_or("Invalid opcode")?;
                    }
                    0x2e => {
                        // cs:
                        sub.flags |= flags::SEG_CS;
                        op = handle.r8(offset);
                        offset += 1;
                        opcode = self.opcodes.get(&op).ok_or("Invalid opcode")?;
                    }
                    0x36 => {
                        // ss:
                        sub.flags |= flags::SEG_SS;
                        op = handle.r8(offset);
                        offset += 1;
                        opcode = self.opcodes.get(&op).ok_or("Invalid opcode")?;
                    }
                    0xf2 => {
                        // repnz
                        sub.flags |= flags::REPNZ;
                        op = handle.r8(offset);
                        offset += 1;
                        opcode = self.opcodes.get(&op).ok_or("Invalid opcode")?;
                    }
                    0xf3 => {
                        // rep
                        sub.flags |= flags::REPZ;
                        op = handle.r8(offset);
                        offset += 1;
                        opcode = self.opcodes.get(&op).ok_or("Invalid opcode")?;
                    }
                    _ => {
                        eprintln!("prefix: {op:x}");
                        return Err("Invalid prefix");
                    }
                }
            } else if opcode.i == InstType::Special {
                opcode = opcode
                    .children
                    .get(&modrm.reg(handle, &mut offset))
                    .ok_or("Invalid modrm")?;
            }
        }

        // loop over args
        let mut it = opcode.args.chars();
        while let Some(ch) = it.next() {
            match ch {
                'a' => {
                    // ah al ap ax av
                    let ch = it.next().unwrap();
                    match ch {
                        'h' => sub.args.push(Arg::reg(REGNAMES[REGOFS[1] + 4])),
                        'l' => sub.args.push(Arg::reg(REGNAMES[REGOFS[1]])),
                        'p' => {
                            let ofs = handle.r16(offset);
                            offset += 2;
                            let seg = handle.r16(offset);
                            offset += 2;
                            sub.args.push(Arg::seg_ofs(seg, ofs));
                        }
                        'x' => sub.args.push(Arg::reg(REGNAMES[REGOFS[width]])),
                        'v' => {
                            let ofs = state.ofs + handle.r16(offset);
                            offset += 2;
                            let seg = if width == 4 {
                                let seg = handle.r16(offset);
                                offset += 2;
                                seg
                            } else {
                                state.seg
                            };
                            let ofs = ofs.wrapping_add(
                                u16::try_from(offset - start_offset).unwrap_or_default(),
                            );
                            sub.args.push(Arg::seg_ofs(seg, ofs));
                        }
                        _ => return Err("unhandled a?")?,
                    }
                }
                'b' => {
                    // bl bp bx
                    let ch = it.next().unwrap();
                    match ch {
                        'l' => sub.args.push(Arg::reg(REGNAMES[REGOFS[1] + 3])),
                        'p' => sub.args.push(Arg::reg(REGNAMES[REGOFS[width] + 5])),
                        'x' => sub.args.push(Arg::reg(REGNAMES[REGOFS[width] + 3])),
                        _ => Err("unhandled b?")?,
                    }
                }
                'c' => {
                    // cl cx
                    let ch = it.next().unwrap();
                    match ch {
                        'l' => sub.args.push(Arg::reg(REGNAMES[REGOFS[1] + 1])),
                        'x' => sub.args.push(Arg::reg(REGNAMES[REGOFS[width] + 1])),
                        _ => Err("unhandled c?")?,
                    }
                }
                'd' => {
                    // dh di dl dx
                    let ch = it.next().unwrap();
                    match ch {
                        'h' => sub.args.push(Arg::reg(REGNAMES[REGOFS[1] + 6])),
                        'i' => sub.args.push(Arg::reg(REGNAMES[REGOFS[width] + 7])),
                        'l' => sub.args.push(Arg::reg(REGNAMES[REGOFS[1] + 2])),
                        'x' => sub.args.push(Arg::reg(REGNAMES[REGOFS[width] + 2])),
                        _ => Err("unhandled d?")?,
                    }
                }
                'e' => {
                    // eb ep ev ew
                    let ch = it.next().unwrap();
                    let md = modrm.md(handle, &mut offset);
                    let rm = modrm.rm(handle, &mut offset);
                    match ch {
                        'b' => sub.args.push(Arg::modrm(md, rm, 1, handle, &mut offset)),
                        'p' => {
                            sub.args
                                .push(Arg::modrm(md, rm, width, handle, &mut offset));
                            sub.flags |= flags::FAR;
                        }
                        'v' => sub
                            .args
                            .push(Arg::modrm(md, rm, width, handle, &mut offset)),
                        'w' => sub.args.push(Arg::modrm(md, rm, 2, handle, &mut offset)),
                        _ => Err("unhandled e?")?,
                    }
                }
                'g' => {
                    // gb gv
                    let ch = it.next().unwrap();
                    let reg = usize::from(modrm.reg(handle, &mut offset));
                    match ch {
                        'b' => sub.args.push(Arg::reg(REGNAMES[reg + REGOFS[1]])),
                        'v' => sub.args.push(Arg::reg(REGNAMES[reg + REGOFS[width]])),
                        _ => Err("unhandled g?")?,
                    }
                }
                'i' => {
                    // ib iv iw
                    let ch = it.next().unwrap();
                    match ch {
                        'b' => {
                            let val = handle.r8(offset);
                            offset += 1;
                            sub.args.push(Arg::imm(u32::from(val)));
                            sub.flags |= flags::BYTE;
                        }
                        'v' => {
                            if width == 4 {
                                let val = handle.r32(offset);
                                offset += 4;
                                sub.args.push(Arg::imm(val));
                                sub.flags |= flags::DWORD;
                            } else {
                                let val = handle.r16(offset);
                                offset += 2;
                                sub.args.push(Arg::imm(u32::from(val)));
                                sub.flags |= flags::WORD;
                            }
                        }
                        'w' => {
                            let val = handle.r16(offset);
                            offset += 2;
                            sub.args.push(Arg::imm(u32::from(val)));
                            sub.flags |= flags::WORD;
                        }
                        _ => Err("unhandled i?")?,
                    }
                }
                'j' => {
                    // jb jv
                    let ch = it.next().unwrap();
                    match ch {
                        'b' => {
                            let ofs = state
                                .ofs
                                .wrapping_add_signed(i16::from(handle.r8(offset).cast_signed()));
                            offset += 1;
                            let ofs = ofs.wrapping_add(
                                u16::try_from(offset - start_offset).unwrap_or_default(),
                            );
                            sub.args.push(Arg::seg_ofs(state.seg, ofs));
                        }
                        'v' => {
                            let ofs = state
                                .ofs
                                .wrapping_add_signed(handle.r16(offset).cast_signed());
                            offset += 2;
                            let seg = if width == 4 {
                                let seg = handle.r16(offset);
                                offset += 2;
                                seg
                            } else {
                                state.seg
                            };
                            let ofs = ofs.wrapping_add(
                                u16::try_from(offset - start_offset).unwrap_or_default(),
                            );
                            sub.args.push(Arg::seg_ofs(seg, ofs));
                        }
                        _ => Err("unhandled j?")?,
                    }
                }
                'm' => {
                    // mm mp
                    let ch = it.next().unwrap();
                    let md = modrm.md(handle, &mut offset);
                    let rm = modrm.rm(handle, &mut offset);
                    match ch {
                        'm' | 'p' => sub
                            .args
                            .push(Arg::modrm(md, rm, width, handle, &mut offset)),
                        _ => Err("unhandled m?")?,
                    }
                }
                'o' => {
                    // ob ov
                    let ch = it.next().unwrap();
                    match ch {
                        'b' => {
                            let mut arg = Arg::imm(u32::from(handle.r16(offset)));
                            offset += 2;
                            arg.typ = Opr::ExpPtr;
                            sub.args.push(arg);
                        }
                        'v' => {
                            if width == 4 {
                                let seg = handle.r16(offset);
                                offset += 2;
                                let ofs = handle.r16(offset);
                                offset += 2;
                                sub.args.push(Arg::seg_ofs(seg, ofs));
                            } else {
                                let mut arg = Arg::imm(u32::from(handle.r16(offset)));
                                offset += 2;
                                arg.typ = Opr::ExpPtr;
                                sub.args.push(arg);
                            }
                        }
                        _ => Err("unhandled o?")?,
                    }
                }
                's' => {
                    // sc sd se si sp ss sw
                    let ch = it.next().unwrap();
                    match ch {
                        'c' => sub.args.push(Arg::reg("cs")),
                        'd' => sub.args.push(Arg::reg("ds")),
                        'e' => sub.args.push(Arg::reg("es")),
                        'i' => sub.args.push(Arg::reg(REGNAMES[REGOFS[width] + 6])),
                        'p' => sub.args.push(Arg::reg(REGNAMES[REGOFS[width] + 4])),
                        's' => sub.args.push(Arg::reg("ss")),
                        'w' => sub.args.push(Arg::reg(
                            REGNAMES[25 + usize::from(modrm.reg(handle, &mut offset))],
                        )),
                        _ => Err("unhandled s?")?,
                    }
                }
                '#' => {
                    it.next();
                    sub.args.push(Arg::imm(1));
                }
                _ => Err("unhandled args")?,
            }
        }

        // parse '|' in name
        let mut name = String::new();
        name.push_str(opcode.name);
        let length = offset - start_offset;
        state.ofs += u16::try_from(length).unwrap_or_default();
        let raw = handle.read(start_offset, length).to_vec();
        Ok(Instruction {
            name,
            inst_type: opcode.i,
            length: u32::try_from(length).unwrap_or_default(),
            raw,
            operands: sub,
        })
    }
    fn disassemble(
        &self,
        address: u32,
        inst: &Instruction,
        lookup: &dyn Fn(u32) -> Option<String>,
    ) -> CpuLine {
        let Some(sub) = inst.operands.as_any().downcast_ref::<Operands86>() else {
            return CpuLine {
                opcode: "invalid operand".to_string(),
                ..Default::default()
            };
        };
        let mut out = CpuLine {
            opcode: format!("{:8}", inst.name),
            address,
            target: self.target(inst),
            ..Default::default()
        };
        for (idx, code) in inst.raw.iter().enumerate() {
            if out.hex.len() == 11 {
                out.hex.push('…');
                break;
            }
            if idx != 0 {
                out.hex.push(' ');
            }
            out.hex.push(HEX[usize::from(code >> 4)]);
            out.hex.push(HEX[usize::from(code & 0xf)]);
        }
        while out.hex.chars().count() < 13 {
            out.hex.push(' ');
        }
        if sub.flags & flags::REPZ != 0 {
            out.opcode = format!("repz {} ", inst.name);
        }
        if sub.flags & flags::REPNZ != 0 {
            out.opcode = format!("repnz {} ", inst.name);
        }
        for arg in &sub.args {
            if !out.operand.is_empty() {
                out.operand.push_str(", ");
            }
            match arg.typ {
                Opr::None => {}
                Opr::Abs => {
                    if let Some(str) = lookup(arg.into()) {
                        out.operand.push_str(&str);
                    } else {
                        out.operand
                            .push_str(format!("{:04x}:{:04x}", arg.seg, arg.ofs).as_str());
                    }
                }
                Opr::Imm => {
                    out.operand
                        .push_str(format!("#{}", to_hex(arg.disp)).as_str());
                }
                Opr::Reg => {
                    out.operand.push_str(arg.base);
                }
                Opr::ExpPtr => exp_ptr(sub, arg, &mut out),
            }
        }
        out
    }
    fn target(&self, inst: &Instruction) -> Option<u32> {
        if let Some(sub) = inst.operands.as_any().downcast_ref::<Operands86>() {
            for arg in &sub.args {
                if arg.typ == Opr::Imm || arg.typ == Opr::Abs {
                    return Some(arg.into());
                }
            }
        }
        None
    }
}

fn exp_ptr(sub: &Operands86, arg: &Arg, out: &mut CpuLine) {
    if sub.flags & flags::BYTE != 0 {
        out.operand.push_str("byte ptr ");
    }
    if sub.flags & flags::WORD != 0 {
        out.operand.push_str("word ptr ");
    }
    if sub.flags & flags::DWORD != 0 {
        out.operand.push_str("dword ptr ");
    }
    if sub.flags & flags::FAR != 0 {
        out.operand.push_str("far ptr ");
    }
    if sub.flags & flags::SEG_ES != 0 {
        out.operand.push_str("es:");
    }
    if sub.flags & flags::SEG_CS != 0 {
        out.operand.push_str("cs:");
    }
    if sub.flags & flags::SEG_SS != 0 {
        out.operand.push_str("ss:");
    }
    out.operand.push('[');
    if !arg.base.is_empty() && !arg.index.is_empty() {
        out.operand.push_str(arg.base);
        out.operand.push('+');
        out.operand.push_str(arg.index);
    } else if !arg.base.is_empty() {
        out.operand.push_str(arg.base);
    } else {
        out.operand.push_str(arg.index);
    }
    if !arg.base.is_empty() || !arg.index.is_empty() {
        if arg.scale > 1 {
            out.operand.push('*');
            out.operand.push_str(SCALES[usize::from(arg.scale)]);
        }
        if arg.disp != 0 {
            if arg.disp & 0x8000_0000 != 0 {
                out.operand.push('+');
            }
            out.operand.push_str(&to_hex(arg.disp));
        }
    } else {
        out.operand.push_str(&to_hex(arg.disp));
    }
    out.operand.push(']');
}
fn to_hex(val: u32) -> String {
    if (val & !0xff) == !0xff {
        format!("-${:02x}", !val + 1)
    } else if (val & !0xff) == 0 {
        format!("${val:02x}")
    } else if (val & !0xffff) == !0xffff {
        format!("-${:04x}", !val + 1)
    } else if (val & !0xffff) == 0 {
        format!("${val:04x}")
    } else if val.cast_signed() < 0 {
        format!("-${:08x}", !val + 1)
    } else {
        format!("${val:08x}")
    }
}

const REGNAMES: [&str; 31] = [
    "", "eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi", "ax", "cx", "dx", "bx", "sp", "bp",
    "si", "di", "al", "cl", "dl", "bl", "ah", "ch", "dh", "bh", "es", "cs", "ss", "ds", "fs", "gs",
];

const REGOFS: [usize; 5] = [
    0, 17, // byte
    9,  // word
    0,  // 3
    1,  // dword
];

const SCALES: [&str; 9] = [
    "", "", // *1 is redundant
    "*2", "", "*4", "", "", "", "*8",
];

const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
