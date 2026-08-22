/*

C64 : D64 disk image parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};

pub struct D64 {}

impl Plugin for D64 {
    fn get_name(&self) -> &str {
        "C64/D64"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if [174848, 175531, 196608, 197376, 205312, 206114].contains(&file.len()) {
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
        let entries = parser.parse()?;
        let mut treegrid = TreeGrid::new(
            &["Filename", "Type", "Length"],
            &format!("d64_{}", file.get_name()),
            move |entry: &Entry| {
                ctx.post(Message::Open(File::new_from_array(
                    &entry.data,
                    &entry.name,
                    &format!("d64/{:x}", entry.filetype),
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
    name: String,
    filetype: u8,
    type_name: String,
    data: Vec<u8>,
}

impl Entry {
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.name);
        row.push_column(&self.type_name);
        row.push_column(&self.data.len().to_formatted_string(&Locale::en));
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
        let mut track = 18;
        let mut sector = 1;
        let mut entries = vec![];
        while track != 0 {
            let block = TRACK_TO_OFFSET[track] + sector * 256;
            track = usize::from(self.file.r8(block));
            sector = usize::from(self.file.r8(block + 1));
            for i in 0..8 {
                let base = block + i * 0x20;
                let filetype = self.file.r8(block + 2) & 0xf;
                if filetype != 0 {
                    entries.push(Entry {
                        name: self.file.rs(base + 5, 16, "petscii"),
                        filetype,
                        type_name: if let Some(tn) = FILE_TYPES.get(usize::from(filetype)) {
                            tn.to_string()
                        } else {
                            format!("${:x}", filetype)
                        },
                        data: self.load_file(self.file.r8(base + 3), self.file.r8(base + 4)),
                    });
                }
            }
        }
        Ok(entries)
    }

    fn load_file(&self, track: u8, sector: u8) -> Vec<u8> {
        let mut track = track;
        let mut sector = sector;
        let mut data = vec![];
        while track != 0 {
            let block = TRACK_TO_OFFSET[usize::from(track)] + usize::from(sector) * 256;
            track = self.file.r8(block);
            sector = self.file.r8(block + 1);
            let blen = if track != 0 { 254 } else { sector };
            data.extend_from_slice(self.file.read(block + 2, usize::from(blen)));
        }
        data
    }
}

const FILE_TYPES: [&str; 5] = ["DEL", "SEQ", "PRG", "USR", "REL"];

const TRACK_TO_OFFSET: [usize; 41] = [
    0x00000, // track 0 doesn't exist
    0x00000, // 1, 21 sectors
    0x01500, // 2
    0x02a00, // 3
    0x03f00, // 4
    0x05400, // 5
    0x06900, // 6
    0x07e00, // 7
    0x09300, // 8
    0x0a800, // 9
    0x0bd00, // 10
    0x0d200, // 11
    0x0e700, // 12
    0x0fc00, // 13
    0x11100, // 14
    0x12600, // 15
    0x13b00, // 16
    0x15000, // 17
    0x16500, // 18, 19 sectors
    0x17800, // 19
    0x18b00, // 20
    0x19e00, // 21
    0x1b100, // 22
    0x1c400, // 23
    0x1d700, // 24
    0x1ea00, // 25, 18 sectors
    0x1fc00, // 26
    0x20e00, // 27
    0x22000, // 28
    0x23200, // 29
    0x24400, // 30
    0x25600, // 31, 17 sectors
    0x26700, // 32
    0x27800, // 33
    0x28900, // 34
    0x29a00, // 35
    0x2ab00, // 36
    0x2bc00, // 37
    0x2cd00, // 38
    0x2de00, // 39
    0x2ef00, // 40
];
