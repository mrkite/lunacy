/*

Virtual file reading.
@copyright 2026 Sean Kasun

*/
use crate::plugins::get_charset;
use std::{cmp, io::Write, path::Path};

#[derive(PartialEq, Clone)]
pub struct File {
    data: Vec<u8>,
    name: String,
    mime: String,
}

impl File {
    pub fn new_from_disk(filename: &str) -> std::io::Result<Self> {
        let path = Path::new(filename).file_name().unwrap();
        let h = Self {
            data: std::fs::read(filename)?,
            name: path.to_str().unwrap().to_string(),
            mime: "".to_string(),
        };
        Ok(h)
    }

    pub fn new_from_array(data: &[u8], name: &str, mime: &str) -> Self {
        Self {
            data: data.to_owned(),
            name: name.to_owned(),
            mime: mime.to_owned(),
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_mime(&self) -> &str {
        &self.mime
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn r8(&self, ofs: usize) -> u8 {
        self.data[ofs]
    }

    pub fn r16(&self, ofs: usize) -> u16 {
        u16::from(self.data[ofs]) | (u16::from(self.data[ofs + 1]) << 8)
    }

    pub fn r16be(&self, ofs: usize) -> u16 {
        (u16::from(self.data[ofs]) << 8) | u16::from(self.data[ofs + 1])
    }

    pub fn r24(&self, ofs: usize) -> u32 {
        u32::from(self.data[ofs])
            | (u32::from(self.data[ofs + 1]) << 8)
            | (u32::from(self.data[ofs + 2]) << 16)
    }

    pub fn r24be(&self, ofs: usize) -> u32 {
        (u32::from(self.data[ofs]) << 16)
            | (u32::from(self.data[ofs + 1]) << 8)
            | u32::from(self.data[ofs + 2])
    }

    pub fn r32(&self, ofs: usize) -> u32 {
        u32::from(self.data[ofs])
            | (u32::from(self.data[ofs + 1]) << 8)
            | (u32::from(self.data[ofs + 2]) << 16)
            | (u32::from(self.data[ofs + 3]) << 24)
    }

    pub fn r32be(&self, ofs: usize) -> u32 {
        (u32::from(self.data[ofs]) << 24)
            | (u32::from(self.data[ofs + 1]) << 16)
            | (u32::from(self.data[ofs + 2]) << 8)
            | u32::from(self.data[ofs + 3])
    }

    pub fn r4(&self, ofs: usize) -> &str {
        std::str::from_utf8(&self.data[ofs..ofs + 4]).unwrap_or_default()
    }

    pub fn rs(&self, ofs: usize, len: usize, charset: &str) -> String {
        let cs = get_charset(charset);
        self.data[ofs..ofs + len]
            .iter()
            .map(|c| cs[usize::from(*c)])
            .collect()
    }

    pub fn rps(&self, ofs: usize, charset: &str) -> String {
        let len = usize::from(self.data[ofs]);
        self.rs(ofs + 1, len, charset)
    }

    pub fn skipps(&self, ofs: usize) -> usize {
        let len = usize::from(self.data[ofs]);
        ofs + len + 1
    }

    pub fn read(&self, ofs: usize, len: usize) -> &[u8] {
        &self.data[ofs..cmp::min(ofs + len, self.data.len())]
    }

    pub fn write(&self, out: &mut std::fs::File) -> Result<usize, std::io::Error> {
        out.write(&self.data)
    }

    pub fn find(&self, from: usize, key: &[u8]) -> Option<usize> {
        self.data[from..]
            .windows(key.len())
            .position(|window| window == key)
            .map(|dest| dest + from)
    }

    pub fn rfind(&self, from: usize, key: &[u8]) -> Option<usize> {
        self.data[0..from]
            .windows(key.len())
            .rposition(|window| window == key)
    }
}
