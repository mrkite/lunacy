/*

TRS80 : BASIC
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
        "Tandy/Basic"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.get_mime() == "rsdos/0" && file.len() > 8 && usize::from(file.r16be(1)) < file.len()
        {
            Some(80)
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
        let cs = get_charset("");
        let end = usize::from(file.r16be(1));
        let mut pos = 3;

        let root = if file.r16be(pos) < 0x25fe {
            0x13fe
        } else {
            0x25fe
        };
        while pos < end {
            let addr = file.r16be(pos);
            if addr == 0 {
                break;
            }

            let line_num = file.r16be(pos + 2);
            pos += 4;
            let mut line = String::new();
            while pos < file.len() {
                let mut token = file.r8(pos);
                pos += 1;
                while pos < file.len() && token != 0 && token < 0x80 {
                    line.push(cs[usize::from(token)]);
                    token = file.r8(pos);
                    pos += 1;
                }
                if token == 0 {
                    break;
                }
                if token == 0xff {
                    token = file.r8(pos);
                    pos += 1;
                    if let Some(val) = FUNCTION_TOKENS.get(&token) {
                        line.push_str(val);
                    }
                } else if let Some(val) = TOKENS.get(&token) {
                    line.push_str(val);
                }
            }
            basic.add(usize::from(line_num), line);
            pos = usize::from(addr - root);
        }
        Ok(Box::new(basic))
    }
}

lazy_static! {
    static ref TOKENS: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0x80, "FOR");
        m.insert(0x81, "GO");
        m.insert(0x82, "REM");
        m.insert(0x83, "'");
        m.insert(0x84, "ELSE");
        m.insert(0x85, "IF");
        m.insert(0x86, "DATA");
        m.insert(0x87, "PRINT");
        m.insert(0x88, "ON");
        m.insert(0x89, "INPUT");
        m.insert(0x8a, "END");
        m.insert(0x8b, "NEXT");
        m.insert(0x8c, "DIM");
        m.insert(0x8d, "READ");
        m.insert(0x8e, "RUN");
        m.insert(0x8f, "RESTORE");
        m.insert(0x90, "RETURN");
        m.insert(0x91, "STOP");
        m.insert(0x92, "POKE");
        m.insert(0x93, "CONT");
        m.insert(0x94, "LIST");
        m.insert(0x95, "CLEAR");
        m.insert(0x96, "NEW");
        m.insert(0x97, "CLOAD");
        m.insert(0x98, "CSAVE");
        m.insert(0x99, "OPEN");
        m.insert(0x9a, "CLOSE");
        m.insert(0x9b, "LLIST");
        m.insert(0x9c, "SET");
        m.insert(0x9d, "RESET");
        m.insert(0x9e, "CLS");
        m.insert(0x9f, "MOTOR");
        m.insert(0xa0, "SOUND");
        m.insert(0xa1, "AUDIO");
        m.insert(0xa2, "EXEC");
        m.insert(0xa3, "SKIPF");
        m.insert(0xa4, "TAB(");
        m.insert(0xa5, "TO");
        m.insert(0xa6, "SUB");
        m.insert(0xa7, "THEN");
        m.insert(0xa8, "NOT");
        m.insert(0xa9, "STEP");
        m.insert(0xaa, "OFF");
        m.insert(0xab, "+");
        m.insert(0xac, "-");
        m.insert(0xad, "*");
        m.insert(0xae, "/");
        m.insert(0xaf, "^");
        m.insert(0xb0, "AND");
        m.insert(0xb1, "OR");
        m.insert(0xb2, ">");
        m.insert(0xb3, "=");
        m.insert(0xb4, "<");
        m.insert(0xb5, "DEL");
        m.insert(0xb6, "EDIT");
        m.insert(0xb7, "TRON");
        m.insert(0xb8, "TROFF");
        m.insert(0xb9, "DEF");
        m.insert(0xba, "LET");
        m.insert(0xbb, "LINE");
        m.insert(0xbc, "PCLS");
        m.insert(0xbd, "PSET");
        m.insert(0xbe, "PRESET");
        m.insert(0xbf, "SCREEN");
        m.insert(0xc0, "PCLEAR");
        m.insert(0xc1, "COLOR");
        m.insert(0xc2, "CIRCLE");
        m.insert(0xc3, "PAINT");
        m.insert(0xc4, "GET");
        m.insert(0xc5, "PUT");
        m.insert(0xc6, "DRAW");
        m.insert(0xc7, "PCOPY");
        m.insert(0xc8, "PMODE");
        m.insert(0xc9, "PLAY");
        m.insert(0xca, "DLOAD");
        m.insert(0xcb, "RENUM");
        m.insert(0xcc, "FN");
        m.insert(0xcd, "USING");
        m.insert(0xce, "DIR");
        m.insert(0xcf, "DRIVE");
        m.insert(0xd0, "FIELD");
        m.insert(0xd1, "FILES");
        m.insert(0xd2, "KILL");
        m.insert(0xd3, "LOAD");
        m.insert(0xd4, "LSET");
        m.insert(0xd5, "MERGE");
        m.insert(0xd6, "RENAME");
        m.insert(0xd7, "RSET");
        m.insert(0xd8, "SAVE");
        m.insert(0xd9, "WRITE");
        m.insert(0xda, "VERIFY");
        m.insert(0xdb, "UNLOAD");
        m.insert(0xdc, "DSKINI");
        m.insert(0xdd, "BACKUP");
        m.insert(0xde, "COPY");
        m.insert(0xdf, "DSKI$");
        m.insert(0xe0, "DSKO$");
        m
    };
    static ref FUNCTION_TOKENS: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0x80, "SGN");
        m.insert(0x81, "INT");
        m.insert(0x82, "ABS");
        m.insert(0x83, "USR");
        m.insert(0x84, "RND");
        m.insert(0x85, "SIN");
        m.insert(0x86, "PEEK");
        m.insert(0x87, "LEN");
        m.insert(0x88, "STR$");
        m.insert(0x89, "VAL");
        m.insert(0x8a, "ASC");
        m.insert(0x8b, "CHR$");
        m.insert(0x8c, "EOF");
        m.insert(0x8d, "JOYSTK");
        m.insert(0x8e, "LEFT$");
        m.insert(0x8f, "RIGHT$");
        m.insert(0x90, "MID$");
        m.insert(0x91, "POINT");
        m.insert(0x92, "INKEY$");
        m.insert(0x93, "MEM");
        m.insert(0x94, "ATN");
        m.insert(0x95, "COS");
        m.insert(0x96, "TAN");
        m.insert(0x97, "EXP");
        m.insert(0x98, "FIX");
        m.insert(0x99, "LOG");
        m.insert(0x9a, "POS");
        m.insert(0x9b, "SQR");
        m.insert(0x9c, "HEX$");
        m.insert(0x9d, "VARPTR");
        m.insert(0x9e, "INSTR");
        m.insert(0x9f, "TIMER");
        m.insert(0xa0, "PPOINT");
        m.insert(0xa1, "STRING$");
        m.insert(0xa2, "CVN");
        m.insert(0xa3, "FREE");
        m.insert(0xa4, "LOC");
        m.insert(0xa5, "LOF");
        m.insert(0xa6, "MKN$");
        m
    };
}
