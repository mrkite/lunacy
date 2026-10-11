/*

Acorn/BBC Micro: UEF tape encoding
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::Plugin,
    widgets::{Label, Link, TreeGrid, TreeRow, Widget},
};

pub struct Uef {}

impl Plugin for Uef {
    fn get_name(&self) -> &'static str {
        "Acorn/UEF"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.len() > 12 && handle.rs(0, 9, "") == "UEF File!" {
            Some(60)
        } else {
            None
        }
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let (data, machine) = parse(handle);

        let mut treegrid = TreeGrid::new(&["UEF", "Machine"]);
        let mut row = TreeRow::default();
        row.push_column(Link::new("View tape", "", move |ctx: &Context| {
            ctx.post(Message::Open(Handle::new_from_array(&data, "tape", "")));
        }));
        row.push_column(Label::new(&machine));
        treegrid.add(row, ctx);

        Ok(Box::new(treegrid))
    }
}

fn parse(handle: &Handle) -> (Vec<u8>, String) {
    let mut tape = vec![];
    let mut machine = String::new();
    let mut ofs = 0xc;
    while ofs < handle.len() {
        let chunkid = handle.r16(ofs);
        let chunklen = handle.r32(ofs + 2) as usize;
        ofs += 6;
        match chunkid {
            // target machine
            5 => {
                let m = handle.r8(ofs);
                machine = MACHINES
                    .get(usize::from(m))
                    .map_or_else(|| format!("${m:x} (unknown)"), ToString::to_string);
            }
            // tape data
            0x100 => tape.extend_from_slice(handle.read(ofs, chunklen)),
            // tape data + parity + stop
            0x104 => tape.extend_from_slice(handle.read(ofs + 3, chunklen - 3)),
            // ignore all others
            _ => {}
        }
        ofs += chunklen;
    }
    (tape, machine)
}

const MACHINES: [&str; 5] = [
    "BBC Model A",
    "Electron",
    "BBC Model B",
    "BBC Master",
    "Atom",
];
