/*

68k Mac : Resource fork parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};
use std::collections::HashMap;

pub struct Resources {}

impl Plugin for Resources {
    fn get_name(&self) -> &str {
        "Mac/Resources"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.get_mime() == "mac/res" {
            Some(90)
        } else {
            None
        }
    }

    fn create(
        &self,
        file: &File,
        ctx: Context,
    ) -> Result<Box<dyn crate::plugins::Widget>, &'static str> {
        let parser = Parser::new(file);
        let entries = parser.parse()?;
        let mut treegrid = TreeGrid::new(
            &["ID", "Length", "Name"],
            &format!("res_{}", file.get_name()),
            move |entry: &Entry| {
                if entry.is_valid {
                    let separator = if entry.name.is_empty() { "" } else { " " };
                    let name = format!(
                        "{} ${:02x}{}{}",
                        entry.res_type, entry.id, separator, entry.name
                    );
                    ctx.post(
                        match parser.get_res(&name, &entry.res_type, entry.offset, entry.attr) {
                            Ok(file) => Message::Open(file),
                            Err(err) => Message::Error(err.to_string()),
                        },
                    );
                }
            },
        );
        let mut keys = Vec::from_iter(entries.keys());
        keys.sort();
        for key in keys {
            let mut row = TreeRow::new(Entry::default());
            row.push_column(key);
            row.push_column("");
            row.push_column("");
            for child in entries[key].iter() {
                row.push_child(child.render());
            }
            treegrid.add(row);
        }
        Ok(Box::new(treegrid))
    }
}

#[derive(Default, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Entry {
    res_type: String,
    id: u16,
    attr: u8,
    name: String,
    offset: usize,
    length: u32,
    is_valid: bool,
}

impl Entry {
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.id.to_formatted_string(&Locale::en));
        row.push_column(&self.length.to_formatted_string(&Locale::en));
        row.push_column(&self.name);
        row
    }
}

struct Parser {
    file: File,
}

impl Parser {
    fn new(file: &File) -> Self {
        Self { file: file.clone() }
    }

    fn parse(&self) -> Result<HashMap<String, Vec<Entry>>, &'static str> {
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
                    is_valid: true,
                });
            }
            res_list.sort();
            resources.insert(res_type.to_owned(), res_list);
        }
        Ok(resources)
    }

    fn get_res(
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
