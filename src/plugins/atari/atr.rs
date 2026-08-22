/*

Atari ST : ATR disk image parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};

pub struct Atr {}

impl Plugin for Atr {
    fn get_name(&self) -> &str {
        "Atari/ATR"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() >= 92176 && file.r16(0) == 0x296 {
            let len = usize::from(file.r16(2)) | (usize::from(file.r8(8)) << 16);
            if len * 16 + 16 == file.len() {
                return Some(90);
            }
        }
        None
    }

    fn create(
        &self,
        file: &File,
        ctx: Context,
    ) -> Result<Box<dyn crate::plugins::Widget>, &'static str> {
        let parser = Parser::new(file);
        let entries = parser.parse()?;
        let mut treegrid = TreeGrid::new(
            &["Filename", "Ext", "Sector"],
            &format!("atr_{}", file.get_name()),
            move |entry: &Entry| {
                if entry.is_file {
                    ctx.post(match parser.load_file(entry.sector) {
                        Ok(data) => Message::Open(File::new_from_array(
                            &data,
                            &entry.name,
                            &format!("ataridos/{}", entry.ext),
                        )),
                        Err(err) => Message::Error(err.to_string()),
                    });
                }
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
    is_file: bool,
    sector_count: u16,
    sector: u16,
    children: Vec<Entry>,
}

impl Entry {
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.name);
        row.push_column(&self.ext);
        row.push_column(&self.sector_count.to_formatted_string(&Locale::en));
        for child in self.children.iter() {
            row.push_child(child.render());
        }
        row
    }
}

struct Parser {
    file: File,
    sector_size: u16,
}

impl Parser {
    fn new(file: &File) -> Self {
        let sector_size = file.r16(4);
        Self {
            file: file.clone(),
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
            let flag = self.file.r8(ofs);
            if flag == 0 {
                break;
            }
            if flag & 0x80 != 0 {
                // deleted
                continue;
            }
            let sector_count = self.file.r16(ofs + 1);
            if sector_count == 0 {
                // empty file
                continue;
            }
            let start = self.file.r16(ofs + 3);
            let name = self.file.rs(ofs + 5, 8, "atascii");
            let ext = self.file.rs(ofs + 13, 3, "atascii");
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

    fn load_file(&self, sector: u16) -> Result<Vec<u8>, &'static str> {
        let mut next = sector;
        let mut data = vec![];
        while next != 0 {
            let fofs = self.seek_to(next);
            let meta = fofs + usize::from(self.sector_size) - 3;
            let edge = self.file.r8(meta + 2);
            let end = if edge & 0x80 != 0 {
                usize::from(edge & 0x7f)
            } else {
                meta - fofs
            };
            data.extend_from_slice(self.file.read(fofs, end));
            next = self.file.r16be(meta) & 0x3ff;
        }
        Ok(data)
    }
}
