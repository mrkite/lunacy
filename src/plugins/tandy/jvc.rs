/*

TRS80 : JVC file system parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};

pub struct Jvc {}

impl Plugin for Jvc {
    fn get_name(&self) -> &str {
        "Tandy/JVC"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() == 35 * 18 * 256 {
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
        let parser = Parser::new(file);
        let entries = parser.parse()?;
        let mut treegrid = TreeGrid::new(
            &["Filename", "Ext", "Type"],
            &format!("jvc_{}", file.get_name()),
            move |entry: &Entry| {
                ctx.post(match parser.load_file(entry) {
                    Ok(data) => Message::Open(File::new_from_array(
                        &data,
                        &format!("{}.{}", entry.name, entry.ext),
                        &format!("rsdos/{:x}", entry.ftype),
                    )),
                    Err(err) => Message::Error(err.to_string()),
                });
            },
        );
        for entry in entries.iter() {
            treegrid.add(entry.render());
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
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.name);
        row.push_column(&self.ext);
        row.push_column(&self.type_name);
        row
    }
}

struct Parser {
    file: File,
}

impl Parser {
    fn new(file: &File) -> Self {
        Self { file: file.clone() }
    }

    fn parse(&self) -> Result<Vec<Entry>, &'static str> {
        let ofs = 17 * 18 * 256 + 2 * 256;
        let mut entries = vec![];
        for i in 0..72 {
            if self.file.r8(ofs + i * 32) == 0xff {
                // end of dir
                break;
            }
            let ftype = self.file.r8(ofs + i * 32 + 0xb);
            entries.push(Entry {
                name: self.file.rs(ofs + i * 32, 8, ""),
                ext: self.file.rs(ofs + i * 32 + 8, 3, ""),
                ftype,
                type_name: if let Some(t) = TYPES.get(usize::from(ftype)) {
                    t.to_string()
                } else {
                    format!("${:x}", ftype)
                },
                start: self.file.r8(ofs + i * 32 + 0xd),
                len: self.file.r16be(ofs + i * 32 + 0xe),
            });
        }
        Ok(entries)
    }

    fn load_file(&self, entry: &Entry) -> Result<Vec<u8>, &'static str> {
        let gran_size = 9 * 256;
        let mut data = vec![];
        let mut granule = usize::from(entry.start);
        while granule & 0x80 == 0 {
            if granule < 34 {
                data.extend_from_slice(self.file.read(granule * gran_size, gran_size));
            } else {
                let base = 18 * 18 * 256; // granules pickup on track 18
                data.extend_from_slice(
                    self.file.read((granule - 34) * gran_size + base, gran_size),
                );
            }
            granule = usize::from(self.file.r8(17 * 18 * 256 + 256 + granule));
        }
        // the final granule is only granule & 0xf sectors long
        // the final sector is only entry.len bytes long
        let len = data.len() - gran_size + (granule & 0xf) * 256 - 256 + usize::from(entry.len);
        data.resize(len, 0);
        Ok(data)
    }
}

const TYPES: [&str; 4] = ["Basic", "Data", "Exec", "Text"];
