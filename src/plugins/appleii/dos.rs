/*

Apple II : DOS 3.3 filesystem parser
@copyright 2026 Sean Kasun

*/
use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};

pub struct Dos {}

impl Plugin for Dos {
    fn get_name(&self) -> &str {
        "Apple/Dos"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        // has to be a multple of the sector size
        if file.len() & 0xff != 0 {
            return None;
        }
        // check for 16-sector tracks (most likely)
        if is_dos(file, 16) {
            return Some(90);
        }
        // check all other sectors
        for sectors in 3..=32 {
            if is_dos(file, sectors) {
                return Some(80);
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
            &["Filename", "Type", "Sectors"],
            &format!("dos_{}", file.get_name()),
            move |entry: &Entry| {
                ctx.post(match parser.load_file(entry) {
                    Ok(data) => Message::Open(File::new_from_array(
                        &data,
                        &entry.name,
                        &format!("dos33/{}", entry.filetype),
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

fn is_dos(file: &File, sectors: usize) -> bool {
    // should be divisible by sector count
    if !file.len().is_multiple_of(sectors) {
        return false;
    }
    // cat track is always 17, so disk must be big enough for 17 tracks
    let catalog = 17 * 256 * sectors;
    if file.len() < catalog + 256 {
        return false;
    }
    // does it look valid?
    if usize::from(file.r8(catalog + 0x35)) != sectors {
        return false;
    }
    if file.r16(catalog + 0x36) != 256 {
        // check # bytes per sector
        return false;
    }
    usize::from(file.r8(catalog + 0x34)) * sectors * 256 == file.len()
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
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.name);
        row.push_column(self.ext);
        row.push_column(&self.length.to_formatted_string(&Locale::en));
        row
    }
}

struct Parser {
    file: File,
    sec_per_track: usize,
}

impl Parser {
    fn new(file: &File) -> Self {
        let mut sec_per_track = 16;
        if !is_dos(file, sec_per_track) {
            for sectors in 3..=32 {
                if is_dos(file, sectors) {
                    sec_per_track = sectors;
                    break;
                }
            }
        }
        Self {
            file: file.clone(),
            sec_per_track,
        }
    }

    fn parse(&self) -> Result<Vec<Entry>, &'static str> {
        let base = 17 * self.sec_per_track * 256;
        let mut track = self.file.r8(base + 1);
        let mut sector = self.file.r8(base + 2);
        let mut entries = vec![];
        while track != 0 {
            let block = usize::from(track) * self.sec_per_track * 256 + usize::from(sector) * 256;
            if block + 256 > self.file.len() {
                return Err("catalog block beyond disk");
            }
            track = self.file.r8(block + 1);
            sector = self.file.r8(block + 2);
            for i in 0..7 {
                let base = block + 11 + i * 35;
                let fid = self.file.r8(base);
                if fid != 0xff && fid != 0 {
                    let filetype = self.file.r8(base + 2) & 0x7f;
                    entries.push(Entry {
                        track: self.file.r8(base),
                        sector: self.file.r8(base + 1),
                        filetype,
                        ext: get_file_type(filetype),
                        name: self.file.rs(base + 3, 30, "iie"),
                        length: self.file.r16(base + 33),
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
            if block + 244 > self.file.len() {
                return Err("invalid file block");
            }
            track = self.file.r8(block); // next t/s
            sector = self.file.r8(block + 1);
            let mut offset = usize::from(self.file.r16(block + 4)) * 0x100;
            let pairs = block + 0xb;
            for i in 0..0x7a {
                let ftrack = self.file.r8(pairs + i * 2);
                let fsec = self.file.r8(pairs + i * 2 + 1);
                if ftrack != 0 {
                    let base =
                        usize::from(ftrack) * 256 * self.sec_per_track + usize::from(fsec) * 256;
                    if base + 256 > self.file.len() {
                        return Err("invalid file pair");
                    }
                    data[offset..offset + 256].copy_from_slice(self.file.read(base, 256));
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

fn get_file_type(ft: u8) -> &'static str {
    const FILETYPES: [&str; 9] = ["INT", "BAS", "BIN", "S", "REL", "A", "B", "E", "TXT"];
    FILETYPES[ft.trailing_zeros() as usize]
}
