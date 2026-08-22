/*

Apple II : ProDOS filesystem parser
@copyright 2026 Sean Kasun

*/
use crate::{
    lunacy::{Context, File, Message},
    plugins::{Plugin, appleii::prodos_files::get_prodos_type},
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};
use std::cmp;

pub struct Prodos {}

impl Plugin for Prodos {
    fn get_name(&self) -> &str {
        "Apple/Prodos"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() < 512 {
            return None;
        }
        if file.r4(0) == "2IMG" {
            if file.r32(0xc) != 1 {
                return None;
            }
            return Some(90);
        }
        // check sector format
        if (file.len() > 1536 && (file.len() & 511) == 0) || file.len() == 256 * 13 * 35 {
            if is_prodos(file, 2 * 512) {
                return Some(90);
            }
            if file.len() >= 2822 && is_prodos(file, 0xb00) {
                return Some(90);
            }
        }
        None
    }

    fn create(
        &self,
        file: &File,
        ctx: Context,
    ) -> Result<Box<dyn crate::plugins::Widget>, &'static str> {
        // should we slice the 2mg header off?
        let file = if file.r4(0) == "2IMG" {
            let len = (file.r32(0x14) * 512) as usize;
            let ofs = file.r32(0x18) as usize;
            &File::new_from_array(file.read(ofs, len), file.get_name(), file.get_mime())
        } else {
            file
        };
        let parser = Parser::new(file);
        let root = parser.get_directory(2)?;
        let mut treegrid = TreeGrid::new(
            &["Filename", "Ext", "Data", "Res", "Date", "Type"],
            &format!("prodos_{}", file.get_name()),
            move |entry: &Entry| {
                if let Some(id) = entry.data.as_ref() {
                    ctx.post(match parser.load_file(id) {
                        Ok(data) => Message::Open(File::new_from_array(&data, &id.name, &id.mime)),
                        Err(err) => Message::Error(err.to_string()),
                    });
                }
                if let Some(id) = entry.res.as_ref() {
                    ctx.post(match parser.load_file(id) {
                        Ok(data) => Message::Open(File::new_from_array(&data, &id.name, &id.mime)),
                        Err(err) => Message::Error(err.to_string()),
                    });
                }
            },
        );
        treegrid.add(root.render());
        Ok(Box::new(treegrid))
    }
}

fn is_prodos(file: &File, start: usize) -> bool {
    if file.r16(start) != 0 {
        return false;
    }
    let t = file.r8(start + 4);
    if (t & 0xf0) != 0xf0 || (t & 0xf) == 0 {
        return false;
    }
    let ch = file.r8(start + 5);
    ch.is_ascii_uppercase()
}

#[derive(Default, Clone)]
struct Entry {
    name: String,
    ext: String,
    desc: String,
    filetype: u8,
    data: Option<FileId>,
    res: Option<FileId>,
    aux: u16,
    mod_date: u16,
    mod_time: u16,
    children: Vec<Entry>,
}
impl Entry {
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.name);
        row.push_column(&self.ext);
        if let Some(data) = &self.data {
            row.push_column(&data.len.to_formatted_string(&Locale::en));
        } else {
            row.push_column("");
        }
        if let Some(res) = self.res.as_ref().cloned() {
            row.push_column(&res.len.to_formatted_string(&Locale::en));
        } else {
            row.push_column("");
        }
        if self.mod_date > 0 {
            let year = ((self.mod_date >> 9) & 0x7f) + 1900;
            row.push_column(&format!(
                "{:04}-{:02}-{:02} {:02}:{:02}",
                year,
                (self.mod_date >> 5) & 0xf,
                self.mod_date & 0x1f,
                (self.mod_time >> 8) & 0x1f,
                self.mod_time & 0x3f
            ));
        } else {
            row.push_column("");
        }
        row.push_column(&self.desc);
        for child in self.children.iter() {
            row.push_child(child.render());
        }
        row
    }
}

struct Parser {
    sec_per_track: usize,
    prodos_order: bool,
    file: File,
}

impl Parser {
    fn new(file: &File) -> Self {
        let sec_per_track = if file.len() == 35 * 13 * 256 { 13 } else { 16 };
        let mut parser = Self {
            sec_per_track,
            prodos_order: false,
            file: file.clone(),
        };
        // if this fails, sector order might be prodos
        parser.prodos_order = !is_prodos(&parser.get_block(2), 0);
        parser
    }

    fn get_directory(&self, key: usize) -> Result<Entry, &'static str> {
        let mut block = self.get_block(key);
        let storage = block.r8(4);

        if storage & 0xe0 != 0xe0 {
            return Err("invalid directory storage");
        }

        let mut dir = Entry {
            name: block.rs(5, usize::from(storage & 0xf), ""),
            ..Entry::default()
        };

        let entry_len = block.r8(0x23);
        let entries_per_block = block.r8(0x24);
        let file_count = block.r8(0x25);

