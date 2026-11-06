/*

CPU plugins
@copyright 2026 Sean Kasun

*/

mod cpu6502;
mod cpu65816;
mod cpu68k;
mod cpu86;

use crate::lunacy::Handle;
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
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Sub;

pub trait Address:
    Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + Eq
    + PartialEq
    + Ord
    + PartialOrd
    + Default
    + Copy
{
    fn as_usize(&self) -> usize;
    fn from_usize(value: usize) -> Self;
}

impl Address for u32 {
    fn as_usize(&self) -> usize {
        usize::try_from(*self).unwrap_or(usize::MAX)
    }
    fn from_usize(value: usize) -> Self {
        Self::try_from(value).unwrap_or(Self::MAX)
    }
}

impl Address for u16 {
    fn as_usize(&self) -> usize {
        usize::from(*self)
    }
    fn from_usize(value: usize) -> Self {
        Self::try_from(value).unwrap_or(Self::MAX)
    }
}

pub trait CpuState<ADDR: Address> {
    fn jump(&mut self, addr: ADDR);
    fn address(&self) -> ADDR;
    fn as_any(&mut self) -> &mut dyn Any;
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
    pub length: usize,
    pub raw: Vec<u8>,
    pub operands: Box<dyn Operand>,
}

#[derive(Default, Clone)]
pub struct CpuLine<ADDR: Address> {
    pub address: ADDR,
    pub hex: String,
    pub opcode: String,
    pub operand: String,
    pub comment: String,
    pub target: Option<ADDR>,
}

pub trait Cpu<ADDR: Address> {
    fn decode(
        &self,
        handle: &Handle,
        offset: usize,
        state: &mut dyn CpuState<ADDR>,
    ) -> Result<Instruction, &'static str>;
    fn target(&self, inst: &Instruction) -> Option<ADDR>;
    fn disassemble(
        &self,
        address: ADDR,
        inst: &Instruction,
        lookup: &dyn Fn(ADDR) -> Option<String>,
    ) -> CpuLine<ADDR>;
}
