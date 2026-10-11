/*

Mac platform
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{Cpu, Cpu68k, CpuLine, CpuState, InstType, Instruction, Operands68k, State68k},
    lunacy::{Context, FilePicker, Handle, Message},
    platforms::{Platform, api::Api, fingerprints::Fingerprint, labelwin::LabelWin},
    screen::{Point, Rect, Screen},
    widgets::{Disassembly, KeyMap, Widget, Windowable},
};
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufReader, BufWriter, Write},
};

pub struct Mac {
    api: Api,
    a5: HashMap<u32, u32>,
    disassembly: Disassembly,
    platform: MacPlatform,
    picker: FilePicker,
    label: LabelWin,
}

impl Mac {
    pub fn new(filename: &str) -> Self {
        let api = Api::new(include_bytes!("../../api/mac.dat"));
        let prints = api.fingerprints();
        let mut picker = FilePicker::default();
        picker.set(&format!("{filename}.lunacy"));
        Self {
            api,
            picker,
            a5: HashMap::new(),
            disassembly: Disassembly::default(),
            platform: MacPlatform::new(prints),
            label: LabelWin::default(),
        }
    }
    pub fn add_section(&mut self, name: &str, start: u32, data: &[u8]) {
        self.disassembly.add_section(name, start, data);
    }

    pub fn add_entry(&mut self, state: State68k) -> Result<(), &'static str> {
        self.disassembly.add_entry(state, &self.platform)
    }

    pub fn disassemble(&mut self) {
        self.disassembly.disassemble(&self.platform);
    }

    pub fn set_a5(&mut self, a5: &HashMap<u32, u32>) {
        a5.clone_into(&mut self.a5);
    }

    fn load_state(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(path) = self.picker.get()
            && path.exists()
        {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let platform = serde_json::from_reader(reader)?;
            self.platform = platform;
            self.platform.prints = self.api.fingerprints();
            for entry in &self.platform.entries {
                let state = State68k::new(*entry);
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

impl Widget for Mac {
    fn layout(&mut self, constraint: &Rect) {
        self.api.layout(constraint);
        self.disassembly.layout(constraint);
        self.picker.layout(constraint);
        self.label.layout(constraint);
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.disassembly.draw(s, origin, clip, &self.platform);
        if self.api.visible() {
            self.api.draw(s, origin, clip);
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
        } else {
            self.disassembly.gestalt()
        }
    }

    fn jump(&mut self, to: usize, ctx: &Context) {
        if let Ok(addr) = u32::try_from(to) {
            self.disassembly.jump(addr, ctx, &self.platform);
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

    fn export(&self, out: &mut std::fs::File) -> Result<(), std::io::Error> {
        self.disassembly.export(out, &self.platform)
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if self.api.visible() {
            return self.api.key(code, modifiers, ctx);
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
                    let state = State68k::new(addr);
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
            KeyCode::Char('a') => {
                self.api.show();
                true
            }
            _ => self.disassembly.key(code, ctx, &self.platform),
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        if self.api.visible() {
            return self.api.mouse(pt, is_down, ctx);
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
        m.insert("ca", "api");
        if self.picker.get().is_some() {
            m.insert("os", "save state");
        } else {
            // don't clutter the help
            m.insert("oo", "open state");
        }
        m
    }
}

#[derive(Default, Serialize, Deserialize)]
struct MacPlatform {
    #[serde(skip)]
    cpu: Cpu68k,
    #[serde(skip)]
    prints: Fingerprint,
    entries: Vec<u32>,
    labels: HashMap<u32, String>,
}

impl MacPlatform {
    fn new(prints: Fingerprint) -> Self {
        Self {
            cpu: Cpu68k::default(),
            prints,
            entries: vec![],
            labels: HashMap::default(),
        }
    }
}

impl Platform for MacPlatform {
    fn address(&self, address: u32) -> String {
        if let Some(addr) = self.labels.get(&address) {
            return addr.clone();
        }
        format!("{address:06x}")
    }

    fn target(&self, inst: &Instruction) -> Option<u32> {
        self.cpu.target(inst)
    }

    fn disassemble(&self, address: u32, inst: &Instruction) -> CpuLine {
        self.cpu.disassemble(address, inst, &|addr| {
            if let Some(addr) = self.labels.get(&addr) {
                return Some(addr.clone());
            }
            None
        })
    }

    fn decode(
        &self,
        handle: &Handle,
        offset: usize,
        state: &mut dyn CpuState,
    ) -> Result<Instruction, &'static str> {
        let mut p = &self.prints;
        let mut pofs = offset;
        while let Some(child) = p.next(handle.r8(pofs)) {
            pofs += 1;
            if let Some(name) = &child.name {
                let length = pofs - offset;
                let ins_len = u32::try_from(length).unwrap_or_default();
                let addr = state.address() + ins_len;
                state.jump(addr);
                return Ok(Instruction {
                    name: name.to_owned(),
                    inst_type: InstType::Special,
                    length: ins_len,
                    raw: handle.read(offset, length).to_vec(),
                    operands: Box::new(Operands68k::default()),
                });
            }
            p = child;
        }
        self.cpu.decode(handle, offset, state)
    }
}

impl MacPlatform {
    pub fn add_entry(&mut self, addr: u32) {
        self.entries.push(addr);
    }
    pub fn add_label(&mut self, addr: &str, label: &str) {
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
            }
        }
        self.labels.insert(ofs, label.to_string());
    }
}
