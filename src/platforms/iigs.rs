/*

IIgs platform
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{
        Cpu, Cpu65816, CpuLine, CpuState, InstType, Instruction, Operand65816, State68k, State65816,
    },
    lunacy::{Context, File, Message},
    platforms::{
        GetAddress, Platform,
        api::Api,
        fingerprints::{Fingerprint, Print},
    },
    widgets::{Disassembly, Widget},
};
use eframe::egui;

pub struct IIgs {
    api: Api,
    get_addr: Option<GetAddress>,
    disassembly: Disassembly,
    platform: IIgsPlatform,
    showing_api: bool,
}

impl IIgs {
    pub fn new(id_salt: &str) -> Self {
        let api = Api::new(include_bytes!("../../api/iigs.dat"));
        let prints = api.fingerprints();
        Self {
            api,
            get_addr: None,
            disassembly: Disassembly::new(id_salt),
            platform: IIgsPlatform::new(prints),
            showing_api: false,
        }
    }

    pub fn add_section(&mut self, name: &str, start: u32, data: &[u8]) {
        self.disassembly.add_section(name, start, data);
    }

    pub fn add_entry(&mut self, state: State65816) -> Result<(), &'static str> {
        self.disassembly.add_entry(state, &self.platform)
    }
}

impl Widget for IIgs {
    fn ui(&mut self, ui: &mut egui::Ui, ctx: Context) {
        self.disassembly.ui(ui, ctx, &self.platform);
    }

    fn status(&self, ui: &mut egui::Ui) {
        self.disassembly.status(ui, &self.platform);
    }

    fn gestalt(&self) -> u8 {
        self.disassembly.gestalt()
    }

    fn jump(&mut self, to: usize, ctx: Context) {
        self.disassembly.jump(to, ctx);
    }

    fn find(&mut self, val: &str, ctx: Context) {
        self.disassembly.find(val, ctx);
    }

    fn find_next(&mut self, ctx: Context) {
        self.disassembly.find_next(ctx);
    }

    fn find_prev(&mut self, ctx: Context) {
        self.disassembly.find_prev(ctx);
    }

    fn save(&self, out: &mut std::fs::File, ctx: Context) -> Result<(), std::io::Error> {
        self.disassembly.save(out, ctx, &self.platform)
    }

    fn menu(&mut self, ui: &mut egui::Ui, ctx: Context) {
        ui.menu_button("Disassembly", |ui| {
            if ui.button("Disassemble...").clicked() {
                self.get_addr = Some(GetAddress::default());
            }
            ui.separator();
            if ui.button("Symbols...").clicked() {
                self.showing_api = true;
            }
        });

        self.get_addr.take_if(|get_addr| {
            if let Some((done, addr)) = get_addr.prompt(ui) {
                if done {
                    let state = State68k { addr: addr as u32 };
                    if let Err(err) = self.disassembly.add_entry(state, &self.platform) {
                        ctx.post(Message::Error(err.to_owned()));
                    }
                }
                true
            } else {
                false
            }
        });
        if self.showing_api {
            self.showing_api = !self.api.show(ui, ctx);
        }
    }
}

struct IIgsPlatform {
    cpu: Cpu65816,
    prints: Fingerprint,
}

impl IIgsPlatform {
    pub fn new(prints: Fingerprint) -> Self {
        Self {
            cpu: Cpu65816::default(),
            prints,
        }
    }
}

impl Platform for IIgsPlatform {
    fn address(&self, address: &u32) -> String {
        format!("{:02x}/{:04x} ", address >> 16, address & 0xffff)
    }

    fn target(&self, inst: &Instruction) -> Option<u32> {
        self.cpu.target(inst)
    }

    fn disassemble(&self, address: u32, inst: &Instruction) -> CpuLine {
        self.cpu.disassemble(address, inst, &|_addr| None)
    }

    fn decode(
        &self,
        file: &File,
        offset: usize,
        state: &mut dyn CpuState,
    ) -> Result<Instruction, &'static str> {
        let mut p = &self.prints;
        let mut pofs = offset;
        while let Some(child) = p.next.get(&Print::Byte(file.r8(pofs))) {
            pofs += 1;
            if let Some(name) = &child.name {
                let length = pofs - offset;
                let addr = state.address() + length as u32;
                state.jump(addr);
                return Ok(Instruction {
                    name: name.to_owned(),
                    inst_type: InstType::Special,
                    length: length as u32,
                    raw: file.read(offset, length).to_vec(),
                    operands: Box::new(Operand65816::default()),
                });
            }
            p = child;
        }
        self.cpu.decode(file, offset, state)
    }
}
