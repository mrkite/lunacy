/*

General Disassembly Viewer widget
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{CpuLine, CpuState, InstType, Instruction},
    lunacy::{Context, Handle, Message, gestalt},
    platforms::{BlockCache, Platform},
    screen::{Color, Point, Rect, Screen},
    widgets::{KeyMap, Scroll, Widget},
};
use crossterm::event::KeyCode;
use std::{
    cmp,
    collections::{BTreeMap, BTreeSet},
    fmt::Write as fmtwrite,
    io::Write,
    ops::Bound,
};

const ADDRESS_COLOR: Color = Some(0x9d_ac_ae);
const HEX_COLOR: Color = Some(0x82_8d_ab);
const OPCODE_COLOR: Color = Some(0xb2_ee_be);
const OPERAND_COLOR: Color = Some(0x9b_cf_c2);
const COMMENT_COLOR: Color = Some(0x15_5e_95);
const ASCII_COLOR: Color = Some(0xb9_90_95);
const DIVIDER_COLOR: Color = Some(0x27_44_72);
const BG: Color = Some(0x00_00_01);
const DIVIDER: char = '│';

struct Section {
    start: u32,
    handle: Handle,
}

enum Render {
    Head(String),
    Asm(CpuLine),
    Ref(u32, Vec<u32>),
    Hex(u32, u32),
}

#[derive(Default, PartialEq)]
enum Search {
    #[default]
    None,
    Hex(Vec<u8>),
    Str(String),
}

#[derive(Default)]
pub struct Disassembly {
    bounds: Rect,
    search: Search,
    scroll: Scroll,
    cursorx: usize,
    cursory: usize,
    labels: BTreeSet<u32>,
    branches: BTreeMap<u32, u32>,
    sections: Vec<Section>,
    insts: BTreeMap<u32, Instruction>,
    lines: Vec<Render>,
    stack: Vec<(usize, usize)>,
}

impl Disassembly {
    pub fn add_section(&mut self, name: &str, start: u32, data: &[u8]) {
        self.sections.push(Section {
            start,
            handle: Handle::new_from_array(data, name, ""),
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
                            &section.handle,
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
        Ok(())
    }

    pub fn disassemble(&mut self, platform: &impl Platform) {
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
            self.lines
                .push(Render::Head(format!("Section {}", idx + 1)));
            self.lines.extend(
                self.dis_section(
                    section.start,
                    section
                        .start
                        .saturating_add(u32::try_from(section.handle.len()).unwrap_or(u32::MAX)),
                    &blocks,
                    platform,
                ),
            );
        }
    }

    fn dis_section(
        &self,
        from: u32,
        to: u32,
        blocks: &BlockCache,
        platform: &impl Platform,
    ) -> Vec<Render> {
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
                let mut preds = vec![];
                for pred in &b.preds {
                    let pred = pred.borrow();
                    if pred.address + pred.length != b.address {
                        preds.push(pred.address + pred.length - pred.branch_len);
                    }
                }
                if !preds.is_empty() {
                    lines.push(Render::Ref(address, preds));
                }
            }
            while address < to && address < b.address + b.length {
                lines.push(Render::Asm(
                    platform.disassemble(address, &self.insts[&address]),
                ));
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

    fn dump_hex(&self, from: u32, to: u32) -> Vec<Render> {
        let mut lines = vec![];
        if self.get_address(from).is_some() {
            let skip = from & 0xf;
            let from = if skip != 0 {
                let end = cmp::min(to, from + 16 - skip);
                lines.push(Render::Hex(from, end));
                end
            } else {
                from
            };
            for from in (from..to).step_by(16) {
                let end = cmp::min(to, from + 16);
                lines.push(Render::Hex(from, end));
            }
        }
        lines
    }

    fn get_address(&self, addr: u32) -> Option<&Section> {
        self.sections
            .iter()
            .find(|&s| {
                addr >= s.start
                    && addr
                        < s.start
                            .saturating_add(u32::try_from(s.handle.len()).unwrap_or(u32::MAX))
            })
            .map(|v| v as _)
    }

    pub fn export(
        &self,
        out: &mut std::fs::File,
        platform: &impl Platform,
    ) -> Result<(), std::io::Error> {
        for line in &self.lines {
            match line {
                Render::Asm(line) => {
                    let addr = platform.address(line.address);
                    writeln!(
                        out,
                        "{addr} | {}{}{}    {}",
                        line.hex, line.opcode, line.operand, line.comment
                    )?;
                }
                Render::Head(label) => {
                    writeln!(out, "    {label}")?;
                }
                Render::Ref(addr, refs) => {
                    write!(out, "    ")?;
                    for from in refs {
                        if from < addr {
                            write!(out, "\u{25b2}{}", platform.address(*from))?;
                        } else {
                            write!(out, "\u{25bc}{}", platform.address(*from))?;
                        }
                        write!(out, " ")?;
                    }
                    writeln!(out)?;
                }
                Render::Hex(from, to) => {
                    if let Some(section) = self.get_address(*from) {
                        let offset = (from - section.start) as usize;
                        let skip = usize::try_from(*from & 0xf).unwrap_or_default();
                        let end = usize::try_from(((*to - 1) & 0xf) + 1).unwrap_or_default();
                        let addr = platform.address(*from);
                        write!(out, "{addr} | ")?;
                        let mut ascii = String::new();
                        for _ in 0..skip {
                            write!(out, "   ")?;
                            ascii.push(' ');
                        }
                        for x in skip..end {
                            let ch = section.handle.r8(offset + x - skip);
                            write!(out, "{ch:02x} ")?;
                            if (32..127).contains(&ch) {
                                ascii.push(ch as char);
                            } else {
                                ascii.push('.');
                            }
                        }
                        for _ in end..16 {
                            write!(out, "   ")?;
                        }
                        writeln!(out, "| {ascii}")?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn layout(&mut self, constraint: &Rect) {
        self.bounds = constraint.remove_scroll();
        self.scroll.layout(&constraint.to_scroll());
        self.scroll.set_range(
            0,
            i32::try_from(self.lines.len()).unwrap_or(i32::MAX),
            self.bounds.h(),
        );
        self.scroll_to_cursor();
    }

    pub fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect, platform: &impl Platform) {
        self.scroll.draw(s, origin, clip);
        let range = clip.intersect(&self.bounds.offset(origin));

        for y in range.t..range.b {
            let row = y - (self.bounds.t + origin.y) + self.scroll.get();
            if let Ok(idx) = usize::try_from(row)
                && let Some(line) = self.lines.get(idx)
            {
                match line {
                    Render::Asm(_) => self.draw_instruction(y, s, origin, &range, platform),
                    Render::Hex(_, _) => self.draw_hex(y, s, origin, &range, platform),
                    Render::Head(line) => {
                        for (x, ch) in (self.bounds.l + origin.x + 4..).zip(line.chars()) {
                            if x >= range.l && x < range.r {
                                if idx == self.cursory {
                                    s.draw(x, y, ch, BG, COMMENT_COLOR);
                                } else {
                                    s.draw(x, y, ch, COMMENT_COLOR, BG);
                                }
                            }
                        }
                    }
                    Render::Ref(_, _) => self.draw_refs(y, s, origin, &range, platform),
                }
            }
        }
    }

    pub fn key(&mut self, code: KeyCode, ctx: &Context, platform: &impl Platform) -> bool {
        let page = isize::try_from(self.bounds.h().saturating_sub(1)).unwrap_or(1);
        match code {
            KeyCode::Up | KeyCode::Char('k') => self.cursory = self.cursory.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => self.cursory = self.cursory.saturating_add(1),
            KeyCode::Left | KeyCode::Char('h') => match self.lines.get(self.cursory) {
                Some(Render::Hex(from, _)) => {
                    if self.cursorx > (from & 0xf) as usize {
                        self.cursorx -= 1;
                    } else {
                        self.cursory = self.cursory.saturating_sub(1);
                        self.cursorx = 15;
                    }
                }
                Some(Render::Ref(_, refs)) => {
                    if self.cursorx > 0 {
                        self.cursorx -= 1;
                    } else {
                        self.cursorx = refs.len().saturating_sub(1);
                    }
                }
                _ => {}
            },
            KeyCode::Right | KeyCode::Char('l') => match self.lines.get(self.cursory) {
                Some(Render::Hex(_, to)) => {
                    if self.cursorx < ((to - 1) & 0xf) as usize {
                        self.cursorx = self.cursorx.saturating_add(1);
                    } else {
                        self.cursorx = 0;
                        self.cursory = self.cursory.saturating_add(1);
                    }
                }
                Some(Render::Ref(_, refs)) => {
                    if self.cursorx < refs.len().saturating_sub(1) {
                        self.cursorx += 1;
                    } else {
                        self.cursorx = 0;
                    }
                }
                _ => {}
            },
            KeyCode::PageUp | KeyCode::Char('K') => {
                self.cursory = self.cursory.saturating_sub_signed(page);
            }
            KeyCode::PageDown | KeyCode::Char('J') => {
                self.cursory = self.cursory.saturating_add_signed(page);
            }
            KeyCode::Home => self.cursory = 0,
            KeyCode::End => self.cursory = self.lines.len(),
            KeyCode::Enter | KeyCode::Char(' ') => match self.lines.get(self.cursory) {
                Some(Render::Asm(line)) => {
                    if let Some(target) = line.target
                        && self.get_address(target).is_some()
                    {
                        self.jump(target, &ctx.clone(), platform);
                    }
                }
                Some(Render::Ref(_, refs)) => {
                    if let Some(target) = refs.get(self.cursorx)
                        && self.get_address(*target).is_some()
                    {
                        self.jump(*target, &ctx.clone(), platform);
                    }
                }
                _ => {}
            },
            KeyCode::Backspace => {
                if let Some((dx, dy)) = self.stack.pop() {
                    self.cursorx = dx;
                    self.cursory = dy;
                } else {
                    return false;
                }
            }
            _ => return false,
        }
        self.scroll_to_cursor();
        self.status(ctx, platform);
        true
    }

    pub fn mouse(
        &mut self,
        pt: &Point,
        is_down: bool,
        ctx: &Context,
        platform: &impl Platform,
    ) -> bool {
        if self.bounds.inside(pt.x, pt.y) {
            self.cursory =
                usize::try_from(self.scroll.get() + pt.y - self.bounds.t).unwrap_or_default();
            let col = pt.x - self.bounds.l;
            match self.lines.get(self.cursory) {
                Some(Render::Hex(from, _)) => {
                    let addr = platform.address(*from);
                    let addr_len = i32::try_from(addr.len().saturating_add(3)).unwrap_or(i32::MAX);
                    let hex_len = addr_len.saturating_add(51);
                    let ascii_len = hex_len.saturating_add(16);

                    if col < addr_len {
                        self.cursorx = 0;
                    } else if col < hex_len {
                        let col = usize::try_from(col - addr_len).unwrap_or_default();
                        if col >= 24 {
                            self.cursorx = col.saturating_sub(1) / 3;
                        } else {
                            self.cursorx = col / 3;
                        }
                    } else if col < ascii_len {
                        self.cursorx = usize::try_from(col - hex_len).unwrap_or_default();
                    } else {
                        self.cursorx = 15;
                    }
                }
                Some(Render::Ref(_, refs)) => {
                    self.cursorx = 0;
                    let mut left = 4;
                    for from in refs {
                        let len = i32::try_from(platform.address(*from).chars().count() + 1)
                            .unwrap_or_default();
                        if col >= left && col < left + len {
                            break;
                        }
                        self.cursorx += 1;
                        left += len;
                    }
                    self.cursorx = self.cursorx.clamp(0, refs.len().saturating_sub(1));
                }
                _ => {}
            }

            self.scroll_to_cursor();
            self.status(ctx, platform);
            true
        } else {
            self.scroll.mouse(pt, is_down, ctx)
        }
    }

    fn status(&self, ctx: &Context, platform: &impl Platform) {
        if let Some(addr) = self.cursor_to_address() {
            ctx.post(Message::Status(platform.address(addr)));
        }
    }

    fn cursor_to_address(&self) -> Option<u32> {
        self.lines.get(self.cursory).and_then(|line| match line {
            Render::Asm(line) => Some(line.address),
            Render::Hex(from, _) => Some(*from + u32::try_from(self.cursorx).unwrap_or_default()),
            _ => None,
        })
    }

    pub fn gestalt(&self) -> u8 {
        let mut mask = gestalt::JUMP | gestalt::FIND | gestalt::EXPORT;
        if self.search != Search::None {
            mask |= gestalt::FIND_NEXT | gestalt::FIND_PREV;
        }
        mask
    }

    pub fn jump(&mut self, dest: u32, ctx: &Context, platform: &impl Platform) {
        self.stack.push((self.cursorx, self.cursory));
        self.cursory = 0;
        self.cursorx = 0;
        for (idx, line) in self.lines.iter().enumerate() {
            self.cursory = idx;
            match line {
                Render::Asm(line) => {
                    if line.address >= dest {
                        break;
                    }
                }
                Render::Hex(_, to) if dest < *to => {
                    self.cursorx = (dest & 0xf) as usize;
                    break;
                }
                _ => {}
            }
        }
        self.scroll_to_cursor();
        self.status(ctx, platform);
    }

    pub fn find(&mut self, key: &str, ctx: &Context, platform: &impl Platform) {
        if key.is_empty() {
            self.search = Search::None;
        } else {
            // assume key is a series of hex values
            let mut ishex = true;
            let mut byte = None;
            let mut search = vec![];
            for ch in key.chars() {
                if ch.is_ascii_hexdigit() {
                    let nib = if ch > '9' {
                        (ch as u8 & 0xf) + 9
                    } else {
                        ch as u8 & 0xf
                    };
                    if let Some(b) = byte {
                        search.push((b << 4) | nib);
                        byte = None;
                    } else {
                        byte = Some(nib);
                    }
                } else if ch.is_whitespace() {
                    if let Some(b) = byte {
                        search.push(b);
                        byte = None;
                    }
                } else {
                    ishex = false;
                }
            }
            if ishex {
                if let Some(b) = byte {
                    search.push(b);
                }
                self.search = Search::Hex(search);
            } else {
                self.search = Search::Str(key.to_string());
            }
            self.find_next(ctx, platform);
        }
    }

    pub fn find_next(&mut self, ctx: &Context, platform: &impl Platform) {
        match &self.search {
            Search::Str(search) => {
                let next_hex = self.find_next_hex(search.as_bytes());
                for cursor in self.cursory.saturating_add(1)..self.lines.len() {
                    match self.lines.get(cursor) {
                        Some(Render::Asm(row)) => {
                            if let Some(nxt) = next_hex
                                && nxt < row.address
                            {
                                self.jump(nxt, ctx, platform);
                                return;
                            } else if row.hex.contains(search)
                                || row.opcode.contains(search)
                                || row.operand.contains(search)
                                || row.comment.contains(search)
                            {
                                self.cursory = cursor;
                                self.scroll_to_cursor();
                                self.status(ctx, platform);
                                return;
                            }
                        }
                        Some(Render::Head(head)) => {
                            if head.contains(search) {
                                self.cursory = cursor;
                                self.scroll_to_cursor();
                                self.status(ctx, platform);
                                return;
                            }
                        }
                        Some(Render::Hex(_, to)) => {
                            if let Some(nxt) = next_hex
                                && nxt < *to
                            {
                                self.jump(nxt, ctx, platform);
                                return;
                            }
                        }
                        _ => {}
                    }
                }
                ctx.post(Message::NotFound);
            }
            Search::Hex(search) => {
                if let Some(addr) = self.find_next_hex(search) {
                    self.jump(addr, ctx, platform);
                } else {
                    ctx.post(Message::NotFound);
                }
            }
            Search::None => {}
        }
    }

    fn find_next_hex(&self, search: &[u8]) -> Option<u32> {
        if let Some(from) = self.cursor_to_address()
            && let Some(section) = self.get_address(from)
            && let Some(cursor) = section.handle.find(
                usize::try_from(from.saturating_add(1).saturating_sub(section.start))
                    .unwrap_or_default(),
                search,
            )
        {
            Some(u32::try_from(cursor).unwrap_or_default() + section.start)
        } else {
            None
        }
    }

    pub fn find_prev(&mut self, ctx: &Context, platform: &impl Platform) {
        match &self.search {
            Search::Str(search) => {
                let prev_hex = self.find_prev_hex(search.as_bytes());
                for cursor in (0..self.cursory.saturating_sub(1)).rev() {
                    match self.lines.get(cursor) {
                        Some(Render::Asm(row)) => {
                            if let Some(prv) = prev_hex
                                && prv >= row.address
                            {
                                self.jump(prv, ctx, platform);
                                return;
                            } else if row.hex.contains(search)
                                || row.opcode.contains(search)
                                || row.operand.contains(search)
                                || row.comment.contains(search)
                            {
                                self.cursory = cursor;
                                self.scroll_to_cursor();
                                self.status(ctx, platform);
                                return;
                            }
                        }
                        Some(Render::Head(head)) => {
                            if head.contains(search) {
                                self.cursory = cursor;
                                self.scroll_to_cursor();
                                self.status(ctx, platform);
                                return;
                            }
                        }
                        Some(Render::Hex(from, _)) => {
                            if let Some(prv) = prev_hex
                                && prv >= *from
                            {
                                self.jump(prv, ctx, platform);
                                return;
                            }
                        }
                        _ => {}
                    }
                }
                ctx.post(Message::NotFound);
            }
            Search::Hex(search) => {
                if let Some(addr) = self.find_prev_hex(search) {
                    self.jump(addr, ctx, platform);
                } else {
                    ctx.post(Message::NotFound);
                }
            }
            Search::None => {}
        }
    }

    fn find_prev_hex(&self, search: &[u8]) -> Option<u32> {
        if let Some(from) = self.cursor_to_address()
            && let Some(section) = self.get_address(from)
            && let Some(cursor) = section.handle.rfind(
                usize::try_from(from.saturating_sub(section.start)).unwrap_or_default(),
                search,
            )
        {
            Some(u32::try_from(cursor).unwrap_or_default() + section.start)
        } else {
            None
        }
    }

    pub fn get_key_map(&self) -> KeyMap {
        let mut m = KeyMap::default();
        if let Some(Render::Asm(line)) = self.lines.get(self.cursory)
            && let Some(target) = line.target
            && self.get_address(target).is_some()
        {
            m.insert("a⏎|␣", "follow");
        }

        m
    }

    pub fn can_disassemble(&self) -> Option<u32> {
        if let Some(Render::Hex(from, _)) = self.lines.get(self.cursory) {
            Some(
                from + u32::try_from(self.cursorx)
                    .unwrap_or_default()
                    .saturating_sub(*from & 0xf),
            )
        } else {
            None
        }
    }

    pub fn cur_address(&self) -> Option<u32> {
        match self.lines.get(self.cursory) {
            Some(Render::Hex(from, _)) => Some(
                from + u32::try_from(self.cursorx)
                    .unwrap_or_default()
                    .saturating_sub(*from & 0xf),
            ),
            Some(Render::Asm(line)) => {
                if let Some(target) = line.target {
                    Some(target)
                } else {
                    Some(line.address)
                }
            }
            Some(Render::Ref(_, refs)) => refs.get(self.cursorx).copied(),
            _ => None,
        }
    }

    fn scroll_to_cursor(&mut self) {
        if !self.lines.is_empty() {
            self.cursory = self.cursory.clamp(0, self.lines.len().saturating_sub(1));
            match self.lines.get(self.cursory) {
                Some(Render::Hex(from, to)) => {
                    self.cursorx = self
                        .cursorx
                        .clamp((*from & 0xf) as usize, ((*to - 1) & 0xf) as usize);
                }
                Some(Render::Ref(_, refs)) => self.cursorx = self.cursorx.clamp(0, refs.len()),
                _ => self.cursorx = 0,
            }
            if let Ok(cursor) = i32::try_from(self.cursory) {
                if cursor < self.scroll.get() {
                    self.scroll.set(cursor);
                }
                let page = self.bounds.h().saturating_sub(1);
                if cursor >= self.scroll.get() + page {
                    self.scroll.set(cursor.saturating_sub(page));
                }
            }
        }
    }

    fn draw_instruction(
        &self,
        y: i32,
        s: &mut Screen,
        origin: &Point,
        range: &Rect,
        platform: &impl Platform,
    ) {
        let row = y - (self.bounds.t + origin.y) + self.scroll.get();
        if let Ok(idx) = usize::try_from(row)
            && let Some(Render::Asm(line)) = self.lines.get(idx)
        {
            let addr = platform.address(line.address);
            let addr_len = i32::try_from(addr.chars().count()).unwrap_or(i32::MAX);
            let hex_len = addr_len
                .saturating_add(3)
                .saturating_add(i32::try_from(line.hex.chars().count()).unwrap_or(i32::MAX));
            let opcode_len = hex_len
                .saturating_add(i32::try_from(line.opcode.chars().count()).unwrap_or(i32::MAX));
            let operand_len = opcode_len.saturating_add(
                i32::try_from(line.operand.chars().count().saturating_add(4)).unwrap_or(i32::MAX),
            );
            let comment_len = operand_len
                .saturating_add(i32::try_from(line.comment.chars().count()).unwrap_or(i32::MAX));

            for x in range.l..range.r {
                let col = x - (self.bounds.l + origin.x);
                if col < 0 {
                    s.draw(x, y, ' ', ADDRESS_COLOR, BG);
                } else if col < addr_len {
                    let ch = addr
                        .chars()
                        .nth(usize::try_from(col).unwrap_or_default())
                        .unwrap_or(' ');
                    if idx == self.cursory {
                        s.draw(x, y, ch, BG, ADDRESS_COLOR);
                    } else {
                        s.draw(x, y, ch, ADDRESS_COLOR, BG);
                    }
                } else if col < addr_len + 3 {
                    if col == addr_len + 1 {
                        s.draw(x, y, DIVIDER, DIVIDER_COLOR, BG);
                    } else {
                        s.draw(x, y, ' ', DIVIDER_COLOR, BG);
                    }
                } else if col < hex_len {
                    let col = usize::try_from(col - (addr_len + 3)).unwrap_or_default();
                    let ch = line.hex.chars().nth(col).unwrap_or(' ');
                    s.draw(x, y, ch, HEX_COLOR, BG);
                } else if col < opcode_len {
                    let col = usize::try_from(col - hex_len).unwrap_or_default();
                    let ch = line.opcode.chars().nth(col).unwrap_or(' ');
                    s.draw(x, y, ch, OPCODE_COLOR, BG);
                } else if col < operand_len {
                    let col = usize::try_from(col - opcode_len).unwrap_or_default();
                    let ch = line.operand.chars().nth(col).unwrap_or(' ');
                    if idx == self.cursory
                        && let Some(target) = line.target
                        && self.get_address(target).is_some()
                    {
                        s.draw(x, y, ch, BG, OPERAND_COLOR);
                    } else {
                        s.draw(x, y, ch, OPERAND_COLOR, BG);
                    }
                } else if col < comment_len
                    && let Ok(col) = usize::try_from(col - operand_len)
                    && let ch = line.comment.chars().nth(col).unwrap_or(' ')
                {
                    s.draw(x, y, ch, COMMENT_COLOR, BG);
                } else {
                    s.draw(x, y, ' ', COMMENT_COLOR, BG);
                }
            }
        }
    }

    fn draw_hex(
        &self,
        y: i32,
        s: &mut Screen,
        origin: &Point,
        range: &Rect,
        platform: &impl Platform,
    ) {
        let row = y - (self.bounds.t + origin.y) + self.scroll.get();
        if let Ok(idx) = usize::try_from(row)
            && let Some(Render::Hex(from, to)) = self.lines.get(idx)
            && let Some(section) = self.get_address(*from)
        {
            let offset = (from - section.start) as usize;
            let skip = usize::try_from(*from & 0xf).unwrap_or_default();
            let end = usize::try_from(((*to - 1) & 0xf) + 1).unwrap_or_default();
            let addr = platform.address(*from);
            let addr_len = i32::try_from(addr.len()).unwrap_or(i32::MAX);
            let hex_len = addr_len + 52; // 61
            let ascii_len = hex_len + 18; // 79
            for x in range.l..range.r {
                let col = x - (self.bounds.l + origin.x);
                if col < 0 {
                    s.draw(x, y, ' ', ADDRESS_COLOR, BG);
                } else if col < addr_len {
                    if let Ok(col) = usize::try_from(col)
                        && let Some(ch) = addr.chars().nth(col)
                    {
                        if idx == self.cursory {
                            s.draw(x, y, ch, BG, ADDRESS_COLOR);
                        } else {
                            s.draw(x, y, ch, ADDRESS_COLOR, BG);
                        }
                    }
                } else if col < addr_len + 3 {
                    if col == addr_len + 1 {
                        s.draw(x, y, DIVIDER, DIVIDER_COLOR, BG);
                    } else {
                        s.draw(x, y, ' ', DIVIDER_COLOR, BG);
                    }
                } else if col < hex_len {
                    let col = usize::try_from(col - (addr_len + 3)).unwrap_or_default();
                    let (nib, col) = if col >= 24 {
                        ((col - 1) % 3, (col - 1) / 3)
                    } else {
                        (col % 3, col / 3)
                    };
                    if col >= skip && col < end && offset + col < section.handle.len() {
                        let ch = section.handle.r8(offset + col - skip);
                        let (fg, bg) = if idx == self.cursory && col == self.cursorx {
                            (BG, HEX_COLOR)
                        } else {
                            (HEX_COLOR, BG)
                        };
                        match nib {
                            0 => s.draw(x, y, HEX[usize::from(ch >> 4)], fg, bg),
                            1 => s.draw(x, y, HEX[usize::from(ch & 0xf)], fg, bg),
                            _ => s.draw(x, y, ' ', HEX_COLOR, BG),
                        }
                    } else {
                        s.draw(x, y, ' ', HEX_COLOR, BG);
                    }
                } else if col < hex_len + 2 {
                    if col == hex_len {
                        s.draw(x, y, DIVIDER, DIVIDER_COLOR, BG);
                    } else {
                        s.draw(x, y, ' ', DIVIDER_COLOR, BG);
                    }
                } else if col < ascii_len {
                    let col = usize::try_from(col - (hex_len + 2)).unwrap_or_default();
                    if col >= skip && col < end && offset + col < section.handle.len() {
                        let ch = section.handle.r8(offset + col - skip);
                        let (fg, bg) = if idx == self.cursory && col == self.cursorx {
                            (BG, ASCII_COLOR)
                        } else {
                            (ASCII_COLOR, BG)
                        };
                        if (32..127).contains(&ch) {
                            s.draw(x, y, ch as char, fg, bg);
                        } else {
                            s.draw(x, y, '.', fg, bg);
                        }
                    }
                } else {
                    s.draw(x, y, ' ', COMMENT_COLOR, BG);
                }
            }
        }
    }
    fn draw_refs(
        &self,
        y: i32,
        s: &mut Screen,
        origin: &Point,
        range: &Rect,
        platform: &impl Platform,
    ) {
        let row = y - (self.bounds.t + origin.y) + self.scroll.get();
        if let Ok(idx) = usize::try_from(row)
            && let Some(Render::Ref(addr, refs)) = self.lines.get(idx)
        {
            let mut startx = None;
            let mut endx = None;

            let mut str = String::new();
            str.push_str("    ");
            for (idx, from) in refs.iter().enumerate() {
                if idx == self.cursorx {
                    startx = Some(str.chars().count());
                }
                if from < addr {
                    let _ = write!(str, "\u{25b2}{}", platform.address(*from));
                } else {
                    let _ = write!(str, "\u{25bc}{}", platform.address(*from));
                }
                if idx == self.cursorx {
                    endx = Some(str.chars().count());
                }
                str.push(' ');
            }

            for x in range.l..range.r {
                let col = usize::try_from(x - (self.bounds.l + origin.x)).unwrap_or(usize::MAX);
                let ch = str.chars().nth(col).unwrap_or(' ');
                if idx == self.cursory
                    && let Some(sx) = startx
                    && col >= sx
                    && let Some(ex) = endx
                    && col < ex
                {
                    s.draw(x, y, ch, BG, ADDRESS_COLOR);
                } else {
                    s.draw(x, y, ch, ADDRESS_COLOR, BG);
                }
            }
        }
    }
}

const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
