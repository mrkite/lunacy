/*

Dos : 16-bit MZ EXE
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{CpuState, State86},
    lunacy::{Context, Handle},
    platforms::Dos,
    plugins::Plugin,
    widgets::Widget,
};

pub struct Mz {}

impl Plugin for Mz {
    fn get_name(&self) -> &'static str {
        "DOS/MZ"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.len() > 28 && handle.r16(0) == 0x5a4d {
            Some(80)
        } else {
            None
        }
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let mut dos = Dos::new(handle.get_name());
        let parser = Parser::new(handle);
        let top = parser.parse(&mut dos)?;
        dos.disassemble();
        if let Some(top) = top
            && let Ok(addr) = usize::try_from(top)
        {
            dos.jump(addr, ctx);
        }
        Ok(Box::new(dos))
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

    fn parse(&self, dos: &mut Dos) -> Result<Option<u32>, &'static str> {
        let base = 0x1000;
        let header = Header::new(&self.handle);
        dos.set_mz(header.to_list());
        dos.add_segment(base);
        dos.add_segment(header.initial_ss + base);
        dos.add_segment(header.initial_cs + base);

        let mut len = usize::from(header.num_pages) * 512;
        if header.extra_bytes != 0 {
            len -= 512 - usize::from(header.extra_bytes);
        }
        let img_start = usize::from(header.size) * 16;
        len -= img_start;

        let mut image = self.handle.read(img_start, len).to_vec();

        let mut fofs = usize::from(header.relocations);
        for _ in 0..header.num_relocation {
            let ofs = self.handle.r16(fofs);
            fofs += 2;
            let seg = self.handle.r16(fofs);
            fofs += 2;
            dos.add_segment(seg + base);
            image[((usize::from(seg) << 4) + usize::from(ofs)) + 1] += (base >> 8) as u8;
        }
        let entry = (usize::from(header.initial_cs) << 4) + usize::from(header.initial_ip);
        let mut top = None;
        dos.add_section("code", u32::from(base) << 4, &image);
        if entry < len {
            let state = State86 {
                seg: header.initial_cs + base,
                ofs: header.initial_ip,
            };
            top = Some(state.address());
            dos.add_entry(state)?;
        }
        Ok(top)
    }
}

struct Header {
    signature: u16,
    extra_bytes: u16,
    num_pages: u16,
    num_relocation: u16,
    size: u16,
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
    pub fn new(handle: &Handle) -> Self {
        Self {
            signature: handle.r16(0),
            extra_bytes: handle.r16(2),
            num_pages: handle.r16(4),
            num_relocation: handle.r16(6),
            size: handle.r16(8),
            min_alloc: handle.r16(0xa),
            max_alloc: handle.r16(0xc),
            initial_ss: handle.r16(0xe),
            initial_sp: handle.r16(0x10),
            checksum: handle.r16(0x12),
            initial_ip: handle.r16(0x14),
            initial_cs: handle.r16(0x16),
            relocations: handle.r16(0x18),
            overlay: handle.r16(0x1a),
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
        l.push(("Header Size".to_owned(), format!("${:04x}", self.size)));
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
