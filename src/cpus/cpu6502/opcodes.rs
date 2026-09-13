/*

6502 opcodes

*/

use crate::cpus::InstType;

#[derive(Eq, Hash, PartialEq)]
pub enum Addressing {
    Imp,
    Imm,
    Abs,
    Abx,
    Aby,
    Zp,
    Zpx,
    Zpy,
    Ind,
    Inx,
    Iny,
    Rel,
}

pub struct Opcode {
    pub n: &'static str,
    pub a: Addressing,
    pub i: InstType,
}

pub const OPCODES: [Opcode; 256] = [
    Opcode {
        n: "brk",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 00
    Opcode {
        n: "ora",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // 01
    Opcode {
        n: "stp",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // 02
    Opcode {
        n: "slo",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // 03
    Opcode {
        n: "nop",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 04
    Opcode {
        n: "ora",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 05
    Opcode {
        n: "asl",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 06
    Opcode {
        n: "slo",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 07
    Opcode {
        n: "php",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 08
    Opcode {
        n: "ora",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // 09
    Opcode {
        n: "asl",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 0a
    Opcode {
        n: "anc",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // 0b
    Opcode {
        n: "nop",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 0c
    Opcode {
        n: "ora",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 0d
    Opcode {
        n: "asl",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 0e
    Opcode {
        n: "slo",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 0f
    Opcode {
        n: "bpl",
        a: Addressing::Rel,
        i: InstType::Branch,
    }, // 10
    Opcode {
        n: "ora",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // 11
    Opcode {
        n: "stp",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // 12
    Opcode {
        n: "slo",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // 13
    Opcode {
        n: "nop",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 14
    Opcode {
        n: "ora",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 15
    Opcode {
        n: "asl",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 16
    Opcode {
        n: "slo",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 17
    Opcode {
        n: "clc",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 18
    Opcode {
        n: "ora",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // 19
    Opcode {
        n: "nop",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 1a
    Opcode {
        n: "slo",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // 1b
    Opcode {
        n: "nop",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 1c
    Opcode {
        n: "ora",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 1d
    Opcode {
        n: "asl",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 1e
    Opcode {
        n: "slo",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 1f
    Opcode {
        n: "jsr",
        a: Addressing::Abs,
        i: InstType::Call,
    }, // 20
    Opcode {
        n: "and",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // 21
    Opcode {
        n: "stp",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // 22
    Opcode {
        n: "rla",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // 23
    Opcode {
        n: "bit",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 24
    Opcode {
        n: "and",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 25
    Opcode {
        n: "rol",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 26
    Opcode {
        n: "rla",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 27
    Opcode {
        n: "plp",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 28
    Opcode {
        n: "and",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // 29
    Opcode {
        n: "rol",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 2a
    Opcode {
        n: "anc",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // 2b
    Opcode {
        n: "bit",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 2c
    Opcode {
        n: "and",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 2d
    Opcode {
        n: "rol",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 2e
    Opcode {
        n: "rla",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 2f
    Opcode {
        n: "bmi",
        a: Addressing::Rel,
        i: InstType::Branch,
    }, // 30
    Opcode {
        n: "and",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // 31
    Opcode {
        n: "stp",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // 32
    Opcode {
        n: "rla",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // 33
    Opcode {
        n: "nop",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 34
    Opcode {
        n: "and",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 35
    Opcode {
        n: "rol",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 36
    Opcode {
        n: "rla",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 37
    Opcode {
        n: "sec",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 38
    Opcode {
        n: "and",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // 39
    Opcode {
        n: "nop",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 3a
    Opcode {
        n: "rla",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // 3b
    Opcode {
        n: "nop",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 3c
    Opcode {
        n: "and",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 3d
    Opcode {
        n: "rol",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 3e
    Opcode {
        n: "rla",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 3f
    Opcode {
        n: "rti",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // 40
    Opcode {
        n: "eor",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // 41
    Opcode {
        n: "stp",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // 42
    Opcode {
        n: "sre",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // 43
    Opcode {
        n: "nop",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 44
    Opcode {
        n: "eor",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 45
    Opcode {
        n: "lsr",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 46
    Opcode {
        n: "sre",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 47
    Opcode {
        n: "pha",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 48
    Opcode {
        n: "eor",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // 49
    Opcode {
        n: "lsr",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 4a
    Opcode {
        n: "alr",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // 4b
    Opcode {
        n: "jmp",
        a: Addressing::Abs,
        i: InstType::Jump,
    }, // 4c
    Opcode {
        n: "eor",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 4d
    Opcode {
        n: "lsr",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 4e
    Opcode {
        n: "sre",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 4f
    Opcode {
        n: "bvc",
        a: Addressing::Rel,
        i: InstType::Branch,
    }, // 50
    Opcode {
        n: "eor",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // 51
    Opcode {
        n: "stp",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // 52
    Opcode {
        n: "sre",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // 53
    Opcode {
        n: "nop",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 54
    Opcode {
        n: "eor",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 55
    Opcode {
        n: "lsr",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 56
    Opcode {
        n: "sre",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 57
    Opcode {
        n: "cli",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 58
    Opcode {
        n: "eor",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // 59
    Opcode {
        n: "nop",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 5a
    Opcode {
        n: "sre",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // 5b
    Opcode {
        n: "nop",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 5c
    Opcode {
        n: "eor",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 5d
    Opcode {
        n: "lsr",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 5e
    Opcode {
        n: "sre",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 5f
    Opcode {
        n: "rts",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // 60
    Opcode {
        n: "adc",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // 61
    Opcode {
        n: "stp",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // 62
    Opcode {
        n: "rra",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // 63
    Opcode {
        n: "nop",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 64
    Opcode {
        n: "adc",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 65
    Opcode {
        n: "ror",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 66
    Opcode {
        n: "rra",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 67
    Opcode {
        n: "pla",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 68
    Opcode {
        n: "adc",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // 69
    Opcode {
        n: "ror",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 6a
    Opcode {
        n: "arr",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // 6b
    Opcode {
        n: "jmp",
        a: Addressing::Ind,
        i: InstType::Jump,
    }, // 6c
    Opcode {
        n: "adc",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 6d
    Opcode {
        n: "ror",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 6e
    Opcode {
        n: "rra",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 6f
    Opcode {
        n: "bvs",
        a: Addressing::Rel,
        i: InstType::Branch,
    }, // 70
    Opcode {
        n: "adc",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // 71
    Opcode {
        n: "stp",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // 72
    Opcode {
        n: "rra",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // 73
    Opcode {
        n: "nop",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 74
    Opcode {
        n: "adc",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 75
    Opcode {
        n: "ror",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 76
    Opcode {
        n: "rra",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 77
    Opcode {
        n: "sei",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 78
    Opcode {
        n: "adc",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // 79
    Opcode {
        n: "nop",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 7a
    Opcode {
        n: "rra",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // 7b
    Opcode {
        n: "nop",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 7c
    Opcode {
        n: "adc",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 7d
    Opcode {
        n: "ror",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 7e
    Opcode {
        n: "rra",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 7f
    Opcode {
        n: "nop",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // 80
    Opcode {
        n: "sta",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // 81
    Opcode {
        n: "nop",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // 82
    Opcode {
        n: "sax",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // 83
    Opcode {
        n: "sty",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 84
    Opcode {
        n: "sta",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 85
    Opcode {
        n: "stx",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 86
    Opcode {
        n: "sax",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // 87
    Opcode {
        n: "dey",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 88
    Opcode {
        n: "nop",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // 89
    Opcode {
        n: "txa",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 8a
    Opcode {
        n: "xaa",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // 8b
    Opcode {
        n: "sty",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 8c
    Opcode {
        n: "sta",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 8d
    Opcode {
        n: "stx",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 8e
    Opcode {
        n: "sax",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // 8f
    Opcode {
        n: "bcc",
        a: Addressing::Rel,
        i: InstType::Branch,
    }, // 90
    Opcode {
        n: "sta",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // 91
    Opcode {
        n: "stp",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 92
    Opcode {
        n: "ahx",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // 93
    Opcode {
        n: "sty",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 94
    Opcode {
        n: "sta",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // 95
    Opcode {
        n: "stx",
        a: Addressing::Zpy,
        i: InstType::Normal,
    }, // 96
    Opcode {
        n: "sax",
        a: Addressing::Zpy,
        i: InstType::Normal,
    }, // 97
    Opcode {
        n: "tya",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 98
    Opcode {
        n: "sta",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // 99
    Opcode {
        n: "txs",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // 9a
    Opcode {
        n: "tas",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // 9b
    Opcode {
        n: "shy",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 9c
    Opcode {
        n: "sta",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // 9d
    Opcode {
        n: "shx",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // 9e
    Opcode {
        n: "ahx",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // 9f
    Opcode {
        n: "ldy",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // a0
    Opcode {
        n: "lda",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // a1
    Opcode {
        n: "ldx",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // a2
    Opcode {
        n: "lax",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // a3
    Opcode {
        n: "ldy",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // a4
    Opcode {
        n: "lda",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // a5
    Opcode {
        n: "ldx",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // a6
    Opcode {
        n: "lax",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // a7
    Opcode {
        n: "tay",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // a8
    Opcode {
        n: "lda",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // a9
    Opcode {
        n: "tax",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // aa
    Opcode {
        n: "lax",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // ab
    Opcode {
        n: "ldy",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // ac
    Opcode {
        n: "lda",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // ad
    Opcode {
        n: "ldx",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // ae
    Opcode {
        n: "lax",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // af
    Opcode {
        n: "bcs",
        a: Addressing::Rel,
        i: InstType::Branch,
    }, // b0
    Opcode {
        n: "lda",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // b1
    Opcode {
        n: "stp",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // b2
    Opcode {
        n: "lax",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // b3
    Opcode {
        n: "ldy",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // b4
    Opcode {
        n: "lda",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // b5
    Opcode {
        n: "ldx",
        a: Addressing::Zpy,
        i: InstType::Normal,
    }, // b6
    Opcode {
        n: "lax",
        a: Addressing::Zpy,
        i: InstType::Normal,
    }, // b7
    Opcode {
        n: "clv",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // b8
    Opcode {
        n: "lda",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // b9
    Opcode {
        n: "tsx",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // ba
    Opcode {
        n: "las",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // bb
    Opcode {
        n: "ldy",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // bc
    Opcode {
        n: "lda",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // bd
    Opcode {
        n: "ldx",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // be
    Opcode {
        n: "lax",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // bf
    Opcode {
        n: "cpy",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // c0
    Opcode {
        n: "cmp",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // c1
    Opcode {
        n: "nop",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // c2
    Opcode {
        n: "dcp",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // c3
    Opcode {
        n: "cpy",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // c4
    Opcode {
        n: "cmp",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // c5
    Opcode {
        n: "dec",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // c6
    Opcode {
        n: "dcp",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // c7
    Opcode {
        n: "iny",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // c8
    Opcode {
        n: "cmp",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // c9
    Opcode {
        n: "dex",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // ca
    Opcode {
        n: "axs",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // cb
    Opcode {
        n: "cpy",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // cc
    Opcode {
        n: "cmp",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // cd
    Opcode {
        n: "dec",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // ce
    Opcode {
        n: "dcp",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // cf
    Opcode {
        n: "bne",
        a: Addressing::Rel,
        i: InstType::Branch,
    }, // d0
    Opcode {
        n: "cmp",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // d1
    Opcode {
        n: "stp",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // d2
    Opcode {
        n: "dcp",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // d3
    Opcode {
        n: "nop",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // d4
    Opcode {
        n: "cmp",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // d5
    Opcode {
        n: "dec",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // d6
    Opcode {
        n: "dcp",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // d7
    Opcode {
        n: "cld",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // d8
    Opcode {
        n: "cmp",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // d9
    Opcode {
        n: "nop",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // da
    Opcode {
        n: "dcp",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // db
    Opcode {
        n: "nop",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // dc
    Opcode {
        n: "cmp",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // dd
    Opcode {
        n: "dec",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // de
    Opcode {
        n: "dcp",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // df
    Opcode {
        n: "cpx",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // e0
    Opcode {
        n: "sbc",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // e1
    Opcode {
        n: "nop",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // e2
    Opcode {
        n: "isc",
        a: Addressing::Inx,
        i: InstType::Normal,
    }, // e3
    Opcode {
        n: "cpx",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // e4
    Opcode {
        n: "sbc",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // e5
    Opcode {
        n: "inc",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // e6
    Opcode {
        n: "isc",
        a: Addressing::Zp,
        i: InstType::Normal,
    }, // e7
    Opcode {
        n: "inx",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // e8
    Opcode {
        n: "sbc",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // e9
    Opcode {
        n: "nop",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // ea
    Opcode {
        n: "sbc",
        a: Addressing::Imm,
        i: InstType::Normal,
    }, // eb
    Opcode {
        n: "cpx",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // ec
    Opcode {
        n: "sbc",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // ed
    Opcode {
        n: "inc",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // ee
    Opcode {
        n: "isc",
        a: Addressing::Abs,
        i: InstType::Normal,
    }, // ef
    Opcode {
        n: "beq",
        a: Addressing::Rel,
        i: InstType::Branch,
    }, // f0
    Opcode {
        n: "sbc",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // f1
    Opcode {
        n: "stp",
        a: Addressing::Imp,
        i: InstType::Return,
    }, // f2
    Opcode {
        n: "isc",
        a: Addressing::Iny,
        i: InstType::Normal,
    }, // f3
    Opcode {
        n: "nop",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // f4
    Opcode {
        n: "sbc",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // f5
    Opcode {
        n: "inc",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // f6
    Opcode {
        n: "isc",
        a: Addressing::Zpx,
        i: InstType::Normal,
    }, // f7
    Opcode {
        n: "sed",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // f8
    Opcode {
        n: "sbc",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // f9
    Opcode {
        n: "nop",
        a: Addressing::Imp,
        i: InstType::Normal,
    }, // fa
    Opcode {
        n: "isc",
        a: Addressing::Aby,
        i: InstType::Normal,
    }, // fb
    Opcode {
        n: "nop",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // fc
    Opcode {
        n: "sbc",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // fd
    Opcode {
        n: "inc",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // fe
    Opcode {
        n: "isc",
        a: Addressing::Abx,
        i: InstType::Normal,
    }, // ff
];
