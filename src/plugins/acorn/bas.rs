/*

Acorn/Electron : BASIC parser
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::Context,
    plugins::{Plugin, get_charset},
    widgets::Basic,
};
use lazy_static::lazy_static;
use std::collections::HashMap;

pub struct Bas {}

impl Plugin for Bas {
    fn get_name(&self) -> &str {
        "Elecron/Basic"
    }

    fn supports(&self, file: &crate::lunacy::File) -> Option<i16> {
        if file.get_mime().starts_with("acorn/")
            && file.len() > 3
            && file.r8(0) == 0xd
            && file.r8(1) != 0xff
            && file.r16be(1) < 2000
        {
            Some(60)
        } else {
            None
        }
    }

    fn create(
        &self,
        file: &crate::lunacy::File,
        _ctx: Context,
    ) -> Result<Box<dyn crate::plugins::Widget>, &'static str> {
        let mut basic = Basic::new(&format!("bas_{}", file.get_name()));
        let cs = get_charset("");

        let mut pos = 1;
        while pos < file.len() && file.r8(pos) != 0xff {
            let line_num = file.r16be(pos);
            let line_len = usize::from(file.r8(pos + 2)) + pos;
            pos += 3;
            let mut line = String::new();
            while pos < line_len {
                let token = usize::from(file.r8(pos));
                pos += 1;
                if token == 0xd {
                    break;
                }
                if token == 0x8d {
                    // line number
                    let mask = file.r8(pos);
                    let lo = ((file.r8(pos + 1) & 0x3f) | ((mask << 2) & 0xc0)) ^ 0x40;
                    let hi = ((file.r8(pos + 2) & 0x3f) | ((mask << 4) & 0xc0)) ^ 0x40;
                    pos += 3;
                    let val = (usize::from(hi) << 8) | usize::from(lo);
                    line.push_str(&val.to_string());
                } else if token < 0x80 {
                    line.push(cs[token]);
                } else if let Some(val) = TOKENS.get(&token) {
                    line.push_str(val);
                }
            }
            basic.add(usize::from(line_num), line);
        }
        Ok(Box::new(basic))
    }
}

lazy_static! {
    static ref TOKENS: HashMap<usize, &'static str> = {
        let mut m = HashMap::new();
         m.insert(0x80, "AND");
         m.insert(0x81, "DIV");
         m.insert(0x82, "EOR");
         m.insert(0x83, "MOD");
         m.insert(0x84, "OR");
         m.insert(0x85, "ERROR");
         m.insert(0x86, "LINE");
         m.insert(0x87, "OFF");
         m.insert(0x88, "STEP");
         m.insert(0x89, "SPC");
         m.insert(0x8a, "TAB(");
         m.insert(0x8b, "ELSE");
         m.insert(0x8c, "THEN");
         m.insert(0x8d, "");  // line number follows
         m.insert(0x8e, "OPENIN");
         m.insert(0x8f, "PTR");
         m.insert(0x90, "PAGE");
         m.insert(0x91, "TIME");
         m.insert(0x92, "LOMEM");
         m.insert(0x93, "HIMEM");
         m.insert(0x94, "ABS");
         m.insert(0x95, "ACS");
         m.insert(0x96, "ADVAL");
         m.insert(0x97, "ASC");
         m.insert(0x98, "ASN");
         m.insert(0x99, "ATN");
         m.insert(0x9a, "BGET");
         m.insert(0x9b, "COS");
         m.insert(0x9c, "COUNT");
         m.insert(0x9d, "DEG");
         m.insert(0x9e, "ERL");
         m.insert(0x9f, "ERR");
         m.insert(0xa0, "EVAL");
         m.insert(0xa1, "EXP");
         m.insert(0xa2, "EXT");
         m.insert(0xa3, "FALSE");
         m.insert(0xa4, "FN");
         m.insert(0xa5, "GET");
         m.insert(0xa6, "INKEY");
         m.insert(0xa7, "INSTR(");
         m.insert(0xa8, "INT");
         m.insert(0xa9, "LEN");
         m.insert(0xaa, "LN");
         m.insert(0xab, "LOG");
         m.insert(0xac, "NOT");
         m.insert(0xad, "OPENUP");
         m.insert(0xae, "OPENOUT");
         m.insert(0xaf, "PI");
         m.insert(0xb0, "POINT(");
         m.insert(0xb1, "POS");
         m.insert(0xb2, "RAD");
         m.insert(0xb3, "RND");
         m.insert(0xb4, "SGN");
         m.insert(0xb5, "SIN");
         m.insert(0xb6, "SQR");
         m.insert(0xb7, "TAN");
         m.insert(0xb8, "TO");
         m.insert(0xb9, "TRUE");
         m.insert(0xba, "USR");
         m.insert(0xbb, "VAL");
         m.insert(0xbc, "VPOS");
         m.insert(0xbd, "CHR$");
         m.insert(0xbe, "GET$");
         m.insert(0xbf, "INKEY$");
         m.insert(0xc0, "LEFT$(");
         m.insert(0xc1, "MID$(");
         m.insert(0xc2, "RIGHT$(");
         m.insert(0xc3, "STR$");
         m.insert(0xc4, "STRING$(");
         m.insert(0xc5, "EOF");
         m.insert(0xc6, "AUTO");
         m.insert(0xc7, "DELETE");
         m.insert(0xc8, "LOAD");
         m.insert(0xc9, "LIST");
         m.insert(0xca, "NEW");
         m.insert(0xcb, "OLD");
         m.insert(0xcc, "RENUMBER");
         m.insert(0xcd, "SAVE");
         m.insert(0xce, "EDIT");
         m.insert(0xcf, "PTR");
         m.insert(0xd0, "PAGE");
         m.insert(0xd1, "TIME");
         m.insert(0xd2, "LOMEM");
         m.insert(0xd3, "HIMEM");
         m.insert(0xd4, "SOUND");
         m.insert(0xd5, "BPUT");
         m.insert(0xd6, "CALL");
         m.insert(0xd7, "CHAIN");
         m.insert(0xd8, "CLEAR");
         m.insert(0xd9, "CLOSE");
         m.insert(0xda, "CLG");
         m.insert(0xdb, "CLS");
         m.insert(0xdc, "DATA");
         m.insert(0xdd, "DEF");
         m.insert(0xde, "DIM");
         m.insert(0xdf, "DRAW");
         m.insert(0xe0, "END");
         m.insert(0xe1, "ENDPROC");
         m.insert(0xe2, "ENVELOPE");
         m.insert(0xe3, "FOR");
         m.insert(0xe4, "GOSUB");
         m.insert(0xe5, "GOTO");
         m.insert(0xe6, "GCOL");
         m.insert(0xe7, "IF");
         m.insert(0xe8, "INPUT");
         m.insert(0xe9, "LET");
         m.insert(0xea, "LOCAL");
         m.insert(0xeb, "MODE");
         m.insert(0xec, "MOVE");
         m.insert(0xed, "NEXT");
         m.insert(0xee, "ON");
         m.insert(0xef, "VDU");
         m.insert(0xf0, "PLOT");
         m.insert(0xf1, "PRINT");
         m.insert(0xf2, "PROC");
         m.insert(0xf3, "READ");
         m.insert(0xf4, "REM");
         m.insert(0xf5, "REPEAT");
         m.insert(0xf6, "REPORT");
         m.insert(0xf7, "RESTORE");
         m.insert(0xf8, "RETURN");
         m.insert(0xf9, "RUN");
         m.insert(0xfa, "STOP");
         m.insert(0xfb, "COLOUR");
         m.insert(0xfc, "TRACE");
         m.insert(0xfd, "UNTIL");
         m.insert(0xfe, "WIDTH");
         m.insert(0xff, "OSCLI");
         m
    };
}
