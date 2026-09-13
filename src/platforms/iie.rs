/*

IIe platform
@copyright 2026 Sean Kasun

*/

use std::collections::HashMap;

use eframe::egui;
use lazy_static::lazy_static;

use crate::{
    cpus::{Cpu, Cpu6502, CpuLine, CpuState, Instruction, State6502},
    lunacy::{Context, File, Message},
    platforms::{GetAddress, Platform},
    widgets::{Disassembly, Widget},
};

pub struct IIe {
    get_addr: Option<GetAddress>,
    disassembly: Disassembly,
    platform: IIePlatform,
}

impl IIe {
    pub fn new(id_salt: &str) -> Self {
        Self {
            get_addr: None,
            disassembly: Disassembly::new(id_salt),
            platform: IIePlatform::default(),
        }
    }

    pub fn add_section(&mut self, name: &str, start: u32, data: &[u8]) {
        self.disassembly.add_section(name, start, data);
    }

    pub fn add_entry(&mut self, state: State6502) -> Result<(), &'static str> {
        self.disassembly.add_entry(state, &self.platform)
    }
}

impl Widget for IIe {
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
        });

        self.get_addr.take_if(|get_addr| {
            if let Some((done, addr)) = get_addr.prompt(ui) {
                if done {
                    let state = State6502 { addr: addr as u16 };
                    if let Err(err) = self.disassembly.add_entry(state, &self.platform) {
                        ctx.post(Message::Error(err.to_owned()));
                    }
                }
                true
            } else {
                false
            }
        });
    }
}

#[derive(Default)]
pub struct IIePlatform {
    cpu: Cpu6502,
}

impl Platform for IIePlatform {
    fn address(&self, address: &u32) -> String {
        format!("{:04x} ", address)
    }

    fn target(&self, inst: &Instruction) -> Option<u32> {
        self.cpu.target(inst)
    }

