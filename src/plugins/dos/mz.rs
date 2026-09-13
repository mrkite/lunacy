/*

Dos : 16-bit MZ EXE
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{CpuState, State86},
    lunacy::File,
    platforms::Dos,
    plugins::Plugin,
    widgets::Widget,
};

pub struct Mz {}

impl Plugin for Mz {
    fn get_name(&self) -> &str {
        "DOS/MZ"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() > 28 && file.r16(0) == 0x5a4d {
            Some(80)
        } else {
            None
        }
    }

    fn create(
        &self,
        file: &File,
        ctx: crate::lunacy::Context,
    ) -> Result<Box<dyn crate::widgets::Widget>, &'static str> {
        let mut dos = Dos::new(&format!("mz_{}", file.get_name()));
        let parser = Parser::new(file);
        let (header, segments) = parser.parse()?;
        dos.set_mz(header);
        for seg in segments.iter() {
            dos.add_section("", u32::from(seg.seg) << 4, &seg.data);
        }

        let mut top = None;
        for seg in segments.iter() {
            if let Some(entry) = seg.entry {
                let state = State86 {
                    seg: seg.seg,
                    ofs: entry,
                };
                if top.is_none() {
                    top = Some(state.address());
                }
                dos.add_entry(state)?;
            }
        }
        if let Some(top) = top {
            dos.jump(top as usize, ctx);
        }
        Ok(Box::new(dos))
    }
}

struct Parser {
    file: File,
}

type KVList = Vec<(String, String)>;

impl Parser {
    fn new(file: &File) -> Self {
        Self { file: file.clone() }
    }

    fn parse(&self) -> Result<(KVList, Vec<Segment>), &'static str> {
        let base = 0x1000;
        let header = Header::new(&self.file);

        let mut len = usize::from(header.num_pages) * 512;
        if header.extra_bytes != 0 {
            len -= 512 - usize::from(header.extra_bytes);
        }
        let img_start = usize::from(header.header_size) * 16;
        len -= img_start;

        let mut image = self.file.read(img_start, len).to_vec();

        let mut fofs = usize::from(header.relocations);
        for _ in 0..header.num_relocation {
            let ofs = self.file.r16(fofs);
            fofs += 2;
            let seg = self.file.r16(fofs);
            fofs += 2;
            image[((usize::from(seg) << 4) + usize::from(ofs)) + 1] += (base >> 8) as u8;
        }
        let entry = (usize::from(header.initial_cs) << 4) + usize::from(header.initial_ip);
        let mut segments = vec![];
        let seg = Segment {
            seg: base,
            data: image,
            entry: if entry < len {
                Some(entry as u16)
            } else {
                None
            },
        };
        segments.push(seg);
        Ok((header.to_list(), segments))
    }
}

struct Header {
    signature: u16,
    extra_bytes: u16,
    num_pages: u16,
    num_relocation: u16,
    header_size: u16,
    min_alloc: u16,
    max_alloc: u16,
    initial_ss: u16,
    initial_sp: u16,
    checksum: u16,
    initial_ip: u16,
    initial_cs: u16,
    relocations: u16,
    overlay: u16,
}

impl Header {
    pub fn new(file: &File) -> Self {
        Self {
            signature: file.r16(0),
            extra_bytes: file.r16(2),
            num_pages: file.r16(4),
            num_relocation: file.r16(6),
            header_size: file.r16(8),
            min_alloc: file.r16(0xa),
            max_alloc: file.r16(0xc),
            initial_ss: file.r16(0xe),
            initial_sp: file.r16(0x10),
            checksum: file.r16(0x12),
            initial_ip: file.r16(0x14),
            initial_cs: file.r16(0x16),
            relocations: file.r16(0x18),
            overlay: file.r16(0x1a),
        }
    }

    pub fn to_list(&self) -> Vec<(String, String)> {
        let mut l = vec![];
        l.push(("Signature".to_owned(), format!("${:04x}", self.signature)));
        l.push((
            "Extra Bytes".to_owned(),
            format!("${:04x}", self.extra_bytes),
        ));
        l.push(("Num Pages".to_owned(), format!("${:04x}", self.num_pages)));
        l.push((
            "Num Relocations".to_owned(),
            format!("${:04x}", self.num_relocation),
        ));
        l.push((
            "Header Size".to_owned(),
            format!("${:04x}", self.header_size),
        ));
        l.push(("Min Alloc".to_owned(), format!("${:04x}", self.min_alloc)));
        l.push(("Max Alloc".to_owned(), format!("${:04x}", self.max_alloc)));
        l.push(("Initial SS".to_owned(), format!("${:04x}", self.initial_ss)));
        l.push(("Initial SP".to_owned(), format!("${:04x}", self.initial_sp)));
        l.push(("Checksum".to_owned(), format!("${:04x}", self.checksum)));
        l.push(("Initial IP".to_owned(), format!("${:04x}", self.initial_ip)));
        l.push(("Initial CS".to_owned(), format!("${:04x}", self.initial_cs)));
        l.push((
            "Relocations".to_owned(),
            format!("${:04x}", self.relocations),
        ));
        l.push(("Overlay".to_owned(), format!("${:04x}", self.overlay)));
        l
    }
}

#[derive(Default)]
struct Segment {
    seg: u16,
    entry: Option<u16>,
    data: Vec<u8>,
}