        dir.children = vec![];
        let mut cur_entry = 1;
        let mut cur_file = 0;
        while cur_file < file_count {
            let ofs = usize::from(entry_len) * usize::from(cur_entry) + 4;
            let entry_storage = block.r8(ofs);
            if entry_storage != 0 {
                let key_pointer = block.r16(ofs + 0x11);
                dir.children.push(match entry_storage & 0xf0 {
                    0x10 | 0x20 | 0x30 | 0x50 => {
                        self.get_file(&block, ofs, key_pointer, entry_storage)
                    }
                    0xd0 => self.get_directory(usize::from(key_pointer))?,
                    _ => Err("invalid entry storage")?,
                });
                cur_file += 1;
            }
            cur_entry += 1;
            if cur_entry == entries_per_block {
                cur_entry = 0;
                let next = block.r16(2);
                if next == 0 {
                    break;
                }
                block = self.get_block(usize::from(next));
            }
        }

        Ok(dir)
    }

    fn get_file(&self, block: &File, ofs: usize, key: u16, storage: u8) -> Entry {
        let mut entry = Entry {
            filetype: block.r8(ofs + 0x10),
            aux: block.r16(ofs + 0x1f),
            name: block.rs(ofs + 1, usize::from(storage & 0xf), ""),
            mod_date: block.r16(ofs + 0x21),
            mod_time: block.r16(ofs + 0x23),
            ..Default::default()
        };
        (entry.ext, entry.desc) = get_prodos_type(entry.filetype, entry.aux);
        let data_len = block.r24(ofs + 0x15);
        let data_storage = storage >> 4;
        if data_len > 0 {
            entry.data = Some(FileId {
                key: usize::from(key),
                len: data_len as usize,
                storage: data_storage,
                name: entry.name.clone(),
                mime: format!("prodos/{:02x}:{:04x}", entry.filetype, entry.aux),
            });
        }

        if data_storage == 5 {
            let gsos = self.get_block(usize::from(key));
            entry.data = Some(FileId {
                storage: gsos.r8(0),
                key: usize::from(gsos.r16(1)),
                len: gsos.r24(5) as usize,
                name: entry.name.clone(),
                mime: format!("prodos/{:02x}:{:04x}", entry.filetype, entry.aux),
            });
            entry.res = Some(FileId {
                storage: gsos.r8(0x100),
                key: usize::from(gsos.r16(0x101)),
                len: gsos.r24(0x105) as usize,
                name: format!("{} [res]", entry.name),
                mime: "prodos/res".to_string(),
            });
        }
        entry
    }

    fn get_block(&self, key: usize) -> File {
        if self.prodos_order {
            File::new_from_array(self.file.read(key * 512, 512), "", "")
        } else {
            let track = key / (self.sec_per_track / 2);
            let block = key % (self.sec_per_track / 2);
            let mut data = vec![];
            data.extend_from_slice(self.read_sector(track, block * 2));
            data.extend_from_slice(self.read_sector(track, block * 2 + 1));
            File::new_from_array(&data, "", "")
        }
    }

    fn read_sector(&self, track: usize, sector: usize) -> &[u8] {
        const PRODOS2RAW: [usize; 16] = [0, 2, 4, 6, 8, 10, 12, 14, 1, 3, 5, 7, 9, 11, 13, 15];
        const RAW2DOS: [usize; 16] = [0, 7, 14, 6, 13, 5, 12, 4, 11, 3, 10, 2, 9, 1, 8, 15];
        let mut offset = track * self.sec_per_track * 256;
        let sector = if self.sec_per_track == 16 || self.sec_per_track == 32 {
            RAW2DOS[PRODOS2RAW[if sector >= 16 {
                offset += 16 * 256;
                sector - 16
            } else {
                sector
            }]]
        } else {
            sector
        };
        offset += sector * 256;
        self.file.read(offset, 256)
    }

    fn load_file(&self, id: &FileId) -> Result<Vec<u8>, &'static str> {
        match id.storage {
            1 => Ok(self.seedling(id.key, id.len)),
            2 => Ok(self.sapling(id.key, id.len)),
            3 => Ok(self.tree(id.key, id.len)),
            _ => Err("invalid file storage"),
        }
    }

    fn seedling(&self, key: usize, len: usize) -> Vec<u8> {
        self.get_block(key).read(0, len).to_vec()
    }

    fn sapling(&self, key: usize, len: usize) -> Vec<u8> {
        let block = self.get_block(key);
        let mut len = len;
        let mut pos = 0;
        let mut data = vec![];
        while len > 0 {
            let bkey = usize::from(block.r8(pos)) | (usize::from(block.r8(pos + 256)) << 8);
            let blen = cmp::min(len, 512);
            if bkey != 0 {
                data.extend_from_slice(&self.seedling(bkey, blen));
            } else {
                data.resize(data.len() + blen, 0);
            }
            pos += 1;
            len -= blen;
        }
        data
    }

    fn tree(&self, key: usize, len: usize) -> Vec<u8> {
        let block = self.get_block(key);
        let mut len = len;
        let mut pos = 0;
        let mut data = vec![];
        while len > 0 {
            let bkey = usize::from(block.r8(pos)) | (usize::from(block.r8(pos + 256)) << 8);
            let blen = cmp::min(len, 256 * 512);
            if bkey != 0 {
                data.extend_from_slice(&self.sapling(bkey, blen));
            } else {
                data.resize(data.len() + blen, 0);
            }
            pos += 1;
            len -= blen;
        }
        data
    }
}

#[derive(Clone)]
struct FileId {
    key: usize,
    len: usize,
    storage: u8,
    name: String,
    mime: String,
}
