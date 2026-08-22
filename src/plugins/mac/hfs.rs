/*

68k Mac : HFS file system parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File, Message},
    plugins::Plugin,
    widgets::{TreeGrid, TreeRow},
};
use num_format::{Locale, ToFormattedString};
use std::{cell::RefCell, cmp, rc::Rc};

pub struct Hfs {}

impl Plugin for Hfs {
    fn get_name(&self) -> &str {
        "Mac/HFS"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() < 0x500 {
            return None;
        }
        if file.r16be(0x400) != 0x4244 {
            if file.r16be(0x454) != 0x4244 || file.r16be(0x462) != 3 || file.r8(0x478) >= 28 {
                None
            } else {
                Some(90)
            }
        } else if file.r16be(0x40e) != 3 || file.r8(0x424) >= 28 {
            None
        } else {
            Some(90)
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
            &["Filename", "Type", "Creator", "Data", "Res"],
            &format!("hfs_{}", file.get_name()),
            move |entry: &Entry| {
                if entry.is_file {
                    if entry.res_len > 0 {
                        ctx.post(
                            match parser.load_file(entry.res_extents, entry.res_len, entry.id, 0xff)
                            {
                                Ok(data) => Message::Open(File::new_from_array(
                                    &data,
                                    &format!("{} [res]", entry.name),
                                    "mac/res",
                                )),
                                Err(err) => Message::Error(err.to_string()),
                            },
                        );
                    }
                    if entry.data_len > 0 {
                        ctx.post(
                            match parser.load_file(entry.data_extents, entry.data_len, entry.id, 0)
                            {
                                Ok(data) => {
                                    Message::Open(File::new_from_array(&data, &entry.name, ""))
                                }
                                Err(err) => Message::Error(err.to_string()),
                            },
                        );
                    }
                }
            },
        );
        for entry in entries.iter() {
            treegrid.add(entry.borrow().render());
        }
        Ok(Box::new(treegrid))
    }
}

#[derive(Default, Clone)]
struct Entry {
    is_file: bool,
    id: u32,
    parent: u32,
    name: String,
    filetype: String,
    creator: String,
    data_len: u32,
    res_len: u32,
    data_extents: usize,
    res_extents: usize,
    children: Vec<EntryPtr>,
}

type EntryPtr = Rc<RefCell<Entry>>;
impl Entry {
    fn render(&self) -> TreeRow<Entry> {
        let mut row = TreeRow::new(self.clone());
        row.push_column(&self.name);
        row.push_column(&self.filetype);
        row.push_column(&self.creator);
        if self.data_len > 0 {
            row.push_column(&self.data_len.to_formatted_string(&Locale::en));
        } else {
            row.push_column("");
        }
        if self.res_len > 0 {
            row.push_column(&self.res_len.to_formatted_string(&Locale::en));
        } else {
            row.push_column("");
        }

        for child in self.children.iter() {
            row.push_child(child.borrow().render());
        }
        row
    }
}

struct Parser {
    file: File,
    block_size: usize,
    block_start: usize,
    over_node: u32,
    ovr_extents: Vec<Extent>,
    cat_extents: Vec<Extent>,
}

struct Extent {
    start: usize,
    count: usize,
}

impl Parser {
    fn new(file: &File) -> Self {
        let start = if file.r16be(0x400) != 0x4244 { 0x5d } else { 0 };
        let block_size = file.r32be(start + 0x414) as usize;
        let block_start = file.r16be(start + 0x41c) as usize;
        let mut ovr_extents = vec![];
        let mut cat_extents = vec![];
        for i in 0..3 {
            ovr_extents.push(Extent {
                start: usize::from(file.r16be(start + 0x486 + i * 4)),
                count: usize::from(file.r16be(start + 0x486 + i * 4 + 2)),
            });
            cat_extents.push(Extent {
                start: usize::from(file.r16be(start + 0x496 + i * 4)),
                count: usize::from(file.r16be(start + 0x496 + i * 4 + 2)),
            });
        }
        // read overflow file first
        let block = (ovr_extents[0].start + block_start) * block_size;
        let over_node = match file.r8(block + 8) {
            1 => {
                // header node
                let pos = usize::from(file.r16be(block + block_size - 2)) + block + 2;
                file.r32be(pos)
            }
            _ => 0xffffffff,
        };

        Self {
            file: file.clone(),
            block_size,
            block_start,
            over_node,
            ovr_extents,
            cat_extents,
        }
    }

    fn parse(&self) -> Result<Vec<EntryPtr>, &'static str> {
        if self.block_size & 0xff000000 != 0 {
            return Err("blocksize is too large");
        }
        if self.over_node == 0xffffffff {
            return Err("invalid overflow table");
        }

        // first node better be a header node
        let block = self.block_ofs(self.cat_extents[0].start);
        let entries = match self.file.r8(block + 8) {
            1 => {
                // header node
                let pos = usize::from(self.file.r16be(block + self.block_size - 2)) + block + 2;
                self.walk(self.file.r32be(pos))?
            }
            _ => return Err("invalid hfs root"),
        };
        let mut root = vec![];
        // rearrange into a tree
        for entry in entries.iter() {
            let mut found = false;
            for parent in entries.iter() {
                if !parent.borrow().is_file && parent.borrow().id == entry.borrow().parent {
                    parent.borrow_mut().children.push(entry.clone());
                    found = true;
                }
            }
            if !found {
                root.push(entry.clone());
            }
        }
        Ok(root)
    }

    fn walk(&self, node: u32) -> Result<Vec<EntryPtr>, &'static str> {
        let mut i = 0;
        let mut node = node as usize;
        while node >= self.cat_extents[i].count {
            node -= self.cat_extents[i].count;
            i += 1;
        }
        let cur = self.block_ofs(self.cat_extents[i].start + node);
        let kind = self.file.r8(cur + 8);
        let num_records = self.file.r16be(cur + 10);
        let mut entries = vec![];
        match kind {
            0 => {
                // index node
                for i in 0..usize::from(num_records) {
                    let ofs = usize::from(self.file.r16be(cur + self.block_size - (i + 1) * 2));
                    let keylen = usize::from(self.file.r8(cur + ofs));
                    entries.extend_from_slice(&self.walk(self.file.r32be(cur + ofs + keylen + 1))?);
                }
            }
            0xff => {
                // leaf node
                for i in 0..usize::from(num_records) {
                    let ofs = usize::from(self.file.r16be(cur + self.block_size - (i + 1) * 2));
                    if let Some(entry) = self.leaf(cur + ofs)? {
                        entries.push(entry);
                    }
                }
            }
            _ => {
                return Err("unknown extent type");
            }
        }
        Ok(entries)
    }

    fn leaf(&self, record: usize) -> Result<Option<EntryPtr>, &'static str> {
        let keylen = self.file.r8(record) + 1;
        if keylen == 0 {
            // deleted
            return Ok(None);
        }
        let keylen = usize::from(keylen + (keylen & 1));
        let parent = self.file.r32be(record + 2);
        let name = self.file.rps(record + 6, "macroman");
        match self.file.r16be(record + keylen) {
            0x100 => {
                // folder
                Ok(Some(Rc::new(RefCell::new(Entry {
                    is_file: false,
                    parent,
                    name,
                    filetype: String::new(),
                    creator: String::new(),
                    id: self.file.r32be(record + keylen + 6),
                    data_len: 0,
                    res_len: 0,
                    data_extents: 0,
                    res_extents: 0,
                    children: vec![],
                }))))
            }
            0x200 => {
                // file
                Ok(Some(Rc::new(RefCell::new(Entry {
                    is_file: true,
                    parent,
                    name,
                    filetype: self.file.rs(record + keylen + 4, 4, "macroman"),
                    creator: self.file.rs(record + keylen + 8, 4, "macroman"),
                    id: self.file.r32be(record + keylen + 0x14),
                    data_len: self.file.r32be(record + keylen + 0x1a),
                    res_len: self.file.r32be(record + keylen + 0x24),
                    data_extents: record + keylen + 0x4a,
                    res_extents: record + keylen + 0x56,
                    children: vec![],
                }))))
            }
            0x300 => {
                // directory thread
                Ok(None)
            }
            0x400 => {
                // file thread
                Ok(None)
            }
            _ => Err("unknown file kind"),
        }
    }

    fn overflow(
        &self,
        node: u32,
        id: u32,
        block: u16,
        fork: u8,
    ) -> Result<Vec<Extent>, &'static str> {
        let mut i = 0;
        let mut node = node as usize;
        while node >= self.ovr_extents[i].count {
            node -= self.ovr_extents[i].count;
            i += 1;
        }
        let cur = self.block_ofs(self.ovr_extents[i].start + node);
        let kind = self.file.r8(cur + 8);
        let num_records = self.file.r16be(cur + 10);

        match kind {
            0 => {
                // index node
                for i in 0..usize::from(num_records) {
                    let ofs = usize::from(self.file.r16be(cur + self.block_size - (i + 1) * 2));
                    let keylen = usize::from(self.file.r8(cur + ofs));
                    let extents =
                        self.overflow(self.file.r32be(cur + ofs + keylen + 1), id, block, fork)?;
                    if !extents.is_empty() {
                        return Ok(extents);
                    }
                }
                Ok(vec![])
            }
            0xff => {
                // leaf node
                for i in 0..usize::from(num_records) {
                    let ofs = usize::from(self.file.r16be(cur + self.block_size - (i + 1) * 2));
                    let extents = self.ovr_leaf(cur + ofs, id, block, fork)?;
                    if !extents.is_empty() {
                        return Ok(extents);
                    }
                }
                Ok(vec![])
            }
            _ => Err("unknown extent kind"),
        }
    }

    fn ovr_leaf(
        &self,
        record: usize,
        id: u32,
        block: u16,
        fork: u8,
    ) -> Result<Vec<Extent>, &'static str> {
        let mut extents = vec![];
        let keylen = self.file.r8(record);
        if keylen == 7
            && self.file.r8(record + 1) == fork
            && self.file.r32be(record + 2) == id
            && self.file.r16be(record + 6) == block
        {
            for i in 0..3 {
                extents.push(Extent {
                    start: usize::from(self.file.r16be(record + 8 + i * 4)),
                    count: usize::from(self.file.r16be(record + 8 + i * 4 + 2)),
                });
            }
        }
        Ok(extents)
    }

    fn load_file(
        &self,
        extent_ofs: usize,
        len: u32,
        id: u32,
        fork: u8,
    ) -> Result<Vec<u8>, &'static str> {
        let mut extents = vec![];
        for i in 0..3 {
            extents.push(Extent {
                start: usize::from(self.file.r16be(extent_ofs + i * 4)),
                count: usize::from(self.file.r16be(extent_ofs + i * 4 + 2)),
            });
        }

        let mut data = vec![];
        let mut block = 0;
        let mut total_blocks = 0;
        let mut ext = 0;
        let mut len = len as usize;

        while len > 0 {
            let size = cmp::min(len, self.block_size);
            let pos = self.block_ofs(extents[ext].start + block);
            data.extend_from_slice(self.file.read(pos, size));
            len -= size;
            block += 1;
            total_blocks += 1;
            if block == extents[ext].count && len > 0 {
                ext += 1;
                block = 0;
                if ext == 3 {
                    ext = 0;
                    extents = self.overflow(self.over_node, id, total_blocks, fork)?;
                    if extents.is_empty() {
                        return Err("corrupt overflow table");
                    }
                }
            }
        }
        Ok(data)
    }

    fn block_ofs(&self, start: usize) -> usize {
        (start + self.block_start) * self.block_size
    }
}
