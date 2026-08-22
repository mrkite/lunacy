/*

Vic-20 : BASIC parser (in PRG format)
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, File},
    plugins::{Plugin, get_charset},
    widgets::Basic,
};
use lazy_static::lazy_static;
use std::collections::HashMap;

pub struct Bas {}

impl Plugin for Bas {
    fn get_name(&self) -> &str {
        "Vic20/Basic"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() < 8 {
            return None;
        }
        let base = file.r16(0);
        // 3 possible starting locations
        if base != 0x1001 && base != 0x401 && base != 0x1201 {
            return None;
        }
        // check if end of fist line makes sense
        let addr = file.r16(2);
        if addr >= base && usize::from(addr - (base - 2)) < file.len() {
            Some(70)
        } else {
            None
        }
    }

    fn create(
        &self,
        file: &File,
        _ctx: Context,
    ) -> Result<Box<dyn crate::plugins::Widget>, &'static str> {
        let mut basic = Basic::new(&format!("bas_{}", file.get_name()));
        let cs = get_charset("petscii"); // vic20 uses petscii too
        let base = file.r16(0);
        let mut pos = 2;
        while pos + 4 < file.len() {
            let addr = file.r16(pos);
            if addr < base {
                break;
            }
            let line_num = file.r16(pos + 2);
            pos += 4;
            let mut in_quote = false;
            let mut line = String::new();
            while pos < file.len() {
                let mut token = file.r8(pos);
                pos += 1;
                while token != 0 && (!(0x80..=0xcb).contains(&token) || in_quote) {
                    let ch = cs[usize::from(token)];
                    if ch == '"' {
                        in_quote = !in_quote;
                    }
                    line.push(ch);
                    token = file.r8(pos);
                    pos += 1;
                }
                if token == 0 {
                    break;
                }
                if let Some(val) = TOKENS.get(&token) {
                    line.push(' ');
                    line.push_str(val);
                    line.push(' ');
                }
            }
            basic.add(usize::from(line_num), line);
            pos = usize::from(addr - (base - 2));
        }
        Ok(Box::new(basic))
    }
}

lazy_static! {
    static ref TOKENS: HashMap<u8, &'static str> = {
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
        m
    };
}
