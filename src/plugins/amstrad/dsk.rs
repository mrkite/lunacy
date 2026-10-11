/*

Amstrad CPC : CPM filesystem parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::{Plugin, get_charset},
    widgets::{Label, Link, TreeGrid, TreeRow, Widget},
};
use num_format::{Locale, ToFormattedString};
use std::cmp;

pub struct Dsk {}

impl Plugin for Dsk {
    fn get_name(&self) -> &'static str {
        "Amstrad/DSK"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.len() > 204 {
            let sig = handle.rs(0, 12, "");
            if sig == "MV - CPCEMU " || sig == "EXTENDED CPC" {
                return Some(90);
            }
        }
        None
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let disk = load_cpc(handle)?;
        let mut treegrid = TreeGrid::new(&["Filename", "Ext", "Length"]);
        let entries = parse(&Handle::new_from_array(&disk, "", ""));
        for entry in &entries {
            treegrid.add(entry.render(), ctx);
        }
        Ok(Box::new(treegrid))
    }
}

fn load_cpc(handle: &Handle) -> Result<Vec<u8>, &'static str> {
    if handle.r4(0) == "EXTE" {
        return load_extended(handle);
    }
    let num_tracks = handle.r8(0x30);
    let num_sides = handle.r8(0x31);
    // track size = r16(0x32)
    let mut ofs = 0x100;
    let mut data = vec![];
    for t in 0..num_tracks {
        for side in 0..num_sides {
            let track_ofs = ofs;
            if handle.r8(ofs + 0x10) != t || handle.r8(ofs + 0x11) != side {
                return Err("invalid track header");
            }
            let sector_size = usize::from(handle.r8(ofs + 0x14)) * 256;
            let num_sectors = handle.r8(ofs + 0x15);
            ofs += 0x18;
            for _ in 0..num_sectors {
                if handle.r8(ofs) != t || handle.r8(ofs + 1) != side {
                    return Err("invalid sector header");
                }
                ofs += 8;
            }
            ofs = track_ofs + 0x100;
            for _ in 0..num_sectors {
                data.extend_from_slice(handle.read(ofs, sector_size));
                ofs += sector_size;
            }
        }
    }
    Ok(data)
}

fn load_extended(handle: &Handle) -> Result<Vec<u8>, &'static str> {
    let num_tracks = handle.r8(0x30);
    let num_sides = handle.r8(0x31);
    let mut ofs = 0x100;
    let mut data = vec![];
    for t in 0..num_tracks {
        for side in 0..num_sides {
            let track_ofs = ofs;
            if handle.r8(ofs + 0x10) != t || handle.r8(ofs + 0x11) != side {
                return Err("invalid track header");
            }
            let num_sectors = handle.r8(ofs + 0x15);
            let mut sector_sizes = vec![];
            ofs += 0x18;
            for _ in 0..num_sectors {
                if handle.r8(ofs) != t || handle.r8(ofs + 1) != side {
                    return Err("invalid sector header");
                }
                sector_sizes.push(usize::from(handle.r16(ofs + 6)));
                ofs += 8;
            }
            ofs = track_ofs + 0x100;
            for size in &sector_sizes {
                data.extend_from_slice(handle.read(ofs, *size));
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
    fn render(&self) -> TreeRow {
        let mut row = TreeRow::default();
        let entry = self.clone();
        row.push_column(Link::new(
            &self.name,
            &entry.ext.clone(),
            move |ctx: &Context| {
                ctx.post(Message::Open(Handle::new_from_array(
                    &entry.data,
                    &entry.name,
                    &entry.ext,
                )));
            },
        ));
        row.push_column(Label::new(&self.ext));
        row.push_column(Label::new(
            &self.data.len().to_formatted_string(&Locale::en),
        ));
        row
    }
}

fn parse(handle: &Handle) -> Vec<Entry> {
    let cs = get_charset("");
    let mut entries: Vec<Entry> = vec![];
    for i in 0..64 {
        let ofs = i * 32;
        if handle.r8(ofs) < 32 {
            let name = handle
                .read(ofs + 1, 8)
                .iter()
                .map(|c| cs[usize::from(c & 0x7f)])
                .collect();
            let ext = handle
                .read(ofs + 9, 3)
                .iter()
                .map(|c| cs[usize::from(c & 0x7f)])
                .collect();
            let extent = u16::from(handle.r8(ofs + 0xc) & 0x1f)
                | (u16::from(handle.r8(ofs + 0xe) & 0x3f) << 5);
            let bc = handle.r8(ofs + 0xd);
            let rc = handle.r8(ofs + 0xf);
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
                    let extofs = handle.r8(ofs + 0x10 + j);
                    if extofs == 0 {
                        break;
                    }
                    let clen = cmp::min(0x400, ext_len);
                    let start = extent * 16 * 0x400 + j * 0x400;
                    entries[idx].data[start..start + clen]
                        .copy_from_slice(handle.read(usize::from(extofs) * 0x400, clen));
                    ext_len -= clen;
                }
            }
        }
    }
    entries
}
