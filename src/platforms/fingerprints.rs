/*

Custom API calls

*/

use std::collections::{HashMap, hash_map::Entry};

#[derive(Default)]
pub struct Fingerprint {
    next: HashMap<u8, Self>,
    wild: HashMap<bool, Self>,
    pub name: Option<String>,
}

impl Fingerprint {
    pub fn next(&self, byte: u8) -> Option<&Self> {
        if self.is_wild() {
            self.wild.get(&true)
        } else {
            self.next.get(&byte)
        }
    }

    pub fn next_entry(&mut self, byte: u8) -> Entry<'_, u8, Self> {
        self.next.entry(byte)
    }

    pub fn next_wild(&mut self) -> Entry<'_, bool, Self> {
        self.wild.entry(true)
    }

    pub fn is_wild(&self) -> bool {
        !self.wild.is_empty()
    }
}
