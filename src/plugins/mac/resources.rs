/*

68k Mac : Resource fork parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::{
        Plugin,
        mac::resparser::{Entry, Parser},
    },
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};

pub struct Resources {}

impl Plugin for Resources {
    fn get_name(&self) -> &str {
        "Mac/Resources"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.get_mime() == "mac/res" {
            Some(90)
        } else {
            None
        }
    }

    fn create(
        &self,
        file: &File,
        ctx: Context,
    ) -> Result<Box<dyn crate::plugins::Widget>, &'static str> {
        let parser = Parser::new(file);
        let entries = parser.parse()?;
        let mut treegrid = TreeGrid::new(
            &["ID", "Length", "Name"],
            &format!("res_{}", file.get_name()),
            move |entry: &Entry| {
                let separator = if entry.name.is_empty() { "" } else { " " };
                let name = format!(
                    "{} ${:02x}{}{}",
                    entry.res_type, entry.id, separator, entry.name
                );
                ctx.post(
                    match parser.get_res(&name, &entry.res_type, entry.offset, entry.attr) {
                        Ok(file) => Message::Open(file),
                        Err(err) => Message::Error(err.to_string()),
                    },
                );
            },
        );
        let mut keys = Vec::from_iter(entries.keys());
        keys.sort();
        for key in keys {
            let mut row = TreeRow::new(Entry::default());
            row.push_column(key);
            row.push_column("");
            row.push_column("");
            for child in entries[key].iter() {
                row.push_child(child.render());
            }
            treegrid.add(row);
        }
        Ok(Box::new(treegrid))
    }
}

impl Entry {
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.id.to_formatted_string(&Locale::en));
        row.push_column(&self.length.to_formatted_string(&Locale::en));
        row.push_column(&self.name);
        row
    }
}
