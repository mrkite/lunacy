/*

TRS80 : JVC file system parser
@copyright 2026 Sean Kasun

*/

use std::rc::Rc;

use crate::{
    lunacy::{Context, Handle, Message},
    plugins::Plugin,
    widgets::{Label, Link, TreeGrid, TreeRow, Widget},
};

pub struct Jvc {}

impl Plugin for Jvc {
    fn get_name(&self) -> &'static str {
        "Tandy/JVC"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.len() == 35 * 18 * 256 {
            Some(60)
        } else {
            None
        }
    }

    fn create(&self, handle: &Handle, ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let parser = Rc::new(Parser::new(handle));
        let entries = parser.parse();
        let mut treegrid = TreeGrid::new(&["Filename", "Ext", "Type"]);
        for entry in &entries {
            treegrid.add(entry.render(parser.clone()), ctx);
        }
        Ok(Box::new(treegrid))
    }
}

#[derive(Clone)]
struct Entry {
    name: String,
    ext: String,
    ftype: u8,
    type_name: String,
    start: u8,
    len: u16,
}

impl Entry {
    fn render(&self, parser: Rc<Parser>) -> TreeRow {
        let mut row = TreeRow::default();
        let entry = self.clone();
        row.push_column(Link::new(
            &self.name,
            &format!("rsdos/{:x}", entry.ftype),
            move |ctx: &Context| {
                let data = parser.load_file(&entry);
                ctx.post(Message::Open(Handle::new_from_array(
                    &data,
                    &format!("{}.{}", entry.name, entry.ext),
                    &format!("rsdos/{:x}", entry.ftype),
                )));
            },
        ));
        row.push_column(Label::new(&self.ext));
        row.push_column(Label::new(&self.type_name));
        row
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

    fn parse(&self) -> Vec<Entry> {
        let ofs = 17 * 18 * 256 + 2 * 256;
        let mut entries = vec![];
        for i in 0..72 {
            if self.handle.r8(ofs + i * 32) == 0xff {
                // end of dir
                break;
            }
            let ftype = self.handle.r8(ofs + i * 32 + 0xb);
            entries.push(Entry {
                name: self.handle.rs(ofs + i * 32, 8, "").trim_end().to_string(),
                ext: self
                    .handle
                    .rs(ofs + i * 32 + 8, 3, "")
                    .trim_end()
                    .to_string(),
                ftype,
                type_name: TYPES
                    .get(usize::from(ftype))
                    .map_or_else(|| format!("${ftype:x}"), ToString::to_string),
                start: self.handle.r8(ofs + i * 32 + 0xd),
                len: self.handle.r16be(ofs + i * 32 + 0xe),
            });
        }
        entries
    }

    fn load_file(&self, entry: &Entry) -> Vec<u8> {
        let gran_size = 9 * 256;
        let mut data = vec![];
        let mut granule = usize::from(entry.start);
        while granule & 0x80 == 0 {
            if granule < 34 {
                data.extend_from_slice(self.handle.read(granule * gran_size, gran_size));
            } else {
                let base = 18 * 18 * 256; // granules pickup on track 18
                data.extend_from_slice(
                    self.handle
                        .read((granule - 34) * gran_size + base, gran_size),
                );
            }
            granule = usize::from(self.handle.r8(17 * 18 * 256 + 256 + granule));
        }
        // the final granule is only granule & 0xf sectors long
        // the final sector is only entry.len bytes long
        let len = data.len() - gran_size + (granule & 0xf) * 256 - 256 + usize::from(entry.len);
        data.resize(len, 0);
        data
    }
}

const TYPES: [&str; 4] = ["Basic", "Data", "Exec", "Text"];
