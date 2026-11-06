/*

Apple IIgs : Resource fork view
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::{
        Plugin,
        appleii::resparser::{Entry, Parser},
    },
    widgets::{Blank, Label, Link, TreeGrid, TreeRow, Widget},
};
use num_format::{Locale, ToFormattedString};
use std::{collections::HashMap, rc::Rc, sync::LazyLock};

pub struct Resources {}

impl Plugin for Resources {
    fn get_name(&self) -> &'static str {
        "Apple/Resources"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.get_mime() == "prodos/res" {
            Some(90)
        } else {
            None
        }
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Rc::new(Parser::new(handle));
        let entries = parser.parse();
        let mut treegrid = TreeGrid::new(&["ID", "Length"]);
        let mut keys = vec![];
        for key in entries.keys() {
            if let Some(res_type) = TYPES.get(key) {
                keys.push((res_type.to_string(), key));
            } else {
                keys.push((format!("${key:x}"), key));
            }
        }
        keys.sort();
        for key in keys {
            let mut row = TreeRow::default();
            row.push_column(Label::new(&key.0));
            row.push_column(Blank::default());
            for child in &entries[key.1] {
                row.push_child(child.render(parser.clone()));
            }
            treegrid.add(row, ctx);
        }
        Ok(Box::new(treegrid))
    }
}

impl Entry {
    fn render(&self, parser: Rc<Parser>) -> TreeRow {
        let mut row = TreeRow::default();
        let entry = self.clone();
        let name = self.id.to_formatted_string(&Locale::en);
        row.push_column(Link::new(&name.clone(), "", move |ctx: &Context| {
            ctx.post(Message::Open(parser.get_res(
                entry.res_type,
                &name,
                entry.offset,
                usize::try_from(entry.length).unwrap_or_default(),
            )));
        }));
        row.push_column(Label::new(&self.length.to_formatted_string(&Locale::en)));
        row
    }
}

static TYPES: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0x8001, "rIcon");
    m.insert(0x8002, "rPicture");
    m.insert(0x8003, "rControlList");
    m.insert(0x8004, "rControlTemplate");
    m.insert(0x8005, "rC1InputString");
    m.insert(0x8006, "rPString");
    m.insert(0x8007, "rStringList");
    m.insert(0x8008, "rMenuBar");
    m.insert(0x8009, "rMenu");
    m.insert(0x800a, "rMenuItem");
    m.insert(0x800b, "rTextForLETextBox2");
    m.insert(0x800d, "rCtlColorTbl");
    m.insert(0x800e, "rWindParam1");
    m.insert(0x800f, "rWindParam2");
    m.insert(0x8010, "rWindColor");
    m.insert(0x8011, "rTextBlock");
    m.insert(0x8012, "rStyleBlock");
    m.insert(0x8013, "rToolStartup");
    m.insert(0x8014, "rResName");
    m.insert(0x8015, "rAlertString");
    m.insert(0x8016, "rText");
    m.insert(0x801a, "rTwoRects");
    m.insert(0x801c, "rListRef");
    m.insert(0x801d, "rCString");
    m.insert(0x8020, "rErrorString");
    m.insert(0x8021, "rKTransTable");
    m.insert(0x8023, "rC1OutputString");
    m.insert(0x8024, "rSoundSample");
    m.insert(0x8025, "rTERuler");
    m.insert(0x8027, "rCursor");
    m.insert(0x8029, "rVersion");
    m.insert(0x802a, "rComment");
    m.insert(0x802e, "rTaggedStrings");
    m.insert(0x802f, "rPatternList");
    m.insert(0xc001, "rRectList");
    m.insert(0xc002, "rPrintRecord");
    m.insert(0xc003, "rFont");
    m
});
