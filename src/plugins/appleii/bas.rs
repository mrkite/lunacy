/*

Apple II : Applesoft BASIC parser
@copyright 2026 Sean Kasun

*/
use crate::{
    lunacy::{Context, Handle},
    plugins::{Plugin, Widget, get_charset},
    widgets::Basic,
};
use std::{collections::HashMap, sync::LazyLock};

pub struct Bas {}

impl Plugin for Bas {
    fn get_name(&self) -> &'static str {
        "Apple/Basic"
    }

    fn supports(&self, handle: &Handle) -> Option<i16> {
        let mime = handle.get_mime();
        if mime == "dos33/2" || mime.starts_with("prodos/fc:") {
            Some(90)
        } else {
            None
        }
    }

    fn create(&self, handle: &Handle, _ctx: &Context) -> Result<Box<dyn Widget>, &'static str> {
        let mut basic = Basic::default();
        let cs = get_charset("");

        let mut pos = 0;
        while pos + 4 < handle.len() {
            let addr = handle.r16(pos);
            if addr < 0x801 {
                break;
            }
            let line_num = handle.r16(pos + 2);
            pos += 4;
            let mut line = String::new();
            while pos < handle.len() {
                let mut token = handle.r8(pos);
                pos += 1;
                while token != 0 && token < 0x80 {
                    line.push(cs[usize::from(token)]);
                    token = handle.r8(pos);
                    pos += 1;
                }
                if token == 0 {
                    break;
                }
                if let Some(val) = TOKENS.get(&token) {
                    line.push(' ');
                    line += val;
                    line.push(' ');
                }
            }
            basic.add(line_num.into(), &line);
            pos = usize::from(addr) - 0x801;
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
    m.insert(0x84, "INPUT");
    m.insert(0x85, "DEL");
    m.insert(0x86, "DIM");
    m.insert(0x87, "READ");
    m.insert(0x88, "GR");
    m.insert(0x89, "TEXT");
    m.insert(0x8a, "PR #");
    m.insert(0x8b, "IN #");
    m.insert(0x8c, "CALL");
    m.insert(0x8d, "PLOT");
    m.insert(0x8e, "HLIN");
    m.insert(0x8f, "VLIN");
    m.insert(0x90, "HGR2");
    m.insert(0x91, "HGR");
    m.insert(0x92, "HCOLOR=");
    m.insert(0x93, "HPLOT");
    m.insert(0x94, "DRAW");
    m.insert(0x95, "XDRAW");
    m.insert(0x96, "HTAB");
    m.insert(0x97, "HOME");
    m.insert(0x98, "ROT=");
    m.insert(0x99, "SCALE=");
    m.insert(0x9a, "SHLOAD");
    m.insert(0x9b, "TRACE");
    m.insert(0x9c, "NOTRACE");
    m.insert(0x9d, "NORMAL");
    m.insert(0x9e, "INVERSE");
    m.insert(0x9f, "FLASH");
    m.insert(0xa0, "COLOR=");
    m.insert(0xa1, "POP");
    m.insert(0xa2, "VTAB");
    m.insert(0xa3, "HIMEM");
    m.insert(0xa4, "LOMEM");
    m.insert(0xa5, "ONERR");
    m.insert(0xa6, "RESUME");
    m.insert(0xa7, "RECALL");
    m.insert(0xa8, "STORE");
    m.insert(0xa9, "SPEED=");
    m.insert(0xaa, "LET");
    m.insert(0xab, "GOTO");
    m.insert(0xac, "RUN");
    m.insert(0xad, "IF");
    m.insert(0xae, "RESTORE");
    m.insert(0xaf, "&");
    m.insert(0xb0, "GOSUB");
    m.insert(0xb1, "RETURN");
    m.insert(0xb2, "REM");
    m.insert(0xb3, "STOP");
    m.insert(0xb4, "ON");
    m.insert(0xb5, "WAIT");
    m.insert(0xb6, "LOAD");
    m.insert(0xb7, "SAVE");
    m.insert(0xb8, "DEF FN");
    m.insert(0xb9, "POKE");
    m.insert(0xba, "PRINT");
    m.insert(0xbb, "CONT");
    m.insert(0xbc, "LIST");
    m.insert(0xbd, "CLEAR");
    m.insert(0xbe, "GET");
    m.insert(0xbf, "NEW");
    m.insert(0xc0, "TAB");
    m.insert(0xc1, "TO");
    m.insert(0xc2, "FN");
    m.insert(0xc3, "SPC(");
    m.insert(0xc4, "THEN");
    m.insert(0xc5, "AT");
    m.insert(0xc6, "NOT");
    m.insert(0xc7, "STEP");
    m.insert(0xc8, "+");
    m.insert(0xc9, "-");
    m.insert(0xca, "*");
    m.insert(0xcb, "/");
    m.insert(0xcc, ";");
    m.insert(0xcd, "AND");
    m.insert(0xce, "OR");
    m.insert(0xcf, ">");
    m.insert(0xd0, "=");
    m.insert(0xd1, "<");
    m.insert(0xd2, "SGN");
    m.insert(0xd3, "INT");
    m.insert(0xd4, "ABS");
    m.insert(0xd5, "USR");
    m.insert(0xd6, "FRE");
    m.insert(0xd7, "SCRN(");
    m.insert(0xd8, "PDL");
    m.insert(0xd9, "POS");
    m.insert(0xda, "SQR");
    m.insert(0xdb, "RND");
    m.insert(0xdc, "LOG");
    m.insert(0xdd, "EXP");
    m.insert(0xde, "COS");
    m.insert(0xdf, "SIN");
    m.insert(0xe0, "TAN");
    m.insert(0xe1, "ATN");
    m.insert(0xe2, "PEEK");
    m.insert(0xe3, "LEN");
    m.insert(0xe4, "STR$");
    m.insert(0xe5, "VAL");
    m.insert(0xe6, "ASC");
    m.insert(0xe7, "CHR$");
    m.insert(0xe8, "LEFT$");
    m.insert(0xe9, "RIGHT$");
    m.insert(0xea, "MID$");
    m.insert(0xfd, "(");
    m.insert(0xfe, "(");
    m.insert(0xff, "(");
    m
});
