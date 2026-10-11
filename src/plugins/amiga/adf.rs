/*

Amiga : ADF filesystem parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::Plugin,
    widgets::{Blank, Label, Link, TreeGrid, TreeRow, Widget},
};
use chrono::{DateTime, prelude::*};
use num_format::{Locale, ToFormattedString};
use std::{cmp, rc::Rc};

const T_HEADER: u32 = 2;
const T_DATA: u32 = 8;
const T_LIST: u32 = 16;
const ST_ROOT: u32 = 1;
const ST_USERDIR: i32 = 2;
const ST_FILE: i32 = -3;

pub struct Adf {}

impl Plugin for Adf {
    fn get_name(&self) -> &'static str {
        "Amiga/ADF"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.len() != 901_120 && handle.len() != 1_802_240 {
            return None;
        }
        if handle.r24be(0) != 0x44_4f53 {
            return None;
        }
        if handle.r32be(8) == 880 {
            Some(90)
        } else {
            Some(40) // might be ADF, but let's not default to it
        }
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Rc::new(Parser::new(handle));
        let root = parser.parse()?;
        let mut treegrid = TreeGrid::new(&["Filename", "Comment", "Length", "Date"]);
        treegrid.add(root.render(&parser), ctx);
        Ok(Box::new(treegrid))
    }
}

#[derive(Default, Clone)]
struct Entry {
    block: u32,
    is_file: bool,
    days: u32,
    mins: u32,
    name: String,
    comment: String,
    length: usize,
    children: Vec<Self>,
}

impl Entry {
    fn render(&self, parser: &Rc<Parser>) -> TreeRow {
        let mut row = TreeRow::default();
        if self.is_file {
            let entry = self.clone();
            let parser = parser.clone();
            row.push_column(Link::new(&self.name, "", move |ctx: &Context| {
                ctx.post(match parser.load_file(&entry) {
                    Ok(data) => Message::Open(Handle::new_from_array(&data, &entry.name, "")),
                    Err(err) => Message::Error(err.to_string()),
                });
            }));
        } else {
            row.push_column(Label::new(&self.name));
        }
        row.push_column(Label::new(&self.comment));
        if self.is_file {
            row.push_column(Label::new(&self.length.to_formatted_string(&Locale::en)));
        } else {
            row.push_column(Blank::default());
        }
        if let Some(dt) = DateTime::from_timestamp(
            i64::from(self.days) * 86400 + i64::from(self.mins) * 60 + 252_460_800,
            0,
        ) {
            row.push_column(Label::new(&format!(
                "{:4}-{:2}-{:2} {:2}:{:2}",
                dt.year(),
                dt.month(),
                dt.day(),
                dt.hour(),
                dt.minute(),
            )));
        } else {
            row.push_column(Blank::default());
        }
        for child in &self.children {
            row.push_child(child.render(parser));
        }
        row
    }
}

struct Parser {
    handle: Handle,
    ffs: bool,
}

impl Parser {
    fn new(handle: &Handle) -> Self {
        let ffs = handle.r8(3) & 1 == 1;
        Self {
            handle: handle.clone(),
            ffs,
        }
    }

    fn parse(&self) -> Result<Entry, &'static str> {
        let start = if self.handle.len() == 901_120 {
            880
        } else {
            176
        };
        self.do_root(start)
    }

    fn do_block(&self, block: u32) -> Result<Vec<Entry>, &'static str> {
        let ofs = block as usize * 512;
        if self.handle.r32be(ofs) != T_HEADER {
            return Err("missing block header");
        }
        match self.handle.r32be(ofs + 0x1fc).cast_signed() {
            ST_USERDIR => self.do_dir(block),
            ST_FILE => self.do_file(block),
            _ => Err("invalid sectype"),
        }
    }

    fn do_root(&self, block: u32) -> Result<Entry, &'static str> {
        let ofs = block as usize * 512;

        if self.handle.r32be(ofs) != T_HEADER || self.handle.r32be(ofs + 0x1fc) != ST_ROOT {
            return Err("invalid directory root");
        }

        let mut entry = Entry {
            block,
            is_file: false,
            days: self.handle.r32be(ofs + 0x1a4),
            mins: self.handle.r32be(ofs + 0x1a8),
            name: self.handle.rps(ofs + 0x1b0, ""),
            ..Default::default()
        };
        let num_ht = self.handle.r32be(ofs + 0xc) as usize;
        for i in 0..num_ht {
            let child = self.handle.r32be(ofs + 24 + i * 4);
            if child != 0 {
                entry.children.extend(self.do_block(child)?);
            }
        }
        Ok(entry)
    }

    fn do_dir(&self, block: u32) -> Result<Vec<Entry>, &'static str> {
        let ofs = block as usize * 512;

        let mut entry = Entry {
            block,
            is_file: false,
            comment: self.handle.rps(ofs + 0x148, ""),
            days: self.handle.r32be(ofs + 0x1a4),
            mins: self.handle.r32be(ofs + 0x1a8),
            name: self.handle.rps(ofs + 0x1b0, ""),
            ..Default::default()
        };
        for i in 0..72 {
            let child = self.handle.r32be(ofs + 24 + i * 4);
            if child != 0 {
                entry.children.extend(self.do_block(child)?);
            }
        }
        let next = self.handle.r32be(ofs + 0x1f0);
        let mut entries = vec![entry];
        if next != 0 {
            entries.extend(self.do_block(next)?);
        }
        Ok(entries)
    }

    fn do_file(&self, block: u32) -> Result<Vec<Entry>, &'static str> {
        let ofs = block as usize * 512;

        let entry = Entry {
            block,
            is_file: true,
            length: self.handle.r32be(ofs + 0x144) as usize,
            comment: self.handle.rps(ofs + 0x148, ""),
            days: self.handle.r32be(ofs + 0x1a4),
            mins: self.handle.r32be(ofs + 0x1a8),
            name: self.handle.rps(ofs + 0x1b0, ""),
            ..Default::default()
        };
        let next = self.handle.r32be(ofs + 0x1f0);
        let mut entries = vec![entry];
        if next != 0 {
            entries.extend(self.do_block(next)?);
        }
        Ok(entries)
    }

    fn load_file(&self, entry: &Entry) -> Result<Vec<u8>, &'static str> {
        let mut block = entry.block as usize;
        let mut first = true;
        let mut data = vec![];
        while data.len() < entry.length {
            let blocktype = self.handle.r32be(block * 512);
            if first && blocktype != T_HEADER {
                return Err("invalid data header");
            }
            if !first && blocktype != T_LIST {
                return Err("invalid data list");
            }
            first = false;
            let high_seq = self.handle.r32be(block * 512 + 8) as usize;
            for i in 0..high_seq {
                let ht = self.handle.r32be(block * 512 + 0x18 + (71 - i) * 4);
                data.extend_from_slice(self.data_block(ht as usize, entry.length - data.len())?);
            }
            block = self.handle.r32be(block * 512 + 0x1f8) as usize;
        }
        Ok(data)
    }

    fn data_block(&self, block: usize, len: usize) -> Result<&[u8], &'static str> {
        let ofs = block * 512;
        if self.ffs {
            let len = cmp::min(len, 512);
            if len + ofs > self.handle.len() {
                Err("invalid data block")
            } else {
                Ok(self.handle.read(ofs, len))
            }
        } else if self.handle.r32be(ofs) != T_DATA {
            Err("invalid data block")
        } else {
            let len = self.handle.r32be(ofs + 0xc) as usize;
            if len + ofs + 0x18 > self.handle.len() {
                Err("invalid data block")
            } else {
                Ok(self.handle.read(ofs + 0x18, len))
            }
        }
    }
}
