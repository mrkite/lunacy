/*

Amstrad CPC : CPM filesystem parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::{Plugin, get_charset},
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};
use std::cmp;

pub struct Dsk {}

impl Plugin for Dsk {
    fn get_name(&self) -> &str {
        "Amstrad/DSK"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() > 204 {
            let sig = file.rs(0, 12, "");
            if sig == "MV - CPCEMU " || sig == "EXTENDED CPC" {
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
        let disk = load_cpc(file)?;
        let mut treegrid = TreeGrid::new(
            &["Filename", "Ext", "Length"],
            &format!("dsk_{}", file.get_name()),
            move |entry: &Entry| {
                ctx.post(Message::Open(File::new_from_array(
                    &entry.data,
                    &entry.name,
                    &entry.ext,
                )));
            },
        );
        let entries = parse(&File::new_from_array(&disk, "", ""))?;
        for entry in entries.iter() {
            treegrid.add(entry.render());
        }
        Ok(Box::new(treegrid))
    }
}

fn load_cpc(file: &File) -> Result<Vec<u8>, &'static str> {
    if file.r4(0) == "EXTE" {
        return load_extended(file);
    }
    let num_tracks = file.r8(0x30);
    let num_sides = file.r8(0x31);
    // track size = r16(0x32)
    let mut ofs = 0x100;
    let mut data = vec![];
    for t in 0..num_tracks {
        for side in 0..num_sides {
            let track_ofs = ofs;
            if file.r8(ofs + 0x10) != t || file.r8(ofs + 0x11) != side {
                return Err("invalid track header");
            }
            let sector_size = usize::from(file.r8(ofs + 0x14)) * 256;
            let num_sectors = file.r8(ofs + 0x15);
            ofs += 0x18;
            for _ in 0..num_sectors {
                if file.r8(ofs) != t || file.r8(ofs + 1) != side {
                    return Err("invalid sector header");
                }
                ofs += 8;
            }
            ofs = track_ofs + 0x100;
            for _ in 0..num_sectors {
                data.extend_from_slice(file.read(ofs, sector_size));
                ofs += sector_size;
            }
        }
    }
    Ok(data)
}

fn load_extended(file: &File) -> Result<Vec<u8>, &'static str> {
    let num_tracks = file.r8(0x30);
    let num_sides = file.r8(0x31);
    let mut ofs = 0x100;
    let mut data = vec![];
    for t in 0..num_tracks {
        for side in 0..num_sides {
            let track_ofs = ofs;
            if file.r8(ofs + 0x10) != t || file.r8(ofs + 0x11) != side {
                return Err("invalid track header");
            }
            let num_sectors = file.r8(ofs + 0x15);
            let mut sector_sizes = vec![];
            ofs += 0x18;
            for _ in 0..num_sectors {
                if file.r8(ofs) != t || file.r8(ofs + 1) != side {
                    return Err("invalid sector header");
                }
                sector_sizes.push(usize::from(file.r16(ofs + 6)));
                ofs += 8;
            }
            ofs = track_ofs + 0x100;
            for size in sector_sizes.iter() {
                data.extend_from_slice(file.read(ofs, *size));
                ofs += size;
            }
        }
    }
    Ok(data)
}

#[derive(Clone)]
struct Entry {
    name: String,
    ext: String,
    data: Vec<u8>,
}

impl Entry {
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.name);
        row.push_column(&self.ext);
        row.push_column(&self.data.len().to_formatted_string(&Locale::en));
        row
    }
}

fn parse(file: &File) -> Result<Vec<Entry>, &'static str> {
    let cs = get_charset("");
    let mut entries: Vec<Entry> = vec![];
    for i in 0..64 {
        let ofs = i * 32;
        if file.r8(ofs) < 32 {
            let name = file
                .read(ofs + 1, 8)
                .iter()
                .map(|c| cs[usize::from(c & 0x7f)])
                .collect();
            let ext = file
                .read(ofs + 9, 3)
                .iter()
                .map(|c| cs[usize::from(c & 0x7f)])
                .collect();
            let extent =
                u16::from(file.r8(ofs + 0xc) & 0x1f) | (u16::from(file.r8(ofs + 0xe) & 0x3f) << 5);
            let bc = file.r8(ofs + 0xd);
            let rc = file.r8(ofs + 0xf);
            let bc = if bc == 0 { 128 } else { bc };
            let mut ext_len = usize::from(rc) * 128 - usize::from(128 - bc);
            let mut idx = None;
            for (i, e) in entries.iter().enumerate() {
                if e.name == name && e.ext == ext {
                    idx = Some(i);
                    break;
                }
            }
            if idx.is_none() {
                idx = Some(entries.len());
                entries.push(Entry {
                    name,
                    ext,
                    data: vec![],
                });
            }
            if let Some(idx) = idx {
                let extent = usize::from(extent);
                let max = cmp::max(extent * 16 * 0x400 + ext_len, entries[idx].data.len());
                entries[idx].data.resize(max, 0);
                for j in 0..16 {
                    let extofs = file.r8(ofs + 0x10 + j);
                    if extofs == 0 {
                        break;
                    }
                    let clen = cmp::min(0x400, ext_len);
                    let start = extent * 16 * 0x400 + j * 0x400;
                    entries[idx].data[start..start + clen]
                        .copy_from_slice(file.read(usize::from(extofs) * 0x400, clen));
                    ext_len -= clen;
                }
            }
        }
    }
    Ok(entries)
}
