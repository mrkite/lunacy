/*

C64 : BASIC parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Handle},
    plugins::{Plugin, get_charset},
    widgets::{Basic, Widget},
};
use std::{collections::HashMap, sync::LazyLock};

pub struct Bas {}

impl Plugin for Bas {
    fn get_name(&self) -> &'static str {
        "C64/Basic"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        if handle.len() > 8 && handle.r16(0) == 0x801 && handle.len() < 0x10000 {
            let addr = handle.r16(2);
            if addr >= 0x801 && usize::from(addr - 0x7ff) < handle.len() {
                return Some(90);
            }
        }
        None
    }

    fn create(&self, handle: &Handle, _ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let mut basic = Basic::default();
        let cs = get_charset("petscii");
        let mut pos = 2;
        while pos + 4 < handle.len() {
            let addr = handle.r16(pos);
            if addr < 0x801 {
                break;
            }
            let line_num = handle.r16(pos + 2);
            pos += 4;
            let mut in_quote = false;
            let mut line = String::new();
            while pos < handle.len() {
                let mut token = handle.r8(pos);
                pos += 1;
                while token != 0 && (!(0x80..=0xe8).contains(&token) || in_quote) {
                    let ch = cs[usize::from(token)];
                    if ch == '"' {
                        in_quote = !in_quote;
                    }
                    line.push(ch);
                    token = handle.r8(pos);
                    pos += 1;
                }
                if token == 0 {
                    break;
                }
                if let Some(val) = TOKENS.get(&token) {
                    line.push_str(val);
                }
            }
            basic.add(line_num.into(), &line);
            pos = usize::from(addr - 0x7ff);
        }
        Ok(Box::new(basic))
    }
}

static TOKENS: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0x80, "END");
    m.insert(0x81, "FOR");
    m.insert(0x82, "NEXT");
    m.insert(0x83, "DATA");
    m.insert(0x84, "INPUT#");
    m.insert(0x85, "INPUT");
    m.insert(0x86, "DIM");
    m.insert(0x87, "READ");
    m.insert(0x88, "LET");
    m.insert(0x89, "GOTO");
    m.insert(0x8a, "RUN");
    m.insert(0x8b, "IF");
    m.insert(0x8c, "RESTORE");
    m.insert(0x8d, "GOSUB");
    m.insert(0x8e, "RETURN");
    m.insert(0x8f, "REM");
    m.insert(0x90, "STOP");
    m.insert(0x91, "ON");
    m.insert(0x92, "WAIT");
    m.insert(0x93, "LOAD");
    m.insert(0x94, "SAVE");
    m.insert(0x95, "VERIFY");
    m.insert(0x96, "DEF");
    m.insert(0x97, "POKE");
    m.insert(0x98, "PRINT#");
    m.insert(0x99, "PRINT");
    m.insert(0x9a, "CONT");
    m.insert(0x9b, "LIST");
    m.insert(0x9c, "CLR");
    m.insert(0x9d, "CMD");
    m.insert(0x9e, "SYS");
    m.insert(0x9f, "OPEN");
    m.insert(0xa0, "CLOSE");
    m.insert(0xa1, "GET");
    m.insert(0xa2, "NEW");
    m.insert(0xa3, "TAB(");
    m.insert(0xa4, "TO");
    m.insert(0xa5, "FN");
    m.insert(0xa6, "SPC(");
    m.insert(0xa7, "THEN");
    m.insert(0xa8, "NOT");
    m.insert(0xa9, "STEP");
    m.insert(0xaa, "+");
    m.insert(0xab, "-");
    m.insert(0xac, "*");
    m.insert(0xad, "/");
    m.insert(0xae, "^");
    m.insert(0xaf, "AND");
    m.insert(0xb0, "OR");
    m.insert(0xb1, ">");
    m.insert(0xb2, "=");
    m.insert(0xb3, "<");
    m.insert(0xb4, "SGN");
    m.insert(0xb5, "INT");
    m.insert(0xb6, "ABS");
    m.insert(0xb7, "USR");
    m.insert(0xb8, "FRE");
    m.insert(0xb9, "POS");
    m.insert(0xba, "SQR");
    m.insert(0xbb, "RND");
    m.insert(0xbc, "LOG");
    m.insert(0xbd, "EXP");
    m.insert(0xbe, "COS");
    m.insert(0xbf, "SIN");
    m.insert(0xc0, "TAN");
    m.insert(0xc1, "ATN");
    m.insert(0xc2, "PEEK");
    m.insert(0xc3, "LEN");
    m.insert(0xc4, "STR$");
    m.insert(0xc5, "VAL");
    m.insert(0xc6, "ASC");
    m.insert(0xc7, "CHR$");
    m.insert(0xc8, "LEFT$");
    m.insert(0xc9, "RIGHT$");
    m.insert(0xca, "MID$");
    m.insert(0xcb, "GO");
    m.insert(0xcc, "CONCAT");
    m.insert(0xcd, "DOPEN");
    m.insert(0xce, "DCLOSE");
    m.insert(0xcf, "RECORD");
    m.insert(0xd0, "HEADER");
    m.insert(0xd1, "COLLECT");
    m.insert(0xd2, "BACKUP");
    m.insert(0xd3, "COPY");
    m.insert(0xd4, "APPEND");
    m.insert(0xd5, "DSAVE");
    m.insert(0xd6, "DLOAD");
    m.insert(0xd7, "CATALOG");
    m.insert(0xd8, "RENAME");
    m.insert(0xd9, "SCRATCH");
    m.insert(0xda, "DIRECTORY");
    m.insert(0xdb, "DCLEAR");
    m.insert(0xdc, "BANK");
    m.insert(0xdd, "BLOAD");
    m.insert(0xde, "BSAVE");
    m.insert(0xdf, "KEY");
    m.insert(0xe0, "DELETE");
    m.insert(0xe1, "ELSE");
    m.insert(0xe2, "TRAP");
    m.insert(0xe3, "RESUME");
    m.insert(0xe4, "DISPOSE");
    m.insert(0xe5, "PUDEF");
    m.insert(0xe6, "USING");
    m.insert(0xe7, "ERR$");
    m.insert(0xe8, "INSTR");
    m
});
