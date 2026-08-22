/*

68k Mac : MFS file system parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};

pub struct Mfs {}

impl Plugin for Mfs {
    fn get_name(&self) -> &str {
        "Mac/MFS"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() < 0x500 {
            return None;
        }
        if file.r16be(0x400) != 0xd2d7 {
            None
        } else {
            Some(90)
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
            &["Filename", "Type", "Creator", "Data", "Res"],
            &format!("mfs_{}", file.get_name()),
            move |entry: &Entry| {
                if entry.res_len > 0 {
                    ctx.post(match parser.load_file(entry.res_start, entry.res_len) {
                        Ok(data) => Message::Open(File::new_from_array(
                            &data,
                            &format!("{} [res]", entry.name),
                            "mac/res",
                        )),
                        Err(err) => Message::Error(err.to_string()),
                    });
                }
                if entry.data_len > 0 {
                    ctx.post(match parser.load_file(entry.data_start, entry.data_len) {
                        Ok(data) => Message::Open(File::new_from_array(&data, &entry.name, "")),
                        Err(err) => Message::Error(err.to_string()),
                    });
                }
            },
        );
        for entry in entries.iter() {
            treegrid.add(entry.render());
        }
        Ok(Box::new(treegrid))
    }
}

#[derive(Default, Clone)]
struct Entry {
    name: String,
    filetype: String,
    creator: String,
    data_len: usize,
    res_len: usize,
    data_start: usize,
    res_start: usize,
}

impl Entry {
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.name);
        row.push_column(&self.filetype);
        row.push_column(&self.creator);
        if self.data_len > 0 {
            row.push_column(&self.data_len.to_formatted_string(&Locale::en));
        } else {
            row.push_column("");
        }
        if self.res_len > 0 {
            row.push_column(&self.res_len.to_formatted_string(&Locale::en));
        } else {
            row.push_column("");
        }
        row
    }
}

struct Parser {
    file: File,
    file_count: usize,
    dir_start: usize,
    allocation_block_size: usize,
    allocation_block_start: usize,
}

impl Parser {
    fn new(file: &File) -> Self {
        let file_count = file.r16be(0x40c) as usize;
        let dir_start = file.r16be(0x40e) as usize * 512;
        let allocation_block_size = file.r32be(0x414) as usize;
        let allocation_block_start = file.r16be(0x41c) as usize * 512;
        Self {
            file: file.clone(),
            file_count,
            dir_start,
            allocation_block_size,
            allocation_block_start,
        }
    }

    fn parse(&self) -> Result<Vec<Entry>, &'static str> {
        let mut base = self.dir_start;
        let mut entries = vec![];
        for _ in 0..self.file_count {
            let namelen = self.file.r8(base + 0x32) as usize;
            entries.push(Entry {
                filetype: self.file.rs(base + 2, 4, "macroman"),
                creator: self.file.rs(base + 6, 4, "macroman"),
                data_start: self.file.r16be(base + 0x16) as usize,
                data_len: self.file.r32be(base + 0x18) as usize,
                res_start: self.file.r16be(base + 0x20) as usize,
                res_len: self.file.r32be(base + 0x22) as usize,
                name: self.file.rs(base + 0x33, namelen, "macroman"),
            });
            let entrylen = namelen + 0x33;
            base += entrylen + (entrylen & 1);
        }
        Ok(entries)
    }

    fn load_file(&self, start: usize, len: usize) -> Result<Vec<u8>, &'static str> {
        let mut data = vec![];
        let ofs = self.allocation_block_start + (start - 2) * self.allocation_block_size;
        data.extend_from_slice(self.file.read(ofs, len));
        Ok(data)
    }
}
