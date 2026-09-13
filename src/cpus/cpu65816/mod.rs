/*

65816 disassembler
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{
        Cpu, CpuLine, CpuState, InstType, Instruction, Operand,
        cpu65816::opcodes::{Addressing, OPCODES},
    },
    lunacy::File,
};
use lazy_static::lazy_static;
use std::collections::HashMap;

mod opcodes;

mod flags {
    pub const X8: u32 = 0x10;
    pub const M8: u32 = 0x20;
    pub const EMU: u32 = 0x100;
    pub const FLAGS: u32 = 0xff0130;
    pub const X8_CHANGED: u32 = 0x200;
    pub const M8_CHANGED: u32 = 0x400;
    pub const EMU_CHANGED: u32 = 0x2000;
}

#[derive(Clone, Default)]
pub struct State65816 {
    pub addr: u32,
    pub flags: u32,
}

impl CpuState for State65816 {
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
    Imm,
    Abs,
    AbsB,
    AbsD,
    AbsX,
    AbsXB,
    AbsXD,
    AbsY,
    AbsYB,
    AbsYD,
    AbsS,
    Ind,
    IndB,
    IndD,
    IndX,
    IndXB,
    IndXD,
    IndY,
    IndL,
    IndLY,
    IndS,
    Bank,
}

#[derive(Default)]
pub struct Operand65816 {
    oper_type: Opr,
    oper: u32,
    flags: u32,
}
impl Operand for Operand65816 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl Operand65816 {
    fn to_hex(&self) -> String {
        if (self.oper & !0xff) == !0xff {
            format!("$-{:02x}", !self.oper + 1)
        } else if (self.oper & !0xff) == 0 {
            format!("${:02x}", self.oper)
        } else if (self.oper & !0xffff) == !0xffff {
            format!("$-{:04x}", !self.oper + 1)
        } else if (self.oper & !0xffff) == 0 {
            format!("${:04x}", self.oper)
        } else if (self.oper & !0xffffff) == 0 {
            format!("${:02x}/{:04x}", self.oper >> 16, self.oper & 0xffff)
        } else if (self.oper & 0x80000000) != 0 {
            format!("$-{:08x}", !self.oper + 1)
        } else {
            format!("${:08x}", self.oper)
        }
    }
}

#[derive(Default)]
pub struct Cpu65816 {}

impl Cpu for Cpu65816 {
    fn decode(
        &self,
        file: &File,
        offset: usize,
        state: &mut dyn CpuState,
    ) -> Result<Instruction, &'static str> {
        let state = if let Some(state) = state.as_any().downcast_mut::<State65816>() {
            state
        } else {
            return Err("65816 disassembler called without a 65816 state");
        };
        let start_offset = offset;
        if offset >= file.len() {
            return Err("end of file reached");
        }
        let op = file.r8(offset);
        let opcode = &OPCODES[usize::from(op)];
        let offset = offset + 1;
        if opcode.i == InstType::Invalid {
            return Err("invalid opcode");
        }
        let mode = &opcode.a;
        let length = if (mode == &Addressing::ImmM && (state.flags & flags::M8) != 0)
            || (mode == &Addressing::ImmX && (state.flags & flags::X8) != 0)
        {
            SIZES[mode] - 1
        } else {
            SIZES[mode]
        };

        let addr = state.addr + length;
        state.jump(addr);
        state.flags &= flags::FLAGS; // clear changed flags
        let old_flags = state.flags;

        let mut operand = Box::new(Operand65816 {
            oper_type: Opr::None,
            oper: 0,
            flags: 0,
        });

        match mode {
            Addressing::Imp => {}
            Addressing::Imm => {
                operand.oper = u32::from(file.r8(offset));
                operand.oper_type = Opr::Imm;
                if op == 0xe2 {
                    state.flags |= operand.oper & flags::FLAGS;
                } else if op == 0xc2 {
                    state.flags &= !operand.oper;
                }
            }
            Addressing::ImmM => {
                if state.flags & flags::M8 == 0 {
                    operand.oper = u32::from(file.r16(offset));
                } else {
                    operand.oper = u32::from(file.r8(offset));
                }
                operand.oper_type = Opr::Imm;
            }
            Addressing::ImmX => {
                if state.flags & flags::X8 == 0 {
                    operand.oper = u32::from(file.r16(offset));
                } else {
                    operand.oper = u32::from(file.r8(offset));
                }
                operand.oper_type = Opr::Imm;
            }
            Addressing::Imms => {
                operand.oper = u32::from(file.r16(offset));
                operand.oper_type = Opr::Imm;
            }
            Addressing::Abs => {
                operand.oper = u32::from(file.r16(offset));
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper |= state.addr & 0xff0000; // K
                    operand.oper_type = Opr::Abs;
                } else {
                    operand.oper_type = Opr::AbsB;
                }
            }
            Addressing::Abl => {
                operand.oper = file.r24(offset);
                operand.oper_type = Opr::Abs;
            }
            Addressing::Abx => {
                operand.oper = u32::from(file.r16(offset));
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper |= state.addr & 0xff0000; // K
                    operand.oper_type = Opr::AbsX;
                } else {
                    operand.oper_type = Opr::AbsXB;
                }
            }
            Addressing::Aby => {
                operand.oper = u32::from(file.r16(offset));
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper |= state.addr & 0xff0000; // K
                    operand.oper_type = Opr::AbsY;
                } else {
                    operand.oper_type = Opr::AbsYB;
                }
            }
            Addressing::Ablx => {
                operand.oper = file.r24(offset);
                operand.oper_type = Opr::AbsX;
            }
            Addressing::Aix => {
                operand.oper = u32::from(file.r16(offset));
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper |= state.addr & 0xff0000; // K
                    operand.oper_type = Opr::IndX;
                } else {
                    operand.oper_type = Opr::IndXB;
                }
            }
            Addressing::Zp => {
                operand.oper = u32::from(file.r8(offset));
                operand.oper_type = Opr::AbsD;
            }
            Addressing::Zpx => {
                operand.oper = u32::from(file.r8(offset));
                operand.oper_type = Opr::AbsXD;
            }
            Addressing::Zpy => {
                operand.oper = u32::from(file.r8(offset));
                operand.oper_type = Opr::AbsYD;
            }
            Addressing::Zps => {
                operand.oper = u32::from(file.r8(offset));
                operand.oper_type = Opr::AbsS;
            }
            Addressing::Ind => {
                operand.oper = u32::from(file.r16(offset));
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper |= state.addr & 0xff0000; // K
                    operand.oper_type = Opr::Ind;
                } else {
                    operand.oper_type = Opr::IndB;
                }
            }
            Addressing::Inz => {
                operand.oper = u32::from(file.r8(offset));
                operand.oper_type = Opr::IndD;
            }
            Addressing::Inl => {
                operand.oper = u32::from(file.r8(offset));
                operand.oper_type = Opr::IndL;
            }
            Addressing::Inx => {
                operand.oper = u32::from(file.r8(offset));
                operand.oper_type = Opr::IndXD;
            }
            Addressing::Iny => {
                operand.oper = u32::from(file.r8(offset));
                operand.oper_type = Opr::IndY;
            }
            Addressing::Inly => {
                operand.oper = u32::from(file.r8(offset));
                operand.oper_type = Opr::IndLY;
            }
            Addressing::Ins => {
                operand.oper = u32::from(file.r8(offset));
                operand.oper_type = Opr::IndS;
            }
            Addressing::Rel => {
                operand.oper = addr.wrapping_add((file.r8(offset) as i8) as u32);
                operand.oper_type = Opr::Abs;
            }
            Addressing::Rell => {
                operand.oper = addr.wrapping_add((file.r16(offset) as i16) as u32);
                operand.oper_type = Opr::Abs;
            }
            Addressing::Bank => {
                operand.oper = u32::from(file.r16(offset));
                operand.oper_type = Opr::Bank;
            }
            Addressing::Db => {}
        }
        if op == 0x18 && file.r8(offset) == 0xfb {
            // CLC XCE
            state.flags &= !flags::EMU;
        }
        if op == 0x38 && file.r8(offset) == 0xfb {
            // SEC XCE
            state.flags |= flags::EMU | flags::M8 | flags::X8;
        }
        if (state.flags ^ old_flags) & flags::EMU != 0 {
            state.flags |= flags::EMU_CHANGED;
        }
        if (state.flags ^ old_flags) & flags::X8 != 0 {
            state.flags |= flags::X8_CHANGED;
        }
        if (state.flags ^ old_flags) & flags::M8 != 0 {
            state.flags |= flags::M8_CHANGED;
        }
        let raw = file.read(start_offset, length as usize).to_vec();

        Ok(Instruction {
            name: opcode.n.to_string(),
            inst_type: opcode.i,
            length,
            raw,
            operands: operand,
        })
    }

    fn disassemble(
        &self,
        address: u32,
        inst: &Instruction,
        lookup: &dyn Fn(u32) -> Option<String>,
    ) -> CpuLine {
        let operand = if let Some(operand) = inst.operands.as_any().downcast_ref::<Operand65816>() {
            operand
        } else {
            return CpuLine {
                opcode: "invalid operand".to_string(),
                ..Default::default()
            };
        };
        let mut out = CpuLine {
            opcode: format!("{:5}", inst.name),
            address,
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
        match operand.oper_type {
            Opr::Imm => {
                out.operand = format!("#{}", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::Abs => {
                out.operand = operand.to_hex();
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::AbsB => {
                out.operand = format!("B:{}", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::AbsD => {
                out.operand = format!("D:{}", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::AbsX => {
                out.operand = format!("{}, x", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::AbsXB => {
                out.operand = format!("B:{}, x", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::AbsXD => {
                out.operand = format!("D:{}, x", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::AbsY => {
                out.operand = format!("{}, y", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::AbsYB => {
                out.operand = format!("B:{}, y", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::AbsYD => {
                out.operand = format!("D:{}, y", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::AbsS => {
                out.operand = format!("{}, s", operand.to_hex());
            }
            Opr::Ind => {
                out.operand = format!("({})", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::IndB => {
                out.operand = format!("(B:{})", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::IndD => {
                out.operand = format!("(D:{})", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::IndX => {
                out.operand = format!("({}, x)", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::IndXB => {
                out.operand = format!("(B:{}, x)", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::IndXD => {
                out.operand = format!("(D:{}, x)", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::IndY => {
                out.operand = format!("({}), y", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::IndL => {
                out.operand = format!("[{}]", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::IndLY => {
                out.operand = format!("[{}], y", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            Opr::IndS => {
                out.operand = format!("({}, s), y", operand.to_hex());
            }
            Opr::Bank => {
                out.operand = format!("{:02x}, {:02x}", operand.oper >> 8, operand.oper & 0xff);
            }
            Opr::None => {}
        }
        if operand.flags & flags::EMU_CHANGED != 0 {
            if operand.flags & flags::EMU == 0 {
                out.comment.push_str(" 16-bit mode");
            } else {
                out.comment.push_str(" 8-bit mode");
            }
        }
        if operand.flags & flags::M8_CHANGED != 0 {
            if operand.flags & flags::M8 == 0 {
                out.comment.push_str(" a.w");
            } else {
                out.comment.push_str(" a.b");
            }
        }
        if operand.flags & flags::X8_CHANGED != 0 {
            if operand.flags & flags::X8 == 0 {
                out.comment.push_str(" x.w");
            } else {
                out.comment.push_str(" x.b");
            }
        }
        out
    }

    fn target(&self, inst: &Instruction) -> Option<u32> {
        if let Some(opr) = inst.operands.as_any().downcast_ref::<Operand65816>()
            && (opr.oper_type == Opr::Imm || opr.oper_type == Opr::Abs)
        {
            Some(opr.oper)
        } else {
            None
        }
    }
}

lazy_static! {
    static ref SIZES: HashMap<Addressing, u32> = {
        let mut m = HashMap::new();
        m.insert(Addressing::Imp, 1);
        m.insert(Addressing::Imm, 2);
        m.insert(Addressing::ImmM, 3);
        m.insert(Addressing::ImmX, 3);
        m.insert(Addressing::Imms, 3);
        m.insert(Addressing::Abs, 3);
        m.insert(Addressing::Abl, 4);
        m.insert(Addressing::Abx, 3);
        m.insert(Addressing::Aby, 3);
        m.insert(Addressing::Ablx, 4);
        m.insert(Addressing::Aix, 3);
        m.insert(Addressing::Zp, 2);
        m.insert(Addressing::Zpx, 2);
        m.insert(Addressing::Zpy, 2);
        m.insert(Addressing::Zps, 2);
        m.insert(Addressing::Ind, 3);
        m.insert(Addressing::Inz, 2);
        m.insert(Addressing::Inl, 2);
        m.insert(Addressing::Inx, 2);
        m.insert(Addressing::Iny, 2);
        m.insert(Addressing::Inly, 2);
        m.insert(Addressing::Ins, 2);
        m.insert(Addressing::Rel, 2);
        m.insert(Addressing::Rell, 3);
        m.insert(Addressing::Bank, 3);
        m
    };
}

const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
