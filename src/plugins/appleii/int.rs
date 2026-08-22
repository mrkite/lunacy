/*

Apple II : Integer BASIC parser
@copyright 2026 Sean Kasun

*/
use crate::{
    lunacy::{Context, File},
    plugins::{Plugin, Widget, get_charset},
    widgets::Basic,
};
use lazy_static::lazy_static;
use std::collections::HashMap;

pub struct Int {}

impl Plugin for Int {
    fn get_name(&self) -> &str {
        "Apple/Integer Basic"
    }

    fn supports(&self, file: &crate::lunacy::File) -> Option<i16> {
        if file.get_mime() == "dos33/1" {
            Some(90)
        } else {
            None
        }
    }

    fn create(&self, file: &File, _ctx: Context) -> Result<Box<dyn Widget>, &'static str> {
        let mut basic = Basic::new(&format!("bas_{}", file.get_name()));
        let cs = get_charset("iie");

        let mut pos = 0;
        while pos + 4 < file.len() {
            let _line_len = file.r8(pos);
            pos += 1;
            let line_num = file.r16(pos);
            pos += 2;
            let mut in_quote = false;
            let mut in_rem = false;
            let mut alpha = false;
            let mut after_token = false;
            let mut line = String::new();
            while pos < file.len() {
                let token = file.r8(pos);
                pos += 1;
                if token == 1 {
                    break;
                }
                if token & 0x80 != 0 {
                    if !in_rem && !in_quote && !alpha && (0xb0..0xc0).contains(&token) {
                        line.push_str(&file.r16(pos).to_string());
                        pos += 2;
                    } else {
                        alpha = (token & 0x7f).is_ascii_alphanumeric();
                        if after_token && alpha {
                            line.push(' ');
                        }
                        line.push(cs[usize::from(token)]);
                    }
                    after_token = false;
                } else {
                    if let Some(val) = TOKENS.get(&token) {
                        let next_alpha =
                            (val.chars().nth(0).unwrap_or('!') as u8).is_ascii_alphabetic();
                        if alpha && next_alpha {
                            line.push(' ');
                        }
                        line.push_str(val);
                        alpha = next_alpha;
                        after_token = true;
                    }
                    match token {
                        0x5d => {
                            in_rem = true;
                        }
                        0x28 => {
                            in_quote = true;
                        }
                        0x29 => {
                            in_quote = false;
                        }
                        _ => {}
                    }
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
        m.insert(0x00, "HIMEM:");
        m.insert(0x02, "_");
        m.insert(0x03, ":");
        m.insert(0x04, "LOAD");
        m.insert(0x05, "SAVE");
        m.insert(0x06, "CON");
        m.insert(0x07, "RUN");
        m.insert(0x08, "RUN");
        m.insert(0x09, "DEL");
        m.insert(0x0a, ",");
        m.insert(0x0b, "NEW");
        m.insert(0x0c, "CLR");
        m.insert(0x0d, "AUTO");
        m.insert(0x0e, ",");
        m.insert(0x0f, "MAN");
        m.insert(0x10, "HIMEM:");
        m.insert(0x11, "LOMEM:");
        m.insert(0x12, "+");
        m.insert(0x13, "-");
        m.insert(0x14, "*");
        m.insert(0x15, "/");
        m.insert(0x16, "=");
        m.insert(0x17, "#");
        m.insert(0x18, ">=");
        m.insert(0x19, ">");
        m.insert(0x1a, "<=");
        m.insert(0x1b, "<>");
        m.insert(0x1c, "<");
        m.insert(0x1d, "AND");
        m.insert(0x1e, "OR");
        m.insert(0x1f, "MOD");
        m.insert(0x20, "^");
        m.insert(0x21, "+");
        m.insert(0x2a, "(");
        m.insert(0x23, ",");
        m.insert(0x24, "THEN");
        m.insert(0x25, "THEN");
        m.insert(0x26, ",");
        m.insert(0x27, ",");
        m.insert(0x28, "\"");
        m.insert(0x29, "\"");
        m.insert(0x2a, "(");
        m.insert(0x2b, "!");
        m.insert(0x2c, "!");
        m.insert(0x2d, "(");
        m.insert(0x2e, "PEEK");
        m.insert(0x2f, "RND");
        m.insert(0x30, "SGN");
        m.insert(0x31, "ABS");
        m.insert(0x32, "PDL");
        m.insert(0x33, "RNDX");
        m.insert(0x34, "(");
        m.insert(0x35, "+");
        m.insert(0x36, "-");
        m.insert(0x37, "NOT");
        m.insert(0x38, "(");
        m.insert(0x39, "=");
        m.insert(0x3a, "#");
        m.insert(0x3b, "LEN(");
        m.insert(0x3c, "ASC(");
        m.insert(0x3d, "SCRN(");
        m.insert(0x3e, ",");
        m.insert(0x3f, "(");
        m.insert(0x40, "$");
        m.insert(0x41, "$");
        m.insert(0x42, "(");
        m.insert(0x43, ",");
        m.insert(0x44, ",");
        m.insert(0x45, ";");
        m.insert(0x46, ";");
        m.insert(0x47, ",");
        m.insert(0x48, ",");
        m.insert(0x49, ",");
        m.insert(0x4a, ",");
        m.insert(0x4b, "TEXT");
        m.insert(0x4c, "GR");
        m.insert(0x4d, "CALL");
        m.insert(0x4e, "DIM");
        m.insert(0x4f, "DIM");
        m.insert(0x50, "TAB");
        m.insert(0x51, "END");
        m.insert(0x52, "INPUT");
        m.insert(0x53, "INPUT");
        m.insert(0x54, "INPUT");
        m.insert(0x55, "FOR");
        m.insert(0x56, "=");
        m.insert(0x57, "TO");
        m.insert(0x58, "STEP");
        m.insert(0x59, "NEXT");
        m.insert(0x5a, ",");
        m.insert(0x5b, "RETURN");
        m.insert(0x5c, "GOSUB");
        m.insert(0x5d, "REM");
        m.insert(0x5e, "LET");
        m.insert(0x5f, "GOTO");
        m.insert(0x60, "IF");
        m.insert(0x61, "PRINT");
        m.insert(0x62, "PRINT");
        m.insert(0x63, "PRINT");
        m.insert(0x64, "POKE");
        m.insert(0x65, ",");
        m.insert(0x66, "COLOR=");
        m.insert(0x67, "PLOT");
        m.insert(0x68, ",");
        m.insert(0x69, "HLIN");
        m.insert(0x6a, ",");
        m.insert(0x6b, "AT");
        m.insert(0x6c, "VLIN");
        m.insert(0x6d, ",");
        m.insert(0x6e, "AT");
        m.insert(0x6f, "VTAB");
        m.insert(0x70, "=");
        m.insert(0x71, "=");
        m.insert(0x72, ")");
        m.insert(0x73, ")");
        m.insert(0x74, "LIST");
        m.insert(0x75, ",");
        m.insert(0x76, "LIST");
        m.insert(0x77, "POP");
        m.insert(0x78, "NODSP");
        m.insert(0x79, "DSP");
        m.insert(0x7a, "NOTRACE");
        m.insert(0x7b, "DSP");
        m.insert(0x7c, "DSP");
        m.insert(0x7d, "TRACE");
        m.insert(0x7e, "PR#");
        m.insert(0x7f, "IN#");
        m
    };
}
