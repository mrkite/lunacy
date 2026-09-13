/*

CPU plugins
@copyright 2026 Sean Kasun

*/

mod cpu6502;
mod cpu65816;
mod cpu68k;
mod cpu86;

use crate::lunacy::File;
pub use cpu68k::Cpu68k;
pub use cpu68k::Operands68k;
pub use cpu68k::State68k;
pub use cpu86::Cpu86;
pub use cpu86::State86;
pub use cpu6502::Cpu6502;
pub use cpu6502::State6502;
pub use cpu65816::Cpu65816;
pub use cpu65816::Operand65816;
pub use cpu65816::State65816;
use std::any::Any;

pub trait CpuState {
    fn jump(&mut self, addr: u32);
    fn address(&self) -> u32;
    fn as_any(&mut self) -> &mut dyn Any;
}

#[derive(Clone, Copy, PartialEq)]
pub enum InstType {
    Normal,
    Call,
    Jump,
    Return,
    Branch,
    Special,
    Prefix,
    Invalid,
}

pub trait Operand {
    fn as_any(&self) -> &dyn Any;
}

pub struct Instruction {
    pub name: String,
    pub inst_type: InstType,
    pub length: u32,
    pub raw: Vec<u8>,
    pub operands: Box<dyn Operand>,
}

#[derive(Default, Clone)]
pub struct CpuLine {
    pub address: u32,
    pub hex: String,
    pub opcode: String,
    pub operand: String,
    pub comment: String,
}

pub trait Cpu {
    fn decode(
        &self,
        file: &File,
        offset: usize,
        state: &mut dyn CpuState,
    ) -> Result<Instruction, &'static str>;
    fn target(&self, inst: &Instruction) -> Option<u32>;
    fn disassemble(
        &self,
        address: u32,
        inst: &Instruction,
        lookup: &dyn Fn(u32) -> Option<String>,
    ) -> CpuLine;
}
