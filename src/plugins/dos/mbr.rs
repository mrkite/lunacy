/*

DOS : Master Boot Record parser
@copyright 2026 Sean Kasun

*/
use crate::{
    lunacy::{Context, Handle, Message},
    plugins::Plugin,
    widgets::{Label, Link, TreeGrid, TreeRow, Widget},
};
use num_format::{Locale, ToFormattedString};
use std::rc::Rc;

pub struct Mbr {}

impl Plugin for Mbr {
    fn get_name(&self) -> &'static str {
        "DOS/MBR"
    }
    fn supports(&self, handle: &Handle) -> Option<i16> {
        // has to be large enough for a partition map
        if handle.len() < 512 {
            return None;
        }
        if handle.r16(0x1fe) != 0xaa55 {
            return None;
        }
        let base = 0x1be;
        if handle.r8(base) != 0x80 {
            // status byte should be 80 on at least first partition
            return None;
        }
        let lba = handle.r32(base + 8);
        let lbalen = handle.r32(base + 0xc);
        if (((lba + lbalen) * 512) as usize) <= handle.len() {
            return Some(80);
        }
        None
    }
    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Rc::new(Parser::new(handle));
        let partitions = parser.parse();

        let mut treegrid = TreeGrid::new(&["Partition", "Length", "Type"]);
        for partition in &partitions {
            let len = partition.len;
            let typ = partition.typ;
            let lba = partition.lba;
            let mut row = TreeRow::default();
            let entry = partition.clone();
            let parser = parser.clone();
            row.push_column(Link::new(
                &format!("Partition {lba}"),
                &format!("mbr/{}", entry.typ),
                move |ctx: &Context| {
                    let data = parser.load_file(&entry);
                    ctx.post(Message::Open(Handle::new_from_array(
                        data,
                        &format!("partition {}", entry.lba),
                        &format!("mbr/{}", entry.typ),
                    )));
                },
            ));
            row.push_column(Label::new(&len.to_formatted_string(&Locale::en)));
            row.push_column(Label::new(&format!("Type {typ}")));
            treegrid.add(row, ctx);
        }
        Ok(Box::new(treegrid))
    }
}

#[derive(Default, Clone)]
struct Entry {
    lba: usize,
    len: usize,
    typ: u8,
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
    fn parse(&self) -> Vec<Entry> {
        let mut partitions = vec![];
        for i in 0..4 {
            let base = 0x1be + i * 0x10;
            if self.handle.r8(base) == 0x80 {
                partitions.push(Entry {
                    lba: self.handle.r32(base + 8) as usize,
                    len: self.handle.r32(base + 0xc) as usize,
                    typ: self.handle.r8(base + 4),
                });
            }
        }
        partitions
    }
    fn load_file(&self, entry: &Entry) -> &[u8] {
        self.handle.read(entry.lba * 512, entry.len * 512)
    }
}
