/*

DOS platform
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{Cpu, Cpu86, CpuLine, CpuState, Instruction, State86},
    lunacy::{Context, FilePicker, Handle, Message},
    platforms::{Platform, api::Api, labelwin::LabelWin, mzheader::MZHeader},
    screen::{Point, Rect, Screen},
    widgets::{Disassembly, KeyMap, Widget, Windowable},
};
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, HashMap},
    error::Error,
    fs::File,
    io::{BufReader, BufWriter, Write},
    ops::Bound,
};

pub struct Dos {
    api: Api,
    disassembly: Disassembly,
    platform: DosPlatform,
    header: MZHeader,
    picker: FilePicker,
    label: LabelWin,
}

impl Dos {
    pub fn new(filename: &str) -> Self {
        let api = Api::new(include_bytes!("../../api/dos.dat"));
        let mut picker = FilePicker::default();
        picker.set(&format!("{filename}.lunacy"));
        Self {
            api,
            picker,
            disassembly: Disassembly::default(),
            platform: DosPlatform::default(),
            header: MZHeader::default(),
            label: LabelWin::default(),
        }
    }
    pub fn set_mz(&mut self, header: Vec<(String, String)>) {
        self.header.set(header);
    }

    pub fn add_section(&mut self, name: &str, start: u32, data: &[u8]) {
        self.disassembly.add_section(name, start, data);
    }

    pub fn add_entry(&mut self, state: State86) -> Result<(), &'static str> {
        self.disassembly.add_entry(state, &self.platform)
    }

    pub fn add_segment(&mut self, seg: u16) {
        self.platform.add_segment(seg);
    }

    pub fn disassemble(&mut self) {
        self.disassembly.disassemble(&self.platform);
    }

    fn load_state(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(path) = self.picker.get()
            && path.exists()
        {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let platform = serde_json::from_reader(reader)?;
            self.platform = platform;
            for entry in &self.platform.entries {
                let state = State86::new(*entry);
                self.disassembly.add_entry(state, &self.platform)?;
            }
            self.disassembly.disassemble(&self.platform);
        }
        Ok(())
    }

    fn save_state(&self) -> Result<Option<String>, Box<dyn Error>> {
        if let Some(path) = self.picker.get() {
            let file = File::create(path)?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer(&mut writer, &self.platform)?;
            writer.flush()?;
            Ok(Some(format!(
                "Saved {}",
                path.file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
            )))
        } else {
            Ok(None)
        }
    }
}

impl Widget for Dos {
    fn layout(&mut self, constraint: &Rect) {
        self.api.layout(constraint);
        self.header.layout(constraint);
        self.picker.layout(constraint);
        self.label.layout(constraint);
        self.disassembly.layout(constraint);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.disassembly.draw(s, origin, clip, &self.platform);
        if self.api.visible() {
            self.api.draw(s, origin, clip);
        }
        if self.header.visible() {
            self.header.draw(s, origin, clip);
        }
        if self.picker.visible() {
            self.picker.draw(s, origin, clip);
        }
        if self.label.visible() {
            self.label.draw(s, origin, clip);
        }
    }

    fn gestalt(&self) -> u8 {
        if self.api.visible() {
            self.api.gestalt()
        } else if self.header.visible() {
            self.header.gestalt()
        } else {
            self.disassembly.gestalt()
        }
    }

    fn jump(&mut self, to: usize, ctx: &Context) {
        if let Ok(to) = u32::try_from(to) {
            self.disassembly.jump(to, ctx, &self.platform);
        }
    }

    fn find(&mut self, val: &str, ctx: &Context) {
        self.disassembly.find(val, ctx, &self.platform);
    }

    fn find_next(&mut self, ctx: &Context) {
        self.disassembly.find_next(ctx, &self.platform);
    }

    fn find_prev(&mut self, ctx: &Context) {
        self.disassembly.find_prev(ctx, &self.platform);
    }

    fn export(&self, out: &mut File) -> Result<(), std::io::Error> {
        self.disassembly.export(out, &self.platform)
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if self.api.visible() {
            return self.api.key(code, modifiers, ctx);
        }
        if self.header.visible() {
            return self.header.key(code, modifiers, ctx);
        }
        if self.picker.visible() {
            return if self.picker.key(code, modifiers, ctx) {
                if let Err(err) = self.load_state() {
                    ctx.post(Message::Error(err.to_string()));
                }
                true
            } else {
                false
            };
        }
        if self.label.visible() {
            return if self.label.key(code, modifiers, ctx) {
                if let Some((addr, label)) = self.label.take() {
                    self.platform.add_label(&addr, &label);
                    self.disassembly.disassemble(&self.platform);
                }
                true
            } else {
                false
            };
        }
        match code {
            KeyCode::Char('d') => {
                if let Some(addr) = self.disassembly.can_disassemble() {
                    let state = State86::new(addr);
                    if let Err(err) = self.disassembly.add_entry(state, &self.platform) {
                        ctx.post(Message::Error(err.to_string()));
                    } else {
                        self.platform.add_entry(addr);
                        self.disassembly.disassemble(&self.platform);
                    }
                    true
                } else {
                    false
                }
            }
            KeyCode::Char('\'') => {
                if let Some(addr) = self.disassembly.cur_address() {
                    self.label.set(&self.platform.address(addr));
                } else {
                    self.label.set("");
                }
                self.label.show();
                true
            }
            KeyCode::Char('o') => {
                self.picker.show();
                true
            }
            KeyCode::Char('s') => {
                match self.save_state() {
                    Ok(Some(success)) => {
                        ctx.post(Message::Status(success));
                    }
                    Ok(None) => {}
                    Err(err) => ctx.post(Message::Error(err.to_string())),
                }
                true
            }
            KeyCode::Char('i') => {
                self.api.show();
                true
            }
            KeyCode::Char('z') => {
                self.header.show();
                true
            }
            _ => self.disassembly.key(code, ctx, &self.platform),
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        if self.api.visible() {
            return self.api.mouse(pt, is_down, ctx);
        }
        if self.header.visible() {
            return self.header.mouse(pt, is_down, ctx);
        }
        if self.picker.visible() {
            return if self.picker.mouse(pt, is_down, ctx) {
                if let Err(err) = self.load_state() {
                    ctx.post(Message::Error(err.to_string()));
                }
                true
            } else {
                false
            };
        }
        if self.label.visible() {
            return self.label.mouse(pt, is_down, ctx);
        }
        self.disassembly.mouse(pt, is_down, ctx, &self.platform)
    }

    fn get_key_map(&self) -> KeyMap {
        if self.api.visible() {
            return self.api.get_key_map();
        }
        if self.header.visible() {
            return self.header.get_key_map();
        }
        if self.picker.visible() {
            return self.picker.get_key_map();
        }
        if self.label.visible() {
            return self.label.get_key_map();
        }
        let mut m = self.disassembly.get_key_map();
        if self.disassembly.can_disassemble().is_some() {
            m.insert("ad", "disassemble");
        }
        m.insert("b'", "label");
        m.insert("ci", "interrupts");
        m.insert("dz", "mz header");
        if self.picker.get().is_some() {
            m.insert("os", "save state");
        } else {
            // don't clutter the help, only show this if you haven't used it yet
            m.insert("oo", "open state");
        }
        m
    }
}

#[derive(Default, Serialize, Deserialize)]
struct DosPlatform {
    #[serde(skip)]
    cpu: Cpu86,
    segments: BTreeSet<u16>,
    entries: Vec<u32>,
    labels: HashMap<u32, String>,
}

impl Platform for DosPlatform {
    fn address(&self, address: u32) -> String {
        if let Some(addr) = self.labels.get(&address) {
            return addr.clone();
        }
        let min_seg = u16::try_from((address >> 4) & 0xffff).unwrap_or_default();
        let seg = self
            .segments
            .range((Bound::Unbounded, Bound::Included(min_seg)))
            .last();
        seg.map_or_else(
            || format!("{:04x}:{:04x}", address >> 4, address & 0xf),
            |seg| format!("{:04x}:{:04x}", seg, address - (u32::from(*seg) << 4)),
        )
    }

    fn target(&self, inst: &Instruction) -> Option<u32> {
        self.cpu.target(inst)
    }

    fn disassemble(&self, address: u32, inst: &Instruction) -> CpuLine {
        self.cpu.disassemble(address, inst, &|addr| {
            if let Some(addr) = self.labels.get(&addr) {
                return Some(addr.clone());
            }
            let min_seg = u16::try_from((addr >> 4) & 0xffff).unwrap_or_default();
            let seg = self
                .segments
                .range((Bound::Unbounded, Bound::Included(min_seg)))
                .last();
            seg.map(|seg| format!("{:04x}:{:04x}", seg, addr - (u32::from(*seg) << 4)))
        })
    }

    fn decode(
        &self,
        handle: &Handle,
        offset: usize,
        state: &mut dyn CpuState,
    ) -> Result<Instruction, &'static str> {
        self.cpu.decode(handle, offset, state)
    }
}

impl DosPlatform {
    pub fn add_segment(&mut self, seg: u16) {
        self.segments.insert(seg);
    }
    pub fn add_entry(&mut self, addr: u32) {
        self.entries.push(addr);
    }
    pub fn add_label(&mut self, addr: &str, label: &str) {
        let mut seg: u32 = 0;
        let mut ofs: u32 = 0;
        for ch in addr.chars() {
            if ch.is_ascii_hexdigit() {
                let nib = if ch > '9' {
                    (ch as u8 & 0xf) + 9
                } else {
                    ch as u8 & 0xf
                };
                ofs <<= 4;
                ofs |= u32::from(nib);
            } else if ch == ':' {
                seg = ofs;
                ofs = 0;
            }
        }
        let abs = (seg << 4) + ofs;
        self.labels.insert(abs, label.to_string());
    }
}
