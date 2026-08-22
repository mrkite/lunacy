/*

Expression parser
@copyright 2026 Sean Kasun

*/
use std::{
    cmp,
    collections::{HashMap, HashSet},
};

#[derive(Clone, PartialEq)]
enum Assoc {
    Left,
    //    Right,
    Unary,
}

#[derive(Clone)]
enum Id {
    Or,
    Xor,
    And,
    Shl,
    Shr,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Pos,
    Not,
}

#[derive(Clone)]
struct Op {
    prec: i16,
    assoc: Assoc,
    id: Id,
}

struct Tree {
    op: Option<Box<Op>>,
    left: Option<Box<Tree>>,
    right: Option<Box<Tree>>,
    leaf: i32,
}

struct Reader {
    chars: Vec<char>,
    pos: usize,
}

impl Reader {
    fn find(&self, needle: &'static str) -> Option<usize> {
        if self.chars.len() < needle.chars().count() {
            return None;
        }
        let end = self.chars.len() - needle.chars().count();
        for i in self.pos..end {
            let mut found = true;
            for (orig, ch) in (i..).zip(needle.chars()) {
                if self.chars.get(orig).is_none_or(|x| *x != ch) {
                    found = false;
                    break;
                }
            }
            if found {
                return Some(i);
            }
        }
        None
    }

    fn read_token(&self, token: &Token) -> String {
        self.chars[token.start..token.start + token.len]
            .iter()
            .collect()
    }

    fn consume(&mut self, token: &Token) {
        self.pos += token.len;
    }

    fn in_base(&mut self, base: i32) -> Option<i32> {
        if self.pos >= self.chars.len() {
            return None;
        }
        let ch = self.chars[self.pos];
        if (base == 16 && ch.is_ascii_hexdigit()) || ch.is_ascii_digit() {
            let ch = ch as i32;
            self.pos += 1;
            if ch > '9' as i32 {
                Some((ch & 0xf) + 9)
            } else {
                Some(ch & 0xf)
            }
        } else {
            None
        }
    }
}

struct Token {
    start: usize,
    len: usize,
}

pub struct ExpParser {
    binaries: HashMap<&'static str, Op>,
    unaries: HashMap<&'static str, Op>,
    terminators: HashSet<&'static str>,
}

impl Default for ExpParser {
    fn default() -> Self {
        let mut p = Self {
            binaries: HashMap::default(),
            unaries: HashMap::default(),
            terminators: HashSet::default(),
        };
        p.add("|", 0, Assoc::Left, Id::Or);
        p.add("^", 1, Assoc::Left, Id::Xor);
        p.add("&", 2, Assoc::Left, Id::And);
        p.add("<<", 3, Assoc::Left, Id::Shl);
        p.add(">>", 3, Assoc::Left, Id::Shr);
        p.add("+", 4, Assoc::Left, Id::Add);
        p.add("-", 4, Assoc::Left, Id::Sub);
        p.add("*", 5, Assoc::Left, Id::Mul);
        p.add("/", 5, Assoc::Left, Id::Div);
        p.add("%", 5, Assoc::Left, Id::Mod);
        p.add("-", 5, Assoc::Unary, Id::Neg);
        p.add("+", 5, Assoc::Unary, Id::Pos);
        p.add("~", 6, Assoc::Unary, Id::Not);
        p.add_terminator("(");
        p.add_terminator(")");
        p
    }
}

