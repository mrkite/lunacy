/*

68k Mac : ICON and ICN# parser
@copyright 2026 Sean Kasun

*/

use crate::{lunacy::File, plugins::Plugin, widgets::ImageView};

pub struct Icon {}

impl Plugin for Icon {
    fn get_name(&self) -> &str {
        "Mac/ICON"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.get_mime() == "mac/res/ICON" || file.get_mime() == "mac/res/ICN#" {
            Some(90)
        } else {
            None
        }
    }

    fn create(
        &self,
        file: &File,
        _ctx: crate::lunacy::Context,
    ) -> Result<Box<dyn crate::widgets::Widget>, &'static str> {
        let has_mask = file.get_mime() == "mac/res/ICN#";
        Ok(Box::new(parse_icon(file, has_mask)?))
    }
}

fn parse_icon(file: &File, has_mask: bool) -> Result<ImageView, &'static str> {
    let mut pixels = vec![];

    if (file.len() < 256 && has_mask) || (file.len() < 128 && !has_mask) {
        return Err("icon truncated");
    }
    let data = file.read(0, 128);
    let mask = if has_mask {
        file.read(128, 128)
    } else {
        &vec![0xff; 128]
    };
    let mut ofs = 0;
    let mut bit = 0;
    for _ in 0..32 {
        for _ in 0..32 {
            if mask[ofs] & (0x80 >> bit) != 0 {
                if data[ofs] & (0x80 >> bit) != 0 {
                    pixels.push(0);
                    pixels.push(0);
                    pixels.push(0);
                    pixels.push(0xff);
                } else {
                    pixels.push(0xff);
                    pixels.push(0xff);
                    pixels.push(0xff);
                    pixels.push(0xff);
                }
            } else {
                pixels.push(0);
                pixels.push(0);
                pixels.push(0);
                pixels.push(0);
            }
            bit += 1;
            if bit == 8 {
                bit = 0;
                ofs += 1;
            }
        }
    }
    Ok(ImageView::new(&pixels, 32, 32))
}
