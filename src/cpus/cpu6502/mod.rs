/*

6502 disassembler
@copyright 2026 Sean Kasun

*/

mod opcodes;

use crate::{
    cpus::{
        Cpu, CpuLine, CpuState, InstType, Instruction, Operand,
        cpu6502::opcodes::{Addressing, OPCODES},
    },
    lunacy::Handle,
};
use std::{collections::HashMap, sync::LazyLock};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct State6502 {
    pub addr: u16,
}

impl State6502 {
    pub const fn new(addr: u16) -> Self {
        Self { addr }
    }
}

impl CpuState<u16> for State6502 {
    fn jump(&mut self, addr: u16) {
        self.addr = addr;
    }
    fn address(&self) -> u16 {
        self.addr
    }
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(PartialEq)]
enum OperType {
    None,
    Imm,
    Abs,
    AbsX,
    AbsY,
    Ind,
    IndX,
    IndY,
}

struct Operand6502 {
    pub oper_type: OperType,
    pub oper: u16,
}
impl Operand for Operand6502 {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl Operand6502 {
    fn to_hex(&self) -> String {
        if (self.oper & 0xff00) == 0 {
            format!("${:02x}", self.oper)
        } else {
            format!("${:04x}", self.oper)
        }
    }
}

#[derive(Default)]
pub struct Cpu6502 {}

impl Cpu<u16> for Cpu6502 {
    fn decode(
        &self,
        handle: &Handle,
        offset: usize,
        state: &mut dyn CpuState<u16>,
    ) -> Result<Instruction, &'static str> {
        let Some(state) = state.as_any().downcast_mut::<State6502>() else {
            return Err("6502 disassembler called without a 6502 state");
        };
        let start_offset = offset;
        if offset >= handle.len() {
            return Err("end of file reached");
        }
        let opcode = &OPCODES[usize::from(handle.r8(offset))];
        let offset = offset + 1;
        if opcode.i == InstType::Invalid {
            return Err("invalid opcode");
        }
        let mode = &opcode.a;
        let length = SIZES[mode];
        let addr = state.addr + u16::try_from(length).unwrap_or_default();
        state.jump(addr);
        if start_offset + length >= handle.len() {
            return Err("end of file reached");
        }

        let mut operand = Box::new(Operand6502 {
            oper_type: OperType::None,
            oper: 0,
        });

        match mode {
            Addressing::Imp => operand.oper_type = OperType::None,
            Addressing::Imm => {
                operand.oper = u16::from(handle.r8(offset));
                operand.oper_type = OperType::Imm;
            }
            Addressing::Abs => {
                operand.oper = handle.r16(offset);
                operand.oper_type = OperType::Abs;
            }
            Addressing::Abx => {
                operand.oper = handle.r16(offset);
                operand.oper_type = OperType::AbsX;
            }
            Addressing::Aby => {
                operand.oper = handle.r16(offset);
                operand.oper_type = OperType::AbsY;
            }
            Addressing::Zp => {
                operand.oper = u16::from(handle.r8(offset));
                operand.oper_type = OperType::Abs;
            }
            Addressing::Zpx => {
                operand.oper = u16::from(handle.r8(offset));
                operand.oper_type = OperType::AbsX;
            }
            Addressing::Zpy => {
                operand.oper = u16::from(handle.r8(offset));
                operand.oper_type = OperType::AbsY;
            }
            Addressing::Ind => {
                operand.oper = handle.r16(offset);
                operand.oper_type = OperType::Ind;
            }
            Addressing::Inx => {
                operand.oper = u16::from(handle.r8(offset));
                operand.oper_type = OperType::IndX;
            }
            Addressing::Iny => {
                operand.oper = u16::from(handle.r8(offset));
                operand.oper_type = OperType::IndY;
            }
            Addressing::Rel => {
                operand.oper = addr.wrapping_add_signed(i16::from(handle.r8(offset).cast_signed()));
                operand.oper_type = OperType::Abs;
            }
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
        address: u16,
        inst: &Instruction,
        lookup: &dyn Fn(u16) -> Option<String>,
    ) -> CpuLine<u16> {
        let Some(operand) = inst.operands.as_any().downcast_ref::<Operand6502>() else {
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
            if idx != 0 {
                out.hex.push(' ');
            }
            out.hex.push(HEX[usize::from(code >> 4)]);
            out.hex.push(HEX[usize::from(code & 0xf)]);
        }
        while out.hex.len() < 9 {
            out.hex.push(' ');
        }
        match operand.oper_type {
            OperType::Imm => {
                out.operand = format!("#{}", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            OperType::Abs => {
                out.operand = operand.to_hex();
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            OperType::AbsX => {
                out.operand = format!("{}, x", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            OperType::AbsY => {
                out.operand = format!("{}, y", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            OperType::Ind => {
                out.operand = format!("({})", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            OperType::IndX => {
                out.operand = format!("({}, x)", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            OperType::IndY => {
                out.operand = format!("({}), y", operand.to_hex());
                out.comment = lookup(operand.oper).unwrap_or_default();
            }
            OperType::None => {}
        }
        out
    }

    fn target(&self, inst: &Instruction) -> Option<u16> {
        if let Some(opr) = inst.operands.as_any().downcast_ref::<Operand6502>()
            && (opr.oper_type == OperType::Imm || opr.oper_type == OperType::Abs)
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
    m.insert(Addressing::Zp, 2);
    m.insert(Addressing::Zpx, 2);
    m.insert(Addressing::Zpy, 2);
    m.insert(Addressing::Abs, 3);
    m.insert(Addressing::Abx, 3);
    m.insert(Addressing::Aby, 3);
    m.insert(Addressing::Ind, 3);
    m.insert(Addressing::Inx, 2);
    m.insert(Addressing::Iny, 2);
    m.insert(Addressing::Rel, 2);
    m
});

const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
