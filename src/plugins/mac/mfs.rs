/*

68k Mac : MFS file system parser
@copyright 2026 Sean Kasun

*/

use std::rc::Rc;

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::Plugin,
    widgets::{Blank, Label, Link, TreeGrid, TreeRow, Widget},
};
use num_format::{Locale, ToFormattedString};

pub struct Mfs {}

impl Plugin for Mfs {
    fn get_name(&self) -> &'static str {
        "Mac/MFS"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.len() < 0x500 {
            return None;
        }
        if handle.r16be(0x400) == 0xd2d7 {
            Some(90)
        } else {
            None
        }
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Rc::new(Parser::new(handle));
        let entries = parser.parse();
        let mut treegrid = TreeGrid::new(&["Filename", "Type", "Creator", "Data", "Res"]);
        for entry in &entries {
            treegrid.add(entry.render(&parser), ctx);
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
    fn render(&self, parser: &Rc<Parser>) -> TreeRow {
        let mut row = TreeRow::default();
        if self.data_len > 0 || self.res_len > 0 {
            let entry = self.clone();
            let parser = parser.clone();
            row.push_column(Link::new(&self.name, "", move |ctx: &Context| {
                if entry.data_len > 0 {
                    let data = parser.load_file(entry.data_start, entry.data_len);
                    ctx.post(Message::Open(Handle::new_from_array(
                        &data,
                        &entry.name,
                        "",
                    )));
                } else if entry.res_len > 0 {
                    let data = parser.load_file(entry.res_start, entry.res_len);
                    ctx.post(Message::Open(Handle::new_from_array(
                        &data,
                        &format!("{} [res]", entry.name),
                        "mac/res",
                    )));
                }
            }));
        } else {
            row.push_column(Label::new(&self.name));
        }
        row.push_column(Label::new(&self.filetype));
        row.push_column(Label::new(&self.creator));
        if self.data_len > 0 {
            row.push_column(Label::new(&self.data_len.to_formatted_string(&Locale::en)));
        } else {
            row.push_column(Blank::default());
        }
        if self.res_len > 0 {
            if self.data_len > 0 {
                let entry = self.clone();
                let parser = parser.clone();
                row.push_column(Link::new(
                    &self.res_len.to_formatted_string(&Locale::en),
                    "",
                    move |ctx: &Context| {
                        let data = parser.load_file(entry.res_start, entry.res_len);
                        ctx.post(Message::Open(Handle::new_from_array(
                            &data,
                            &format!("{} [res]", entry.name),
                            "mac/res",
                        )));
                    },
                ));
            } else {
                row.push_column(Label::new(&self.res_len.to_formatted_string(&Locale::en)));
            }
        } else {
            row.push_column(Blank::default());
        }
        row
    }
}

struct Parser {
    handle: Handle,
    file_count: usize,
    dir_start: usize,
    allocation_block_size: usize,
    allocation_block_start: usize,
}

impl Parser {
    fn new(handle: &Handle) -> Self {
        let file_count = handle.r16be(0x40c) as usize;
        let dir_start = handle.r16be(0x40e) as usize * 512;
        let allocation_block_size = handle.r32be(0x414) as usize;
        let allocation_block_start = handle.r16be(0x41c) as usize * 512;
        Self {
            handle: handle.clone(),
            file_count,
            dir_start,
            allocation_block_size,
            allocation_block_start,
        }
    }

    fn parse(&self) -> Vec<Entry> {
        let mut base = self.dir_start;
        let mut entries = vec![];
        for _ in 0..self.file_count {
            let namelen = self.handle.r8(base + 0x32) as usize;
            entries.push(Entry {
                filetype: self.handle.rs(base + 2, 4, "macroman"),
                creator: self.handle.rs(base + 6, 4, "macroman"),
                data_start: self.handle.r16be(base + 0x16) as usize,
                data_len: self.handle.r32be(base + 0x18) as usize,
                res_start: self.handle.r16be(base + 0x20) as usize,
                res_len: self.handle.r32be(base + 0x22) as usize,
                name: self.handle.rs(base + 0x33, namelen, "macroman"),
            });
            let entrylen = namelen + 0x33;
            base += entrylen + (entrylen & 1);
        }
        entries
    }

    fn load_file(&self, start: usize, len: usize) -> Vec<u8> {
        let mut data = vec![];
        let ofs = self.allocation_block_start + (start - 2) * self.allocation_block_size;
        data.extend_from_slice(self.handle.read(ofs, len));
        data
    }
}