    fn disassemble(&self, address: u32, inst: &Instruction) -> CpuLine {
        self.cpu.disassemble(address, inst, &|addr| {
            ADDRESSES
                .get(&(addr as u16))
                .map(|name| format!(" ;{}", name))
        })
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

lazy_static! {
    static ref ADDRESSES: HashMap<u16, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0xc000, "KBD");
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
        m.insert(0xc01f, " RD80VID");
        m.insert(0xc020, "TAPEOUT");
        m.insert(0xc030, "SPKR");
        m.insert(0xc040, "STROBE");
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
        m.insert(0xc060, "TAPEIN");
        m.insert(0xc061, "BUTN0");
        m.insert(0xc062, "BUTN1");
        m.insert(0xc063, "BUTN2");
        m.insert(0xc064, "PADDL0");
        m.insert(0xc065, " PADDL1");
        m.insert(0xc066, "PADDL2");
        m.insert(0xc067, "PADDL3");
        m.insert(0xc06a, "FASTCHIP_6A");
        m.insert(0xc06b, "FASTCHIP_6B");
        m.insert(0xc06c, "FASTCHIP_6C");
        m.insert(0xc06e, "FASTCHIP_6D");
        m.insert(0xc06f, "FASTCHIP_6F");
        m.insert(0xc070, "PTRIG");
        m.insert(0xc073, "BANKSEL");
        m.insert(0xc074, "TRANSWARP");
        m.insert(0xc07e, "IOUDISON");
        m.insert(0xc07f, "IOUDISOFF");
        m.insert(0xc081, "ROMIN");
        m.insert(0xc083, "LCBANK2");
        m.insert(0xc085, "ROMIN");
        m.insert(0xc087, "LCBANK2");
        m.insert(0xc311, "AUXMOVE");
        m.insert(0xc314, "XFER");
        m.insert(0xf800, "PLOT");
        m.insert(0xf80c, "RTMASK");
        m.insert(0xf80e, "PLOT1");
        m.insert(0xf819, "HLINE");
        m.insert(0xf81c, "HLINE1");
        m.insert(0xf826, "VLINEZ");
        m.insert(0xf828, "VLINE");
        m.insert(0xf831, "RTS1");
        m.insert(0xf832, "CLRSCR");
        m.insert(0xf836, "CLRTOP");
        m.insert(0xf838, "CLRSC2");
        m.insert(0xf83c, "CLRSC3");
        m.insert(0xf847, "GBASCALC");
        m.insert(0xf856, "GBCALC");
        m.insert(0xf85f, "NXTCOL");
        m.insert(0xf864, "SETCOL");
        m.insert(0xf871, "SCRN");
        m.insert(0xf879, "SCRN2");
        m.insert(0xf87f, "RTMSKZ");
        m.insert(0xf882, "INSDS1");
        m.insert(0xf88e, "INSDS2");
        m.insert(0xf89b, "IEVEN");
        m.insert(0xf8a5, "ERR");
        m.insert(0xf8a9, "GETFMT");
        m.insert(0xf8be, "MNNDX1");
        m.insert(0xf8c2, "MNNDX2");
        m.insert(0xf8c9, "MNNDX3");
        m.insert(0xf8d0, "INSTDSP");
        m.insert(0xf8d4, "PRNTOP");
        m.insert(0xf8db, "PRNTBL");
        m.insert(0xf8f5, "PRMN1");
        m.insert(0xf8f9, "PRMN2");
        m.insert(0xf910, "PRADR1");
        m.insert(0xf914, "PRADR2");
        m.insert(0xf926, "PRADR3");
        m.insert(0xf92a, "PRADR4");
        m.insert(0xf930, "PRADR5");
        m.insert(0xf938, "RELADR");
        m.insert(0xf940, "PRNTYX");
        m.insert(0xf941, "PRNTAX");
        m.insert(0xf944, "PRNTX");
        m.insert(0xf948, "PRBLNK");
        m.insert(0xf94a, "PRBL2");
        m.insert(0xf94c, "PRBL3");
        m.insert(0xf953, "PCADJ");
        m.insert(0xf954, "PCADJ2");
        m.insert(0xf956, "PCADJ3");
        m.insert(0xf95c, "PCADJ4");
        m.insert(0xf961, "RTS2");
        m.insert(0xfa40, "IRQ");
        m.insert(0xfa4c, "BREAK");
        m.insert(0xfa59, "OLDBRK");
        m.insert(0xfa62, "RESET");
        m.insert(0xfa6f, "INITAN");
        m.insert(0xfa81, "NEWMON");
        m.insert(0xfa9b, "FIXSEV");
        m.insert(0xfaa3, "NOFIX");
        m.insert(0xfaa6, "PWRUP");
        m.insert(0xfaab, "SETPLP");
        m.insert(0xfaba, "SLOOP");
        m.insert(0xfac7, "NXTBYT");
        m.insert(0xfad7, "REGDSP");
        m.insert(0xfada, "RGDSP1");
        m.insert(0xfae4, "RDSP1");
        m.insert(0xfafd, "PWRCON");
        m.insert(0xfb02, "DSKID");
        m.insert(0xfb09, "TITLE");
        m.insert(0xfb11, "XLTBL");
        m.insert(0xfb19, "RTBL");
        m.insert(0xfb1e, "PREAD");
        m.insert(0xfb25, "PREAD2");
        m.insert(0xfb2e, "RTS2D");
        m.insert(0xfb2f, "INIT");
        m.insert(0xfb39, "SETTXT");
        m.insert(0xfb40, "SETGR");
        m.insert(0xfb4b, "SETWND");
        m.insert(0xfb60, "APPLEII");
        m.insert(0xfb65, "STITLE");
        m.insert(0xfb6f, "SETPWRC");
        m.insert(0xfb78, "VIDWAIT");
        m.insert(0xfb88, "KBDWAIT");
        m.insert(0xfb94, "NOWAIT");
        m.insert(0xfb97, "ESCOLD");
        m.insert(0xfb9b, "ESCNOW");
        m.insert(0xfba5, "ESCNEW");
        m.insert(0xfbb3, "VERSION");
        m.insert(0xfbb4, "GOTOCX");
        m.insert(0xfbc1, "BASCALC");
        m.insert(0xfbd0, "BSCLC2");
        m.insert(0xfbd9, "BELL1");
        m.insert(0xfbe4, "BELL2");
        m.insert(0xfbef, "RTS2B");
        m.insert(0xfbf0, "STOADV");
        m.insert(0xfbf4, "ADVANCE");
        m.insert(0xfbfc, "RTS3");
        m.insert(0xfbfd, "VIDOUT");
        m.insert(0xfc10, "BS");
        m.insert(0xfc1a, "UP");
        m.insert(0xfc22, "VTAB");
        m.insert(0xfc24, "VTABZ");
        m.insert(0xfc2b, "RTS4");
        m.insert(0xfc2c, "ESC1");
        m.insert(0xfc42, "CLREOP");
        m.insert(0xfc58, "HOME");
        m.insert(0xfc62, "CR");
        m.insert(0xfc66, "LF");
        m.insert(0xfc70, "SCROLL");
        m.insert(0xfc72, "XGOTOCX");
        m.insert(0xfc84, "RDCX");
        m.insert(0xfc91, "ISSLOTS");
        m.insert(0xfc99, "ISPAGE1");
        m.insert(0xfc9c, "CLREOL");
        m.insert(0xfc9e, "CLREOLZ");
        m.insert(0xfca8, "WAIT");
        m.insert(0xfca9, "WAIT2");
        m.insert(0xfcaa, "WAIT3");
        m.insert(0xfcb4, "NXTA4");
        m.insert(0xfcba, "NXTA1");
        m.insert(0xfcc8, "RTS4B");
        m.insert(0xfcc9, "HEADR");
        m.insert(0xfcd6, "WRBIT");
        m.insert(0xfcdb, "ZERDLY");
        m.insert(0xfce2, "ONEDLY");
        m.insert(0xfce5, "WRTAPE");
        m.insert(0xfcec, "RDBYTE");
        m.insert(0xfcee, "RDBYT2");
        m.insert(0xfcfa, "RD2BIT");
        m.insert(0xfcfd, "RDBIT");
        m.insert(0xfd0c, "RDKEY");
        m.insert(0xfd1b, "KEYIN");
        m.insert(0xfd21, "RDESC");
        m.insert(0xfd2f, "ESC");
        m.insert(0xfd35, "RDCHAR");
        m.insert(0xfd3d, "NOTCR");
        m.insert(0xfd5f, "NOTCR1");
        m.insert(0xfd62, "CANCEL");
        m.insert(0xfd67, "GETLNZ");
        m.insert(0xfd6a, "GETLN");
        m.insert(0xfd71, "BCKSPC");
        m.insert(0xfd75, "NXTCHAR");
        m.insert(0xfd7e, "CAPTST");
        m.insert(0xfd84, "ADDINP");
        m.insert(0xfd8e, "CROUT");
        m.insert(0xfd92, "PRA1");
        m.insert(0xfd96, "PRYX2");
        m.insert(0xfda3, "XAM8");
        m.insert(0xfdad, "MOD8CHK");
        m.insert(0xfdb3, "XAM");
        m.insert(0xfdb6, "DATACUT");
        m.insert(0xfdc5, "RTS4C");
        m.insert(0xfdc6, "XAMPM");
        m.insert(0xfdd1, "ADD");
        m.insert(0xfdda, "PRBYTE");
        m.insert(0xfde3, "PRHEX");
        m.insert(0xfde5, "PRHEXZ");
        m.insert(0xfded, "COUT");
        m.insert(0xfdf0, "COUT1");
        m.insert(0xfdf6, "COUTZ");
        m.insert(0xfe00, "BL1");
        m.insert(0xfe04, "BLANK");
        m.insert(0xfe0b, "STOR");
        m.insert(0xfe17, "RTS5");
        m.insert(0xfe18, "SETMODE");
        m.insert(0xfe1d, "SETMDZ");
        m.insert(0xfe20, "LT");
        m.insert(0xfe22, "LT2");
        m.insert(0xfe2c, "MOVE");
        m.insert(0xfe36, "VFY");
        m.insert(0xfe58, "VFYOK");
        m.insert(0xfe5e, "LIST");
        m.insert(0xfe63, "LIST2");
        m.insert(0xfe75, "A1PC");
        m.insert(0xfe78, "A1PCLP");
        m.insert(0xfe7f, "A1PCRTS");
        m.insert(0xfe80, "SETINV");
        m.insert(0xfe84, "SETNORM");
        m.insert(0xfe86, "SETIFLG");
        m.insert(0xfe89, "SETKBD");
        m.insert(0xfe8b, "INPORT");
        m.insert(0xfe8d, "INPRT");
        m.insert(0xfe93, "SETVID");
        m.insert(0xfe95, "OUTPORT");
        m.insert(0xfe97, "OUTPRT");
        m.insert(0xfe9b, "IOPRT");
        m.insert(0xfea7, "IOPRT1");
        m.insert(0xfea9, "IOPRT2");
        m.insert(0xfeaf, "CKSUMFIX");
        m.insert(0xfeb0, "XBASIC");
        m.insert(0xfeb3, "BASCONT");
        m.insert(0xfeb6, "GO");
        m.insert(0xfebf, "REGZ");
        m.insert(0xfec2, "TRACE");
        m.insert(0xfec4, "STEPZ");
        m.insert(0xfeca, "USR");
        m.insert(0xfecd, "WRITE");
        m.insert(0xfed4, "WR1");
        m.insert(0xfeed, "WRBYTE");
        m.insert(0xfeef, "WRBYT2");
        m.insert(0xfef6, "CRMON");
        m.insert(0xfefd, "READ");
        m.insert(0xff0a, "RD2");
        m.insert(0xff16, "RD3");
        m.insert(0xff2d, "PRERR");
        m.insert(0xff3a, "BELL");
        m.insert(0xff3f, "RESTORE");
        m.insert(0xff44, "RESTR1");
        m.insert(0xff4a, "SAVE");
        m.insert(0xff4c, "SAV1");
        m.insert(0xff59, "OLDRST");
        m.insert(0xff65, "MON");
        m.insert(0xff69, "MONZ");
        m.insert(0xff73, "NXTITM");
        m.insert(0xff7a, "CHRSRCH");
        m.insert(0xff8a, "DIG");
        m.insert(0xff90, "NXTBIT");
        m.insert(0xff98, "NXTBAS");
        m.insert(0xffa2, "NXTBS2");
        m.insert(0xffa7, "GETNUM");
        m.insert(0xffad, "NXTCHR");
        m.insert(0xffbe, "TOSUB");
        m.insert(0xffc7, "ZMODE");
        m.insert(0xffcc, "CHRTBL");
        m.insert(0xffe3, "SUBTBL");
        m
    };
}
