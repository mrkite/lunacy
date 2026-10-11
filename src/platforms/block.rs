/*

Represents a continuous block of code with only 1 entrance and exit
@copyright 2026 Sean Kasun

*/

use std::{cell::RefCell, rc::Rc};

#[derive(Ord, PartialOrd, PartialEq, Eq)]
pub struct Block {
    pub address: u32,
    pub length: u32,
    pub branch_len: u32,
    pub preds: Vec<BlockPtr>,
    pub succs: Vec<BlockPtr>,
}

type BlockPtr = Rc<RefCell<Block>>;

#[derive(Default)]
pub struct BlockCache {
    cache: Vec<BlockPtr>,
}

impl BlockCache {
    pub fn get(&mut self, addr: u32) -> BlockPtr {
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

    pub fn iter(&self) -> std::slice::Iter<'_, BlockPtr> {
        self.cache.iter()
    }
}
