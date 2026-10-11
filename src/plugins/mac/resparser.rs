/*

Mac: Resource fork parser
@copyright 2026 Sean Kasun

*/

use crate::lunacy::Handle;
use std::collections::HashMap;

pub struct Parser {
    handle: Handle,
}

impl Parser {
    pub fn new(handle: &Handle) -> Self {
        Self {
            handle: handle.clone(),
        }
    }

    pub fn parse(&self) -> HashMap<String, Vec<Entry>> {
        let dataofs = self.handle.r32be(0) as usize;
        let mapofs = self.handle.r32be(4) as usize;
        let typelist = mapofs + usize::from(self.handle.r16be(mapofs + 24));
        let namelist = mapofs + usize::from(self.handle.r16be(mapofs + 26));
        let num_types = self.handle.r16be(mapofs + 28);
        let mut resources: HashMap<String, Vec<Entry>> = HashMap::new();

        for t in 0..=num_types {
            let ofs = typelist + 2 + usize::from(t) * 8;
            let res_type = self.handle.rs(ofs, 4, "macroman");
            let num_resources = self.handle.r16be(ofs + 4);
            let listofs = usize::from(self.handle.r16be(ofs + 6));
            let mut res_list = vec![];
            for r in 0..=usize::from(num_resources) {
                let ofs = typelist + listofs + r * 12;
                let nameofs = usize::from(self.handle.r16be(ofs + 2));
                let offset = self.handle.r24be(ofs + 5) as usize + dataofs;
                let name = if nameofs == 0xffff {
                    String::new()
                } else {
                    self.handle.rps(namelist + nameofs, "macroman")
                };
                res_list.push(Entry {
                    res_type: res_type.clone(),
                    id: self.handle.r16be(ofs),
                    attr: self.handle.r8(ofs + 4),
                    name,
                    offset,
                    length: self.handle.r32be(offset),
                });
            }
            res_list.sort();
            resources.insert(res_type.clone(), res_list);
        }
        resources
    }

    pub fn get_res(&self, name: &str, res_type: &str, offset: usize, attr: u8) -> Handle {
        let len = self.handle.r32be(offset) as usize;
        if (attr & 1) != 0 && self.handle.r32be(offset + 4) == 0xa89f_6572 {
            // extended resource
            Handle::new_from_array(
                self.handle.read(offset + 4, len),
                name,
                &format!("mac/res/extended/{res_type}"),
            )
        } else {
            Handle::new_from_array(
                self.handle.read(offset + 4, len),
                name,
                &format!("mac/res/{res_type}"),
            )
        }
    }
}

#[derive(Default, Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct Entry {
    pub res_type: String,
    pub id: u16,
    pub attr: u8,
    pub name: String,
    pub offset: usize,
    pub length: u32,
}
