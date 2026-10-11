/*

Atari ST : ATR disk image parser
@copyright 2026 Sean Kasun

*/

use std::rc::Rc;

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::Plugin,
    widgets::{Label, Link, TreeGrid, TreeRow, Widget},
};
use num_format::{Locale, ToFormattedString};

pub struct Atr {}

impl Plugin for Atr {
    fn get_name(&self) -> &'static str {
        "Atari/ATR"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.len() >= 92176 && handle.r16(0) == 0x296 {
            let len = usize::from(handle.r16(2)) | (usize::from(handle.r8(8)) << 16);
            if len * 16 + 16 == handle.len() {
                return Some(90);
            }
        }
        None
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Rc::new(Parser::new(handle));
        let entries = parser.parse()?;
        let mut treegrid = TreeGrid::new(&["Filename", "Ext", "Sector"]);
        for entry in &entries {
            treegrid.add(entry.render(&parser), ctx);
        }
        Ok(Box::new(treegrid))
    }
}

#[derive(Default, Clone)]
struct Entry {
    name: String,
    ext: String,
    is_file: bool,
    sector_count: u16,
    sector: u16,
    children: Vec<Self>,
}

impl Entry {
    fn render(&self, parser: &Rc<Parser>) -> TreeRow {
        let mut row = TreeRow::default();
        if self.is_file {
            let entry = self.clone();
            let parser = parser.clone();
            row.push_column(Link::new(
                &self.name,
                &format!("ataridos/{}", entry.ext),
                move |ctx: &Context| {
                    let data = parser.load_file(entry.sector);
                    ctx.post(Message::Open(Handle::new_from_array(
                        &data,
                        &entry.name,
                        &format!("ataridos/{}", entry.ext),
                    )));
                },
            ));
        } else {
            row.push_column(Label::new(&self.name));
        }
        row.push_column(Label::new(&self.ext));
        row.push_column(Label::new(
            &self.sector_count.to_formatted_string(&Locale::en),
        ));
        for child in &self.children {
            row.push_child(child.render(parser));
        }
        row
    }
}

struct Parser {
    handle: Handle,
    sector_size: u16,
}

impl Parser {
    fn new(handle: &Handle) -> Self {
        let sector_size = handle.r16(4);
        Self {
            handle: handle.clone(),
            sector_size,
        }
    }

    fn parse(&self) -> Result<Vec<Entry>, &'static str> {
        // header is 16 bytes long
        // first 3 sectors are always 128 bytes long
        // file directory starts at sector 361
        self.load_directory(361, 8)
    }

    fn load_directory(&self, sector: u16, count: u16) -> Result<Vec<Entry>, &'static str> {
        let ofs = self.seek_to(sector);
        let mut files = vec![];
        for i in 0..count * 8 {
            let ofs = ofs + usize::from(i) * 16;
            let flag = self.handle.r8(ofs);
            if flag == 0 {
                break;
            }
            if flag & 0x80 != 0 {
                // deleted
                continue;
            }
            let sector_count = self.handle.r16(ofs + 1);
            if sector_count == 0 {
                // empty file
                continue;
            }
            let start = self.handle.r16(ofs + 3);
            let name = self.handle.rs(ofs + 5, 8, "atascii").trim_end().to_string();
            let ext = self
                .handle
                .rs(ofs + 13, 3, "atascii")
                .trim_end()
                .to_string();
            if flag & 0x10 != 0 {
                // directory
                files.push(Entry {
                    name,
                    ext,
                    is_file: false,
                    sector_count,
                    sector: 0,
                    children: self.load_directory(start, sector_count)?,
                });
            } else {
                files.push(Entry {
                    name,
                    ext,
                    is_file: true,
                    sector_count,
                    sector: start,
                    children: vec![],
                });
            }
        }
        Ok(files)
    }

    fn seek_to(&self, sector: u16) -> usize {
        if sector < 4 {
            16 + usize::from(sector.saturating_sub(1)) * 128
        } else {
            0x190 + usize::from(sector - 4) * usize::from(self.sector_size)
        }
    }

    fn load_file(&self, sector: u16) -> Vec<u8> {
        let mut next = sector;
        let mut data = vec![];
        while next != 0 {
            let fofs = self.seek_to(next);
            let meta = fofs + usize::from(self.sector_size) - 3;
            let edge = self.handle.r8(meta + 2);
            let end = if edge & 0x80 != 0 {
                usize::from(edge & 0x7f)
            } else {
                meta - fofs
            };
            data.extend_from_slice(self.handle.read(fofs, end));
            next = self.handle.r16be(meta) & 0x3ff;
        }
        data
    }
}
