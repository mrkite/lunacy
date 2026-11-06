/*

Platforms for disassembly
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{Address, CpuLine, CpuState, Instruction},
    lunacy::Handle,
};

mod api;
mod block;
mod definition;
mod dos;
mod fingerprints;
mod iie;
mod iigs;
mod labelwin;
mod mac;
mod mzheader;
mod symbols;

pub use block::BlockCache;
pub use dos::Dos;
pub use iie::IIe;
pub use iigs::IIgs;
pub use mac::Mac;

pub trait Platform<ADDR: Address> {
    fn address(&self, address: ADDR) -> String;
    fn decode(
        &self,
        handle: &Handle,
        offset: usize,
        state: &mut dyn CpuState<ADDR>,
    ) -> Result<Instruction, &'static str>;
    fn target(&self, inst: &Instruction) -> Option<ADDR>;
    fn disassemble(&self, address: ADDR, inst: &Instruction) -> CpuLine<ADDR>;
}
