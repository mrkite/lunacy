use std::rc::Rc;

/*

Apple II : DOS 3.3 filesystem parser
@copyright 2026 Sean Kasun

*/
use crate::{
    lunacy::{Context, Handle, Message},
    plugins::Plugin,
    widgets::{Label, Link, TreeGrid, TreeRow, Widget},
};
use num_format::{Locale, ToFormattedString};

pub struct Dos {}

impl Plugin for Dos {
    fn get_name(&self) -> &'static str {
        "Apple/Dos"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        // has to be a multple of the sector size
        if handle.len() & 0xff != 0 {
            return None;
        }
        // check for 16-sector tracks (most likely)
        if is_dos(handle, 16) {
            return Some(90);
        }
        // check all other sectors
        for sectors in 3..=32 {
            if is_dos(handle, sectors) {
                return Some(80);
            }
        }
        None
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Rc::new(Parser::new(handle));
        let entries = parser.parse()?;
        let mut treegrid = TreeGrid::new(&["Filename", "Type", "Sectors"]);
        for entry in &entries {
            treegrid.add(entry.render(parser.clone()), ctx);
        }
        Ok(Box::new(treegrid))
    }
}

fn is_dos(handle: &Handle, sectors: usize) -> bool {
    // should be divisible by sector count
    if !handle.len().is_multiple_of(sectors) {
        return false;
    }
    // cat track is always 17, so disk must be big enough for 17 tracks
    let catalog = 17 * 256 * sectors;
    if handle.len() < catalog + 256 {
        return false;
    }
    // does it look valid?
    if usize::from(handle.r8(catalog + 0x35)) != sectors {
        return false;
    }
    if handle.r16(catalog + 0x36) != 256 {
        // check # bytes per sector
        return false;
    }
    usize::from(handle.r8(catalog + 0x34)) * sectors * 256 == handle.len()
}

#[derive(Default, Clone)]
struct Entry {
    track: u8,
    sector: u8,
    filetype: u8,
    ext: &'static str,
    name: String,
    length: u16,
}

impl Entry {
    fn render(&self, parser: Rc<Parser>) -> TreeRow {
        let mut row = TreeRow::default();
        let entry = self.clone();
        row.push_column(Link::new(
            &self.name,
            &format!("dos33/{}", entry.filetype),
            move |ctx: &Context| {
                ctx.post(match parser.load_file(&entry) {
                    Ok(data) => Message::Open(Handle::new_from_array(
                        &data,
                        &entry.name,
                        &format!("dos33/{}", entry.filetype),
                    )),
                    Err(err) => Message::Error(err.to_string()),
                });
            },
        ));
        row.push_column(Label::new(self.ext));
        row.push_column(Label::new(&self.length.to_formatted_string(&Locale::en)));
        row
    }
}

struct Parser {
    handle: Handle,
    sec_per_track: usize,
}

impl Parser {
    fn new(handle: &Handle) -> Self {
        let mut sec_per_track = 16;
        if !is_dos(handle, sec_per_track) {
            for sectors in 3..=32 {
                if is_dos(handle, sectors) {
                    sec_per_track = sectors;
                    break;
                }
            }
        }
        Self {
            handle: handle.clone(),
            sec_per_track,
        }
    }

    fn parse(&self) -> Result<Vec<Entry>, &'static str> {
        let base = 17 * self.sec_per_track * 256;
        let mut track = self.handle.r8(base + 1);
        let mut sector = self.handle.r8(base + 2);
        let mut entries = vec![];
        while track != 0 {
            let block = usize::from(track) * self.sec_per_track * 256 + usize::from(sector) * 256;
            if block + 256 > self.handle.len() {
                return Err("catalog block beyond disk");
            }
            track = self.handle.r8(block + 1);
            sector = self.handle.r8(block + 2);
            for i in 0..7 {
                let base = block + 11 + i * 35;
                let fid = self.handle.r8(base);
                if fid != 0xff && fid != 0 {
                    let filetype = self.handle.r8(base + 2) & 0x7f;
                    entries.push(Entry {
                        track: self.handle.r8(base),
                        sector: self.handle.r8(base + 1),
                        filetype,
                        ext: get_file_type(filetype),
                        name: self.handle.rs(base + 3, 30, "iie").trim_end().to_string(),
                        length: self.handle.r16(base + 33),
                    });
                }
            }
        }
        Ok(entries)
    }

    fn load_file(&self, entry: &Entry) -> Result<Vec<u8>, &'static str> {
        let mut track = entry.track;
        let mut sector = entry.sector;
        let mut data = vec![0; usize::from(entry.length * 256)];
        while track != 0 {
            let block =
                usize::from(track) * self.sec_per_track * 256 + usize::from(sector) * 256 + 1;
            if block + 244 > self.handle.len() {
                return Err("invalid file block");
            }
            track = self.handle.r8(block); // next t/s
            sector = self.handle.r8(block + 1);
            let mut offset = usize::from(self.handle.r16(block + 4)) * 0x100;
            let pairs = block + 0xb;
            for i in 0..0x7a {
                let ftrack = self.handle.r8(pairs + i * 2);
                let fsec = self.handle.r8(pairs + i * 2 + 1);
                if ftrack != 0 {
                    let base =
                        usize::from(ftrack) * 256 * self.sec_per_track + usize::from(fsec) * 256;
                    if base + 256 > self.handle.len() {
                        return Err("invalid file pair");
                    }
                    data[offset..offset + 256].copy_from_slice(self.handle.read(base, 256));
                    offset += 256;
                }
            }
        }

        match entry.filetype {
            // text, nil terminated
            0 => {
                if let Some(end) = data.iter().position(|&c| c == 0) {
                    data.resize(end, 0);
                }
            }
            // int or applesoft
            1 | 2 if data.len() > 2 => {
                let len = data
                    .drain(0..2)
                    .enumerate()
                    .map(|(idx, val)| usize::from(val) << (idx * 8))
                    .sum();
                data.resize(len, 0);
            }
            _ => {}
        }

        Ok(data)
    }
}

const fn get_file_type(ft: u8) -> &'static str {
    const FILETYPES: [&str; 9] = ["INT", "BAS", "BIN", "S", "REL", "A", "B", "E", "TXT"];
    FILETYPES[ft.trailing_zeros() as usize]
}
