/*

C64 : D64 disk image parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::Plugin,
    widgets::{Label, Link, TreeGrid, TreeRow, Widget},
};
use num_format::{Locale, ToFormattedString};

pub struct D64 {}

impl Plugin for D64 {
    fn get_name(&self) -> &'static str {
        "C64/D64"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if [174_848, 175_531, 196_608, 197_376, 205_312, 206_114].contains(&handle.len()) {
            Some(90)
        } else {
            None
        }
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Parser::new(handle);
        let entries = parser.parse();
        let mut treegrid = TreeGrid::new(&["Filename", "Type", "Length"]);
        for entry in &entries {
            treegrid.add(entry.render(), ctx);
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
    fn render(&self) -> TreeRow {
        let mut row = TreeRow::default();
        let entry = self.clone();
        row.push_column(Link::new(
            &self.name,
            &format!("d64/{:x}", entry.filetype),
            move |ctx: &Context| {
                ctx.post(Message::Open(Handle::new_from_array(
                    &entry.data,
                    &entry.name,
                    &format!("d64/{:x}", entry.filetype),
                )));
            },
        ));
        row.push_column(Label::new(&self.type_name));
        row.push_column(Label::new(
            &self.data.len().to_formatted_string(&Locale::en),
        ));
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

    fn parse(&self) -> Vec<Entry> {
        let mut track = 18;
        let mut sector = 1;
        let mut entries = vec![];
        while track != 0 {
            let block = TRACK_TO_OFFSET[track] + sector * 256;
            track = usize::from(self.handle.r8(block));
            sector = usize::from(self.handle.r8(block + 1));
            for i in 0..8 {
                let base = block + i * 0x20;
                let filetype = self.handle.r8(block + 2) & 0xf;
                if filetype != 0 {
                    entries.push(Entry {
                        name: self
                            .handle
                            .rs(base + 5, 16, "petscii")
                            .trim_end()
                            .to_string(),
                        filetype,
                        type_name: FILE_TYPES
                            .get(usize::from(filetype))
                            .map_or_else(|| format!("${filetype:x}"), ToString::to_string),
                        data: self.load_file(self.handle.r8(base + 3), self.handle.r8(base + 4)),
                    });
                }
            }
        }
        entries
    }

    fn load_file(&self, track: u8, sector: u8) -> Vec<u8> {
        let mut track = track;
        let mut sector = sector;
        let mut data = vec![];
        while track != 0 {
            let block = TRACK_TO_OFFSET[usize::from(track)] + usize::from(sector) * 256;
            track = self.handle.r8(block);
            sector = self.handle.r8(block + 1);
            let blen = if track != 0 { 254 } else { sector };
            data.extend_from_slice(self.handle.read(block + 2, usize::from(blen)));
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
