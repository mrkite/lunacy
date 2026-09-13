/*

DOS platform
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{Cpu, Cpu86, CpuLine, CpuState, Instruction, State86},
    lunacy::{Context, File, Message},
    platforms::{GetAddress, Platform, api::Api},
    widgets::{Disassembly, Widget},
};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub struct Dos {
    api: Api,
    get_addr: Option<GetAddress>,
    disassembly: Disassembly,
    platform: DosPlatform,
    header: Vec<(String, String)>,
    showing_header: bool,
    showing_api: bool,
}

impl Dos {
    pub fn new(id_salt: &str) -> Self {
        let api = Api::new(include_bytes!("../../api/dos.dat"));
        Self {
            api,
            get_addr: None,
            disassembly: Disassembly::new(id_salt),
            platform: DosPlatform::default(),
            header: vec![],
            showing_header: false,
            showing_api: false,
        }
    }

    pub fn set_mz(&mut self, header: Vec<(String, String)>) {
        self.header = header;
    }

    pub fn add_section(&mut self, name: &str, start: u32, data: &[u8]) {
        self.disassembly.add_section(name, start, data);
    }

    pub fn add_entry(&mut self, state: State86) -> Result<(), &'static str> {
        self.disassembly.add_entry(state, &self.platform)
    }
}

impl Widget for Dos {
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
            if !self.header.is_empty() && ui.button("Show MZ...").clicked() {
                self.showing_header = true;
            }
            if ui.button("Interrupts...").clicked() {
                self.showing_api = true;
            }
        });

        self.get_addr.take_if(|get_addr| {
            if let Some((done, addr)) = get_addr.prompt(ui) {
                if done {
                    let state = State86 {
                        seg: (addr >> 4) as u16,
                        ofs: (addr & 0xf) as u16,
                    };
                    if let Err(err) = self.disassembly.add_entry(state, &self.platform) {
                        ctx.post(Message::Error(err.to_owned()));
                    }
                }
                true
            } else {
                false
            }
        });

        if self.showing_header {
            let mut open = true;
            egui::Window::new("MZ").open(&mut open).show(ui, |ui| {
                let table = TableBuilder::new(ui)
                    .column(Column::initial(100.0))
                    .column(Column::remainder());
                table.body(|body| {
                    body.rows(18.0, self.header.len(), |mut row| {
                        let row_index = row.index();
                        let hdr = &self.header[row_index];
                        row.col(|ui| {
                            ui.strong(&hdr.0);
                        });
                        row.col(|ui| {
                            ui.label(&hdr.1);
                        });
                    });
                });
            });
            self.showing_header = open;
        }
        if self.showing_api {
            self.showing_api = !self.api.show(ui, ctx);
        }
    }
}

#[derive(Default)]
struct DosPlatform {
    cpu: Cpu86,
}

impl Platform for DosPlatform {
    fn address(&self, address: &u32) -> String {
        format!("{:04x}:{:04x} ", address >> 16, address & 0xffff)
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
        self.cpu.decode(file, offset, state)
    }
}
