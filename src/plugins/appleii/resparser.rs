/*

Apple IIgs : Resource fork parser
@copyright 2026 Sean Kasun

*/

use std::collections::HashMap;

use crate::lunacy::Handle;

pub struct Parser {
    handle: Handle,
}

impl Parser {
    pub fn new(handle: &Handle) -> Self {
        Self {
            handle: handle.clone(),
        }
    }

    pub fn parse(&self) -> HashMap<u16, Vec<Entry>> {
        let mut resources: HashMap<u16, Vec<Entry>> = HashMap::new();
        let file_to_map = usize::try_from(self.handle.r32(4)).unwrap_or_default();
        let map_to_index = file_to_map + usize::from(self.handle.r16(file_to_map + 0xe));
        let map_index_used =
            usize::try_from(self.handle.r32(file_to_map + 0x18)).unwrap_or_default();

        for t in 0..map_index_used {
            let ofs = map_to_index + t * 0x14;
            let entry = Entry {
                res_type: self.handle.r16(ofs),
                id: self.handle.r32(ofs + 2),
                offset: usize::try_from(self.handle.r32(ofs + 6)).unwrap_or_default(),
                length: self.handle.r32(ofs + 0xc),
            };
            let list = resources.entry(entry.res_type).or_default();
            list.push(entry);
        }
        resources
    }

    pub fn get_res(&self, res_type: u16, name: &str, offset: usize, length: usize) -> Handle {
        Handle::new_from_array(
            self.handle.read(offset, length),
            name,
            &format!("prodos/res/{res_type}"),
        )
    }
}

#[derive(Default, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct Entry {
    pub res_type: u16,
    pub id: u32,
    pub offset: usize,
    pub length: u32,
}
