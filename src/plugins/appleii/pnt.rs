/*

Apple II : PNT C0/06 parser
@copyright 2026 Sean Kasun

*/

use crate::{lunacy::File, plugins::Plugin, widgets::ImageView};

pub struct Pnt {}

const PALETTE_MAP: [u8; 16] = [
    // pixel 0
    8, 9, 10, 11, // pixel 1
    12, 13, 14, 15, // pixel 2
    0, 1, 2, 3, // pixel 3
    4, 5, 6, 7,
];

impl Plugin for Pnt {
    fn get_name(&self) -> &str {
        "Apple/PNT"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        match file.get_mime() {
            s if s.starts_with("prodos/c0") => Some(90),
            s if s.starts_with("prodos/c1") => Some(90),
            s if s.starts_with("prodos/06") && file.len() == 0x8000 => Some(40),
            _ => None,
        }
    }

    fn create(
        &self,
        file: &File,
        _ctx: crate::lunacy::Context,
    ) -> Result<Box<dyn crate::widgets::Widget>, &'static str> {
        match file.get_mime() {
            "prodos/c0:0000" => Ok(Box::new(load_paintworks(file)?)),
            "prodos/c0:0001" => Ok(Box::new(load_raw(&unpack_bytes(file))?)),
            "prodos/c0:0002" => Ok(Box::new(load_mode2(file)?)),
            "prodos/c1:0000" => {
                if file.len() == 0x8000 {
                    Ok(Box::new(load_raw(file.read(0, 0x8000))?))
                } else {
                    Err("truncated file")
                }
            }
            s if s.starts_with("prodos/06") => Ok(Box::new(load_raw(file.read(0, 0x8000))?)),
            _ => Err("Unsupported type"),
        }
    }
}

fn unpack_bytes(file: &File) -> Vec<u8> {
    let mut mem = vec![];
    // unpack btes
    let mut ofs = 0;
    while ofs < file.len() {
        let tc = file.r8(ofs);
        ofs += 1;
        let count = usize::from(tc & 0x3f) + 1;
        match (tc >> 6) & 3 {
            0 => {
                mem.extend_from_slice(file.read(ofs, count));
                ofs += count;
            }
            1 => {
                let val = file.r8(ofs);
                ofs += 1;
                mem.resize(mem.len() + count, val);
            }
            2 => {
                let val = file.read(ofs, 4);
                ofs += 4;
                for _ in 0..count {
                    mem.extend_from_slice(val);
                }
            }
            3 => {
                let val = file.r8(ofs);
                ofs += 1;
                mem.resize(mem.len() + count * 4, val);
            }
            _ => (),
        }
    }
    mem
}

fn load_paintworks(file: &File) -> Result<ImageView, &'static str> {
    if file.len() < 0x222 {
        return Err("truncated file");
    }
    let palette = file.read(0, 32);
    let mem = unpack_bytes(&File::new_from_array(
        file.read(0x222, file.len() - 0x222),
        "",
        "",
    ));
    let mut out = vec![];
    let height = mem.len() / 160;
    for y in 0..height {
        let mut ofs = y * 160;
        for x in 0..320 {
            let pix = mem[ofs];
            let pix = pix >> (4 * (1 - (x & 1)));
            if x & 1 == 1 {
                ofs += 1;
            }
            let palofs = usize::from(pix & 0xf) * 2;
            let b = (palette[palofs] & 0xf) * 0x11;
            let g = (palette[palofs] >> 4) * 0x11;
            let r = (palette[palofs + 1] & 0xf) * 0x11;
            out.push(r);
            out.push(g);
            out.push(b);
            out.push(0xff);
        }
    }
    Ok(ImageView::new(&out, 320, height))
}

