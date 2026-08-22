/*

68k Mac : STR and STR# parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::File,
    plugins::{Plugin, get_charset},
    widgets::Widget,
};
use egui_extras::{Column, TableBuilder};
use lazy_static::lazy_static;
use num_format::{Locale, ToFormattedString};
use std::collections::HashMap;

pub struct Res {}

impl Plugin for Res {
    fn get_name(&self) -> &str {
        "Mac/STR"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        match file.get_mime() {
            "mac/res/ALRT" | "mac/res/CNTL" | "mac/res/DITL" | "mac/res/DLOG" | "mac/res/MENU"
            | "mac/res/STR " | "mac/res/STR#" | "mac/res/WIND" => Some(90),
            _ => None,
        }
    }

    fn create(
        &self,
        file: &File,
        _ctx: crate::lunacy::Context,
    ) -> Result<Box<dyn crate::widgets::Widget>, &'static str> {
        match file.get_mime() {
            "mac/res/ALRT" => Ok(Box::new(KVView::parse_alrt(file)?)),
            "mac/res/CNTL" => Ok(Box::new(KVView::parse_cntl(file)?)),
            "mac/res/DITL" => Ok(Box::new(KVView::parse_ditl(file)?)),
            "mac/res/DLOG" => Ok(Box::new(KVView::parse_wind(file, true)?)),
            "mac/res/MENU" => Ok(Box::new(KVView::parse_menu(file)?)),
            "mac/res/STR " | "mac/res/STR#" => Ok(Box::new(StrView::parse(file)?)),
            "mac/res/WIND" => Ok(Box::new(KVView::parse_wind(file, false)?)),
            _ => Err("unsupported resource type"),
        }
    }
}

struct StrView {
    strings: Vec<String>,
}

impl StrView {
    fn parse(file: &File) -> Result<Self, &'static str> {
        let (ofs, num) = if file.get_mime() == "mac/res/STR#" {
            (2, file.r16be(0))
        } else {
            (0, 1)
        };

        let mut strings = vec![];

        let mut ofs = ofs;
        for _ in 0..num {
            strings.push(file.rps(ofs, "macroman"));
            ofs += usize::from(file.r8(ofs)) + 1;
        }
        Ok(Self { strings })
    }
}

impl Widget for StrView {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _ctx: crate::lunacy::Context) {
        let table = TableBuilder::new(ui)
            .column(Column::initial(200.0))
            .column(Column::remainder());
        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Index");
                });
                header.col(|ui| {
                    ui.strong("String");
                });
            })
            .body(|body| {
                body.rows(18.0, self.strings.len(), |mut row| {
                    let row_index = row.index();
                    row.col(|ui| {
                        ui.label(format!("{}", row_index + 1));
                    });
                    row.col(|ui| {
                        ui.label(self.strings[row_index].clone());
                    });
                });
            });
    }
}

struct KVView {
    kvs: Vec<(&'static str, String)>,
}

impl KVView {
    fn parse_alrt(file: &File) -> Result<Self, &'static str> {
        if file.len() < 10 {
            return Err("Resource truncated");
        }
        let mut kvs = vec![];
        kvs.push(("top", format!("{}", file.r16be(0) as i16)));
        kvs.push(("left", format!("{}", file.r16be(2) as i16)));
        kvs.push(("bottom", format!("{}", file.r16be(4) as i16)));
        kvs.push(("right", format!("{}", file.r16be(6) as i16)));
        kvs.push(("ditl", file.r16be(8).to_formatted_string(&Locale::en)));
        Ok(Self { kvs })
    }
    fn parse_cntl(file: &File) -> Result<Self, &'static str> {
        if file.len() < 23 {
            return Err("Resource truncated");
        }
        let mut kvs = vec![];
        kvs.push(("top", format!("{}", file.r16be(0) as i16)));
        kvs.push(("left", format!("{}", file.r16be(2) as i16)));
        kvs.push(("bottom", format!("{}", file.r16be(4) as i16)));
        kvs.push(("right", format!("{}", file.r16be(6) as i16)));
        kvs.push(("value", format!("{}", file.r16be(8) as i16)));
        kvs.push((
            "visible",
            if file.r8(10) == 0 { "no" } else { "yes" }.to_string(),
        ));
        kvs.push(("maxValue", format!("{}", file.r16be(12) as i16)));
        kvs.push(("minValue", format!("{}", file.r16be(14) as i16)));
        let cdef = file.r16be(16);
        kvs.push((
            "cdef",
            format!("${:04x} {}", cdef, CDEFS.get(&cdef).unwrap_or(&"")),
        ));
        kvs.push(("refcon", format!("${:08x}", file.r32be(18))));
        kvs.push(("title", file.rps(22, "macroman")));
        Ok(Self { kvs })
    }
    fn parse_ditl(file: &File) -> Result<Self, &'static str> {
        if file.len() < 2 {
            return Err("Resource truncated");
        }
        let num_items = file.r16be(0);
        let mut ofs = 2;
        let mut item = 0;
        let mut kvs = vec![];
        while ofs < file.len() && item < num_items {
            ofs += 4; // reserved
            kvs.push(("item", format!("{}", item + 1)));
            item += 1;
            kvs.push(("\ttop", format!("{}", file.r16be(ofs) as i16)));
            ofs += 2;
            kvs.push(("\tleft", format!("{}", file.r16be(ofs) as i16)));
            ofs += 2;
            kvs.push(("\tbottom", format!("{}", file.r16be(ofs) as i16)));
            ofs += 2;
            kvs.push(("\tright", format!("{}", file.r16be(ofs) as i16)));
            ofs += 2;
            let ditltyp = file.r8(ofs);
            ofs += 1;
            kvs.push((
                "\ttype",
                format!(
                    "${:04x} {}",
                    ditltyp,
                    DITL_TYPES.get(&ditltyp).unwrap_or(&"")
                ),
            ));
            let titlelen = usize::from(file.r8(ofs));
            kvs.push(("\ttitle", file.rps(ofs, "macroman")));
            ofs += titlelen + 1 + (titlelen & 1);
        }
        Ok(Self { kvs })
    }
    fn parse_menu(file: &File) -> Result<Self, &'static str> {
        let cs = get_charset("macroman");
        if file.len() < 15 {
            return Err("Resource truncated");
        }
        let mut kvs = vec![];
        kvs.push(("menuID", format!("{}", file.r16be(0))));
        kvs.push(("resourceID", format!("{}", file.r16be(6) as i16)));
        kvs.push(("placeholder", format!("{}", file.r16be(8))));
        let enabled = file.r32be(10);
        kvs.push((
            "enabled",
            if enabled & 1 == 0 { "no" } else { "yes" }.to_string(),
        ));
        let titlelen = file.r8(14);
        kvs.push(("title", file.rps(14, "macroman")));
        let mut ofs = 15 + usize::from(titlelen);
        let mut item = 1;
        while ofs < file.len() && file.r8(ofs) != 0 {
            kvs.push(("menu item", format!("{}", item)));
            let namelen = usize::from(file.r8(ofs));
            kvs.push(("\tname", file.rps(ofs, "macroman")));
            ofs += namelen + 1;
            let icon = file.r8(ofs);
            if icon != 0 {
                kvs.push(("\ticon", format!("{}", icon)));
            }
            ofs += 1;
            let key = file.r8(ofs);
            if key != 0 {
                kvs.push(("\tkey", format!("{}", cs[usize::from(key)])));
            }
            ofs += 1;
            let mark = file.r8(ofs);
            if mark != 0 {
                kvs.push(("\tmark", format!("{}", cs[usize::from(mark)])));
            }
            ofs += 1;
            kvs.push(("\tstyle", format!("{}", file.r8(ofs))));
            ofs += 1;
            kvs.push((
                "\tenabled",
                if item < 32 && enabled & (1 << item) == 0 {
                    "no"
                } else {
                    "yes"
                }
                .to_string(),
            ));
            item += 1;
        }
        Ok(Self { kvs })
    }
    fn parse_wind(file: &File, is_dlog: bool) -> Result<Self, &'static str> {
        let base_len = if is_dlog { 20 } else { 18 };
        if file.len() < base_len + 1 {
            return Err("Resource truncated");
        }
        let mut kvs = vec![];
        kvs.push(("top", format!("{}", file.r16be(0) as i16)));
        kvs.push(("left", format!("{}", file.r16be(2) as i16)));
        kvs.push(("bottom", format!("{}", file.r16be(4) as i16)));
        kvs.push(("right", format!("{}", file.r16be(6) as i16)));
        let wintyp = file.r16be(8);
        kvs.push((
            "type",
            format!("${:04x} {}", wintyp, WIND_TYPES.get(&wintyp).unwrap_or(&"")),
        ));
        kvs.push((
            "visible",
            if file.r16be(10) == 0 { "no" } else { "yes" }.to_string(),
        ));
        kvs.push((
            "closeBox",
            if file.r16be(12) == 0 { "no" } else { "yes" }.to_string(),
        ));
        kvs.push(("refcon", format!("${:08x}", file.r32be(14))));
        if is_dlog {
            kvs.push(("ditl", file.r16be(18).to_formatted_string(&Locale::en)));
        }
        kvs.push(("title", file.rps(base_len, "macroman")));
        Ok(Self { kvs })
    }
}

impl Widget for KVView {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _ctx: crate::lunacy::Context) {
        let table = TableBuilder::new(ui)
            .column(Column::initial(200.0))
            .column(Column::remainder());
        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Key");
                });
                header.col(|ui| {
                    ui.strong("Value");
                });
            })
            .body(|body| {
                body.rows(18.0, self.kvs.len(), |mut row| {
                    let row_index = row.index();
                    if let Some(kv) = self.kvs.get(row_index) {
                        row.col(|ui| {
                            ui.label(kv.0);
                        });
                        row.col(|ui| {
                            ui.label(kv.1.clone());
                        });
                    }
                });
            });
    }
}

lazy_static! {
    static ref WIND_TYPES: HashMap<u16, &'static str> = {
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
    };
    static ref DITL_TYPES: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(4, "button");
        m.insert(8, "label");
        m.insert(16, "textedit");
        m
    };
    static ref CDEFS: HashMap<u16, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0, "button");
        m.insert(1, "checkbox");
        m.insert(2, "radio");
        m.insert(16, "scrollbar");
        m
    };
}
