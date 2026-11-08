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
pub use address::Address65816;
use std::{collections::HashMap, fmt, sync::LazyLock};

mod address;
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
    pub addr: Address65816,
    pub flags: u32,
}

impl State65816 {
    pub const fn new(addr: Address65816, flags: u32) -> Self {
        Self { addr, flags }
    }
}

impl CpuState<Address65816> for State65816 {
    fn jump(&mut self, addr: Address65816) {
        self.addr = addr;
    }
    fn address(&self) -> Address65816 {
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
    AbsX,
    AbsY,
    AbsS,
    Ind,
    IndX,
    IndY,
    IndL,
    IndLY,
    IndS,
    Bank,
}

#[derive(Default)]
pub struct Operand65816 {
    oper_type: Opr,
    oper: Address65816,
    flags: u32,
}
impl Operand for Operand65816 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl fmt::Display for Operand65816 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.oper_type {
            Opr::Imm => write!(f, "#{}", self.oper),
            Opr::Abs => write!(f, "{}", self.oper),
            Opr::AbsX => write!(f, "{}, x", self.oper),
            Opr::AbsY => write!(f, "{}, y", self.oper),
            Opr::AbsS => write!(f, "{}, s", self.oper),
            Opr::Ind => write!(f, "({})", self.oper),
            Opr::IndX => write!(f, "({}, x)", self.oper),
            Opr::IndY => write!(f, "({}), y", self.oper),
            Opr::IndL => write!(f, "[{}]", self.oper),
            Opr::IndLY => write!(f, "[{}], y", self.oper),
            Opr::IndS => write!(f, "({}, s), y", self.oper),
            Opr::Bank => {
                if let Address65816::Normal(v) = self.oper {
                    write!(f, "{:02x}, {:02x}", v >> 8, v & 0xff)
                } else {
                    Ok(())
                }
            }
            Opr::None => Ok(()),
        }
    }
}

impl Operand65816 {
    pub const fn or(&mut self, byte: u32) {
        self.oper_type = Opr::Imm;
        self.oper.or(byte);
    }
}

#[derive(Default)]
pub struct Cpu65816 {}

impl Cpu<Address65816> for Cpu65816 {
    #[allow(clippy::too_many_lines)]
    fn decode(
        &self,
        handle: &Handle,
        offset: usize,
        state: &mut dyn CpuState<Address65816>,
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

        let mut operand = Box::new(Operand65816::default());

        match mode {
            Addressing::Imm => {
                let flags = handle.r8(offset);
                operand.oper = flags.into();
                operand.oper_type = Opr::Imm;
                if op == 0xe2 {
                    state.flags |= u32::from(flags) & flags::FLAGS;
                } else if op == 0xc2 {
                    state.flags &= !u32::from(flags);
                }
            }
            Addressing::ImmM => {
                if state.flags & flags::M8 == 0 {
                    operand.oper = handle.r16(offset).into();
                } else {
                    operand.oper = handle.r8(offset).into();
                }
                operand.oper_type = Opr::Imm;
            }
            Addressing::ImmX => {
                if state.flags & flags::X8 == 0 {
                    operand.oper = handle.r16(offset).into();
                } else {
                    operand.oper = handle.r8(offset).into();
                }
                operand.oper_type = Opr::Imm;
            }
            Addressing::Imms => {
                operand.oper = handle.r16(offset).into();
                operand.oper_type = Opr::Imm;
            }
            Addressing::Abs => {
                let addr = handle.r16(offset);
                operand.oper_type = Opr::Abs;
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper = Address65816::Normal(state.addr.k() | u32::from(addr));
                } else {
                    operand.oper = Address65816::B(addr);
                }
            }
            Addressing::Abl => {
                operand.oper = handle.r24(offset).into();
                operand.oper_type = Opr::Abs;
            }
            Addressing::Abx => {
                let addr = handle.r16(offset);
                operand.oper_type = Opr::AbsX;
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper = Address65816::Normal(state.addr.k() | u32::from(addr));
                } else {
                    operand.oper = Address65816::B(addr);
                }
            }
            Addressing::Aby => {
                let addr = handle.r16(offset);
                operand.oper_type = Opr::AbsY;
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper = Address65816::Normal(state.addr.k() | u32::from(addr));
                } else {
                    operand.oper = Address65816::B(addr);
                }
            }
            Addressing::Ablx => {
                operand.oper = handle.r24(offset).into();
                operand.oper_type = Opr::AbsX;
            }
            Addressing::Aix => {
                let addr = handle.r16(offset);
                operand.oper_type = Opr::IndX;
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper = Address65816::Normal(state.addr.k() | u32::from(addr));
                } else {
                    operand.oper = Address65816::B(addr);
                }
            }
            Addressing::Zp => {
                operand.oper = Address65816::D(handle.r8(offset));
                operand.oper_type = Opr::Abs;
            }
            Addressing::Zpx => {
                operand.oper = Address65816::D(handle.r8(offset));
                operand.oper_type = Opr::AbsX;
            }
            Addressing::Zpy => {
                operand.oper = Address65816::D(handle.r8(offset));
                operand.oper_type = Opr::AbsY;
            }
            Addressing::Zps => {
                operand.oper = handle.r8(offset).into();
                operand.oper_type = Opr::AbsS;
            }
            Addressing::Ind => {
                let addr = handle.r16(offset);
                operand.oper_type = Opr::Ind;
                if opcode.i == InstType::Jump || opcode.i == InstType::Call {
                    operand.oper = Address65816::Normal(state.addr.k() | u32::from(addr));
                } else {
                    operand.oper = Address65816::B(addr);
                }
            }
            Addressing::Inz => {
                operand.oper = Address65816::D(handle.r8(offset));
                operand.oper_type = Opr::Ind;
            }
            Addressing::Inl => {
                operand.oper = handle.r8(offset).into();
                operand.oper_type = Opr::IndL;
            }
            Addressing::Inx => {
                operand.oper = Address65816::D(handle.r8(offset));
                operand.oper_type = Opr::IndX;
            }
            Addressing::Iny => {
                operand.oper = handle.r8(offset).into();
                operand.oper_type = Opr::IndY;
            }
            Addressing::Inly => {
                operand.oper = handle.r8(offset).into();
                operand.oper_type = Opr::IndLY;
            }
            Addressing::Ins => {
                operand.oper = handle.r8(offset).into();
                operand.oper_type = Opr::IndS;
            }
            Addressing::Rel => {
                operand.oper = u32::from(state.addr)
                    .wrapping_add_signed(i32::from(handle.r8(offset).cast_signed()))
                    .into();
                operand.oper_type = Opr::Abs;
            }
            Addressing::Rell => {
                operand.oper = u32::from(state.addr)
                    .wrapping_add_signed(i32::from(handle.r16(offset).cast_signed()))
                    .into();
                operand.oper_type = Opr::Abs;
            }
            Addressing::Bank => {
                operand.oper = handle.r16(offset).into();
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
        let raw = handle.read(start_offset, length).to_vec();

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
        address: Address65816,
        inst: &Instruction,
        lookup: &dyn Fn(Address65816) -> Option<String>,
    ) -> CpuLine<Address65816> {
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
        out.operand = operand.to_string();
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

    fn target(&self, inst: &Instruction) -> Option<Address65816> {
        if let Some(opr) = inst.operands.as_any().downcast_ref::<Operand65816>()
            && (opr.oper_type == Opr::Imm || opr.oper_type == Opr::Abs)
        {
            Some(opr.oper)
        } else {
            None
        }
    }
}

static SIZES: LazyLock<HashMap<Addressing, usize>> = LazyLock::new(|| {
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
