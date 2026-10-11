/*

ZX-series : TAP tape format parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::Plugin,
    widgets::{Label, Link, TreeGrid, TreeRow, Widget},
};
use num_format::{Locale, ToFormattedString};

pub struct Tap {}

impl Plugin for Tap {
    fn get_name(&self) -> &'static str {
        "ZXSpectrum/TAP"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.len() > 20 {
            let crclen = usize::from(handle.r16(0));
            if crclen > 0 && crclen < handle.len() {
                let mut crc = 0;
                for i in 2..crclen + 2 {
                    crc ^= handle.r8(i);
                }
                if crc == 0 {
                    return Some(70);
                }
            }
        }
        None
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Parser::new(handle);
        let entries = parser.parse()?;
        let mut treegrid = TreeGrid::new(&["Filename", "Length"]);
        for entry in &entries {
            treegrid.add(entry.render(), ctx);
        }
        Ok(Box::new(treegrid))
    }
}

#[derive(Clone)]
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

    fn parse(&self) -> Result<Vec<Entry>, &'static str> {
        let mut entries = vec![];
        let mut ofs = 0;
        let mut entry = None;
        while ofs < self.handle.len() {
            let length = usize::from(self.handle.r16(ofs));
            ofs += 2;
            match self.handle.r8(ofs) {
                0 => {
                    // headers
                    if let Some(entry) = entry {
                        entries.push(entry);
                    }
                    entry = Some(Entry {
                        filetype: self.handle.r8(ofs + 1),
                        name: self.handle.rs(ofs + 2, 10, "").trim_end().to_string(),
                        length: self.handle.r16(ofs + 12),
                        p1: self.handle.r16(ofs + 14),
                        p2: self.handle.r16(ofs + 16),
                        data: vec![],
                    });
                }
                0xff => {
                    // data
                    if let Some(entry) = entry.as_mut() {
                        entry
                            .data
                            .extend_from_slice(self.handle.read(ofs + 1, length - 2));
                    } else {
                        return Err("data block without a file header");
                    }
                }
                _ => return Err("unknown block type"),
            }
            ofs += length;
        }
        if let Some(entry) = entry {
            entries.push(entry);
        }
        Ok(entries)
    }
}
