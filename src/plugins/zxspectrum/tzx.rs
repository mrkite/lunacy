/*

ZX-series : TZX tape format parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::Plugin,
    widgets::{Label, Link, TreeGrid, TreeRow, Widget},
};
use num_format::{Locale, ToFormattedString};
use std::collections::HashMap;

pub struct Tzx {}

impl Plugin for Tzx {
    fn get_name(&self) -> &'static str {
        "ZXSpectrum/TZX"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.len() > 10 && handle.rs(0, 7, "") == "ZXTape!" && handle.r16(7) == 0x11a {
            Some(90)
        } else {
            None
        }
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Parser::new(handle);
        let (entries, info) = parser.parse()?;
        let mut treegrid = TreeGrid::new(&["Filename", "Length"]);
        for entry in &entries {
            treegrid.add(entry.render(), ctx);
        }
        for item in &info {
            let mut row = TreeRow::default();
            row.push_column(Label::new(item.0));
            row.push_column(Label::new(item.1));
            treegrid.add(row, ctx);
        }
        Ok(Box::new(treegrid))
    }
}

#[derive(Default, Clone)]
struct Entry {
    filetype: u8,
    name: String,
    length: u16,
    p1: u16,
    p2: u16,
    data: Vec<u8>,
}

impl Entry {
    fn render(&self) -> TreeRow {
        let mut row = TreeRow::default();
        let entry = self.clone();
        row.push_column(Link::new(
            &self.name,
            &format!("zx/{:x}/{:x}:{:x}", entry.filetype, entry.p1, entry.p2),
            move |ctx: &Context| {
                ctx.post(Message::Open(Handle::new_from_array(
                    &entry.data,
                    &entry.name,
                    &format!("zx/{:x}/{:x}:{:x}", entry.filetype, entry.p1, entry.p2),
                )));
            },
        ));
        row.push_column(Label::new(&self.length.to_formatted_string(&Locale::en)));
        row
    }
}

struct Parser {
    handle: Handle,
}

impl Parser {
    fn new(handle: &Handle) -> Self {
        Self {
            handle: handle.clone(),
        }
    }

    fn parse(&self) -> Result<(Vec<Entry>, HashMap<&'static str, String>), &'static str> {
        let mut ofs = 0xa;
        let mut data = vec![];
        let mut info = HashMap::new();
        while ofs + 3 < self.handle.len() {
            let id = self.handle.r8(ofs);
            ofs += 1;
            match id {
                0x10 => data.push(self.standard_data(&mut ofs)),
                0x11 => data.push(self.turbo_data(&mut ofs)),
                0x19 => data.push(self.general_data(&mut ofs)),
                0x20 => {
                    if self.handle.r16(ofs) == 0 {
                        // stop the tape!
                        ofs = self.handle.len();
                    } else {
                        ofs += 2; // pause
                    }
                }
                0x30 => {
                    // skip comment
                    ofs += usize::from(self.handle.r8(ofs)) + 1;
                }
                0x32 => {
                    info = self.archive_info(&mut ofs);
                }
                _ => return Err("unknown block"),
            }
        }
        let mut entries = vec![];
        let mut entry = None;
        for block in &data {
            let handle = Handle::new_from_array(block, "", "");
            match handle.r8(0) {
                0 => {
                    // headers
                    if let Some(entry) = entry {
                        entries.push(entry);
                    }
                    entry = Some(Entry {
                        filetype: handle.r8(1),
                        name: handle.rs(2, 10, "").trim_end().to_string(),
                        length: handle.r16(12),
                        p1: handle.r16(14),
                        p2: handle.r16(16),
                        data: vec![],
                    });
                }
                0xff => {
                    // data
                    if let Some(entry) = entry.as_mut() {
                        entry
                            .data
                            .extend_from_slice(handle.read(1, handle.len() - 2));
                    } else {
                        return Err("data block without a file header");
                    }
                }
                _ => return Err("unknown block type"),
            }
        }
        Ok((entries, info))
    }

    fn standard_data(&self, ofs: &mut usize) -> &[u8] {
        *ofs += 2; // pause
        let length = self.handle.r16(*ofs);
        *ofs += 2;
        self.tap(ofs, usize::from(length))
    }

    fn turbo_data(&self, ofs: &mut usize) -> &[u8] {
        *ofs += 0xf; // header
        let length = self.handle.r24(*ofs) as usize;
        *ofs += 3;
        self.tap(ofs, length)
    }

    fn general_data(&self, ofs: &mut usize) -> &[u8] {
        let _length = self.handle.r32(*ofs) as usize;
        *ofs += 4;
        *ofs += 2; // skip pause
        let tot_p = self.handle.r32(*ofs) as usize;
        *ofs += 4;
        let npp = self.handle.r8(*ofs);
        *ofs += 1;
        let asp = usize::from(self.handle.r8(*ofs));
        *ofs += 1;
        let asp = if asp == 0 { 256 } else { asp };
        let tot_d = self.handle.r32(*ofs) as usize;
        *ofs += 4;
        let npd = self.handle.r8(*ofs);
        *ofs += 1;
        let asd = usize::from(self.handle.r8(*ofs));
        *ofs += 1;
        let asd = if asd == 0 { 256 } else { asd };
        // skip pilot/sync
        if tot_p != 0 {
            *ofs += (2 * usize::from(npp) + 1) * asp;
            *ofs += 3 * tot_p;
        }
        if tot_d != 0 {
            // skip symbol defs
            *ofs += (2 * usize::from(npd) + 1) * asd;
            self.tap(ofs, tot_d)
        } else {
            &[]
        }
    }

    fn archive_info(&self, ofs: &mut usize) -> HashMap<&'static str, String> {
        *ofs += 2; // skip len
        let num_str = self.handle.r8(*ofs);
        *ofs += 1;
        let mut info: HashMap<&'static str, String> = HashMap::new();
        for _ in 0..num_str {
            let tp = self.handle.r8(*ofs);
            *ofs += 1;
            let val = self.handle.rps(*ofs, "");
            *ofs += 1 + usize::from(self.handle.r8(*ofs));
            match tp {
                0 => {
                    info.insert("title", val);
                }
                1 => {
                    info.insert("publisher", val);
                }
                2 => {
                    info.insert("authors", val);
                }
                3 => {
                    info.insert("year", val);
                }
                4 => {
                    info.insert("language", val);
                }
                5 => {
                    info.insert("category", val);
                }
                6 => {
                    info.insert("price", val);
                }
                0xff => {
                    info.insert("comments", val);
                }
                _ => {}
            }
        }
        info
    }

    fn tap(&self, ofs: &mut usize, length: usize) -> &[u8] {
        let data = self.handle.read(*ofs, length);
        *ofs += length;
        data
    }
}
