/*

DOS : Fat 12/16 Partition parser
@copyright 2026 Sean kasun

*/
use crate::{
    lunacy::{Context, Handle, Message},
    plugins::Plugin,
    widgets::{Label, Link, TreeGrid, TreeRow, Widget},
};
use num_format::{Locale, ToFormattedString};
use std::{cmp, rc::Rc};

pub struct Fat {}

impl Plugin for Fat {
    fn get_name(&self) -> &'static str {
        "DOS/Fat16"
    }
    fn supports(&self, handle: &Handle) -> Option<i16> {
        // has to be large enough for boot block
        if handle.len() < 512 {
            return None;
        }
        if handle.r32(0) & 0xff_ffff != 0x90_3ceb {
            return None;
        }
        // only fat16
        let per_sector = usize::from(handle.r16(0xb));
        let num_reserved = usize::from(handle.r16(0xe));
        let fat = num_reserved * per_sector;
        if handle.r32(fat) != 0xffff_fff8 {
            return None;
        }
        Some(60)
    }
    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Rc::new(Parser::new(handle));
        let entries = parser.get_directory(parser.root)?;
        let mut treegrid = TreeGrid::new(&["Filename", "Ext", "Modified", "Length"]);
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
    mod_time: u16,
    mod_date: u16,
    start: usize,
    len: usize,
    children: Vec<Self>,
}

impl Entry {
    fn render(&self, parser: &Rc<Parser>) -> TreeRow {
        let mut row = TreeRow::default();
        let entry = self.clone();
        {
            let parser = parser.clone();
            row.push_column(Link::new(
                &self.name,
                &format!("dos/{}", entry.ext),
                move |ctx: &Context| {
                    let data = parser.load_file(&entry);
                    ctx.post(Message::Open(Handle::new_from_array(
                        &data,
                        &entry.name,
                        &format!("dos/{}", entry.ext),
                    )));
                },
            ));
        }
        row.push_column(Label::new(&self.ext));
        row.push_column(Label::new(&format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            ((self.mod_date >> 9) & 0x7f) + 1980,
            (self.mod_date >> 5) & 0xf,
            self.mod_date & 0x1f,
            (self.mod_time >> 11) & 0x1f,
            (self.mod_time >> 5) & 0x3f
        )));
        row.push_column(Label::new(&self.len.to_formatted_string(&Locale::en)));
        for child in &self.children {
            row.push_child(child.render(parser));
        }
        row
    }
}

struct Parser {
    per_cluster: usize,
    fat: usize,
    root: usize,
    root_entries: usize,
    cluster: usize,
    handle: Handle,
}

impl Parser {
    fn new(handle: &Handle) -> Self {
        let per_sector = usize::from(handle.r16(0xb));
        let per_cluster = usize::from(handle.r8(0xd)) * per_sector;
        let num_reserved = usize::from(handle.r16(0xe));
        let fat = num_reserved * per_sector;
        let num_fats = usize::from(handle.r8(0x10));
        let per_fat = usize::from(handle.r16(0x16));
        let root = fat + num_fats * per_fat * per_sector;
        let root_entries = usize::from(handle.r16(0x11));
        let dir_sectors = (root_entries * 32) / per_sector;
        let cluster = root + (dir_sectors - 2) * per_sector;
        Self {
            per_cluster,
            fat,
            root,
            root_entries,
            cluster,
            handle: handle.clone(),
        }
    }
    fn get_directory(&self, offset: usize) -> Result<Vec<Entry>, &'static str> {
        let mut entries = vec![];
        for dir in 0..self.root_entries {
            let ofs = offset + dir * 32;
            let exists = self.handle.r8(ofs);
            if exists != 0 && exists != 0x2e && exists != 0xe5 {
                let mut children = vec![];
                let flags = self.handle.r8(ofs + 0xb);
                if flags & 0x10 != 0 {
                    // subdirectory
                    let start = usize::from(self.handle.r16(ofs + 0x1a));
                    children = self.get_directory(start * self.per_cluster)?;
                }
                entries.push(Entry {
                    name: self.handle.rs(ofs, 8, "ibmpc").trim_end().to_string(),
                    ext: self.handle.rs(ofs + 0x8, 3, "ibmpc").trim_end().to_string(),
                    mod_time: self.handle.r16(ofs + 0x16),
                    mod_date: self.handle.r16(ofs + 0x18),
                    start: usize::from(self.handle.r16(ofs + 0x1a)),
                    len: self.handle.r32(ofs + 0x1c) as usize,
                    children,
                });
            }
        }
        Ok(entries)
    }
    fn load_file(&self, id: &Entry) -> Vec<u8> {
        let mut cluster = id.start;
        let mut len = id.len;
        let mut data = vec![];
        while cluster < 0xfff8 && len > 0 {
            let blen = cmp::min(len, self.per_cluster);
            data.extend_from_slice(
                self.handle
                    .read(self.cluster + cluster * self.per_cluster, blen),
            );
            cluster = usize::from(self.handle.r16(self.fat + cluster * 2));
            len -= blen;
        }
        data
    }
}
