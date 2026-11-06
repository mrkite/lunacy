/*

Represents a continuous block of code with only 1 entrance and exit
@copyright 2026 Sean Kasun

*/

use crate::cpus::Address;
use std::{cell::RefCell, rc::Rc};

#[derive(Ord, PartialOrd, PartialEq, Eq)]
pub struct Block<ADDR: Address> {
    pub address: ADDR,
    pub length: usize,
    pub branch_len: usize,
    pub preds: Vec<BlockPtr<ADDR>>,
    pub succs: Vec<BlockPtr<ADDR>>,
}

type BlockPtr<ADDR> = Rc<RefCell<Block<ADDR>>>;

#[derive(Default)]
pub struct BlockCache<ADDR: Address> {
    cache: Vec<BlockPtr<ADDR>>,
}

impl<ADDR: Address> BlockCache<ADDR> {
    pub fn get(&mut self, addr: ADDR) -> BlockPtr<ADDR> {
        for b in &self.cache {
            if b.borrow().address == addr {
                return b.clone();
            }
        }
        let b = Rc::new(RefCell::new(Block {
            address: addr,
            length: 0,
            branch_len: 0,
            preds: vec![],
            succs: vec![],
        }));
        self.cache.push(b.clone());
        b
    }

    pub fn sort(&mut self) {
        self.cache.sort();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, BlockPtr<ADDR>> {
        self.cache.iter()
    }
}
