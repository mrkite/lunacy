/*

General hex view widget
@copyright 2026 Sean Kasun

*/

use std::cmp;

use crate::{
    lunacy::{Context, File, Message, gestalt},
    widgets::Widget,
};
use eframe::egui::{self, Color32, Key, Pos2, Rect, Sense, Vec2};

const ADDRESS_COLOR: Color32 = Color32::from_rgb(0x41, 0x72, 0x9f);
const HEX_COLOR: Color32 = Color32::from_rgb(0x58, 0x85, 0xaf);
const ASCII_COLOR: Color32 = Color32::from_rgb(0xb9, 0x90, 0x95);
const DIVIDER_COLOR: Color32 = Color32::from_rgb(0x27, 0x44, 0x72);
const DIVIDER: &str = "│";

pub struct HexView {
    file: File,
    cursor: usize,
    jump_cursor: Option<usize>,
    search: Option<Vec<u8>>,
}

impl Widget for HexView {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _ctx: Context) {
        egui::ScrollArea::both().show_viewport(ui, |ui, visible| {
            // measure a single line
            let font_id = egui::TextStyle::Monospace.resolve(ui.style());
            let line =
                "00000000 | 00 01 02 03 04 05 06 07  08 09 0a 0b 0c 0d 0e 0f | 01234567 89abcdef ";
            let galley = ui.painter().layout_no_wrap(
                line.to_string(),
                font_id.clone(),
                Color32::PLACEHOLDER,
            );
            let hex_size = galley.size();
            let total_rows = self.file.len().div_ceil(16);

            let baserect = ui.available_rect_before_wrap();
            let minrect = baserect
                .with_max_x(f32::max(baserect.left() + hex_size.x, baserect.right()))
                .with_max_y(f32::max(
                    baserect.top() + hex_size.y * total_rows as f32,
                    baserect.bottom(),
                ));

            ui.allocate_rect(minrect, Sense::hover());

            let char_width = hex_size.x / line.len() as f32;

            ui.input(|i| {
                if i.key_pressed(Key::ArrowUp) || i.key_pressed(Key::K) {
                    self.jump_cursor = Some(self.cursor.saturating_sub(16));
                }
                if i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::J) {
                    self.jump_cursor = Some(cmp::min(
                        self.cursor.saturating_add(16),
                        self.file.len().saturating_sub(1),
                    ));
                }
                if i.key_pressed(Key::ArrowLeft) || i.key_pressed(Key::H) {
                    self.jump_cursor = Some(self.cursor.saturating_sub(1));
                }
                if i.key_pressed(Key::ArrowRight) || i.key_pressed(Key::L) {
                    self.jump_cursor = Some(cmp::min(
                        self.cursor.saturating_add(1),
                        self.file.len().saturating_sub(1),
                    ));
                }
                if i.key_pressed(Key::PageUp) || (i.key_pressed(Key::K) && i.modifiers.shift) {
                    self.jump_cursor = Some(self.cursor.saturating_sub(
                        16 * ((baserect.height() / hex_size.y).floor() as usize).saturating_sub(1),
                    ))
                }
                if i.key_pressed(Key::PageDown) || (i.key_pressed(Key::J) && i.modifiers.shift) {
                    self.jump_cursor = Some(cmp::min(
                        self.cursor.saturating_add(
                            16 * ((baserect.height() / hex_size.y).floor() as usize)
                                .saturating_sub(1),
                        ),
                        self.file.len().saturating_sub(1),
                    ));
                }
                if i.pointer.any_down()
                    && self.jump_cursor.is_none()
                    && let Some(mouse_pos) = i.pointer.latest_pos()
                    && visible
                        .translate(Vec2 {
                            x: minrect.left(),
                            y: minrect.top(),
                        })
                        .contains(mouse_pos)
                {
                    let cx = ((mouse_pos.x - minrect.left()) / char_width).floor() as usize;
                    let cy = ((mouse_pos.y - minrect.top()) / hex_size.y).floor() as usize * 16;
                    if cx < 10 {
                        self.jump_cursor = Some(cy);
                    } else if cx < 34 {
                        self.jump_cursor = Some(cy + (cx.saturating_sub(10)) / 3);
                    } else if cx < 60 {
                        self.jump_cursor = Some(cy + 8 + cx.saturating_sub(36) / 3);
                    } else if cx < 70 {
                        self.jump_cursor = Some(cy + cx.saturating_sub(62));
                    } else if cx < 79 {
                        self.jump_cursor = Some(cy + 8 + cx.saturating_sub(71));
                    }
                }
            });

            let cursor_rect = self.get_cursor(char_width, hex_size.y).translate(Vec2 {
                x: baserect.left(),
                y: baserect.top(),
            });

            let start = (visible.top() / hex_size.y).floor() as usize;
            let end = (visible.bottom() / hex_size.y).ceil() as usize;

            let divider =
                ui.painter()
                    .layout_no_wrap(DIVIDER.to_string(), font_id.clone(), DIVIDER_COLOR);

            let mut pos = baserect.left_top();
            pos.y += start as f32 * hex_size.y;
            for y in start..end {
                pos.x = baserect.left();
                let addr = y * 16;
                let mut line = String::new();
                if addr < self.file.len() {
                    for i in 0..8 {
                        line.push(HEX[(addr >> (28 - i * 4)) & 0xf]);
                    }
                    let galley =
                        ui.painter()
                            .layout_no_wrap(line, font_id.clone(), Color32::PLACEHOLDER);
                    if y == self.cursor / 16 {
                        ui.painter().rect_filled(
                            cursor_rect
                                .with_min_x(pos.x)
                                .with_max_x(pos.x + galley.size().x),
                            0,
                            ADDRESS_COLOR,
                        );
                        ui.painter().galley(pos, galley, Color32::WHITE);
                    } else {
                        ui.painter().galley(pos, galley, ADDRESS_COLOR);
                    }
                }
                pos.x += char_width * 9.0;
                ui.painter().galley(pos, divider.clone(), DIVIDER_COLOR);
                pos.x += char_width * 2.0;

                for i in 0..16 {
                    if i == 8 {
                        pos.x += char_width;
                    }
                    if addr + i < self.file.len() {
                        let ch = usize::from(self.file.r8(addr + i));
                        let mut byte = String::new();
                        byte.push(HEX[ch >> 4]);
                        byte.push(HEX[ch & 0xf]);
                        let galley = ui.painter().layout_no_wrap(
                            byte,
                            font_id.clone(),
                            Color32::PLACEHOLDER,
                        );
                        if addr + i == self.cursor {
                            ui.painter().rect_filled(cursor_rect, 0, HEX_COLOR);
                            ui.painter().galley(pos, galley, Color32::WHITE);
                        } else {
                            ui.painter().galley(pos, galley, HEX_COLOR);
                        }
                    }
                    pos.x += char_width * 3.0;
                }
                ui.painter().galley(pos, divider.clone(), DIVIDER_COLOR);
                pos.x += char_width * 2.0;

                for i in 0..16 {
                    if i == 8 {
                        pos.x += char_width;
                    }
                    if addr + i < self.file.len() {
                        let ch = self.file.r8(addr + i);
                        let mut byte = String::new();
                        if (32..127).contains(&ch) {
                            byte.push(ch as char);
                        } else {
                            byte.push('.');
                        }
                        let galley = ui.painter().layout_no_wrap(
                            byte,
                            font_id.clone(),
                            Color32::PLACEHOLDER,
                        );
                        if addr + i == self.cursor {
                            ui.painter().rect_filled(
                                cursor_rect
                                    .with_min_x(pos.x)
                                    .with_max_x(pos.x + galley.size().x),
                                0,
                                ASCII_COLOR,
                            );
                            ui.painter().galley(pos, galley, Color32::WHITE);
                        } else {
                            ui.painter().galley(pos, galley, ASCII_COLOR);
                        }
                    }
                    pos.x += char_width;
                }

                pos.y += hex_size.y;
            }

            if let Some(cursor) = self.jump_cursor.take() {
                self.cursor = cursor;
                ui.scroll_to_rect(
                    self.get_cursor(char_width, hex_size.y).translate(Vec2 {
                        x: baserect.left(),
                        y: baserect.top(),
                    }),
                    None,
                );
            }
        });
    }

    fn status(&self, ui: &mut egui::Ui) {
        if self.cursor < self.file.len() {
            let ch = self.file.r8(self.cursor);
            ui.horizontal(|ui| {
                ui.label(format!("${:08x}", self.cursor));
                ui.label(format!("${:02x} ({})", ch, ch));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.label(self.file.get_mime());
                });
            });
        }
    }

    fn gestalt(&self) -> u8 {
        let mut mask = gestalt::JUMP | gestalt::FIND | gestalt::EXPORT;
        if self.search.is_some() {
            mask |= gestalt::FIND_NEXT | gestalt::FIND_PREV;
        }
        mask
    }

    fn jump(&mut self, to: usize, _ctx: Context) {
        self.jump_cursor = Some(to);
    }

    fn find(&mut self, key: &str, ctx: Context) {
        if key.is_empty() {
            self.search = None;
        } else {
            // assume key is a series of hex values
            let mut ishex = true;
            let mut state = 0;
            let mut byte = 0;
            let mut search = vec![];
            for ch in key.chars() {
                match state {
                    0 | 2 => {
                        if ch.is_ascii_hexdigit() {
                            byte = if ch > '9' {
                                (ch as u8 & 0xf) + 9
                            } else {
                                ch as u8 & 0xf
                            };
                            state = 1;
                        } else if !ch.is_whitespace() || state == 2 {
                            ishex = false;
                        }
                    }
                    1 => {
                        // second nibble of byte or '0x'
                        if ch.is_ascii_hexdigit() {
                            byte <<= 4;
                            byte |= if ch > '9' {
                                (ch as u8 & 0xf) + 9
                            } else {
                                ch as u8 & 0xf
                            };
                            search.push(byte);
                            state = 0;
                        } else if ch == 'x' && byte == 0 {
                            // 0x
                            state = 2;
                        } else if ch.is_whitespace() {
                            search.push(byte);
                            state = 0;
                        } else {
                            ishex = false;
                        }
                    }
                    _ => {}
                };
            }
            if state == 1 && ishex {
                search.push(byte);
            }
            if !ishex {
                search.clear();
                for ch in key.bytes() {
                    search.push(ch);
                }
            }
            self.search = Some(search);
            self.find_next(ctx);
        }
    }

    fn find_next(&mut self, ctx: Context) {
        if let Some(search) = self.search.as_ref()
            && let Some(cursor) = self.file.find(self.cursor + 1, search)
        {
            self.jump_cursor = Some(cursor);
        } else {
            ctx.post(Message::Error("not found".to_string()));
        }
    }

    fn find_prev(&mut self, ctx: Context) {
        if let Some(search) = self.search.as_ref()
            && let Some(cursor) = self.file.rfind(self.cursor, search)
        {
            self.jump_cursor = Some(cursor);
        } else {
            ctx.post(Message::Error("not found".to_string()));
        }
    }

    fn save(&self, out: &mut std::fs::File, _ctx: Context) -> Result<(), std::io::Error> {
        self.file.write(out)?;
        Ok(())
    }
}

impl HexView {
    pub fn new(file: &File) -> Self {
        Self {
            file: file.clone(),
            cursor: 0,
            jump_cursor: Some(0),
            search: None,
        }
    }

    fn get_cursor(&self, w: f32, h: f32) -> Rect {
        let cursorx =
            (self.cursor & 0xf) as f32 * w * 3.0 + (11 + ((self.cursor & 8) >> 3)) as f32 * w;
        let cursory = (self.cursor / 16) as f32 * h;
        Rect {
            min: Pos2 {
                x: cursorx,
                y: cursory,
            },
            max: Pos2 {
                x: cursorx + w * 2.0,
                y: cursory + h,
            },
        }
    }
}

const HEX: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];
