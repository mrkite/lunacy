/*

Mac: Resource fork parser
@copyright 2026 Sean Kasun

*/

use std::collections::HashMap;

use crate::lunacy::File;

pub struct Parser {
    file: File,
}

impl Parser {
    pub fn new(file: &File) -> Self {
        Self { file: file.clone() }
    }

    pub fn parse(&self) -> Result<HashMap<String, Vec<Entry>>, &'static str> {
        let dataofs = self.file.r32be(0) as usize;
        let mapofs = self.file.r32be(4) as usize;
        let typelist = mapofs + usize::from(self.file.r16be(mapofs + 24));
        let namelist = mapofs + usize::from(self.file.r16be(mapofs + 26));
        let num_types = self.file.r16be(mapofs + 28);
        let mut resources: HashMap<String, Vec<Entry>> = HashMap::new();

        for t in 0..=num_types {
            let ofs = typelist + 2 + usize::from(t) * 8;
            let res_type = self.file.rs(ofs, 4, "macroman");
            let num_resources = self.file.r16be(ofs + 4);
            let listofs = usize::from(self.file.r16be(ofs + 6));
            let mut res_list = vec![];
            for r in 0..=usize::from(num_resources) {
                let ofs = typelist + listofs + r * 12;
                let nameofs = usize::from(self.file.r16be(ofs + 2));
                let offset = self.file.r24be(ofs + 5) as usize + dataofs;
                let name = if nameofs != 0xffff {
                    self.file.rps(namelist + nameofs, "macroman")
                } else {
                    String::new()
                };
                res_list.push(Entry {
                    res_type: res_type.to_owned(),
                    id: self.file.r16be(ofs),
                    attr: self.file.r8(ofs + 4),
                    name,
                    offset,
                    length: self.file.r32be(offset),
                });
            }
            res_list.sort();
            resources.insert(res_type.to_owned(), res_list);
        }
        Ok(resources)
    }

    pub fn get_res(
        &self,
        name: &str,
        res_type: &str,
        offset: usize,
        attr: u8,
    ) -> Result<File, &'static str> {
        let len = self.file.r32be(offset) as usize;
        if (attr & 1) != 0 && self.file.r32be(offset + 4) == 0xa89f6572 {
            // extended resource
            Ok(File::new_from_array(
                self.file.read(offset + 4, len),
                name,
                &format!("mac/res/extended/{}", res_type),
            ))
        } else {
            Ok(File::new_from_array(
                self.file.read(offset + 4, len),
                name,
                &format!("mac/res/{}", res_type),
            ))
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
