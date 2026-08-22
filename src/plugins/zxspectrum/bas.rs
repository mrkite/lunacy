/*

ZXSpectrum : BASIC parser
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
        "ZXSpectrum/Basic"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() > 8 && file.get_mime().starts_with("zx/0/") {
            let ofs = usize::from(file.r16(2));
            if ofs + 4 < file.len() && file.r8(ofs + 3) == 0xd {
                return Some(60);
            }
        }
        None
    }

    fn create(
        &self,
        file: &File,
        _ctx: Context,
    ) -> Result<Box<dyn crate::plugins::Widget>, &'static str> {
        let mut basic = Basic::new(&format!("bas_{}", file.get_name()));
        let cs = get_charset("zxspectrum");
        let mut pos = 0;
        while pos + 4 < file.len() {
            let line_num = file.r16be(pos); // yes, this is bigendian
            let line_len = file.r16(pos + 2);
            pos += 4;
            let line_len = usize::from(line_len) + pos;
            if line_len >= file.len() {
                break;
            }
            let mut line = String::new();
            while pos < line_len {
                let token = file.r8(pos);
                pos += 1;
                if let Some(val) = TOKENS.get(&token) {
                    line.push_str(val);
                } else if token == 0xe || token == 0x7e {
                    // these always follow a text-representation of the numeric literal
                    // so there's no reason to interpret them
                    pos += 5;
                } else if token != 0xd {
                    line.push(cs[usize::from(token)]);
                }
            }
            basic.add(usize::from(line_num), line);
        }
        Ok(Box::new(basic))
    }
}

lazy_static! {
    static ref TOKENS: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
        m.insert(0xa3, "SPECTRUM");
        m.insert(0xa4, "PLAY");
        m.insert(0xa5, "RND");
        m.insert(0xa6, "INKEY$");
        m.insert(0xa7, "PI");
        m.insert(0xa8, "FN");
        m.insert(0xa9, "POINT");
        m.insert(0xaa, "SCREEN$");
        m.insert(0xab, "ATTR");
        m.insert(0xac, "AT");
        m.insert(0xad, "TAB");
        m.insert(0xae, "VAL$");
        m.insert(0xaf, "CODE");
        m.insert(0xb0, "VAL");
        m.insert(0xb1, "LEN");
        m.insert(0xb2, "SIN");
        m.insert(0xb3, "COS");
        m.insert(0xb4, "TAN");
        m.insert(0xb5, "ASN");
        m.insert(0xb6, "ACS");
        m.insert(0xb7, "ATN");
        m.insert(0xb8, "LN");
        m.insert(0xb9, "EXP");
        m.insert(0xba, "INT");
        m.insert(0xbb, "SQR");
        m.insert(0xbc, "SGN");
        m.insert(0xbd, "ABS");
        m.insert(0xbe, "PEEK");
        m.insert(0xbf, "IN");
        m.insert(0xc0, "USR");
        m.insert(0xc1, "STR$");
        m.insert(0xc2, "CHR$");
        m.insert(0xc3, "NOT");
        m.insert(0xc4, "BIN");
        m.insert(0xc5, "OR");
        m.insert(0xc6, "AND");
        m.insert(0xc7, "<=");
        m.insert(0xc8, ">=");
        m.insert(0xc9, "<>");
        m.insert(0xca, "LINE");
        m.insert(0xcb, "THEN");
        m.insert(0xcc, "TO");
        m.insert(0xcd, "STEP");
        m.insert(0xce, "DEF FN");
        m.insert(0xcf, "CAT");
        m.insert(0xd0, "FORMAT");
        m.insert(0xd1, "MOVE");
        m.insert(0xd2, "ERASE");
        m.insert(0xd3, "OPEN #");
        m.insert(0xd4, "CLOSE #");
        m.insert(0xd5, "MERGE");
        m.insert(0xd6, "VERIFY");
        m.insert(0xd7, "BEEP");
        m.insert(0xd8, "CIRCLE");
        m.insert(0xd9, "INK");
        m.insert(0xda, "PAPER");
        m.insert(0xdb, "FLASH");
        m.insert(0xdc, "BRIGHT");
        m.insert(0xdd, "INVERSE");
        m.insert(0xde, "OVER");
        m.insert(0xdf, "OUT");
        m.insert(0xe0, "LPRINT");
        m.insert(0xe1, "LLIST");
        m.insert(0xe2, "STOP");
        m.insert(0xe3, "READ");
        m.insert(0xe4, "DATA");
        m.insert(0xe5, "RESTORE");
        m.insert(0xe6, "NEW");
        m.insert(0xe7, "BORDER");
        m.insert(0xe8, "CONTINUE");
        m.insert(0xe9, "DIM");
        m.insert(0xea, "REM");
        m.insert(0xeb, "FOR");
        m.insert(0xec, "GO TO");
        m.insert(0xed, "GO SUB");
        m.insert(0xee, "INPUT");
        m.insert(0xef, "LOAD");
        m.insert(0xf0, "LIST");
        m.insert(0xf1, "LET");
        m.insert(0xf2, "PAUSE");
        m.insert(0xf3, "NEXT");
        m.insert(0xf4, "POKE");
        m.insert(0xf5, "PRINT");
        m.insert(0xf6, "PLOT");
        m.insert(0xf7, "RUN");
        m.insert(0xf8, "SAVE");
        m.insert(0xf9, "RANDOMIZE");
        m.insert(0xfa, "IF");
        m.insert(0xfb, "CLS");
        m.insert(0xfc, "DRAW");
        m.insert(0xfd, "CLEAR");
        m.insert(0xfe, "RETURN");
        m.insert(0xff, "COPY");
        m
    };
}
