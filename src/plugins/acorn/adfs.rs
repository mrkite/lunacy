/*

Acorn/BBC Micro : ADFS-S,ADFS-M, and ADFS-L filesystem parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::{Plugin, get_charset},
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};

pub struct Adfs {}

impl Plugin for Adfs {
    fn get_name(&self) -> &str {
        "Acorn/ADFS"
    }

    fn supports(&self, file: &crate::lunacy::File) -> Option<i16> {
        if file.len() < 6500 {
            return None;
        }
        // multiple of sector size
        if file.len() & 0xff != 0 {
            return None;
        }

        // 800k files have "Nick" at 2047 which doesn't make any sense
        // I'll need to spend some time figuring it out, until then we'll only support ADFS-S to ADFS-L
        if file.r4(513) == "Hugo" {
            Some(70)
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
        let mut treegrid = TreeGrid::new(&["Filename", "Length"], "adfs", move |entry: &Entry| {
            if entry.is_file {
                ctx.post(match parser.load_file(entry) {
                    Ok(data) => {
                        Message::Open(File::new_from_array(&data, &entry.name, &entry.mime))
                    }
                    Err(err) => Message::Error(err.to_string()),
                })
            }
        });
        for entry in entries.iter() {
            treegrid.add(entry.render());
        }
        Ok(Box::new(treegrid))
    }
}

#[derive(Default, Clone)]
struct Entry {
    name: String,
    mime: String,
    sector: u32,
    length: u32,
    is_file: bool,
    children: Vec<Entry>,
}

impl Entry {
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        if self.is_file {
            row.push_column(&self.name);
            row.push_column(&self.length.to_formatted_string(&Locale::en));
        } else {
            row.push_column(&self.name);
            for child in self.children.iter() {
                row.push_child(child.render());
            }
        }
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

    fn parse(&self) -> Result<Vec<Entry>, &'static str> {
        self.dir(2)
    }

    fn dir(&self, dir_sector: usize) -> Result<Vec<Entry>, &'static str> {
        let cs = get_charset("");
        let mut entries = vec![];
        let start = dir_sector * 256 + 5;
        for i in 0..47 {
            let ofs = start + i * 0x1a;
            let mut name = String::new();
            for j in 0..9 {
                let ch = self.file.r8(ofs + j) & 0x7f;
                if ch == 0 || ch == 0xd {
                    break;
                }
                name.push(cs[usize::from(ch)]);
            }
            let load = self.file.r32(ofs + 0xa);
            let exec = self.file.r32(ofs + 0xe);
            let length = self.file.r32(ofs + 0x12);
            let sector = self.file.r24(ofs + 0x16);
            if !name.is_empty() {
                let mut entry = Entry {
                    name,
                    mime: format!("acorn/{:x}/{:x}", load, exec),
                    sector,
                    length,
                    is_file: true,
                    children: vec![],
                };
                if self.file.r8(ofs + 3) & 0x80 != 0 {
                    // D
                    entry.is_file = false;
                    entry.children = self.dir(entry.sector as usize)?;
                }
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    fn load_file(&self, entry: &Entry) -> Result<Vec<u8>, &'static str> {
        Ok(self
            .file
            .read(entry.sector as usize * 256, entry.length as usize)
            .to_vec())
    }
}
