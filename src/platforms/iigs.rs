/*

IIgs platform
@copyright 2026 Sean Kasun

*/

use crate::{
    cpus::{
        Address65816, Cpu, Cpu65816, CpuLine, CpuState, InstType, Instruction, Operand65816,
        State65816,
    },
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
    sync::LazyLock,
};

pub struct IIgs {
    api: Api,
    disassembly: Disassembly<Address65816>,
    platform: IIgsPlatform,
    picker: FilePicker,
    label: LabelWin<Address65816>,
}

impl IIgs {
    pub fn new(filename: &str) -> Self {
        let api = Api::new(include_bytes!("../../api/iigs.dat"));
        let prints = api.fingerprints();
        let mut picker = FilePicker::default();
        picker.set(&format!("{filename}.lunacy"));
        Self {
            api,
            picker,
            disassembly: Disassembly::default(),
            platform: IIgsPlatform::new(prints),
            label: LabelWin::default(),
        }
    }
    pub fn add_section(&mut self, name: &str, start: Address65816, data: &[u8]) {
        self.disassembly.add_section(name, start, data);
    }

    pub fn add_entry(&mut self, state: State65816) -> Result<(), &'static str> {
        self.disassembly.add_entry(state, &self.platform)
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
            self.platform.prints = self.api.fingerprints();
            for entry in &self.platform.entries {
                let state = State65816::new(entry.0, entry.1);
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

impl Widget for IIgs {
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
            self.disassembly
                .jump(Address65816::Normal(addr), ctx, &self.platform);
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
                    self.platform.add_label(addr, &label);
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
                    // TODO: show dialog to choose cpu flags
                    let state = State65816::new(addr, 0);
                    if let Err(err) = self.disassembly.add_entry(state, &self.platform) {
                        ctx.post(Message::Error(err.to_string()));
                    } else {
                        self.platform.add_entry(addr, 0);
                        self.disassembly.disassemble(&self.platform);
                    }
                    true
                } else {
                    false
                }
            }
            KeyCode::Char('\'') => {
                self.label.set(self.disassembly.cur_address());
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
            // don't clutter the help, only show if we haven't use it yet
            m.insert("oo", "open state");
        }
        m
    }
}

#[derive(Default, Serialize, Deserialize)]
struct IIgsPlatform {
    #[serde(skip)]
    cpu: Cpu65816,
    #[serde(skip)]
    prints: Fingerprint,
    entries: Vec<(Address65816, u32)>,
    labels: HashMap<Address65816, String>,
}

impl IIgsPlatform {
    pub fn new(prints: Fingerprint) -> Self {
        Self {
            cpu: Cpu65816::default(),
            prints,
            entries: vec![],
            labels: HashMap::default(),
        }
    }
}

impl Platform<Address65816> for IIgsPlatform {
    fn address(&self, address: Address65816) -> String {
        if let Some(addr) = self.labels.get(&address) {
            return addr.clone();
        }
        address.to_string()
    }

    fn target(&self, inst: &Instruction) -> Option<Address65816> {
        self.cpu.target(inst)
    }

    fn disassemble(&self, address: Address65816, inst: &Instruction) -> CpuLine<Address65816> {
        self.cpu.disassemble(address, inst, &|addr| {
            if let Some(addr) = self.labels.get(&addr) {
                return Some(addr.clone());
            }
            if let Address65816::Normal(v) = addr {
                ADDRESSES
                    .get(&v)
                    .or(ADDRESSES.get(&(v & 0xffff)))
                    .map(|name| format!(" ;{name}"))
            } else if let Address65816::B(v) = addr {
                ADDRESSES.get(&u32::from(v)).map(|name| format!(" ;{name}"))
            } else {
                None
            }
        })
    }

    fn decode(
        &self,
        file: &Handle,
        offset: usize,
        state: &mut dyn CpuState<Address65816>,
    ) -> Result<Instruction, &'static str> {
        let mut p = &self.prints;
        let mut pofs = offset;

        let mut shift = 0;
        let mut operands = Operand65816::default();