fn load_mode2(file: &File) -> Result<ImageView, &'static str> {
    let mut ofs = 0;
    let mut palettes = vec![];
    let mut dir_entries = vec![];
    let mut width = 0;
    let mut height = 0;
    let mut scanlines = vec![];
    let mut master = 0;
    let mut multipal = false;
    let mut num_palettes = 0;

    while ofs < file.len() {
        let len = file.r32(ofs);
        let next_chunk = ofs + len as usize;
        ofs += 4;
        let kindlen = file.r8(ofs);
        let kind = file.rps(ofs, "");
        ofs += usize::from(kindlen) + 1;
        if kind == "MAIN" {
            master = file.r16(ofs);
            ofs += 2;
            width = usize::from(file.r16(ofs));
            ofs += 2;
            num_palettes = usize::from(file.r16(ofs));
            ofs += 2;
            if num_palettes > 0 {
                let pallen = num_palettes * 32;
                palettes.extend_from_slice(file.read(ofs, pallen));
                ofs += pallen;
            }
            height = usize::from(file.r16(ofs));
            ofs += 2;
            for _ in 0..height * 2 {
                dir_entries.push(file.r16(ofs));
                ofs += 2;
            }
            for y in 0..height {
                let inlen = usize::from(dir_entries[y * 2]);
                let line = file.read(ofs, inlen);
                ofs += inlen;
                scanlines.push(unpack_bytes(&File::new_from_array(line, "", "")));
            }
        } else if kind == "MULTIPAL" {
            num_palettes = usize::from(file.r16(ofs));
            ofs += 2;
            palettes.clear();
            palettes.extend_from_slice(file.read(ofs, num_palettes * 32));
            multipal = true;
        }
        ofs = next_chunk;
    }

    let mut out = vec![];
    for y in 0..height {
        let de = dir_entries[y * 2 + 1];
        let scb = if de & 0xff00 != 0 {
            usize::from(de & 0xff)
        } else {
            usize::from(master & 0xff)
        };
        let mut ofs = 0;
        for x in 0..width {
            let pix = scanlines[y][ofs];
            let idx = if scb & 0x80 != 0 {
                // 640 mode
                let pix = pix >> (2 * (3 - (x & 3)));
                if (x & 3) == 3 {
                    ofs += 1;
                }
                PALETTE_MAP[(x & 3) * 4 + usize::from(pix & 3)]
            } else {
                // 320 mode
                let pix = pix >> (4 * (1 - (x & 1)));
                if (x & 1) == 1 {
                    ofs += 1;
                }
                pix & 0xf
            };
            let palofs = if multipal {
                y * 32 + usize::from(idx) * 2
            } else {
                ((scb & 0xf) % num_palettes) * 32 + usize::from(idx) * 2
            };
            let b = (palettes[palofs] & 0xf) * 0x11;
            let g = (palettes[palofs] >> 4) * 0x11;
            let r = (palettes[palofs + 1] & 0xf) * 0x11;
            out.push(r);
            out.push(g);
            out.push(b);
            out.push(0xff);
        }
    }

    Ok(ImageView::new(&out, width, height))
}

fn load_raw(mem: &[u8]) -> Result<ImageView, &'static str> {
    // pixel data is 0-7d00
    // scbs are from 7d00 to 7dc8
    // palettes are from 7e00 to 8000

    let mut out = vec![];

    // we'll create a 640x400 image to handle any 640 mode scanlines
    for y in 0..200 {
        let scb = mem[0x7d00 + y];
        let mut ofs = y * 160;
        for x in 0..640 {
            let pix = mem[ofs];
            let idx = if scb & 0x80 != 0 {
                // 640 mode
                let pix = pix >> (2 * (3 - (x & 3)));
                PALETTE_MAP[(x & 3) * 4 + usize::from(pix & 3)]
            } else {
                // 320 mode
                let pix = pix >> (2 * (2 - (x & 2)));
                pix & 0xf
            };
            if (x & 3) == 3 {
                ofs += 1;
            }
            let palofs = 0x7e00 + usize::from(scb & 0xf) * 32 + usize::from(idx) * 2;
            let b = (mem[palofs] & 0xf) * 0x11;
            let g = (mem[palofs] >> 4) * 0x11;
            let r = (mem[palofs + 1] & 0xf) * 0x11;
            out.push(r);
            out.push(g);
            out.push(b);
            out.push(0xff);
        }
    }

    Ok(ImageView::new(&out, 640, 200))
}