impl ExpParser {
    pub fn calculate(&self, expression: &str) -> Result<i32, &'static str> {
        let mut reader = Reader {
            chars: expression.chars().collect(),
            pos: 0,
        };
        let tree = self.parse(0, &mut reader)?;
        if reader.pos != reader.chars.len() {
            Err("expected operator")
        } else {
            Self::eval(tree)
        }
    }

    fn eval(tree: Tree) -> Result<i32, &'static str> {
        if tree.op.is_none() {
            return Ok(tree.leaf);
        }
        let l = Self::eval(*tree.left.unwrap())?;
        let r = if let Some(r) = tree.right {
            Self::eval(*r)?
        } else {
            0
        };
        match tree.op.unwrap().id {
            Id::Or => Ok(l | r),
            Id::Xor => Ok(l ^ r),
            Id::And => Ok(l & r),
            Id::Shl => Ok(l << r),
            Id::Shr => Ok(l >> r),
            Id::Add => Ok(l + r),
            Id::Sub => Ok(l - r),
            Id::Mul => Ok(l * r),
            Id::Div => {
                if r == 0 {
                    Err("divide by zero")
                } else {
                    Ok(l / r)
                }
            }
            Id::Mod => {
                if r == 0 {
                    Err("divide by zero")
                } else {
                    Ok(l % r)
                }
            }
            Id::Pos => Ok(l),
            Id::Neg => Ok(-l),
            Id::Not => Ok(!l),
        }
    }

    fn parse(&self, prec: i16, reader: &mut Reader) -> Result<Tree, &'static str> {
        let mut t = self.primary(reader)?;
        let mut token = self.next(reader);
        while let Some(op) = self.binaries.get(reader.read_token(&token).as_str())
            && op.prec >= prec
        {
            reader.consume(&token);
            let mut subprec = op.prec;
            if op.assoc == Assoc::Left {
                subprec += 1;
            }
            let r = self.parse(subprec, reader)?;
            t = Tree {
                op: Some(Box::new(op.clone())),
                left: Some(Box::new(t)),
                right: Some(Box::new(r)),
                leaf: 0,
            };
            token = self.next(reader);
        }
        Ok(t)
    }

    fn primary(&self, reader: &mut Reader) -> Result<Tree, &'static str> {
        // either starts with a unary or a leaf
        let token = self.next(reader);
        if token.len == 0 {
            return Err("unexpected end");
        }
        if let Some(op) = self.unaries.get(reader.read_token(&token).as_str()) {
            reader.consume(&token);
            let t = self.parse(op.prec, reader)?;
            Ok(Tree {
                op: Some(Box::new(op.clone())),
                left: Some(Box::new(t)),
                right: None,
                leaf: 0,
            })
        } else if reader.chars[token.start] == '(' {
            reader.consume(&token);
            let t = self.parse(0, reader)?;
            if reader.chars[reader.pos] != ')' {
                Err("expected ')'")
            } else {
                reader.pos += 1;
                Ok(t)
            }
        } else {
            Ok(self.leaf(reader))
        }
    }

    fn next(&self, reader: &mut Reader) -> Token {
        // skip whitespace
        while reader.pos < reader.chars.len() && reader.chars[reader.pos].is_whitespace() {
            reader.pos += 1;
        }
        let mut token = Token {
            start: reader.pos,
            len: 0,
        };
        // ended?
        if reader.pos >= reader.chars.len() {
            return token;
        }
        let mut token_pos = None;
        for t in self.terminators.iter() {
            if let Some(ofs) = reader.find(t)
                && token_pos.is_none_or(|v| v >= ofs)
            {
                // we want the earliest
                if token_pos.is_some_and(|v| v == ofs) {
                    // already found here, match the longest
                    token.len = cmp::max(token.len, t.chars().count());
                } else {
                    token_pos = Some(ofs);
                    token.len = t.chars().count();
                }
            }
        }
        if token_pos.is_none() {
            // no terminators found
            token.len = reader.chars.len() - reader.pos;
        }
        token
    }

    fn leaf(&self, reader: &mut Reader) -> Tree {
        let mut base: i32 = 10;
        let mut val: i32 = 0;
        if reader.chars[reader.pos] == '$' {
            reader.pos += 1;
            base = 16;
        } else if reader.chars[reader.pos] == '0'
            && reader.pos + 1 < reader.chars.len()
            && (reader.chars[reader.pos + 1] == 'b'
                || reader.chars[reader.pos + 1] == 'o'
                || reader.chars[reader.pos + 1] == 'x')
        {
            reader.pos += 1;
            if reader.chars[reader.pos] == 'b' {
                base = 2;
            } else if reader.chars[reader.pos] == 'o' {
                base = 8;
            } else {
                base = 16;
            }
            reader.pos += 1;
        }
        while let Some(v) = reader.in_base(base) {
            val *= base;
            val += v;
        }
        Tree {
            op: None,
            left: None,
            right: None,
            leaf: val,
        }
    }

    fn add(&mut self, token: &'static str, prec: i16, assoc: Assoc, id: Id) {
        if assoc == Assoc::Unary {
            self.unaries.insert(token, Op { prec, assoc, id });
        } else {
            self.binaries.insert(token, Op { prec, assoc, id });
        }
        self.terminators.insert(token);
    }

    fn add_terminator(&mut self, token: &'static str) {
        self.terminators.insert(token);
    }
}
