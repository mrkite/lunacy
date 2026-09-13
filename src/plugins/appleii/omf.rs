/*

Apple IIgs : OMF executable or library

*/

use crate::{
    cpus::State65816,
    lunacy::{Context, File},
    platforms::IIgs,
    plugins::Plugin,
    widgets::Widget,
};
use std::collections::HashMap;

pub struct Omf {}

impl Plugin for Omf {
    fn get_name(&self) -> &str {
        "Apple/OMF"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        let mime = file.get_mime();
        if mime.starts_with("prodos/b3:") || mime.starts_with("prodos/ba:") {
            Some(90)
        } else {
            None
        }
    }

    fn create(&self, file: &File, ctx: Context) -> Result<Box<dyn Widget>, &'static str> {
        let mut iigs = IIgs::new(&format!("omf_{}", file.get_name()));
        let parser = Parser::new(file);
        let segments = parser.parse()?;
        for seg in segments.iter() {
            iigs.add_section(&seg.name, seg.mapped, &seg.data);
        }
        let mut top = 0;
        for seg in segments.iter() {
            if seg.kind & 0x1f == 0 {
                if top == 0 {
                    top = seg.mapped + seg.entry;
                }
                // code
                iigs.add_entry(State65816 {
                    addr: seg.mapped + seg.entry,
                    ..Default::default()
                })?;
            }
        }
        iigs.jump(top as usize, ctx);
        Ok(Box::new(iigs))
    }
}

mod seg_op {
    pub const DONE: u8 = 0x00;
    pub const RELOC: u8 = 0xe2;
    pub const INTERSEG: u8 = 0xe3;
    pub const DS: u8 = 0xf1;
    pub const LCONST: u8 = 0xf2;
    pub const CRELOC: u8 = 0xf5;
    pub const CINTERSEG: u8 = 0xf6;
    pub const SUPER: u8 = 0xf7;
}

#[derive(Default, Ord, PartialOrd, Eq, PartialEq)]
struct Segment {
    mapped: u32, // first for sorting
    name: String,
    kind: u16,
    bytecnt: u32,
    resspc: u32,
    length: u32,
    lablen: u8,
    numlen: u8,
    banksize: u32,
    offset: usize,
    org: u32,
    align: u32,
    segnum: u16,
    entry: u32,
    data: Vec<u8>,
}

impl Segment {
    fn is_dps(&self) -> bool {
        self.kind & 0x1f == 0x12
    }

    fn is_jump(&self) -> bool {
        self.kind & 0x1f == 0x02
    }

    fn init_dps(&mut self) {
        self.kind = 0x12;
        self.length = 1024;
        self.resspc = 1024;
        self.bytecnt = 0;
        self.name = "Direct".to_string();
    }

    fn map(&mut self, memory: &mut Vec<u32>, force: bool) -> Result<(), &'static str> {
        if self.mapped != 0 {
            return Ok(());
        }
        if self.org != 0 && (self.org & 0xffff != 0 || self.kind & 0x800 == 0) {
            self.mapped = self.org;
            // verify we're not overlapping
            for m in memory.iter() {
                if self.mapped < *m && self.mapped + self.length > *m {
                    return Err("Segment collision");
                }
            }
            memory.push(self.mapped);
            memory.push(self.mapped + self.length);
            if self.mapped < memory[0] {
                // below the min?
                memory[0] = self.mapped;
            }
            return Ok(());
        }
        if force {
            for pair in memory.chunks(2) {
                let base = pair[0];
                // skip special?
                let base = if (self.kind & 0x1f) != 0x12 && (base & 0xff0000) == 0 {
                    base + 0x10000 // not in bank 0
                } else {
                    base
                };
                // are we aligned?
                let base = if self.align != 0 && (base & (self.align - 1)) != 0 {
                    (base + self.align) & !(self.align - 1)
                } else {
                    base
                };
                // does it cross bank boundaries?
                let base = if self.banksize != 0
                    && (((base & (self.banksize - 1)) + self.length) & !(self.banksize - 1)) != 0
                {
                    (base + self.banksize) & !(self.banksize - 1)
                } else {
                    base
                };
                let end = pair[1];
                if base < end && end - base >= self.length {
                    self.mapped = base;
                    break;
                }
            }
            if self.mapped != 0 {
                memory.push(self.mapped);
                memory.push(self.mapped + self.length);
                Ok(())
            } else {
                Err("Segment doesn't fit in memory")
            }
        } else {
            Ok(())
        }
    }
}

struct Parser {
    file: File,
}

impl Parser {
    fn new(file: &File) -> Self {
        Self { file: file.clone() }
    }

