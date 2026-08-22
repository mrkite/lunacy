/*

ZX-series : TZX tape format parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};
use std::collections::HashMap;

pub struct Tzx {}

impl Plugin for Tzx {
    fn get_name(&self) -> &str {
        "ZXSpectrum/TZX"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() > 10 && file.rs(0, 7, "") == "ZXTape!" && file.r16(7) == 0x11a {
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
        let (entries, info) = parser.parse()?;
        let mut treegrid = TreeGrid::new(
            &["Filename", "Length"],
            &format!("tzx_{}", file.get_name()),
            move |entry: &Entry| {
                if entry.is_file {
                    ctx.post(Message::Open(File::new_from_array(
                        &entry.data,
                        &entry.name,
                        &format!("zx/{:x}/{:x}:{:x}", entry.filetype, entry.p1, entry.p2),
                    )));
                }
            },
        );
        for entry in entries.iter() {
            treegrid.add(entry.render());
        }
        for item in info.iter() {
            let mut row = TreeRow::new(Entry::default());
            row.push_column(item.0);
            row.push_column(item.1);
            treegrid.add(row);
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
    is_file: bool,
}

impl Entry {
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.name);
        row.push_column(&self.length.to_formatted_string(&Locale::en));
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

    fn parse(&self) -> Result<(Vec<Entry>, HashMap<&'static str, String>), &'static str> {
        let mut ofs = 0xa;
        let mut data = vec![];
        let mut info = HashMap::new();
        while ofs + 3 < self.file.len() {
            let id = self.file.r8(ofs);
            ofs += 1;
            match id {
                0x10 => data.push(self.standard_data(&mut ofs)),
                0x11 => data.push(self.turbo_data(&mut ofs)),
                0x19 => data.push(self.general_data(&mut ofs)),
                0x20 => {
                    if self.file.r16(ofs) == 0 {
                        // stop the tape!
                        ofs = self.file.len();
                    } else {
                        ofs += 2; // pause
                    }
                }
                0x30 => {
                    // skip comment
                    ofs += usize::from(self.file.r8(ofs)) + 1;
                }
                0x32 => {
                    info = self.archive_info(&mut ofs);
                }
                _ => return Err("unknown block"),
            }
        }
        let mut entries = vec![];
        let mut entry = None;
        for block in data.iter() {
            let file = File::new_from_array(block, "", "");
            match file.r8(0) {
                0 => {
                    // headers
                    if let Some(entry) = entry {
                        entries.push(entry);
                    }
                    entry = Some(Entry {
                        filetype: file.r8(1),
                        name: file.rs(2, 10, ""),
                        length: file.r16(12),
                        p1: file.r16(14),
                        p2: file.r16(16),
                        data: vec![],
                        is_file: true,
                    });
                }
                0xff => {
                    // data
                    if let Some(entry) = entry.as_mut() {
                        entry.data.extend_from_slice(file.read(1, file.len() - 2));
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
        let length = self.file.r16(*ofs);
        *ofs += 2;
        self.tap(ofs, usize::from(length))
    }

    fn turbo_data(&self, ofs: &mut usize) -> &[u8] {
        *ofs += 0xf; // header
        let length = self.file.r24(*ofs) as usize;
        *ofs += 3;
        self.tap(ofs, length)
    }

    fn general_data(&self, ofs: &mut usize) -> &[u8] {
        let _length = self.file.r32(*ofs) as usize;
        *ofs += 4;
        *ofs += 2; // skip pause
        let totp = self.file.r32(*ofs) as usize;
        *ofs += 4;
        let npp = self.file.r8(*ofs);
        *ofs += 1;
        let asp = usize::from(self.file.r8(*ofs));
        *ofs += 1;
        let asp = if asp == 0 { 256 } else { asp };
        let totd = self.file.r32(*ofs) as usize;
        *ofs += 4;
        let npd = self.file.r8(*ofs);
        *ofs += 1;
        let asd = usize::from(self.file.r8(*ofs));
        *ofs += 1;
        let asd = if asd == 0 { 256 } else { asd };
        // skip pilot/sync
        if totp != 0 {
            *ofs += (2 * usize::from(npp) + 1) * asp;
            *ofs += 3 * totp;
        }
        if totd != 0 {
            // skip symbol defs
            *ofs += (2 * usize::from(npd) + 1) * asd;
            self.tap(ofs, totd)
        } else {
            &[]
        }
    }

    fn archive_info(&self, ofs: &mut usize) -> HashMap<&'static str, String> {
        *ofs += 2; // skip len
        let num_str = self.file.r8(*ofs);
        *ofs += 1;
        let mut info: HashMap<&'static str, String> = HashMap::new();
        for _ in 0..num_str {
            let tp = self.file.r8(*ofs);
            *ofs += 1;
            let val = self.file.rps(*ofs, "");
            *ofs += 1 + usize::from(self.file.r8(*ofs));
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
            };
        }
        info
    }

    fn tap(&self, ofs: &mut usize, length: usize) -> &[u8] {
        let data = self.file.read(*ofs, length);
        *ofs += length;
        data
    }
}
