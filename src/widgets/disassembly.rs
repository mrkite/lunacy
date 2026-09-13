/*

General Disassembly Viewer widget
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{CpuLine, CpuState, InstType, Instruction},
    lunacy::{Context, File, gestalt},
    platforms::{BlockCache, Platform},
};
use eframe::egui::{self, Color32, Key, Pos2, Rect, Sense};
use std::{
    cmp,
    collections::{BTreeMap, BTreeSet},
    io::Write,
    ops::Bound,
};

const ADDRESS_COLOR: Color32 = Color32::from_rgb(0x9d, 0xac, 0xae);
const HEX_COLOR: Color32 = Color32::from_rgb(0x82, 0x8d, 0xab);
const OPCODE_COLOR: Color32 = Color32::from_rgb(0xb2, 0xee, 0xbe);
const OPERAND_COLOR: Color32 = Color32::from_rgb(0x9b, 0xcf, 0xc2);
const COMMENT_COLOR: Color32 = Color32::from_rgb(0x15, 0x5e, 0x95);
const HILITE_COLOR: Color32 = Color32::from_rgb(0x15, 0x5e, 0x95);

struct Section {
    start: u32,
    file: File,
}

pub struct Disassembly {
    id_salt: String,
    search: Option<String>,
    cursor: usize,
    jump_cursor: Option<usize>,
    labels: BTreeSet<u32>,
    branches: BTreeMap<u32, u32>,
    sections: Vec<Section>,
    insts: BTreeMap<u32, Instruction>,
    lines: Vec<CpuLine>,
}

impl Disassembly {
    pub fn new(id_salt: &str) -> Self {
        Self {
            id_salt: id_salt.to_string(),
            search: None,
            cursor: 0,
            jump_cursor: None,
            labels: BTreeSet::default(),
            branches: BTreeMap::default(),
            sections: vec![],
            insts: BTreeMap::default(),
            lines: vec![],
        }
    }

    pub fn add_section(&mut self, name: &str, start: u32, data: &[u8]) {
        self.sections.push(Section {
            start,
            file: File::new_from_array(data, name, ""),
        });
    }

    pub fn add_entry(
        &mut self,
        state: impl CpuState + Clone,
        platform: &impl Platform,
    ) -> Result<(), &'static str> {
        // TODO if state is outside of the current address space.. error out
        self.labels.insert(state.address());
        let mut work_list = vec![state];
        while let Some(mut state) = work_list.pop() {
            let mut done = false;
            while !done {
                let addr = state.address();
                if let Some(section) = self.get_address(addr) {
                    self.insts.insert(
                        addr,
                        platform.decode(
                            &section.file,
                            (addr - section.start) as usize,
                            &mut state,
                        )?,
                    );
                    let inst = &self.insts[&addr];
                    if (inst.inst_type == InstType::Jump
                        || inst.inst_type == InstType::Branch
                        || inst.inst_type == InstType::Call)
                        && let Some(target) = platform.target(inst)
                        && self.get_address(target).is_some()
                        && !self.labels.contains(&target)
                    {
                        // only insert new targets
                        self.labels.insert(target);
                        let mut state = state.clone();
                        state.jump(target);
                        work_list.push(state);
                    }
                    if inst.inst_type == InstType::Jump
                        || inst.inst_type == InstType::Branch
                        || inst.inst_type == InstType::Return
                    {
                        // we want branches includd in the block
                        self.branches.insert(state.address(), addr);
                    } else if inst.inst_type == InstType::Invalid {
                        self.branches.insert(addr, addr);
                    }
                    done = inst.inst_type == InstType::Invalid
                        || inst.inst_type == InstType::Return
                        || inst.inst_type == InstType::Jump;
                } else {
                    // section doesn't exist, something went wrong.
                    done = true;
                }
            }
        }
        self.disassemble(platform);
        Ok(())
    }

    fn disassemble(&mut self, platform: &impl Platform) {
        self.lines.clear();
        let mut blocks = BlockCache::default();
        // always start at a label
        if let Some(address) = self.labels.first() {
            let mut address = *address;
            let mut block = blocks.get(address);
            let mut done = false;
            while !done {
                let label = self
                    .labels
                    .range((Bound::Excluded(address), Bound::Unbounded))
                    .next();
                let branch = self
                    .branches
                    .range((Bound::Excluded(address), Bound::Unbounded))
                    .next();
                if let Some(label) = label
                    && branch.is_none_or(|branch| label < branch.1)
                {
                    // label was earliest or no branch at all
                    address = *label;
                    let len = address - block.borrow().address;
                    block.borrow_mut().length = len;
                    let next = blocks.get(address);
                    next.borrow_mut().preds.push(block.clone());
                    block.borrow_mut().succs.push(next.clone());
                    block = next;
                } else if let Some(branch) = branch
                    && label.is_none_or(|label| branch.1 <= label)
                {
                    // branch was earliest, equal, or no label at all
                    let b = &self.insts[branch.1];
                    block.borrow_mut().branch_len = b.length;
                    let len = branch.0 - block.borrow().address;
                    block.borrow_mut().length = len;
                    if b.inst_type != InstType::Return
                        && b.inst_type != InstType::Invalid
                        && let Some(target) = platform.target(b)
                    {
                        // branch has a dest
                        let next = blocks.get(target);
                        next.borrow_mut().preds.push(block.clone());
                        block.borrow_mut().succs.push(next.clone());
                    }
                    if b.inst_type == InstType::Jump
                        || b.inst_type == InstType::Return
                        || b.inst_type == InstType::Invalid
                    {
                        // branch doesn't continue
                        let next = self
                            .labels
                            .range((Bound::Excluded(branch.1), Bound::Unbounded))
                            .next();
                        let next = if next.is_some_and(|next| *next == address) {
                            self.labels
                                .range((Bound::Excluded(branch.1), Bound::Unbounded))
                                .next()
                        } else {
                            next
                        };
                        if let Some(next) = next {
                            address = *next;
                            block = blocks.get(address);
                        } else {
                            done = true;
                        }
                    } else {
                        // branch continues
                        let next = blocks.get(*branch.0);
                        next.borrow_mut().preds.push(block.clone());
                        block.borrow_mut().succs.push(next.clone());
                        block = next;
                        address = block.borrow().address;
                    }
                } else {
                    // no more labels or branches
                    let last_key = self.insts.last_key_value().unwrap();
                    let len = last_key.0 + last_key.1.length - block.borrow().address;
                    block.borrow_mut().length = len;
                    done = true;
                }
            }
            blocks.sort();
        }

        for (idx, section) in self.sections.iter().enumerate() {
            self.lines.push(CpuLine {
                comment: format!("Section {}", idx + 1),
                ..Default::default()
            });
            self.lines.extend(self.dis_section(
                section.start,
                section.start + section.file.len() as u32,
                &blocks,
                platform,
            ));
        }
    }

    fn dis_section(
        &self,
        from: u32,
        to: u32,
        blocks: &BlockCache,
        platform: &impl Platform,
    ) -> Vec<CpuLine> {
        let mut lines = vec![];
        let mut address = from;
        for b in blocks.iter() {
            let b = b.borrow();
            if b.address + b.length <= address {
                continue;
            }
            if address < b.address {
                let last = cmp::min(to, b.address);
                lines.extend(self.dump_hex(address, last));
                address = last;
            }
            if address == b.address {
                let mut count = 0;
                let mut preds = String::new();
                for pred in b.preds.iter() {
                    let pred = pred.borrow();
                    if pred.address + pred.length != b.address && count < 4 {
                        count += 1;
                        let adr_str =
                            platform.address(&(pred.address + pred.length - pred.branch_len));
                        if pred.address < b.address {
                            preds.push_str(&format!("\u{25b2}{} ", adr_str));
                        } else {
                            preds.push_str(&format!("\u{25bc}{} ", adr_str));
                        }
                    }
                }
                if count > 0 {
                    lines.push(CpuLine {
                        address,
                        comment: preds,
                        ..Default::default()
                    });
                }
            }
            while address < to && address < b.address + b.length {
                lines.push(platform.disassemble(address, &self.insts[&address]));
                address += self.insts[&address].length;
            }
            if address < b.address + b.length {
                break;
            }
        }
        if address < to {
            lines.extend(self.dump_hex(address, to));
        }
        lines
    }

    fn dump_hex(&self, from: u32, to: u32) -> Vec<CpuLine> {
        let mut lines = vec![];
        if let Some(section) = self.get_address(from) {
            let mut offset = (from - section.start) as usize;
            let mut from = from;
            for _ in (from..to).step_by(16) {
                let mut hex = String::new();
                let mut ascii = String::new();
                let start = from;
                let skip = from & 0xf;
                for x in 0..16 {
                    if x == 8 {
                        hex.push(' ');
                        ascii.push(' ');
                    }
                    if x < skip {
                        hex.push_str("   ");
                        ascii.push(' ');
                    } else if from < to && offset < section.file.len() {
                        let ch = section.file.r8(offset);
                        offset += 1;
                        hex.push(HEX[usize::from(ch >> 4) & 0xf]);
                        hex.push(HEX[usize::from(ch & 0xf)]);
                        hex.push(' ');
                        ascii.push(if (32..127).contains(&ch) {
                            char::from_u32(u32::from(ch)).unwrap_or('.')
                        } else {
                            '.'
                        });
                        from += 1;
                    } else {
                        hex.push_str("   ");
                    }
                }
                lines.push(CpuLine {
                    address: start,
                    hex,
                    comment: ascii,
                    ..Default::default()
                });
            }
        }
        lines
    }

    fn get_address(&self, addr: u32) -> Option<&Section> {
        self.sections
            .iter()
            .find(|&s| addr >= s.start && addr < s.start + s.file.len() as u32)
            .map(|v| v as _)
    }

    pub fn save(
        &self,
        out: &mut std::fs::File,
        _ctx: Context,
        platform: &impl Platform,
    ) -> Result<(), std::io::Error> {
        for line in self.lines.iter() {
            let addr = platform.address(&line.address);
            let raw = addr.as_bytes();
            out.write_all(raw)?;
            out.write_all(line.hex.as_bytes())?;
            out.write_all(line.opcode.as_bytes())?;
            out.write_all(line.operand.as_bytes())?;
            out.write_all(line.comment.as_bytes())?;
            let nl = "\n".as_bytes();
            out.write_all(nl)?;
        }
        Ok(())
    }

    pub fn ui(&mut self, ui: &mut eframe::egui::Ui, _ctx: Context, platform: &impl Platform) {
        egui::ScrollArea::vertical()
            .id_salt(&self.id_salt)
            .show_viewport(ui, |ui, visible| {
                let baserect = ui.available_rect_before_wrap();
                let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                let line_height = font_id.size;

                let minrect =
                    baserect.with_max_y(baserect.top() + self.lines.len() as f32 * font_id.size);
                ui.allocate_rect(minrect, Sense::hover());

                ui.input(|i| {
                    if i.key_pressed(Key::ArrowUp) || i.key_pressed(Key::K) {
                        self.jump_cursor = Some(self.cursor.saturating_sub(1));
                    }
                    if i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::J) {
                        self.jump_cursor = Some(cmp::min(
                            self.cursor.saturating_add(1),
                            self.lines.len().saturating_sub(1),
                        ));
                    }
                    if i.key_pressed(Key::PageUp) || (i.key_pressed(Key::K) && i.modifiers.shift) {
                        self.jump_cursor = Some(self.cursor.saturating_sub(20));
                    }
                    if i.key_pressed(Key::PageDown) || (i.key_pressed(Key::J) && i.modifiers.shift)
                    {
                        self.jump_cursor = Some(cmp::min(
                            self.cursor.saturating_add(20),
                            self.lines.len().saturating_sub(1),
                        ));
                    }
                });

                let mut pos = baserect.left_top();
                for (idx, line) in self.lines.iter().enumerate() {
                    pos.x = baserect.left();
                    let rect = Rect {
                        min: pos,
                        max: Pos2 {
                            x: baserect.right(),
                            y: pos.y + line_height,
                        },
                    };
                    if self.jump_cursor.take_if(|cursor| *cursor == idx).is_some() {
                        self.cursor = idx;
                        ui.scroll_to_rect(rect, None);
                    }
                    if pos.y - baserect.top() < visible.bottom()
                        && pos.y + line_height - baserect.top() > visible.top()
                    {
                        let addr = platform.address(&line.address);
                        let galley = ui.painter().layout_no_wrap(
                            addr,
                            font_id.clone(),
                            Color32::PLACEHOLDER,
                        );
                        let stride = galley.size().x;

                        if self.cursor == idx {
                            ui.painter().rect_filled(rect, 0, HILITE_COLOR);
                            ui.painter().galley(pos, galley, Color32::WHITE);
                        } else {
                            ui.painter().galley(pos, galley, ADDRESS_COLOR);
                        }
                        pos.x += stride;

                        let galley = ui.painter().layout_no_wrap(
                            line.hex.clone(),
                            font_id.clone(),
                            Color32::PLACEHOLDER,
                        );
                        let stride = galley.size().x;
                        if self.cursor == idx {
                            ui.painter().galley(pos, galley, Color32::WHITE);
                        } else {
                            ui.painter().galley(pos, galley, HEX_COLOR);
                        }

                        pos.x += stride;

                        let galley = ui.painter().layout_no_wrap(
                            line.opcode.clone(),
                            font_id.clone(),
                            Color32::PLACEHOLDER,
                        );
                        let stride = galley.size().x;
                        if self.cursor == idx {
                            ui.painter().galley(pos, galley, Color32::WHITE);
                        } else {
                            ui.painter().galley(pos, galley, OPCODE_COLOR);
                        }

                        pos.x += stride;

                        let galley = ui.painter().layout_no_wrap(
                            line.operand.clone(),
                            font_id.clone(),
                            Color32::PLACEHOLDER,
                        );
                        let stride = galley.size().x;
                        if self.cursor == idx {
                            ui.painter().galley(pos, galley, Color32::WHITE);
                        } else {
                            ui.painter().galley(pos, galley, OPERAND_COLOR);
                        }

                        pos.x += stride;

                        let galley = ui.painter().layout_no_wrap(
                            line.comment.clone(),
                            font_id.clone(),
                            Color32::PLACEHOLDER,
                        );
                        if self.cursor == idx {
                            ui.painter().galley(pos, galley, Color32::WHITE);
                        } else {
                            ui.painter().galley(pos, galley, COMMENT_COLOR);
                        }
                    }
                    pos.y += line_height;
                }
            });
    }

    pub fn status(&self, ui: &mut egui::Ui, platform: &impl Platform) {
        if let Some(line) = self.lines.get(self.cursor) {
            ui.label(platform.address(&line.address));
        }
    }

    pub fn gestalt(&self) -> u8 {
        let mut mask = gestalt::JUMP | gestalt::FIND | gestalt::EXPORT | gestalt::MENU;
        if self.search.is_some() {
            mask |= gestalt::FIND_NEXT | gestalt::FIND_PREV;
        }
        mask
    }

    pub fn jump(&mut self, to: usize, _ctx: Context) {
        self.jump_cursor = Some(0);
        for (idx, line) in self.lines.iter().enumerate() {
            self.jump_cursor = Some(idx);
            if line.address >= to as u32 {
                return;
            }
        }
    }

    pub fn find(&mut self, key: &str, ctx: Context) {
        self.search = Some(key.to_string());
        self.find_next(ctx);
    }

    pub fn find_next(&mut self, _ctx: Context) {
        if let Some(search) = self.search.as_ref() {
            for cursor in self.cursor + 1..self.lines.len() {
                if let Some(row) = self.lines.get(cursor)
                    && (row.hex.contains(search)
                        || row.opcode.contains(search)
                        || row.operand.contains(search)
                        || row.comment.contains(search))
                {
                    self.jump_cursor = Some(cursor);
                    return;
                }
            }
        }
        // not found
    }

    pub fn find_prev(&mut self, _ctx: Context) {
        if let Some(search) = self.search.as_ref() {
            for cursor in (0..self.cursor.saturating_sub(1)).rev() {
                if let Some(row) = self.lines.get(cursor)
                    && (row.hex.contains(search)
                        || row.opcode.contains(search)
                        || row.operand.contains(search)
                        || row.comment.contains(search))
                {
                    self.jump_cursor = Some(cursor);
                    return;
                }
            }
        }
    }
}

const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