    fn parse(&self) -> Result<Vec<Segment>, &'static str> {
        let mut segments = self.load_segments()?;
        self.map_segments(&mut segments)?;
        self.reloc_segments(&mut segments)?;
        segments.sort();
        Ok(segments)
    }

    fn load_segments(&self) -> Result<Vec<Segment>, &'static str> {
        let mut ofs = 0;
        let mut segments = vec![];
        while ofs < self.file.len() {
            let mut seg = Segment {
                bytecnt: self.file.r32(ofs),
                resspc: self.file.r32(ofs + 4),
                length: self.file.r32(ofs + 8),
                lablen: self.file.r8(ofs + 0xd),
                numlen: self.file.r8(ofs + 0xe),
                banksize: self.file.r32(ofs + 0x10),
                kind: self.file.r16(ofs + 0x14),
                // unknown word at +16
                org: self.file.r32(ofs + 0x18),
                align: self.file.r32(ofs + 0x1c),
                // byte ordering at +20
                segnum: self.file.r16(ofs + 0x22),
                entry: self.file.r32(ofs + 0x24),
                ..Default::default()
            };
            let kind = u16::from(self.file.r8(ofs + 0xc));
            let version = self.file.r8(ofs + 0xf);
            let dispname = self.file.r16(ofs + 0x28);
            let dispdata = self.file.r16(ofs + 0x2a);
            let skip = if seg.lablen == 0 {
                seg.lablen = self.file.r8(ofs + usize::from(dispname) + 0xa);
                if seg.lablen == 0 {
                    seg.lablen = 10;
                    0
                } else {
                    11
                }
            } else {
                10
            };

            // there are 2 names, one is always 10 bytes at dispname
            // this is the "loadname" and it specifies the name by the linker
            // it is followed by the segname, which is the actual segment name.

            // first check if loadname is valid
            seg.name = self.file.rs(
                ofs + usize::from(dispname) + skip,
                usize::from(seg.lablen),
                "",
            );
            seg.offset = ofs + usize::from(dispdata);
            if version == 1 {
                // convert to v2
                seg.bytecnt *= 512;
                seg.kind = (kind & 0xf1f) | ((kind & 0xe0) << 8);
            }
            seg.mapped = 0;
            ofs += seg.bytecnt as usize;
            segments.push(seg);
        }
        Ok(segments)
    }

    fn map_segments(&self, segments: &mut Vec<Segment>) -> Result<(), &'static str> {
        // use a memory map that denotes runs of available ram
        let mut memory = vec![
            0x300,    // min
            0x7f0000, // max
        ];
        // first map any segments that know where they belong
        for seg in segments.iter_mut() {
            seg.map(&mut memory, false)?;
            memory.sort();
        }

        // now map everything else by first fit
        for seg in segments.iter_mut() {
            seg.map(&mut memory, true)?;
            memory.sort();
        }

        // create a direct page if it doesn't exist
        let mut found = false;
        for seg in segments.iter() {
            if seg.is_dps() {
                found = true;
            }
        }
        if !found {
            let mut seg: Segment = Default::default();
            seg.init_dps();
            seg.map(&mut memory, true)?;
            segments.push(seg);
        }

        Ok(())
    }

    fn reloc_segments(&self, segments: &mut [Segment]) -> Result<(), &'static str> {
        let mut seg_map: HashMap<u16, u32> = HashMap::new();
        for seg in segments.iter() {
            seg_map.insert(seg.segnum, seg.mapped);
        }

        for seg in segments.iter_mut() {
            let mut pc = seg.mapped;
            let mut data = vec![0; seg.length as usize];
            let mut ofs = seg.offset;
            let mut done = false;
            while !done {
                let opcode = self.file.r8(ofs);
                ofs += 1;
                match opcode {
                    seg_op::DONE => done = true,
                    seg_op::RELOC => {
                        let num_bytes = self.file.r8(ofs);
                        let bit_shift = self.file.r8(ofs + 1) as i8;
                        let offset = self.file.r32(ofs + 2) as usize;
                        let sub_offset = self.file.r32(ofs + 6) + seg.mapped;
                        ofs += 10;
                        let sub_offset = if bit_shift < 0 {
                            sub_offset >> -bit_shift
                        } else {
                            sub_offset << bit_shift
                        };
                        patch(&mut data, offset, num_bytes, sub_offset);
                    }
                    seg_op::INTERSEG => {
                        let num_bytes = self.file.r8(ofs);
                        let bit_shift = self.file.r8(ofs + 1) as i8;
                        let offset = self.file.r32(ofs + 2) as usize;
                        // filenum
                        let segnum = self.file.r16(ofs + 8);
                        let sub_offset = self.file.r32(ofs + 10);
                        ofs += 14;
                        let sub_offset = if let Some(remap) = seg_map.get(&segnum) {
                            sub_offset + remap
                        } else {
                            sub_offset
                        };
                        let sub_offset = if bit_shift < 0 {
                            sub_offset >> -bit_shift
                        } else {
                            sub_offset << bit_shift
                        };
                        patch(&mut data, offset, num_bytes, sub_offset);
                    }
                    seg_op::DS => {
                        pc += self.file.r32(ofs);
                        ofs += 4;
                    }
                    seg_op::LCONST => {
                        let count = self.file.r32(ofs) as usize;
                        ofs += 4;
                        let v = self.file.read(ofs, count);
                        ofs += count;
                        let start = (pc - seg.mapped) as usize;
                        data[start..start + count].copy_from_slice(v);
                        pc += count as u32;
                    }
                    seg_op::CRELOC => {
                        let num_bytes = self.file.r8(ofs);
                        let bit_shift = self.file.r8(ofs + 1) as i8;
                        let offset = usize::from(self.file.r16(ofs + 2));
                        let sub_offset = u32::from(self.file.r16(ofs + 4)) + seg.mapped;
                        ofs += 6;
                        let sub_offset = if bit_shift < 0 {
                            sub_offset >> -bit_shift
                        } else {
                            sub_offset << bit_shift
                        };
                        patch(&mut data, offset, num_bytes, sub_offset);
                    }
                    seg_op::CINTERSEG => {
                        let num_bytes = self.file.r8(ofs);
                        let bit_shift = self.file.r8(ofs + 1) as i8;
                        let offset = usize::from(self.file.r16(ofs + 2));
                        let segnum = u16::from(self.file.r8(ofs + 4));
                        let sub_offset = u32::from(self.file.r16(ofs + 5));
                        ofs += 7;
                        let sub_offset = if let Some(remap) = seg_map.get(&segnum) {
                            sub_offset + remap
                        } else {
                            sub_offset
                        };
                        let sub_offset = if bit_shift < 0 {
                            sub_offset >> -bit_shift
                        } else {
                            sub_offset << bit_shift
                        };
                        patch(&mut data, offset, num_bytes, sub_offset);
                    }
                    seg_op::SUPER => {
                        let super_len = self.file.r32(ofs) as usize + ofs + 4;
                        let super_type = self.file.r8(ofs + 4);
                        ofs += 5;
                        let mut super_page = 0;
                        while ofs < super_len {
                            let num_ofs = self.file.r8(ofs);
                            ofs += 1;
                            if num_ofs & 0x80 != 0 {
                                super_page += 256 * usize::from(num_ofs & 0x7f);
                                continue;
                            }
                            for _ in 0..=num_ofs {
                                let offset = super_page | usize::from(self.file.r8(ofs));
                                ofs += 1;
                                let mut subofs = offset;
                                let mut sub_offset =
                                    u32::from(data[subofs]) | u32::from(data[subofs + 1]) << 8;
                                subofs += 2;
                                let num_bytes = if super_type == 0 || super_type == 1 {
                                    sub_offset += seg.mapped;
                                    super_type + 2
                                } else if super_type < 14 {
                                    // INTERSEG1-12
                                    let segnum = u16::from(data[subofs]);
                                    if let Some(sub) = seg_map.get(&segnum) {
                                        sub_offset += sub;
                                    }
                                    3
                                } else if super_type < 26 {
                                    // INTERSEG13-14
                                    let segnum = u16::from(super_type - 13);
                                    if let Some(sub) = seg_map.get(&segnum) {
                                        sub_offset += sub;
                                    }
                                    2
                                } else {
                                    // INTERSEG25-36
                                    let segnum = u16::from(super_type - 25);
                                    if let Some(sub) = seg_map.get(&segnum) {
                                        sub_offset += sub;
                                    }
                                    sub_offset >>= 16;
                                    2
                                };
                                patch(&mut data, offset, num_bytes, sub_offset);
                            }
                            super_page += 256;
                        }
                    }
                    opcode => {
                        if opcode < 0xe0 {
                            let count = usize::from(opcode);
                            let v = self.file.read(ofs, count);
                            ofs += count;
                            let start = (pc - seg.mapped) as usize;
                            data[start..start + count].copy_from_slice(v);
                        } else {
                            return Err("unknown segment opcode");
                        }
                    }
                }
            }
            if seg.is_jump() {
                // patch jumptable
                for i in (8..seg.length - 14).step_by(14) {
                    let i = i as usize;
                    let segnum = u16::from(data[i + 4]) | u16::from(data[i + 5]) << 8;
                    let sub_offset = u32::from(data[i + 6])
                        | u32::from(data[i + 7]) << 8
                        | u32::from(data[i + 8]) << 16
                        | u32::from(data[i + 9]) << 24;
                    let sub_offset = if let Some(sub) = seg_map.get(&segnum) {
                        sub_offset + sub
                    } else {
                        sub_offset
                    };
                    patch(&mut data, i + 11, 3, sub_offset);
                }
            }
            seg.data = data;
        }
        Ok(())
    }
}

fn patch(data: &mut [u8], offset: usize, num_bytes: u8, value: u32) {
    for i in 0..num_bytes {
        data[offset + usize::from(i)] = ((value >> (i * 8)) & 0xff) as u8;
    }
}