        while let Some(child) = p.next(file.r8(pofs)) {
            pofs += 1;
            if child.is_wild() {
                operands.or(u32::from(file.r8(pofs)) << shift);
                shift += 8;
            }
            if let Some(name) = &child.name {
                let length = pofs - offset;
                let ins_len = length;
                let addr = state.address() + u32::try_from(ins_len).unwrap_or_default();
                state.jump(addr);
                return Ok(Instruction {
                    name: format!("{name} "),
                    inst_type: InstType::Special,
                    length: ins_len,
                    raw: file.read(offset, length).to_vec(),
                    operands: Box::new(operands),
                });
            }
            p = child;
        }
        self.cpu.decode(file, offset, state)
    }
}

impl IIgsPlatform {
    pub fn add_entry(&mut self, addr: Address65816, flags: u32) {
        self.entries.push((addr, flags));
    }
    pub fn add_label(&mut self, addr: Address65816, label: &str) {
        self.labels.insert(addr, label.to_string());
    }
}

static ADDRESSES: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0x03d0, "Enter BASIC");
    m.insert(0x03d2, "Reconnect DOS");
    m.insert(0x03d9, "Cow Sound");
    m.insert(0x03ea, "Reconnect IO");
    m.insert(0x03f2, "Control Reset Vector");
    m.insert(0x03f5, "Ampersand Vector");
    m.insert(0x03f8, "Control Y Vector");
    m.insert(0xc000, "KBD/80STOREOFF");
    m.insert(0xc001, "80STOREON");
    m.insert(0xc002, "RDMAINRAM");
    m.insert(0xc003, "RDCARDRAM");
    m.insert(0xc004, "WRMAINRAM");
    m.insert(0xc005, "WRCARDRAM");
    m.insert(0xc006, "SETSLOTCXROM");
    m.insert(0xc007, "SETINTCXROM");
    m.insert(0xc008, "SETSTDZP");
    m.insert(0xc009, "SETALTZP");
    m.insert(0xc00a, "SETINTC3ROM");
    m.insert(0xc00b, "SETSLOTC3ROM");
    m.insert(0xc00c, "CLR80VID");
    m.insert(0xc00d, "SET80VID");
    m.insert(0xc00e, "CLRALTCHAR");
    m.insert(0xc00f, "SETALTCHAR");
    m.insert(0xc010, "KBDSTRB");
    m.insert(0xc011, "RDLCBNK2");
    m.insert(0xc012, "RDLCRAM");
    m.insert(0xc013, "RDRAMRD");
    m.insert(0xc014, "RDRAMWRT");
    m.insert(0xc015, "RDCXROM");
    m.insert(0xc016, "RDALTZP");
    m.insert(0xc017, "RDC3ROM");
    m.insert(0xc018, "RD80STORE");
    m.insert(0xc019, "RDVBL");
    m.insert(0xc01a, "RDTEXT");
    m.insert(0xc01b, "RDMIXED");
    m.insert(0xc01c, "RDPAGE2");
    m.insert(0xc01d, "RDHIRES");
    m.insert(0xc01e, "RDALTCHAR");
    m.insert(0xc01f, "RD80VID");
    m.insert(0xc020, "TAPEOUT");
    m.insert(0xc021, "MONOCOLOR");
    m.insert(0xc022, "TBCOLOR");
    m.insert(0xc023, "VGCINT");
    m.insert(0xc024, "MOUSEDATA");
    m.insert(0xc025, "KEYMODREG");
    m.insert(0xc026, "DATAREG");
    m.insert(0xc027, "KMSTATUS");
    m.insert(0xc028, "ROMBANK");
    m.insert(0xc029, "NEWVIDEO");
    m.insert(0xc02b, "LANGSEL");
    m.insert(0xc02c, "CHARROM");
    m.insert(0xc02d, "SLTROMSEL");
    m.insert(0xc02e, "VERTCNT");
    m.insert(0xc02f, "HORIZCNT");
    m.insert(0xc030, "SPRK");
    m.insert(0xc031, "DISKREG");
    m.insert(0xc032, "SCANINT");
    m.insert(0xc033, "CLOCKDATA");
    m.insert(0xc034, "CLOCKCTL");
    m.insert(0xc035, "SHADOW");
    m.insert(0xc036, "CYAREG");
    m.insert(0xc037, "BMAREG");
    m.insert(0xc038, "SCCBREG");
    m.insert(0xc03a, "SCCBDATA");
    m.insert(0xc03c, "SCCADATA");
    m.insert(0xc03d, "SOUNDDATA");
    m.insert(0xc03e, "SOUNDADRL");
    m.insert(0xc03f, "SOUNDADRH");
    m.insert(0xc040, "STROBE");
    m.insert(0xc044, "MMDELTAX");
    m.insert(0xc045, "MMDELTAY");
    m.insert(0xc046, "DIAGTYPE");
    m.insert(0xc047, "CLRVBLINT");
    m.insert(0xc04f, "EMUBYTE");
    m.insert(0xc050, "TXTCLR");
    m.insert(0xc051, "TXTSET");
    m.insert(0xc052, "MIXCLR");
    m.insert(0xc053, "MIXSET");
    m.insert(0xc054, "TXTPAGE1");
    m.insert(0xc055, "TXTPAGE2");
    m.insert(0xc056, "LORES");
    m.insert(0xc057, "HIRES");
    m.insert(0xc058, "CLRAN0");
    m.insert(0xc059, "SETAN0");
    m.insert(0xc05a, "CLRAN1");
    m.insert(0xc05b, "SETAN1");
    m.insert(0xc05c, "CLRAN2");
    m.insert(0xc05d, "SETAN2");
    m.insert(0xc05e, "DHIRESON");
    m.insert(0xc05f, "DHIRESOFF");
    m.insert(0xc060, "BUTN3");
    m.insert(0xc061, "RDBTN0");
    m.insert(0xc062, "BUTN1");
    m.insert(0xc063, "RD63");
    m.insert(0xc064, "PADDL0");
    m.insert(0xc065, "PADDL1");
    m.insert(0xc066, "PADDL2");
    m.insert(0xc067, "PADDL3");
    m.insert(0xc068, "STATEREG");
    m.insert(0xc06d, "TESTREG");
    m.insert(0xc06e, "CLRTM");
    m.insert(0xc06f, "ENTM");
    m.insert(0xc070, "PTRIG");
    m.insert(0xc073, "BANKSEL");
    m.insert(0xc077, "BLOSSOM");
    m.insert(0xc07e, "IOUDISON");
    m.insert(0xc07f, "IOUDISOFF");
    m.insert(0xc081, "ROMIN");
    m.insert(0xc083, "LCBANK2");
    m.insert(0xc085, "ROMIN");
    m.insert(0xc087, "LCBANK2");
    m.insert(0xc0e0, "PH0 off");
    m.insert(0xc0e1, "PH0 on");
    m.insert(0xc0e2, "PH1 off");
    m.insert(0xc0e3, "PH1 on");
    m.insert(0xc0e4, "PH2 off");
    m.insert(0xc0e5, "PH2 on");
    m.insert(0xc0e6, "PH3 off");
    m.insert(0xc0e7, "PH3 on");
    m.insert(0xc0e8, "motor off");
    m.insert(0xc0e9, "motor on");
    m.insert(0xc0ea, "drive 1");
    m.insert(0xc0eb, "drive 2");
    m.insert(0xc0ec, "Q6 off");
    m.insert(0xc0ed, "Q6 on");
    m.insert(0xc0ee, "Q7 off");
    m.insert(0xc0ef, "Q7 on");
    m.insert(0xc311, "AUXMOVE");
    m.insert(0xc314, "XFER");
    m.insert(0xcfff, "CLRROM");
    m.insert(0xe5ad, "NEW");
    m.insert(0xf317, "RESUME");
    m.insert(0xf3de, "HGR");
    m.insert(0xf800, "PLOT");
    m.insert(0xf80e, "PLOT1");
    m.insert(0xf819, "HLINE");
    m.insert(0xf828, "VLINE");
    m.insert(0xf832, "CLRSCR");
    m.insert(0xf836, "CLRTOP");
    m.insert(0xf847, "GBASCALC");
    m.insert(0xf85f, "NXTCOL");
    m.insert(0xf864, "SETCOL");
    m.insert(0xf871, "SCRN");
    m.insert(0xf88c, "INSDS1");
    m.insert(0xf88e, "INSDS2");
    m.insert(0xf890, "GET816LEN");
    m.insert(0xf8d0, "INSTDSP");
    m.insert(0xf940, "PRNTYX");
    m.insert(0xf941, "PRNTAX");
    m.insert(0xf944, "PRNTX");
    m.insert(0xf948, "PRBLNK");
    m.insert(0xf94a, "PRBL2");
    m.insert(0xf953, "PCADJ");
    m.insert(0xf962, "TEXT2COPY");
    m.insert(0xfa40, "OLDIRQ");
    m.insert(0xfa4c, "BREAK");
    m.insert(0xfa59, "OLDBRK");
    m.insert(0xfa62, "RESET");
    m.insert(0xfaa6, "PWRUP");
    m.insert(0xfaba, "SLOOP");
    m.insert(0xfad7, "REGDSP");
    m.insert(0xfb19, "RTBL");
    m.insert(0xfb1e, "PREAD");
    m.insert(0xfb21, "PREAD4");
    m.insert(0xfb2f, "INIT");
    m.insert(0xfb39, "SETTXT");
    m.insert(0xfb40, "SETGR");
    m.insert(0xfb4b, "SETWND");
    m.insert(0xfb51, "SETWND2");
    m.insert(0xfb5b, "TABV");
    m.insert(0xfb60, "APPLEII");
    m.insert(0xfb6f, "SETPWRC");
    m.insert(0xfb78, "VIDWAIT");
    m.insert(0xfb88, "KBDWAIT");
    m.insert(0xfbb3, "VERSION");
    m.insert(0xfbbf, "ZIDBYTE2");
    m.insert(0xfbc1, "BASCALC");
    m.insert(0xfbdd, "BELL1");
    m.insert(0xfbe2, "BELL12");
    m.insert(0xfbe4, "BELL2");
    m.insert(0xfbf0, "STORADV");
    m.insert(0xfbf4, "ADVANCE");
    m.insert(0xfbfd, "VIDOUT");
    m.insert(0xfc10, "BS");
    m.insert(0xfc1a, "UP");
    m.insert(0xfc22, "VTAB");
    m.insert(0xfc24, "VTABZ");
    m.insert(0xfc2c, "ESC");
    m.insert(0xfc42, "CLREOP");
    m.insert(0xfc58, "HOME");
    m.insert(0xfc62, "CR");
    m.insert(0xfc66, "LF");
    m.insert(0xfc70, "SCROLL");
    m.insert(0xfc9c, "CLREOL");
    m.insert(0xfc9e, "CLREOLZ");
    m.insert(0xfca8, "WAIT");
    m.insert(0xfcb4, "NXTA4");
    m.insert(0xfcba, "NXTA1");
    m.insert(0xfcc9, "HEADR");
    m.insert(0xfd0c, "RDKEY");
    m.insert(0xfd10, "FD10");
    m.insert(0xfd18, "RDKEY1");
    m.insert(0xfd1b, "KEYIN");
    m.insert(0xfd35, "RDCHAR");
    m.insert(0xfd67, "GETLNZ");
    m.insert(0xfd6a, "GETLN");
    m.insert(0xfd6c, "GETLN0");
    m.insert(0xfd6f, "GETLN1");
    m.insert(0xfd8b, "CROUT1");
    m.insert(0xfd8e, "CROUT");
    m.insert(0xfd92, "PRA1");
    m.insert(0xfdda, "PRBYTE");
    m.insert(0xfde3, "PRHEX");
    m.insert(0xfded, "COUT");
    m.insert(0xfdf0, "COUT1");
    m.insert(0xfdf6, "COUTZ");
    m.insert(0xfe1f, "IDROUTINE");
    m.insert(0xfe2c, "MOVE");
    m.insert(0xfe5e, "LIST");
    m.insert(0xfe80, "INVERSE");
    m.insert(0xfe84, "NORMAL");
    m.insert(0xfe89, "SETKBD");
    m.insert(0xfe8b, "INPORT");
    m.insert(0xfe93, "SETVID");
    m.insert(0xfe95, "OUTPORT");
    m.insert(0xfeb6, "GO");
    m.insert(0xfecd, "WRITE");
    m.insert(0xfefd, "READ");
    m.insert(0xff2d, "PRERR");
    m.insert(0xff3a, "BELL");
    m.insert(0xff3f, "RESTORE");
    m.insert(0xff44, "RSTR1");
    m.insert(0xff4a, "SAVE");
    m.insert(0xff4c, "SAV1");
    m.insert(0xff58, "IORTS");
    m.insert(0xff59, "OLDRST");
    m.insert(0xff65, "MON");
    m.insert(0xff69, "MONZ");
    m.insert(0xff6c, "MONZ2");
    m.insert(0xff70, "MONZ4");
    m.insert(0xff8a, "DIG");
    m.insert(0xffa7, "GETNUM");
    m.insert(0xffad, "NXTCHR");
    m.insert(0xffbe, "TOSUB");
    m.insert(0xffc7, "ZMODE");
    m.insert(0xe0_1e00, "ToSlab");
    m.insert(0xe0_1e04, "StdText");
    m.insert(0xe0_1e08, "StdLine");
    m.insert(0xe0_1e0c, "StdRect");
    m.insert(0xe0_1e10, "StdRRect");
    m.insert(0xe0_1e14, "StdOval");
    m.insert(0xe0_1e18, "StdArc");
    m.insert(0xe0_1e1c, "StdPoly");
    m.insert(0xe0_1e20, "StdRgn");
    m.insert(0xe0_1e24, "StdPixels");
    m.insert(0xe0_1e28, "StdComment");
    m.insert(0xe0_1e2c, "StdTxMeas");
    m.insert(0xe0_1e30, "StdTxBnds");
    m.insert(0xe0_1e34, "StdGetPic");
    m.insert(0xe0_1e38, "StdPutPic");
    m.insert(0xe0_1e3c, "PortLoc2ZP");
    m.insert(0xe0_1e40, "UserPat2ZP");
    m.insert(0xe0_1e44, "GetPenPReady");
    m.insert(0xe0_1e48, "GetBackPReady");
    m.insert(0xe0_1e4c, "Pen2ZP");
    m.insert(0xe0_1e50, "Back2ZP");
    m.insert(0xe0_1e54, "RotatePat");
    m.insert(0xe0_1e58, "GetMaskReady");
    m.insert(0xe0_1e5c, "Mask2ZP");
    m.insert(0xe0_1e60, "ExpandMask");
    m.insert(0xe0_1e64, "RotateMask");
    m.insert(0xe0_1e68, "RotateLong");
    m.insert(0xe0_1e6c, "PreDraw");
    m.insert(0xe0_1e70, "XSetup");
    m.insert(0xe0_1e74, "DrawRectB");
    m.insert(0xe0_1e78, "DrawRect");
    m.insert(0xe0_1e7c, "SliceSetup");
    m.insert(0xe0_1e80, "vShiftState");
    m.insert(0xe0_1e84, "vXShiftStay");
    m.insert(0xe0_1e88, "vShiftOut");
    m.insert(0xe0_1e8c, "vXShiftOut");
    m.insert(0xe0_1e90, "BMakeMinRect");
    m.insert(0xe0_1e94, "MakeMinRect");
    m.insert(0xe0_1e98, "ShieldCursor");
    m.insert(0xe0_1e9c, "UnshieldCursor");
    m.insert(0xe0_1ea0, "SetUpPortRegions");
    m.insert(0xe0_1ea4, "SetUpScanPtrs");
    m.insert(0xe0_1ea8, "FreePortRegions");
    m.insert(0xe0_1eac, "Free1Region");
    m.insert(0xe0_1eb0, "Free2Regions");
    m.insert(0xe0_1eb4, "Free3Regions");
    m.insert(0xe0_1eb8, "UnlockBufferHandle");
    m.insert(0xe0_1ebc, "Init1Region");
    m.insert(0xe0_1ec0, "Init2Regions");
    m.insert(0xe0_1ec4, "Init3Regions");
    m.insert(0xe0_1ec8, "Seek1Region");
    m.insert(0xe0_1ecc, "Seek2Regions");
    m.insert(0xe0_1ed0, "Seek3Regions");
    m.insert(0xe0_1ed4, "InitRgn");
    m.insert(0xe0_1ed8, "SeekRgn");
    m.insert(0xe0_1edc, "Join1Buffer");
    m.insert(0xe0_1ee0, "Join2Buffers");
    m.insert(0xe0_1ee4, "Join3Buffers");
    m.insert(0xe0_1ee8, "SetSlabAdr");
    m.insert(0xe0_1eec, "SetSliceAdr");
    m.insert(0xe0_1ef0, "SetStuffAdr");
    m.insert(0xe0_1ef4, "BlowItUp");
    m.insert(0xe0_1ef8, "PutPictStuff");
    m.insert(0xe0_1efc, "AddToPoly");
    m.insert(0xe0_1f14, "GimmeVersion");
    m.insert(0xe0_1f18, "v_FrRgn");
    m.insert(0xe0_1f1c, "v_PRgn");
    m.insert(0xe0_1f20, "v_ERgn");
    m.insert(0xe0_1f24, "v_IRgn");
    m.insert(0xe0_1f28, "v_FRgn");
    m.insert(0xe0_1f2c, "v_FrRect");
    m.insert(0xe0_1f30, "v_PRect");
    m.insert(0xe0_1f34, "v_ERect");
    m.insert(0xe0_1f38, "v_IRect");
    m.insert(0xe0_1f3c, "v_FRect");
    m.insert(0xe0_1f40, "DChar");
    m.insert(0xe0_1f44, "DText");
    m.insert(0xe0_1f48, "BChar");
    m.insert(0xe0_1f4c, "BText");
    m.insert(0xe0_1f50, "ValidChar");
    m.insert(0xe0_1f54, "GetNextChar");
    m.insert(0xe0_1f58, "CalcDrawStatus");
    m.insert(0xe0_1f5c, "TextAndMinRect");
    m.insert(0xe0_1f60, "CalcTextBounds");
    m.insert(0xe0_1f64, "SetUpFontInfo");
    m.insert(0xe0_1f68, "SetUpBufInfo");
    m.insert(0xe0_1f6c, "GetTextHandle");
    m.insert(0xe0_1f70, "ClearBufPart");
    m.insert(0xe0_1f74, "PutChar");
    m.insert(0xe0_1f78, "BumpPen");
    m.insert(0xe0_1f7c, "CleanUpBuffer");
    m.insert(0xe0_1f80, "PutLine");
    m.insert(0xe0_1f84, "PutRgn");
    m.insert(0xe0_1f88, "PutRect");
    m.insert(0xe0_1f8c, "CullPoint");
    m.insert(0xe0_1f90, "SortPoints");
    m.insert(0xe0_1f94, "PackRgn");
    m.insert(0xe0_1f98, "RgnOp");
    m.insert(0xe0_1f9c, "PutOval");
    m.insert(0xe0_1fa0, "BumpOval");
    m.insert(0xe0_1fa4, "InitOval");
    m.insert(0xe0_1fa8, "SetTB");
    m.insert(0xe0_1fac, "ResetTB");
    m.insert(0xe0_1fb0, "StyleText");
    m.insert(0xe0_1fb4, "StyleExtents");
    m.insert(0xe0_1fb8, "DoBold");
    m.insert(0xe0_1fbc, "DoUnderline");
    m.insert(0xe0_1fc0, "StdConic");
    m.insert(0xe0_1fc4, "vDrawConic");
    m.insert(0xe0_1fc8, "vSetScaleAndMap");
    m.insert(0xe0_1fcc, "DrawCursor");
    m.insert(0xe0_1fd0, "UndrawCursor");
    m.insert(0xe0_1fd4, "vAngleFromSlope");
    m.insert(0xe0_1fd8, "vSlopeFromAngle");
    m.insert(0xe0_1fdc, "RgnBlt");
    m.insert(0xe0_1fe0, "SharedMRSetup");
    m.insert(0xe0_1fe4, "ShareShieldStuff");
    m.insert(0xe0_1fe8, "GetCharVals");
    m.insert(0xe0_1fec, "DoFntFlgMods");
    m.insert(0xe0_1ff0, "AddExtras");
    m.insert(0xe0_1ff4, "CalcSlop");
    m.insert(0xe0_1ff8, "GetFntInf");
    m.insert(0xe1_0028, "IRQSCAN");
    m.insert(0xe1_002c, "IRQSOUND");
    m.insert(0xe1_0030, "IRQVBL");
    m.insert(0xe1_0034, "IRQMOUSE");
    m.insert(0xe1_0038, "IRQQTR");
    m.insert(0xe1_003c, "IRQKBD");
    m.insert(0xe1_0040, "IRQRESPONSE");
    m.insert(0xe1_0044, "IRQSRQ");
    m.insert(0xe1_0048, "IRQDSKACC");
    m.insert(0xe1_004c, "IRQFLUSH");
    m.insert(0xe1_0050, "IRQMICRO");
    m.insert(0xe1_0054, "IRQ1SEC");
    m.insert(0xe1_0058, "IRQEXT");
    m.insert(0xe1_005c, "IRQOTHER");
    m.insert(0xe1_0060, "CUPDATE");
    m.insert(0xe1_0064, "INCBUSYFLG");
    m.insert(0xe1_0068, "DECBUSYFLG");
    m.insert(0xe1_006c, "BELLVECTOR");
    m.insert(0xe1_0070, "BREAKVECTOR");
    m.insert(0xe1_0074, "TRACEVECTOR");
    m.insert(0xe1_0078, "STEPVECTOR");
    m.insert(0xe1_0080, "TOWRITEBR");
    m.insert(0xe1_0084, "TOREADBR");
    m.insert(0xe1_0088, "TOWRITETIME");
    m.insert(0xe1_008c, "TOREADTIME");
    m.insert(0xe1_0090, "TOCTRLPANEL");
    m.insert(0xe1_0094, "TOBRAMSETUP");
    m.insert(0xe1_0098, "TOPRINTMSG8");
    m.insert(0xe1_009c, "TOPRNTMSG16");
    m.insert(0xe1_00a0, "CTRLYVECTOR");
    m.insert(0xe1_00a4, "TOTEXTPG2DA");
    m.insert(0xe1_00a8, "OS_Entry");
    m.insert(0xe1_00b0, "OS_StackEntry");
    m.insert(0xe1_00bc, "OS_Kind");
    m.insert(0xe1_00bd, "OS_Boot");
    m.insert(0xe1_00be, "OS_Flag");
    m.insert(0xe1_00c0, "MSGPOINTER");
    m.insert(0xe1_00ca, "SystemVolume");
    m.insert(0xe1_00cf, "SIOFLAG");
    m.insert(0xe1_00d0, "InSlot");
    m.insert(0xe1_00d1, "InOr");
    m.insert(0xe1_00d2, "InAnd");
    m.insert(0xe1_00d3, "OutSlot");
    m.insert(0xe1_00d4, "OutOr");
    m.insert(0xe1_00d5, "OutAnd");
    m.insert(0xe1_00d6, "DOCFlag");
    m.insert(0xe1_00d7, "IrqMouseVBL");
    m.insert(0xe1_00d8, "HBPtr");
    m.insert(0xe1_00dc, "TickCnt");
    m.insert(0xe1_00e0, "MouseSlot");
    m.insert(0xe1_00e4, "EvMgrData");
    m.insert(0xe1_00e3, "JournalPtr");
    m.insert(0xe1_00e7, "EMBStat");
    m.insert(0xe1_00e9, "EMXPos");
    m.insert(0xe1_00ea, "EMYPos");
    m.insert(0xe1_00ec, "TaskHead");
    m
});
