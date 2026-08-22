/*

DOS : Master Boot Record parser
@copyright 2026 Sean Kasun

*/
use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};
use std::rc::Rc;

pub struct Mbr {}

impl Plugin for Mbr {
    fn get_name(&self) -> &str {
        "DOS/MBR"
    }
    fn supports(&self, file: &File) -> Option<i16> {
        // has to be large enough for a partition map
        if file.len() < 512 {
            return None;
        }
        if file.r16(0x1fe) != 0xaa55 {
            return None;
        }
        let base = 0x1be;
        if file.r8(base) != 0x80 {
            // status byte should be 80 on at least first partition
            return None;
        }
        let lba = file.r32(base + 8);
        let lbalen = file.r32(base + 0xc);
        if (((lba + lbalen) * 512) as usize) <= file.len() {
            return Some(80);
        }
        None
    }
    fn create(
        &self,
        file: &File,
        ctx: Context,
    ) -> Result<Box<dyn crate::plugins::Widget>, &'static str> {
        let parser = Rc::new(Parser::new(file));
        let partitions = parser.parse(file)?;

        let mut treegrid = TreeGrid::new(
            &["Partition", "Length", "Type"],
            &format!("mbr_{}", file.get_name()),
            move |entry: &Entry| {
                ctx.post(match parser.load_file(entry) {
                    Ok(data) => Message::Open(File::new_from_array(
                        data,
                        &format!("partition {}", entry.lba),
                        &format!("mbr/{}", entry.typ),
                    )),
                    Err(err) => Message::Error(err.to_string()),
                });
            },
        );
        for partition in partitions.iter() {
            let len = partition.len;
            let typ = partition.typ;
            let lba = partition.lba;
            let mut row = TreeRow::new(partition.clone());
            row.push_column(&format!("Partition {}", lba));
            row.push_column(&len.to_formatted_string(&Locale::en));
            row.push_column(&format!("Type {}", typ));
            treegrid.add(row);
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
    file: File,
}

impl Parser {
    fn new(file: &File) -> Self {
        Self { file: file.clone() }
    }
    fn parse(&self, file: &File) -> Result<Vec<Entry>, &'static str> {
        let mut partitions = vec![];
        for i in 0..4 {
            let base = 0x1be + i * 0x10;
            if file.r8(base) == 0x80 {
                partitions.push(Entry {
                    lba: file.r32(base + 8) as usize,
                    len: file.r32(base + 0xc) as usize,
                    typ: file.r8(base + 4),
                });
            }
        }
        Ok(partitions)
    }
    fn load_file(&self, entry: &Entry) -> Result<&[u8], &'static str> {
        Ok(self.file.read(entry.lba * 512, entry.len * 512))
    }
}
