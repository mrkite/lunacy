/*

68k Mac : Code parser
@copyright 2026 Sean Kasun

*/

use std::collections::HashMap;

use crate::{
    cpus::State68k,
    lunacy::File,
    platforms::Mac,
    plugins::{Plugin, mac::resparser::Parser},
    widgets::Widget,
};

pub struct Code {}

impl Plugin for Code {
    fn get_name(&self) -> &str {
        "Mac/Code"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.get_mime() == "mac/res" && has_code(file) {
            Some(60) // less than mac/res
        } else {
            None
        }
    }

    fn create(
        &self,
        file: &File,
        ctx: crate::lunacy::Context,
    ) -> Result<Box<dyn crate::widgets::Widget>, &'static str> {
        let mut mac = Mac::new(&format!("code_{}", file.get_name()));
        let parser = Parser::new(file);
        let entries = parser.parse()?;
        let mut a5 = HashMap::new();
        let mut first_entry = None;
        if let Some(codes) = entries.get("CODE") {
            for entry in codes.iter() {
                let c = parser.get_res("", &entry.res_type, entry.offset, entry.attr)?;
                let name = format!("CODE {}{}", entry.id, entry.name);
                mac.add_section(&name, u32::from(entry.id) << 16, c.read(0, c.len()));
            }
            if let Some(entry) = codes.first() {
                let c = parser.get_res("", &entry.res_type, entry.offset, entry.attr)?;
                let mut ofs = 8;
                let jump_len = c.r32be(ofs);
                ofs += 4;
                let jump_ofs = c.r32be(ofs);
                ofs += 4;
                for i in (0..jump_len).step_by(8) {
                    let mut dest = u32::from(c.r16be(ofs));
                    ofs += 4; // plus unknown
                    dest |= u32::from(c.r16be(ofs)) << 16;
                    ofs += 2;
                    let trap = c.r16be(ofs);
                    ofs += 2;
                    if trap == 0xa9f0 {
                        a5.insert(jump_ofs + i + 2, dest);
                        if first_entry.is_none() {
                            first_entry = Some(dest);
                        }
                    }
                }
            }
        }
        let entry = first_entry.unwrap_or(0x10000);
        mac.add_entry(State68k { addr: entry })?;
        mac.set_a5(&a5);
        mac.jump(entry as usize, ctx);

        Ok(Box::new(mac))
    }
}

fn has_code(file: &File) -> bool {
    let mapofs = file.r32be(4) as usize;
    let typelist = mapofs + usize::from(file.r16be(mapofs + 24));
    let num_types = file.r16be(mapofs + 28);
    for t in 0..=num_types {
        let ofs = typelist + 2 + usize::from(t) * 8;
        let res_type = file.rs(ofs, 4, "macroman");
        if res_type == "CODE" {
            return true;
        }
    }
    false
}
