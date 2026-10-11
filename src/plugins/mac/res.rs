/*

68k Mac : STR and STR# parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle},
    plugins::{Plugin, get_charset},
    widgets::{Table, Widget},
};
use num_format::{Locale, ToFormattedString};
use std::{collections::HashMap, sync::LazyLock};

pub struct Res {}

impl Plugin for Res {
    fn get_name(&self) -> &'static str {
        "Mac/Res"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        match handle.get_mime() {
            "mac/res/ALRT" | "mac/res/CNTL" | "mac/res/DITL" | "mac/res/DLOG" | "mac/res/MENU"
            | "mac/res/STR " | "mac/res/STR#" | "mac/res/WIND" => Some(90),
            _ => None,
        }
    }

    fn create(&self, handle: &Handle, _ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        match handle.get_mime() {
            "mac/res/ALRT" => Ok(Box::new(parse_alrt(handle)?)),
            "mac/res/CNTL" => Ok(Box::new(parse_cntl(handle)?)),
            "mac/res/DITL" => Ok(Box::new(parse_ditl(handle)?)),
            "mac/res/DLOG" => Ok(Box::new(parse_wind(handle, true)?)),
            "mac/res/MENU" => Ok(Box::new(parse_menu(handle)?)),
            "mac/res/STR " | "mac/res/STR#" => Ok(Box::new(parse_str(handle))),
            "mac/res/WIND" => Ok(Box::new(parse_wind(handle, false)?)),
            _ => Err("unsupported resource type"),
        }
    }
}

fn parse_str(handle: &Handle) -> Table {
    let (ofs, num) = if handle.get_mime() == "mac/res/STR#" {
        (2, handle.r16be(0))
    } else {
        (0, 1)
    };

    let mut table = Table::default();
    let mut ofs = ofs;
    for i in 0..num {
        table.add(&format!("{}", i + 1), &handle.rps(ofs, "macroman"));
        ofs += usize::from(handle.r8(ofs)) + 1;
    }
    table
}

fn parse_alrt(handle: &Handle) -> Result<Table, &'static str> {
    if handle.len() < 10 {
        return Err("Resource truncated");
    }
    let mut table = Table::default();
    table.add("top", &format!("{}", handle.r16be(0).cast_signed()));
    table.add("left", &format!("{}", handle.r16be(2).cast_signed()));
    table.add("bottom", &format!("{}", handle.r16be(4).cast_signed()));
    table.add("right", &format!("{}", handle.r16be(6).cast_signed()));
    table.add("ditl", &handle.r16be(8).to_formatted_string(&Locale::en));
    Ok(table)
}

fn parse_cntl(handle: &Handle) -> Result<Table, &'static str> {
    if handle.len() < 23 {
        return Err("Resource truncated");
    }
    let mut table = Table::default();
    table.add("top", &format!("{}", handle.r16be(0).cast_signed()));
    table.add("left", &format!("{}", handle.r16be(2).cast_signed()));
    table.add("bottom", &format!("{}", handle.r16be(4).cast_signed()));
    table.add("right", &format!("{}", handle.r16be(6).cast_signed()));
    table.add("value", &format!("{}", handle.r16be(8).cast_signed()));
    table.add("visible", if handle.r8(10) == 0 { "no" } else { "yes" });
    table.add("maxValue", &format!("{}", handle.r16be(12).cast_signed()));
    table.add("minValue", &format!("{}", handle.r16be(14).cast_signed()));
    let cdef = handle.r16be(16);
    table.add(
        "cdef",
        &format!("${:04x} {}", cdef, CDEFS.get(&cdef).unwrap_or(&"")),
    );
    table.add("refcon", &format!("${:08x}", handle.r32be(18)));
    table.add("title", &handle.rps(22, "macroman"));
    Ok(table)
}

fn parse_ditl(handle: &Handle) -> Result<Table, &'static str> {
    if handle.len() < 2 {
        return Err("Resource truncated");
    }
    let num_items = handle.r16be(0);
    let mut ofs = 2;
    let mut item = 0;
    let mut table = Table::default();
    while ofs < handle.len() && item < num_items {
        ofs += 4; // reserved
        table.add("item", &format!("{}", item + 1));
        item += 1;
        table.add("\ttop", &format!("{}", handle.r16be(ofs).cast_signed()));
        ofs += 2;
        table.add("\tleft", &format!("{}", handle.r16be(ofs).cast_signed()));
        ofs += 2;
        table.add("\tbottom", &format!("{}", handle.r16be(ofs).cast_signed()));
        ofs += 2;
        table.add("\tright", &format!("{}", handle.r16be(ofs).cast_signed()));
        ofs += 2;
        let ditltyp = handle.r8(ofs);
        ofs += 1;
        table.add(
            "\ttype",
            &format!(
                "${:04x} {}",
                ditltyp,
                DITL_TYPES.get(&ditltyp).unwrap_or(&"")
            ),
        );
        let titlelen = usize::from(handle.r8(ofs));
        table.add("\ttitle", &handle.rps(ofs, "macroman"));
        ofs += titlelen + 1 + (titlelen & 1);
    }
    Ok(table)
}

fn parse_menu(handle: &Handle) -> Result<Table, &'static str> {
    let cs = get_charset("macroman");
    if handle.len() < 15 {
        return Err("Resource truncated");
    }
    let mut table = Table::default();
    table.add("menuID", &format!("{}", handle.r16be(0)));
    table.add("resourceID", &format!("{}", handle.r16be(6).cast_signed()));
    table.add("placeholder", &format!("{}", handle.r16be(8)));
    let enabled = handle.r32be(10);
    table.add("enabled", if enabled & 1 == 0 { "no" } else { "yes" });
    let titlelen = handle.r8(14);
    table.add("title", &handle.rps(14, "macroman"));
    let mut ofs = 15 + usize::from(titlelen);
    let mut item = 1;
    while ofs < handle.len() && handle.r8(ofs) != 0 {
        table.add("menu item", &format!("{item}"));
        let namelen = usize::from(handle.r8(ofs));
        table.add("\tname", &handle.rps(ofs, "macroman"));
        ofs += namelen + 1;
        let icon = handle.r8(ofs);
        if icon != 0 {
            table.add("\ticon", &format!("{icon}"));
        }
        ofs += 1;
        let key = handle.r8(ofs);
        if key != 0 {
            table.add("\tkey", &format!("{}", cs[usize::from(key)]));
        }
        ofs += 1;
        let mark = handle.r8(ofs);
        if mark != 0 {
            table.add("\tmark", &format!("{}", cs[usize::from(mark)]));
        }
        ofs += 1;
        table.add("\tstyle", &format!("{}", handle.r8(ofs)));
        ofs += 1;
        table.add(
            "\tenabled",
            if item < 32 && enabled & (1 << item) == 0 {
                "no"
            } else {
                "yes"
            },
        );
        item += 1;
    }
    Ok(table)
}

fn parse_wind(handle: &Handle, is_dlog: bool) -> Result<Table, &'static str> {
    let base_len = if is_dlog { 20 } else { 18 };
    if handle.len() < base_len + 1 {
        return Err("Resource truncated");
    }
    let mut table = Table::default();
    table.add("top", &format!("{}", handle.r16be(0).cast_signed()));
    table.add("left", &format!("{}", handle.r16be(2).cast_signed()));
    table.add("bottom", &format!("{}", handle.r16be(4).cast_signed()));
    table.add("right", &format!("{}", handle.r16be(6).cast_signed()));
    let wintyp = handle.r16be(8);
    table.add(
        "type",
        &format!("${:04x} {}", wintyp, WIND_TYPES.get(&wintyp).unwrap_or(&"")),
    );
    table.add("visible", if handle.r16be(10) == 0 { "no" } else { "yes" });
    table.add("closeBox", if handle.r16be(12) == 0 { "no" } else { "yes" });
    table.add("refcon", &format!("${:08x}", handle.r32be(14)));
    if is_dlog {
        table.add("ditl", &handle.r16be(18).to_formatted_string(&Locale::en));
    }
    table.add("title", &handle.rps(base_len, "macroman"));
    Ok(table)
}

static WIND_TYPES: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "document");
    m.insert(1, "dBox");
    m.insert(2, "plainDBox");
    m.insert(3, "altDBox");
    m.insert(4, "noGrowDoc");
    m.insert(5, "movableDBox");
    m.insert(8, "zoomDoc");
    m.insert(0xc, "zoomNoGrow");
    m.insert(0x10, "rDoc16");
    m.insert(0x12, "rDoc4");
    m.insert(0x14, "rDoc6");
    m.insert(0x16, "rDoc10");
    m
});
static DITL_TYPES: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(4, "button");
    m.insert(8, "label");
    m.insert(16, "textedit");
    m
});
static CDEFS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "button");
    m.insert(1, "checkbox");
    m.insert(2, "radio");
    m.insert(16, "scrollbar");
    m
});
