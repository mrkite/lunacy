/*

Amiga : ILBM parser
@copyright 2026 Sean Kasun

*/

use crate::{lunacy::File, plugins::Plugin, widgets::ImageView};

pub struct Ilbm {}

impl Plugin for Ilbm {
    fn get_name(&self) -> &str {
        "Amiga/ILBM"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() > 12
            && file.r4(0) == "FORM"
            && file.r4(8) == "ILBM"
            && file.r32be(4) as usize == file.len() - 8
        {
            Some(80)
        } else {
            None
        }
    }

    fn create(
        &self,
        file: &File,
        _ctx: crate::lunacy::Context,
    ) -> Result<Box<dyn crate::widgets::Widget>, &'static str> {
        Ok(Box::new(parse(file)?))
    }
}

fn parse(file: &File) -> Result<ImageView, &'static str> {
    let mut ofs = 12;

    let mut width = 0;
    let mut height = 0;
    let mut num_planes = 0;
    let mut mask = 0;
    let mut compression = 0;
    let mut trans_color = 0;
    let mut cmap = vec![];
    let mut pixels = vec![];

    while ofs < file.len() {
        let subtyp = file.r4(ofs);
        let sublen = file.r32be(ofs + 4) as usize;
        ofs += 8;
        match subtyp {
            "BMHD" => {
                width = usize::from(file.r16be(ofs));
                height = usize::from(file.r16be(ofs + 2));
                num_planes = usize::from(file.r8(ofs + 8));
                mask = file.r8(ofs + 9);
                compression = file.r8(ofs + 10);
                trans_color = usize::from(file.r16be(ofs + 12));
            }
            "BODY" => {
                if mask == 1 {
                    num_planes += 1;
                }
                let final_len = width.div_ceil(16) * height * 2 * num_planes;

                let planes = match compression {
                    0 => file.read(ofs, final_len),
                    1 => &decompress(file, ofs, final_len),
                    _ => return Err("unhandled compression"),
                };
                pixels = unplane(planes, width, height, num_planes, mask, trans_color, &cmap)
            }
            "CMAP" => {
                cmap.extend_from_slice(file.read(ofs, sublen));
            }
            _ => {}
        }

        ofs += sublen + (sublen & 1);
    }

    Ok(ImageView::new(&pixels, width, height))
}

fn unplane(
    bitmap: &[u8],
    width: usize,
    height: usize,
    num_planes: usize,
    mask: u8,
    trans_color: usize,
    palette: &[u8],
) -> Vec<u8> {
    let byte_width = width.div_ceil(16) * 2;

    let mask_bit = 1 << num_planes.saturating_sub(1);

    let mut out = vec![];

    for y in 0..height {
        let src = y * num_planes * byte_width;
        for x in 0..byte_width {
            for bit in 0..8 {
                let mut pix = 0;
                for depth in 0..num_planes {
                    let val = (bitmap[src + x + depth * byte_width] & (0x80 >> bit)) >> (7 - bit);
                    pix |= usize::from(val) << depth;
                }
                if x * 8 + bit < width {
                    if (mask == 1 && pix & mask_bit != 0) || (mask >= 2 && pix == trans_color) {
                        // transparent
                        out.push(0);
                        out.push(0);
                        out.push(0);
                        out.push(0);
                    } else {
                        out.push(palette[pix * 3]);
                        out.push(palette[pix * 3 + 1]);
                        out.push(palette[pix * 3 + 2]);
                        out.push(0xff);
                    }
                }
            }
        }
    }

    out
}

fn decompress(file: &File, ofs: usize, final_len: usize) -> Vec<u8> {
    let mut out = vec![];

    let mut ofs = ofs;
    while out.len() < final_len && ofs < file.len() {
        let rle = file.r8(ofs);
        ofs += 1;
        if rle > 128 {
            let val = file.r8(ofs);
            ofs += 1;
            out.resize(out.len() + 257 - usize::from(rle), val);
        } else if rle < 128 {
            let count = usize::from(rle) + 1;
            out.extend_from_slice(file.read(ofs, count));
            ofs += count;
        } else {
            break;
        }
    }

    out
}
