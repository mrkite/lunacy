/*

Acorn/BBC Micro : ADFS-S,ADFS-M, and ADFS-L filesystem parser
@copyright 2026 Sean Kasun

*/

use std::rc::Rc;

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::{Plugin, get_charset},
    widgets::{Label, Link, TreeGrid, TreeRow, Widget},
};
use num_format::{Locale, ToFormattedString};

pub struct Adfs {}

impl Plugin for Adfs {
    fn get_name(&self) -> &'static str {
        "Acorn/ADFS"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.len() < 6500 {
            return None;
        }
        // multiple of sector size
        if handle.len() & 0xff != 0 {
            return None;
        }

        // 800k files have "Nick" at 2047 which doesn't make any sense
        // I'll need to spend some time figuring it out, until then we'll only support ADFS-S to ADFS-L
        if handle.r4(513) == "Hugo" {
            Some(70)
        } else {
            None
        }
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Rc::new(Parser::new(handle));
        let entries = parser.parse()?;
        let mut treegrid = TreeGrid::new(&["Filename", "Length"]);
        for entry in &entries {
            treegrid.add(entry.render(&parser), ctx);
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
                &entry.mime.clone(),
                move |ctx: &Context| {
                    let data = parser.load_file(&entry);
                    ctx.post(Message::Open(Handle::new_from_array(
                        &data,
                        &entry.name,
                        &entry.mime,
                    )));
                },
            ));
            row.push_column(Label::new(&self.length.to_formatted_string(&Locale::en)));
        } else {
            row.push_column(Label::new(&self.name));
            for child in &self.children {
                row.push_child(child.render(parser));
            }
        }
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
                let ch = self.handle.r8(ofs + j) & 0x7f;
                if ch == 0 || ch == 0xd {
                    break;
                }
                name.push(cs[usize::from(ch)]);
            }
            let load = self.handle.r32(ofs + 0xa);
            let exec = self.handle.r32(ofs + 0xe);
            let length = self.handle.r32(ofs + 0x12);
            let sector = self.handle.r24(ofs + 0x16);
            if !name.is_empty() {
                let mut entry = Entry {
                    name,
                    mime: format!("acorn/{load:x}/{exec:x}"),
                    sector,
                    length,
                    is_file: true,
                    children: vec![],
                };
                if self.handle.r8(ofs + 3) & 0x80 != 0 {
                    // D
                    entry.is_file = false;
                    entry.children = self.dir(entry.sector as usize)?;
                }
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    fn load_file(&self, entry: &Entry) -> Vec<u8> {
        self.handle
            .read(entry.sector as usize * 256, entry.length as usize)
            .to_vec()
    }
}
