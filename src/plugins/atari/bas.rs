/*

Atari ST : BASIC
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
        "Atari/Basic"
    }

    fn supports(&self, file: &File) -> Option<i16> {
        if file.len() < 32 || !file.get_mime().starts_with("ataridos/") {
            return None;
        }
        if file.r16(0) == 0
            && file.r16(2) >= 256
            && file.r16(6) >= 256
            && file.r16(8) >= 256
            && usize::from(file.r16(0xc)) + 0xe - 256 <= file.len()
        {
            Some(55) // that reflects our lack of certainty
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
        let cs = get_charset("atascii");

        let vnt_ofs = usize::from(file.r16(2)) - 256 + 0xe;
        let vvt_ofs = usize::from(file.r16(6)) - 256 + 0xe;
        let st_ofs = usize::from(file.r16(8)) - 256 + 0xe;
        let end = usize::from(file.r16(0xc)) - 256 + 0xe;

        let mut vars = vec![];
        let mut cur_str = String::new();

        // read the variables
        for i in vnt_ofs..vvt_ofs {
            let ch = usize::from(file.r8(i));
            if ch & 0x80 != 0 {
                cur_str.push(cs[ch & 0x7f]);
                vars.push(cur_str.clone());
                cur_str.clear();
            } else {
                cur_str.push(cs[ch])
            }
        }

        // now read the statements
        let mut ofs = st_ofs;
        while ofs < end {
            let line_num = file.r16(ofs);
            // line_len and stmt_len aren't always accurate, so we'll just look for statement and line endings
            ofs += 3;
            let mut eol = false;
            let mut line = String::new();
            while ofs < file.len() && !eol {
                // skip stmt_len
                ofs += 1;
                let stmt = file.r8(ofs);
                ofs += 1;
                if let Some(name) = STATEMENTS.get(&stmt) {
                    if !name.is_empty() {
                        line.push_str(name);
                        line.push(' ');
                    }
                } else {
                    return Err("invalid statement token");
                }
                if stmt == 0 || stmt == 1 {
                    // REM or DATA
                    while ofs < file.len() && file.r8(ofs) != 0x9b {
                        line.push(cs[usize::from(file.r8(ofs))]);
                        ofs += 1;
                    }
                    ofs += 1;
                    eol = true;
                } else {
                    let mut eos = false;
                    while ofs < file.len() && !eos {
                        let oper = file.r8(ofs);
                        ofs += 1;
                        if oper == 0x14 {
                            eos = true;
                        } else if oper == 0x16 {
                            eol = true;
                            break;
                        }
                        if oper == 0xe {
                            line.push_str(&parse_float(file, ofs));
                            ofs += 6;
                        } else if oper == 0xf {
                            line.push('"');
                            line.push_str(&file.rps(ofs, "atascii"));
                            line.push('"');
                            ofs += 1 + usize::from(file.r8(ofs));
                        } else if let Some(op) = OPERATORS.get(&oper) {
                            line.push_str(op);
                            if oper == 0x1b {
                                // "then" has a special case
                                if file.r8(ofs) != 0xe {
                                    ofs += 2; // if not an implied GOTO, next 2 bytes are garbage
                                }
                            }
                        } else if oper & 0x80 != 0
                            && let Some(v) = vars.get(usize::from(oper & 0x7f))
                        {
                            line.push_str(v);
                        } else {
                            return Err("invalid operator token");
                        }
                    }
                }
            }
            basic.add(usize::from(line_num), line);
        }
        Ok(Box::new(basic))
    }
}

fn parse_float(file: &File, ofs: usize) -> String {
    let mut out = String::new();

    let expval = (((file.r8(ofs) & 0x7f) as i8) - 0x40) * 2;
    let dot = expval + 2;

    if file.r8(ofs) & 0x80 != 0 {
        out.push('-');
    }
    if dot == -1 || dot == 0 {
        out.push_str("0.");
    }
    let mut tail = 9;
    while bcd(file, ofs, tail) == '0' && tail > 0 && tail >= dot {
        tail -= 1;
    }
    if expval == -128 {
        out.push('0');
    } else {
        let expmode = !(0..=12).contains(&dot);
        for i in 0..=tail {
            let digit = bcd(file, ofs, i);
            if i != 0 || digit != '0' {
                out.push(digit);
            }
            if (expmode && i == 0) || (i + 1 == dot && i != tail) {
                out.push('.');
            }
        }
        if dot < 0 {
            out.push('E');
            out.push_str(&(expval / 10).to_string());
        } else if dot > 12 {
            out.push_str("E+");
            out.push_str(&(expval / 10).to_string());
        }
    }
    out
}

fn bcd(file: &File, ofs: usize, idx: i8) -> char {
    let mut ch = file.r8(ofs + (idx as usize >> 1) + 1);
    if idx & 1 != 0 {
        ch >>= 4;
    }
    char::from_digit(u32::from(ch & 0xf), 10).unwrap_or('0')
}

lazy_static! {
    static ref STATEMENTS: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
  m.insert(0x00, "REM");
  m.insert(0x01, "DATA");
  m.insert(0x02, "INPUT");
  m.insert(0x03, "COLOR");
  m.insert(0x04, "LIST");
  m.insert(0x05, "ENTER");
  m.insert(0x06, "LET");
  m.insert(0x07, "IF");
  m.insert(0x08, "FOR");
  m.insert(0x09, "NEXT");
  m.insert(0x0a, "GOTO");
  m.insert(0x0b, "GO TO");
  m.insert(0x0c, "GOSUB");
  m.insert(0x0d, "TRAP");
  m.insert(0x0e, "BYE");
  m.insert(0x0f, "CONT");
  m.insert(0x10, "COM");
  m.insert(0x11, "CLOSE");
  m.insert(0x12, "CLR");
  m.insert(0x13, "DEG");
  m.insert(0x14, "DIM");
  m.insert(0x15, "END");
  m.insert(0x16, "NEW");
  m.insert(0x17, "OPEN");
  m.insert(0x18, "LOAD");
  m.insert(0x19, "SAVE");
  m.insert(0x1a, "STATUS");
  m.insert(0x1b, "NOTE");
  m.insert(0x1c, "POINT");
  m.insert(0x1d, "XIO");
  m.insert(0x1e, "ON");
  m.insert(0x1f, "POKE");
  m.insert(0x20, "PRINT");
  m.insert(0x21, "RAD");
  m.insert(0x22, "READ");
  m.insert(0x23, "RESTORE");
  m.insert(0x24, "RETURN");
  m.insert(0x25, "RUN");
  m.insert(0x26, "STOP");
  m.insert(0x27, "POP");
  m.insert(0x28, "?");
  m.insert(0x29, "GET");
  m.insert(0x2a, "PUT");
  m.insert(0x2b, "GRAPHICS");
  m.insert(0x2c, "PLOT");
  m.insert(0x2d, "POSITION");
  m.insert(0x2e, "DOS");
  m.insert(0x2f, "DRAWTO");
  m.insert(0x30, "SETCOLOR");
  m.insert(0x31, "LOCATE");
  m.insert(0x32, "SOUND");
  m.insert(0x33, "LPRINT");
  m.insert(0x34, "CSAVE");
  m.insert(0x35, "CLOAD");
  m.insert(0x36, "");  // implied LET
        m
    };
    static ref OPERATORS: HashMap<u8, &'static str> = {
        let mut m = HashMap::new();
  m.insert(0x10, "🮇");
  m.insert(0x11, "♥");
  m.insert(0x12, ",");
  m.insert(0x13, "$");
  m.insert(0x14, ":");  // statement end
  m.insert(0x15, ";");
  m.insert(0x16, "└");  // line end
  m.insert(0x17, " GOTO ");
  m.insert(0x18, " GOSUB ");
  m.insert(0x19, " TO ");
  m.insert(0x1a, " STEP ");
  m.insert(0x1b, " THEN ");
  m.insert(0x1c, "#");
  m.insert(0x1d, "<=");
  m.insert(0x1e, "<>");
  m.insert(0x1f, ">=");
  m.insert(0x20, "<");
  m.insert(0x21, ">");
  m.insert(0x22, "=");
  m.insert(0x23, "^");
  m.insert(0x24, "*");
  m.insert(0x25, "+");
  m.insert(0x26, "-");
  m.insert(0x27, "/");
  m.insert(0x28, " NOT ");
  m.insert(0x29, " OR ");
  m.insert(0x2a, " AND ");
  m.insert(0x2b, "(");
  m.insert(0x2c, ")");
  m.insert(0x2d, "=");
  m.insert(0x2e, "=");
  m.insert(0x2f, "<=");
  m.insert(0x30, "<>");
  m.insert(0x31, ">=");
  m.insert(0x32, "<");
  m.insert(0x33, ">");
  m.insert(0x34, "=");
  m.insert(0x35, "+");
  m.insert(0x36, "-");
  m.insert(0x37, "(");
  m.insert(0x38, "🮇");  // array
  m.insert(0x39, "🮇");  // dim array
  m.insert(0x3a, "(");  // function
  m.insert(0x3b, "(");  // dim string
  m.insert(0x3c, ",");
  m.insert(0x3d, "STR$");
  m.insert(0x3e, "CHR$");
  m.insert(0x3f, "USR");
  m.insert(0x40, "ASC");
  m.insert(0x41, "VAL");
  m.insert(0x42, "LEN");
  m.insert(0x43, "ADR");
  m.insert(0x44, "ATN");
  m.insert(0x45, "COS");
  m.insert(0x46, "PEEK");
  m.insert(0x47, "SIN");
  m.insert(0x48, "RND");
  m.insert(0x49, "FRE");
  m.insert(0x4a, "EXP");
  m.insert(0x4b, "LOG");
  m.insert(0x4c, "CLOG");
  m.insert(0x4d, "SQR");
  m.insert(0x4e, "SGN");
  m.insert(0x4f, "ABS");
  m.insert(0x50, "INT");
  m.insert(0x51, "PADDLE");
  m.insert(0x52, "STICK");
  m.insert(0x53, "PTRIG");
  m.insert(0x54, "STRIG");
        m
    };
}
