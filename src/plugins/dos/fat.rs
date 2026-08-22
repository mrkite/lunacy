/*

DOS : Fat 12/16 Partition parser
@copyright 2026 Sean kasun

*/
use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};
use std::cmp;

pub struct Fat {}

impl Plugin for Fat {
    fn get_name(&self) -> &str {
        "DOS/Fat16"
    }
    fn supports(&self, file: &File) -> Option<i16> {
        // has to be large enough for boot block
        if file.len() < 512 {
            return None;
        }
        if file.r32(0) & 0xffffff != 0x903ceb {
            return None;
        }
        // only fat16
        let per_sector = usize::from(file.r16(0xb));
        let num_reserved = usize::from(file.r16(0xe));
        let fat = num_reserved * per_sector;
        if file.r32(fat) != 0xfffffff8 {
            return None;
        }
        Some(60)
    }
    fn create(
        &self,
        file: &File,
        ctx: Context,
    ) -> Result<Box<dyn crate::plugins::Widget>, &'static str> {
        let parser = Parser::new(file);
        let entries = parser.get_directory(parser.root)?;
        let mut treegrid = TreeGrid::new(
            &["Filename", "Ext", "Modified", "Length"],
            &format!("fat_{}", file.get_name()),
            move |entry: &Entry| {
                ctx.post(match parser.load_file(entry) {
                    Ok(data) => Message::Open(File::new_from_array(
                        &data,
                        &entry.name,
                        &format!("dos/{}", entry.ext),
                    )),
                    Err(err) => Message::Error(err.to_string()),
                });
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
    ext: String,
    mod_time: u16,
    mod_date: u16,
    start: usize,
    len: usize,
    children: Vec<Entry>,
}

impl Entry {
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.name);
        row.push_column(&self.ext);
        row.push_column(&format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            ((self.mod_date >> 9) & 0x7f) + 1980,
            (self.mod_date >> 5) & 0xf,
            self.mod_date & 0x1f,
            (self.mod_time >> 11) & 0x1f,
            (self.mod_time >> 5) & 0x3f
        ));
        row.push_column(&self.len.to_formatted_string(&Locale::en));
        for child in self.children.iter() {
            row.push_child(child.render())
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
    file: File,
}

impl Parser {
    fn new(file: &File) -> Self {
        let per_sector = usize::from(file.r16(0xb));
        let per_cluster = usize::from(file.r8(0xd)) * per_sector;
        let num_reserved = usize::from(file.r16(0xe));
        let fat = num_reserved * per_sector;
        let num_fats = usize::from(file.r8(0x10));
        let per_fat = usize::from(file.r16(0x16));
        let root = fat + num_fats * per_fat * per_sector;
        let root_entries = usize::from(file.r16(0x11));
        let dir_sectors = (root_entries * 32) / per_sector;
        let cluster = root + (dir_sectors - 2) * per_sector;
        Self {
            per_cluster,
            fat,
            root,
            root_entries,
            cluster,
            file: file.clone(),
        }
    }
    fn get_directory(&self, offset: usize) -> Result<Vec<Entry>, &'static str> {
        let mut entries = vec![];
        for dir in 0..self.root_entries {
            let ofs = offset + dir * 32;
            let exists = self.file.r8(ofs);
            if exists != 0 && exists != 0x2e && exists != 0xe5 {
                let mut children = vec![];
                let flags = self.file.r8(ofs + 0xb);
                if flags & 0x10 != 0 {
                    // subdirectory
                    let start = usize::from(self.file.r16(ofs + 0x1a));
                    children = self.get_directory(start * self.per_cluster)?;
                }
                entries.push(Entry {
                    name: self.file.rs(ofs, 8, "ibmpc"),
                    ext: self.file.rs(ofs + 0x8, 3, "ibmpc"),
                    mod_time: self.file.r16(ofs + 0x16),
                    mod_date: self.file.r16(ofs + 0x18),
                    start: usize::from(self.file.r16(ofs + 0x1a)),
                    len: self.file.r32(ofs + 0x1c) as usize,
                    children,
                })
            }
        }
        Ok(entries)
    }
    fn load_file(&self, id: &Entry) -> Result<Vec<u8>, &'static str> {
        let mut cluster = id.start;
        let mut len = id.len;
        let mut data = vec![];
        while cluster < 0xfff8 && len > 0 {
            let blen = cmp::min(len, self.per_cluster);
            data.extend_from_slice(
                self.file
                    .read(self.cluster + cluster * self.per_cluster, blen),
            );
            cluster = usize::from(self.file.r16(self.fat + cluster * 2));
            len -= blen;
        }
        Ok(data)
    }
}
