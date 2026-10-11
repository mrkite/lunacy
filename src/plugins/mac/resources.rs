/*

68k Mac : Resource fork parser
@copyright 2026 Sean Kasun

*/

use std::rc::Rc;

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::{
        Plugin,
        mac::resparser::{Entry, Parser},
    },
    widgets::{Blank, Label, Link, TreeGrid, TreeRow, Widget},
};
use num_format::{Locale, ToFormattedString};

pub struct Resources {}

impl Plugin for Resources {
    fn get_name(&self) -> &'static str {
        "Mac/Resources"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.get_mime() == "mac/res" {
            Some(90)
        } else {
            None
        }
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Rc::new(Parser::new(handle));
        let entries = parser.parse();
        let mut treegrid = TreeGrid::new(&["ID", "Length", "Name"]);
        let mut keys = entries.keys().collect::<Vec<_>>();
        keys.sort();
        for key in keys {
            let mut row = TreeRow::default();
            row.push_column(Label::new(key));
            row.push_column(Blank::default());
            row.push_column(Blank::default());
            for child in &entries[key] {
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
        row.push_column(Link::new(
            &self.id.to_formatted_string(&Locale::en),
            "",
            move |ctx: &Context| {
                let separator = if entry.name.is_empty() { "" } else { " " };
                let name = format!(
                    "{} ${:02x}{}{}",
                    entry.res_type, entry.id, separator, entry.name
                );
                ctx.post(Message::Open(parser.get_res(
                    &name,
                    &entry.res_type,
                    entry.offset,
                    entry.attr,
                )));
            },
        ));
        row.push_column(Label::new(&self.length.to_formatted_string(&Locale::en)));
        row.push_column(Label::new(&self.name));
        row
    }
}
