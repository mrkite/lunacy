/*

Acorn/BBC Micro: UEF tape encoding
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};

pub struct Uef {}

impl Plugin for Uef {
    fn get_name(&self) -> &str {
        "Acorn/UEF"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() > 12 && file.rs(0, 9, "") == "UEF File!" {
            Some(60)
        } else {
            None
        }
    }

    fn create(
        &self,
        file: &File,
        ctx: Context,
    ) -> Result<Box<dyn crate::plugins::Widget>, &'static str> {
        let parser = Parser::default();
        let (data, machine) = parser.parse(file)?;

        let mut treegrid = TreeGrid::new(&["UEF", "Machine"], "uef", move |_| {
            ctx.post(Message::Open(File::new_from_array(&data, "tape", "")));
        });
        let mut row = TreeRow::new(());
        row.push_column("View tape");
        row.push_column(&machine);
        treegrid.add(row);

        Ok(Box::new(treegrid))
    }
}

#[derive(Default)]
struct Parser {}

impl Parser {
    fn parse(&self, file: &File) -> Result<(Vec<u8>, String), &'static str> {
        let mut tape = vec![];
        let mut machine = String::new();
        let mut ofs = 0xc;
        while ofs < file.len() {
            let chunkid = file.r16(ofs);
            let chunklen = file.r32(ofs + 2) as usize;
            ofs += 6;
            match chunkid {
                // target machine
                5 => {
                    let m = file.r8(ofs);
                    machine = if let Some(mach) = MACHINES.get(usize::from(m)) {
                        mach.to_string()
                    } else {
                        format!("${:x} (unknown)", m)
                    };
                }
                // tape data
                0x100 => tape.extend_from_slice(file.read(ofs, chunklen)),
                // tape data + parity + stop
                0x104 => tape.extend_from_slice(file.read(ofs + 3, chunklen - 3)),
                // ignore all others
                _ => {}
            }
            ofs += chunklen;
        }
        Ok((tape, machine))
    }
}

const MACHINES: [&str; 5] = [
    "BBC Model A",
    "Electron",
    "BBC Model B",
    "BBC Master",
    "Atom",
];
