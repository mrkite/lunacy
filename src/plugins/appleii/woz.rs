/*

Apple II : WOZ parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::Plugin,
    widgets::{Link, TreeGrid, TreeRow, Widget},
};
use std::collections::BTreeMap;

const TRACK_LEN: usize = 6656;
const ADDRESS_16: u8 = 0x96;
const ADDRESS_13: u8 = 0xb5;
const DATA: u8 = 0xad;
const BLOCK53: usize = 0x33;
const BLOCK62: usize = 0x56;

pub struct Woz {}

impl Plugin for Woz {
    fn get_name(&self) -> &'static str {
        "Apple/WOZ"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.len() > 12 {
            let sig = handle.rs(0, 4, "");
            if (sig == "WOZ1" || sig == "WOZ2")
                && handle.r32(4) == 0x0a0d_0aff
                && handle.r4(12) == "INFO"
            {
                Some(80)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let data = parse(handle)?;

        let mut treegrid = TreeGrid::new(&["WOZ"]);
        let mut row = TreeRow::default();
        row.push_column(Link::new("View disk", "", move |ctx: &Context| {
            ctx.post(Message::Open(Handle::new_from_array(&data, "woz", "")));
        }));
        treegrid.add(row, ctx);

        Ok(Box::new(treegrid))
    }
}

fn parse(handle: &Handle) -> Result<Vec<u8>, &'static str> {
    // first chunk is always INFO
    let info_size = usize::try_from(handle.r32(16)).unwrap_or_default();
    let mut ofs = 20;
    let info = parse_info(handle, ofs, info_size)?;
    ofs += info_size;
    // second chunk is always TMAP
    if handle.r4(ofs) != "TMAP" {
        return Err("TMAP missing");
    }
    let tmap_size = usize::try_from(handle.r32(ofs + 4)).unwrap_or_default();
    ofs += 8;
    let tmap = parse_tmap(handle, ofs, tmap_size, info.disk_type == 1);
    ofs += tmap_size;

    let is_v1 = handle.r8(3) == 0x31;
    let mut data = vec![];

    while ofs + 8 <= handle.len() {
        let chunk_id = handle.r4(ofs);
        let chunk_size = usize::try_from(handle.r32(ofs + 4)).unwrap_or_default();
        ofs += 8;
        if chunk_id == "TRKS" {
            data = if is_v1 {
                parse_trks_v1(handle, ofs, chunk_size, &tmap)?
            } else {
                parse_trks_v2(handle, ofs, chunk_size, &tmap)?
            };
        }
        ofs += chunk_size;
    }
    Ok(data)
}

#[derive(Default, Debug)]
struct Info {
    disk_type: u8,
    _write_protected: u8,
    _synchronized: u8,
    _cleaned: u8,
    _creator: String,
    // v2
    sides: u8,
    boot_sector_format: u8,
    optimal_bit_timing: u8,
    compatible_hardware: u16,
    required_ram: u16,
    largest_track: u16,
    // v3
    flux_block: u16,
    largest_flux_track: u16,
}

fn parse_info(handle: &Handle, ofs: usize, len: usize) -> Result<Info, &'static str> {
    let info_version = handle.r8(ofs);
    if len < 37 {
        return Err("invalid INFO size");
    }
    let mut info = Info {
        disk_type: handle.r8(ofs + 1),
        _write_protected: handle.r8(ofs + 2),
        _synchronized: handle.r8(ofs + 3),
        _cleaned: handle.r8(ofs + 4),
        _creator: handle.rs(ofs + 5, 32, ""),
        ..Default::default()
    };
    if info_version >= 2 {
        if len < 46 {
            return Err("invalid INFO size");
        }
        info.sides = handle.r8(ofs + 37);
        info.boot_sector_format = handle.r8(ofs + 38);
        info.optimal_bit_timing = handle.r8(ofs + 39);
        info.compatible_hardware = handle.r16(ofs + 40);
        info.required_ram = handle.r16(ofs + 42);
        info.largest_track = handle.r16(ofs + 44);
    }
    if info_version >= 3 {
        if len < 50 {
            return Err("invalid INFO size");
        }
        info.flux_block = handle.r16(ofs + 46);
        info.largest_flux_track = handle.r16(ofs + 48);
    }
    Ok(info)
}

fn parse_tmap(handle: &Handle, ofs: usize, len: usize, is_525: bool) -> Vec<u8> {
    let mut tmap = vec![];
    // we only want every 4th entry, since we don't care about quarter tracks
    if is_525 {
        for i in 0..len {
            let idx = handle.r8(ofs + i);
            if i.trailing_zeros() >= 2 && idx != 0xff {
                tmap.push(idx);
            }
        }
    } else {
        for i in 0..len {
            let idx = handle.r8(ofs + i);
            if idx != 0xff {
                tmap.push(idx);
            }
        }
    }
    tmap
}

fn parse_trks_v1(
    handle: &Handle,
    ofs: usize,
    len: usize,
    tmap: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut tracks = BTreeMap::new();
    for trk in tmap {
        let trk_ofs = ofs + usize::from(*trk) * TRACK_LEN;
        if trk_ofs + TRACK_LEN > ofs + len {
            return Err("track out of bounds");
        }
        let bytes_used = handle.r16(trk_ofs + 6646);
        let bit_count = handle.r16(trk_ofs + 6648);
        tracks.insert(
            *trk,
            load_track(
                *trk,
                handle,
                trk_ofs,
                usize::from(bytes_used),
                usize::from(bit_count),
            )?,
        );
    }
    let mut prev = None;
    let mut data = vec![];
    for track in &tracks {
        let track_len = track.1.len();
        if let Some(prev) = prev
            && prev + 1 != *track.0
        {
            let padlen = usize::from(track.0 - prev) * track_len;
            data.resize(data.len() + padlen, 0);
        }
        data.extend_from_slice(track.1);
        prev = Some(track.0);
    }
    Ok(data)
}

fn parse_trks_v2(
    handle: &Handle,
    ofs: usize,
    len: usize,
    tmap: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let mut tracks = BTreeMap::new();
    for trk in tmap {
        let trk_ofs = ofs + usize::from(*trk) * 8;
        if trk_ofs + 8 > ofs + len {
            return Err("track out of bounds");
        }
        let starting_block = handle.r16(trk_ofs);
        let block_count = handle.r16(trk_ofs + 2);
        let bit_count = handle.r16(trk_ofs + 4);
        tracks.insert(
            *trk,
            load_track(
                *trk,
                handle,
                usize::from(starting_block) * 512,
                usize::from(block_count) * 512,
                usize::from(bit_count),
            )?,
        );
    }
    let mut prev = None;
    let mut data = vec![];
    for track in &tracks {
        let track_len = track.1.len();
        if let Some(prev) = prev
            && prev + 1 != *track.0
        {
            let padlen = usize::from(track.0 - prev) * track_len;
            data.resize(data.len() + padlen, 0);
        }
        data.extend_from_slice(track.1);
        prev = Some(track.0);
    }
    Ok(data)
}

fn load_track(
    track: u8,
    handle: &Handle,
    ofs: usize,
    len: usize,
    bit_count: usize,
) -> Result<Vec<u8>, &'static str> {
    let mut scan = vec![];
    let mut byte: u8 = 0;
    if len < bit_count / 8 {
        return Err("track truncated");
    }
    for bit in 0..bit_count {
        byte <<= 1;
        if handle.r8(ofs + (bit >> 3)) & (0x80 >> (bit & 7)) != 0 {
            byte |= 1;
        }
        if byte & 0x80 != 0 {
            scan.push(byte);
            byte = 0;
        }
    }
    // overlap track by 1/3rd
    scan.extend_from_within(0..scan.len() / 3);
    let address_prologue = if find_prologue(&scan, ADDRESS_16, 0).is_some() {
        ADDRESS_16
    } else if find_prologue(&scan, ADDRESS_13, 0).is_some() {
        ADDRESS_13
    } else {
        return Err("no address prologue");
    };

    let mut sectors = BTreeMap::new();

    // scan for next sector
    let mut last_sector = 0;
    while let Some(next_sector) = find_prologue(&scan, address_prologue, last_sector) {
        let track_id = ((scan[next_sector + 5] << 1) | 1) & scan[next_sector + 6];
        let mut sector = ((scan[next_sector + 7] << 1) | 1) & scan[next_sector + 8];
        if address_prologue == ADDRESS_16 {
            sector = SECTOR_ORDER_16[usize::from(sector)];
        }
        if track_id == track
            && !sectors.contains_key(&sector)
            && let Some(data_sector) = find_prologue(&scan, DATA, next_sector)
            && data_sector - next_sector < 256
        {
            if address_prologue == ADDRESS_13 {
                sectors.insert(sector, decode_53(&scan, data_sector + 3)?);
            } else {
                sectors.insert(sector, decode_62(&scan, data_sector + 3)?);
            }
        }
        last_sector = next_sector + 1;
    }

    let mut data = vec![];
    let mut prev = None;
    for sector in &sectors {
        if let Some(prev) = prev
            && prev + 1 != *sector.0
        {
            let padlen = usize::from(sector.0 - prev) * 256;
            data.resize(data.len() + padlen, 0);
        }
        data.extend_from_slice(sector.1);
        prev = Some(sector.0);
    }

    Ok(data)
}

fn decode_53(scan: &[u8], ofs: usize) -> Result<Vec<u8>, &'static str> {
    let mut five: [u8; 256] = [0; 256];
    let mut three: [u8; 154] = [0; 154];
    let mut ofs = ofs;
    let mut chk = 0;
    for i in (0..three.len()).rev() {
        chk ^= GCR_53[usize::from(scan[ofs])];
        three[i] = chk;
        ofs += 1;
    }
    for bits in &mut five {
        chk ^= GCR_53[usize::from(scan[ofs])];
        *bits = chk << 3;
        ofs += 1;
    }
    chk ^= GCR_53[usize::from(scan[ofs])];
    if chk != 0 {
        return Err("checksum failed");
    }

    let mut sector = vec![];

    for i in (0..BLOCK53).rev() {
        let b0 = three[i];
        let b1 = three[i + BLOCK53];
        let b2 = three[i + BLOCK53 * 2];
        sector.push(five[i] | ((b0 >> 2) & 7));
        sector.push(five[i + BLOCK53] | ((b1 >> 2) & 7));
        sector.push(five[i + BLOCK53 * 2] | ((b2 >> 2) & 7));
        sector.push(five[i + BLOCK53 * 3] | (b0 & 2) << 1 | (b1 & 2) | (b2 & 2) >> 1);
        sector.push(five[i + BLOCK53 * 4] | (b0 & 1) << 2 | (b1 & 1) << 1 | (b2 & 1));
    }
    sector.push(five[255] | (*three.last().unwrap_or(&0) & 7));

    Ok(sector)
}

fn decode_62(scan: &[u8], ofs: usize) -> Result<Vec<u8>, &'static str> {
    let mut two: [u8; BLOCK62 * 3] = [0; BLOCK62 * 3];
    let mut ofs = ofs;
    let mut chk = 0;
    for i in 0..BLOCK62 {
        chk ^= GCR_62[usize::from(scan[ofs])];
        two[i] = (chk & 1) << 1 | (chk & 2) >> 1;
        two[i + BLOCK62] = (chk & 4) >> 1 | (chk & 8) >> 3;
        two[i + BLOCK62 * 2] = ((chk & 0x10) >> 3) | (chk & 0x20) >> 5;
        ofs += 1;
    }
    let mut sector = vec![];
    for bits in two.iter().take(256) {
        chk ^= GCR_62[usize::from(scan[ofs])];
        sector.push((chk << 2) | bits);
        ofs += 1;
    }
    chk ^= GCR_62[usize::from(scan[ofs])];
    if chk != 0 {
        return Err("checksum failed");
    }
    Ok(sector)
}

fn find_prologue(scan: &[u8], prologue: u8, from: usize) -> Option<usize> {
    let needle = [0xd5, 0xaa, prologue];

    for i in from..=(scan.len() - needle.len()) {
        if scan[i..i + needle.len()] == needle[..] {
            return Some(i);
        }
    }
    None
}

const SECTOR_ORDER_16: [u8; 16] = [0, 7, 14, 6, 13, 5, 12, 4, 11, 3, 10, 2, 9, 1, 8, 15];

const GCR_53: [u8; 256] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0xff, 0x01, 0x02, 0x03,
    0xff, 0xff, 0xff, 0xff, 0xff, 0x04, 0x05, 0x06, 0xff, 0xff, 0x07, 0x08, 0xff, 0x09, 0x0a, 0x0b,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0c, 0x0d, 0xff, 0xff, 0x0e, 0x0f, 0xff, 0x10, 0x11, 0x12,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x13, 0x14, 0xff, 0x15, 0x16, 0x17,
    0xff, 0xff, 0xff, 0xff, 0xff, 0x18, 0x19, 0x1a, 0xff, 0xff, 0x1b, 0x1c, 0xff, 0x1d, 0x1e, 0x1f,
];

const GCR_62: [u8; 256] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x01, 0xff, 0xff, 0x02, 0x03, 0xff, 0x04, 0x05, 0x06,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x07, 0x08, 0xff, 0xff, 0xff, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
    0xff, 0xff, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0xff, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x1b, 0xff, 0x1c, 0x1d, 0x1e,
    0xff, 0xff, 0xff, 0x1f, 0xff, 0xff, 0x20, 0x21, 0xff, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
    0xff, 0xff, 0xff, 0xff, 0xff, 0x29, 0x2a, 0x2b, 0xff, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32,
    0xff, 0xff, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0xff, 0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f,
];
