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
    lunacy::File,
};
use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct State6502 {
    pub addr: u16,
}

impl CpuState for State6502 {
    fn jump(&mut self, addr: u32) {
        self.addr = addr as u16;
    }
    fn address(&self) -> u32 {
        self.addr as u32
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

impl Cpu for Cpu6502 {
    fn decode(
        &self,
        file: &File,
        offset: usize,
        state: &mut dyn CpuState,
    ) -> Result<Instruction, &'static str> {
        let state = if let Some(state) = state.as_any().downcast_mut::<State6502>() {
            state
        } else {
            return Err("6502 disassembler called without a 6502 state");
        };
        let start_offset = offset;
        if offset >= file.len() {
            return Err("end of file reached");
        }
        let opcode = &OPCODES[usize::from(file.r8(offset))];
        let offset = offset + 1;
        if opcode.i == InstType::Invalid {
            return Err("invalid opcode");
        }
        let mode = &opcode.a;
        let length = SIZES[mode];
        let addr = u32::from(state.addr) + length;
        state.jump(addr);
        if start_offset + length as usize >= file.len() {
            return Err("end of file reached");
        }

        let mut operand = Box::new(Operand6502 {
            oper_type: OperType::None,
            oper: 0,
        });

        match mode {
            Addressing::Imp => operand.oper_type = OperType::None,
            Addressing::Imm => {
                operand.oper = u16::from(file.r8(offset));
                operand.oper_type = OperType::Imm;
            }
            Addressing::Abs => {
                operand.oper = file.r16(offset);
                operand.oper_type = OperType::Abs;
            }
            Addressing::Abx => {
                operand.oper = file.r16(offset);
                operand.oper_type = OperType::AbsX;
            }
            Addressing::Aby => {
                operand.oper = file.r16(offset);
                operand.oper_type = OperType::AbsY;
            }
            Addressing::Zp => {
                operand.oper = u16::from(file.r8(offset));
                operand.oper_type = OperType::Abs;
            }
            Addressing::Zpx => {
                operand.oper = u16::from(file.r8(offset));
                operand.oper_type = OperType::AbsX;
            }
            Addressing::Zpy => {
                operand.oper = u16::from(file.r8(offset));
                operand.oper_type = OperType::AbsY;
            }
            Addressing::Ind => {
                operand.oper = file.r16(offset);
                operand.oper_type = OperType::Ind;
            }
            Addressing::Inx => {
                operand.oper = u16::from(file.r8(offset));
                operand.oper_type = OperType::IndX;
            }
            Addressing::Iny => {
                operand.oper = u16::from(file.r8(offset));
                operand.oper_type = OperType::IndY;
            }
            Addressing::Rel => {
                operand.oper = (addr as u16).wrapping_add(i16::from(file.r8(offset) as i8) as u16);
                operand.oper_type = OperType::Abs;
            }
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
        let operand = if let Some(operand) = inst.operands.as_any().downcast_ref::<Operand6502>() {
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
                out.comment = lookup(u32::from(operand.oper)).unwrap_or_default();
            }
            OperType::Abs => {
                out.operand = operand.to_hex();
                out.comment = lookup(u32::from(operand.oper)).unwrap_or_default();
            }
            OperType::AbsX => {
                out.operand = format!("{}, x", operand.to_hex());
                out.comment = lookup(u32::from(operand.oper)).unwrap_or_default();
            }
            OperType::AbsY => {
                out.operand = format!("{}, y", operand.to_hex());
                out.comment = lookup(u32::from(operand.oper)).unwrap_or_default();
            }
            OperType::Ind => {
                out.operand = format!("({})", operand.to_hex());
                out.comment = lookup(u32::from(operand.oper)).unwrap_or_default();
            }
            OperType::IndX => {
                out.operand = format!("({}, x)", operand.to_hex());
                out.comment = lookup(u32::from(operand.oper)).unwrap_or_default();
            }
            OperType::IndY => {
                out.operand = format!("({}), y", operand.to_hex());
                out.comment = lookup(u32::from(operand.oper)).unwrap_or_default();
            }
            _ => {}
        }
        out
    }

    fn target(&self, inst: &Instruction) -> Option<u32> {
        if let Some(opr) = inst.operands.as_any().downcast_ref::<Operand6502>()
            && (opr.oper_type == OperType::Imm || opr.oper_type == OperType::Abs)
        {
            Some(u32::from(opr.oper))
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
    };
}

const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
