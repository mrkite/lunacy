/*

Platforms for disassembly
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{CpuLine, CpuState, Instruction},
    lunacy::File,
};

mod api;
mod block;
mod dos;
mod fingerprints;
mod get_addr;
mod iie;
mod iigs;
mod mac;

pub use block::BlockCache;
pub use dos::Dos;
pub use get_addr::GetAddress;
pub use iie::IIe;
pub use iigs::IIgs;
pub use mac::Mac;

pub trait Platform {
    fn address(&self, address: &u32) -> String;
    fn decode(
        &self,
        file: &File,
        offset: usize,
        state: &mut dyn CpuState,
    ) -> Result<Instruction, &'static str>;
    fn target(&self, inst: &Instruction) -> Option<u32>;
    fn disassemble(&self, address: u32, inst: &Instruction) -> CpuLine;
}
