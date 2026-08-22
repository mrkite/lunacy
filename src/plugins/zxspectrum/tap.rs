/*

ZX-series : TAP tape format parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};

pub struct Tap {}

impl Plugin for Tap {
    fn get_name(&self) -> &str {
        "ZXSpectrum/TAP"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() > 20 {
            let crclen = usize::from(file.r16(0));
            if crclen > 0 && crclen < file.len() {
                let mut crc = 0;
                for i in 2..crclen + 2 {
                    crc ^= file.r8(i);
                }
                if crc == 0 {
                    return Some(70);
                }
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
            &["Filename", "Length"],
            &format!("tap_{}", file.get_name()),
            move |entry: &Entry| {
                ctx.post(Message::Open(File::new_from_array(
                    &entry.data,
                    &entry.name,
                    &format!("zx/{:x}/{:x}:{:x}", entry.filetype, entry.p1, entry.p2),
                )));
            },
        );
        for entry in entries.iter() {
            treegrid.add(entry.render());
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

    fn parse(&self) -> Result<Vec<Entry>, &'static str> {
        let mut entries = vec![];
        let mut ofs = 0;
        let mut entry = None;
        while ofs < self.file.len() {
            let length = usize::from(self.file.r16(ofs));
            ofs += 2;
            match self.file.r8(ofs) {
                0 => {
                    // headers
                    if let Some(entry) = entry {
                        entries.push(entry);
                    }
                    entry = Some(Entry {
                        filetype: self.file.r8(ofs + 1),
                        name: self.file.rs(ofs + 2, 10, ""),
                        length: self.file.r16(ofs + 12),
                        p1: self.file.r16(ofs + 14),
                        p2: self.file.r16(ofs + 16),
                        data: vec![],
                    });
                }
                0xff => {
                    // data
                    if let Some(entry) = entry.as_mut() {
                        entry
                            .data
                            .extend_from_slice(self.file.read(ofs + 1, length - 2));
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
