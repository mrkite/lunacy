/*

65816 disassembler
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{
        Cpu, CpuLine, CpuState, InstType, Instruction, Operand,
        cpu65816::opcodes::{Addressing, OPCODES},
    },
    lunacy::Handle,
};
use std::{collections::HashMap, sync::LazyLock};

mod opcodes;

mod flags {
    pub const X8: u32 = 0x10;
    pub const M8: u32 = 0x20;
    pub const EMU: u32 = 0x100;
    pub const FLAGS: u32 = 0xff_0130;
    pub const X8_CHANGED: u32 = 0x200;
    pub const M8_CHANGED: u32 = 0x400;
    pub const EMU_CHANGED: u32 = 0x2000;
}

#[derive(Clone, Default)]
pub struct State65816 {
    pub addr: u32,
    pub flags: u32,
}

impl State65816 {
    pub fn new(addr: u32, flags: u32) -> Self {
        Self { addr, flags }
    }
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
        } else if (self.oper & !0xff_ffff) == 0 {
            format!("${:02x}/{:04x}", self.oper >> 16, self.oper & 0xffff)
        } else if (self.oper & 0x8000_0000) != 0 {
            format!("$-{:08x}", !self.oper + 1)
        } else {
            format!("${:08x}", self.oper)
        }
    }

    pub fn or(&mut self, byte: u32) {
        self.oper_type = Opr::Imm;
        self.oper |= byte;
    }
}

#[derive(Default)]
pub struct Cpu65816 {}

impl Cpu for Cpu65816 {
    #[allow(clippy::too_many_lines)]
    fn decode(
        &self,
        handle: &Handle,
        offset: usize,
        state: &mut dyn CpuState,
    ) -> Result<Instruction, &'static str> {
        let Some(state) = state.as_any().downcast_mut::<State65816>() else {
            return Err("65816 disassembler called without a 65816 state");
        };
        let start_offset = offset;
        if offset >= handle.len() {
            return Err("end of file reached");
        }
        let op = handle.r8(offset);
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

        state.addr += length;
        state.flags &= flags::FLAGS; // clear changed flags
        let old_flags = state.flags;

        let mut operand = Box::new(Operand65816 {
            oper_type: Opr::None,
            oper: 0,
            flags: 0,
        });

        match mode {
            Addressing::Imm => {
                operand.oper = u32::from(handle.r8(offset));
                operand.oper_type = Opr::Imm;
                if op == 0xe2 {
                    state.flags |= operand.oper & flags::FLAGS;
                } else if op == 0xc2 {
                    state.flags &= !operand.oper;
                }
            }
            Addressing::ImmM => {
                if state.flags & flags::M8 == 0 {
                    operand.oper = u32::from(handle.r16(offset));
                } else {
                    operand.oper = u32::from(handle.r8(offset));
                }
                operand.oper_type = Opr::Imm;
            }
            Addressing::ImmX => {
                if state.flags & flags::X8 == 0 {
                    operand.oper = u32::from(handle.r16(offset));
                } else {
                    operand.oper = u32::from(handle.r8(offset));
                }
                operand.oper_type = Opr::Imm;
            }
            Addressing::Imms => {
                operand.oper = u32::from(handle.r16(offset));
                operand.oper_type = Opr::Imm;
            }
            Addressing::Abs => {
                operand.oper = u32::from(handle.r16(offset));
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper |= state.addr & 0xff_0000; // K
                    operand.oper_type = Opr::Abs;
                } else {
                    operand.oper_type = Opr::AbsB;
                }
            }
            Addressing::Abl => {
                operand.oper = handle.r24(offset);
                operand.oper_type = Opr::Abs;
            }
            Addressing::Abx => {
                operand.oper = u32::from(handle.r16(offset));
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper |= state.addr & 0xff_0000; // K
                    operand.oper_type = Opr::AbsX;
                } else {
                    operand.oper_type = Opr::AbsXB;
                }
            }
            Addressing::Aby => {
                operand.oper = u32::from(handle.r16(offset));
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper |= state.addr & 0xff_0000; // K
                    operand.oper_type = Opr::AbsY;
                } else {
                    operand.oper_type = Opr::AbsYB;
                }
            }
            Addressing::Ablx => {
                operand.oper = handle.r24(offset);
                operand.oper_type = Opr::AbsX;
            }
            Addressing::Aix => {
                operand.oper = u32::from(handle.r16(offset));
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper |= state.addr & 0xff_0000; // K
                    operand.oper_type = Opr::IndX;
                } else {
                    operand.oper_type = Opr::IndXB;
                }
            }
            Addressing::Zp => {
                operand.oper = u32::from(handle.r8(offset));
                operand.oper_type = Opr::AbsD;
            }
            Addressing::Zpx => {
                operand.oper = u32::from(handle.r8(offset));
                operand.oper_type = Opr::AbsXD;
            }
            Addressing::Zpy => {
                operand.oper = u32::from(handle.r8(offset));
                operand.oper_type = Opr::AbsYD;
            }
            Addressing::Zps => {
                operand.oper = u32::from(handle.r8(offset));
                operand.oper_type = Opr::AbsS;
            }
            Addressing::Ind => {
                operand.oper = u32::from(handle.r16(offset));
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper |= state.addr & 0xff_0000; // K
                    operand.oper_type = Opr::Ind;
                } else {
                    operand.oper_type = Opr::IndB;
                }
            }
            Addressing::Inz => {
                operand.oper = u32::from(handle.r8(offset));
                operand.oper_type = Opr::IndD;
            }
            Addressing::Inl => {
                operand.oper = u32::from(handle.r8(offset));
                operand.oper_type = Opr::IndL;
            }
            Addressing::Inx => {
                operand.oper = u32::from(handle.r8(offset));
                operand.oper_type = Opr::IndXD;
            }
            Addressing::Iny => {
                operand.oper = u32::from(handle.r8(offset));
                operand.oper_type = Opr::IndY;
            }
            Addressing::Inly => {
                operand.oper = u32::from(handle.r8(offset));
                operand.oper_type = Opr::IndLY;
            }
            Addressing::Ins => {
                operand.oper = u32::from(handle.r8(offset));
                operand.oper_type = Opr::IndS;
            }
            Addressing::Rel => {
                operand.oper = state
                    .addr
                    .wrapping_add_signed(i32::from(handle.r8(offset).cast_signed()));
                operand.oper_type = Opr::Abs;
            }
            Addressing::Rell => {
                operand.oper = state
                    .addr
                    .wrapping_add_signed(i32::from(handle.r16(offset).cast_signed()));
                operand.oper_type = Opr::Abs;
            }
            Addressing::Bank => {
                operand.oper = u32::from(handle.r16(offset));
                operand.oper_type = Opr::Bank;
            }
            Addressing::Imp | Addressing::Db => {}
        }
        if op == 0x18 && handle.r8(offset) == 0xfb {
            // CLC XCE
            state.flags &= !flags::EMU;
        }
        if op == 0x38 && handle.r8(offset) == 0xfb {
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
        let raw = handle
            .read(start_offset, usize::try_from(length).unwrap_or_default())
            .to_vec();

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
        let Some(operand) = inst.operands.as_any().downcast_ref::<Operand65816>() else {
            return CpuLine {
                opcode: "invalid operand".to_string(),
                ..Default::default()
            };
        };
        let mut out = CpuLine {
            opcode: format!("{:5}", inst.name),
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
        out.operand = match operand.oper_type {
            Opr::Imm => format!("#{}", operand.to_hex()),
            Opr::Abs => operand.to_hex(),
            Opr::AbsB => format!("B:{}", operand.to_hex()),
            Opr::AbsD => format!("D:{}", operand.to_hex()),
            Opr::AbsX => format!("{}, x", operand.to_hex()),
            Opr::AbsXB => format!("B:{}, x", operand.to_hex()),
            Opr::AbsXD => format!("D:{}, x", operand.to_hex()),
            Opr::AbsY => format!("{}, y", operand.to_hex()),
            Opr::AbsYB => format!("B:{}, y", operand.to_hex()),
            Opr::AbsYD => format!("D:{}, y", operand.to_hex()),
            Opr::AbsS => format!("{}, s", operand.to_hex()),
            Opr::Ind => format!("({})", operand.to_hex()),
            Opr::IndB => format!("(B:{})", operand.to_hex()),
            Opr::IndD => format!("(D:{})", operand.to_hex()),
            Opr::IndX => format!("({}, x)", operand.to_hex()),
            Opr::IndXB => format!("(B:{}, x)", operand.to_hex()),
            Opr::IndXD => format!("(D:{}, x)", operand.to_hex()),
            Opr::IndY => format!("({}), y", operand.to_hex()),
            Opr::IndL => format!("[{}]", operand.to_hex()),
            Opr::IndLY => format!("[{}], y", operand.to_hex()),
            Opr::IndS => format!("({}, s), y", operand.to_hex()),
            Opr::Bank => format!("{:02x}, {:02x}", operand.oper >> 8, operand.oper & 0xff),
            Opr::None => String::new(),
        };
        if let Some(cmt) = lookup(operand.oper) {
            out.comment = cmt;
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

static SIZES: LazyLock<HashMap<Addressing, u32>> = LazyLock::new(|| {
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
});

const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
