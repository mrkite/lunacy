/*

Amiga : ADF filesystem parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};
use chrono::{DateTime, prelude::*};
use num_format::{Locale, ToFormattedString};
use std::cmp;

const T_HEADER: u32 = 2;
const T_DATA: u32 = 8;
const T_LIST: u32 = 16;
const ST_ROOT: u32 = 1;
const ST_USERDIR: i32 = 2;
const ST_FILE: i32 = -3;

pub struct Adf {}

impl Plugin for Adf {
    fn get_name(&self) -> &str {
        "Amiga/ADF"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() != 901120 && file.len() != 1802240 {
            return None;
        }
        if file.r24be(0) != 0x444f53 {
            return None;
        }
        if file.r32be(8) == 880 {
            Some(90)
        } else {
            Some(40) // might be ADF, but let's not default to it
        }
    }

    fn create(
        &self,
        file: &File,
        ctx: Context,
    ) -> Result<Box<dyn crate::plugins::Widget>, &'static str> {
        let parser = Parser::new(file);
        let root = parser.parse()?;
        let mut treegrid = TreeGrid::new(
            &["Filename", "Comment", "Length", "Date"],
            &format!("adf_{}", file.get_name()),
            move |entry: &Entry| {
                if entry.is_file {
                    ctx.post(match parser.load_file(entry) {
                        Ok(data) => Message::Open(File::new_from_array(&data, &entry.name, "")),
                        Err(err) => Message::Error(err.to_string()),
                    });
                }
            },
        );
        treegrid.add(root.render());
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
    children: Vec<Entry>,
}

impl Entry {
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.name);
        row.push_column(&self.comment);
        if self.is_file {
            row.push_column(&self.length.to_formatted_string(&Locale::en));
        } else {
            row.push_column("");
        }
        if let Some(dt) = DateTime::from_timestamp(
            i64::from(self.days) * 86400 + i64::from(self.mins) * 60 + 252460800,
            0,
        ) {
            row.push_column(&format!(
                "{:4}-{:2}-{:2} {:2}:{:2}",
                dt.year(),
                dt.month(),
                dt.day(),
                dt.hour(),
                dt.minute(),
            ));
        } else {
            row.push_column("");
        }
        for child in self.children.iter() {
            row.push_child(child.render());
        }
        row
    }
}

struct Parser {
    file: File,
    ffs: bool,
}

impl Parser {
    fn new(file: &File) -> Self {
        let ffs = file.r8(3) & 1 == 1;
        Self {
            file: file.clone(),
            ffs,
        }
    }

    fn parse(&self) -> Result<Entry, &'static str> {
        let start = if self.file.len() == 901120 { 880 } else { 176 };
        self.do_root(start)
    }

    fn do_block(&self, block: u32) -> Result<Vec<Entry>, &'static str> {
        let ofs = block as usize * 512;
        if self.file.r32be(ofs) != T_HEADER {
            return Err("missing block header");
        }
        match self.file.r32be(ofs + 0x1fc) as i32 {
            ST_USERDIR => self.do_dir(block),
            ST_FILE => self.do_file(block),
            _ => Err("invalid sectype"),
        }
    }

    fn do_root(&self, block: u32) -> Result<Entry, &'static str> {
        let ofs = block as usize * 512;

        if self.file.r32be(ofs) != T_HEADER || self.file.r32be(ofs + 0x1fc) != ST_ROOT {
            return Err("invalid directory root");
        }

        let mut entry = Entry {
            block,
            is_file: false,
            days: self.file.r32be(ofs + 0x1a4),
            mins: self.file.r32be(ofs + 0x1a8),
            name: self.file.rps(ofs + 0x1b0, ""),
            ..Default::default()
        };
        let num_ht = self.file.r32be(ofs + 0xc) as usize;
        for i in 0..num_ht {
            let child = self.file.r32be(ofs + 24 + i * 4);
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
            comment: self.file.rps(ofs + 0x148, ""),
            days: self.file.r32be(ofs + 0x1a4),
            mins: self.file.r32be(ofs + 0x1a8),
            name: self.file.rps(ofs + 0x1b0, ""),
            ..Default::default()
        };
        for i in 0..72 {
            let child = self.file.r32be(ofs + 24 + i * 4);
            if child != 0 {
                entry.children.extend(self.do_block(child)?);
            }
        }
        let next = self.file.r32be(ofs + 0x1f0);
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
            length: self.file.r32be(ofs + 0x144) as usize,
            comment: self.file.rps(ofs + 0x148, ""),
            days: self.file.r32be(ofs + 0x1a4),
            mins: self.file.r32be(ofs + 0x1a8),
            name: self.file.rps(ofs + 0x1b0, ""),
            ..Default::default()
        };
        let next = self.file.r32be(ofs + 0x1f0);
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
            let blocktype = self.file.r32be(block * 512);
            if first && blocktype != T_HEADER {
                return Err("invalid data header");
            }
            if !first && blocktype != T_LIST {
                return Err("invalid data list");
            }
            first = false;
            let high_seq = self.file.r32be(block * 512 + 8) as usize;
            for i in 0..high_seq {
                let ht = self.file.r32be(block * 512 + 0x18 + (71 - i) * 4);
                data.extend_from_slice(self.data_block(ht as usize, entry.length - data.len())?);
            }
            block = self.file.r32be(block * 512 + 0x1f8) as usize;
        }
        Ok(data)
    }

    fn data_block(&self, block: usize, len: usize) -> Result<&[u8], &'static str> {
        let ofs = block * 512;
        if self.ffs {
            let len = cmp::min(len, 512);
            if len + ofs > self.file.len() {
                Err("invalid data block")
            } else {
                Ok(self.file.read(ofs, len))
            }
        } else if self.file.r32be(ofs) != T_DATA {
            Err("invalid data block")
        } else {
            let len = self.file.r32be(ofs + 0xc) as usize;
            if len + ofs + 0x18 > self.file.len() {
                Err("invalid data block")
            } else {
                Ok(self.file.read(ofs + 0x18, len))
            }
        }
    }
}
