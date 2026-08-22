/*

General BASIC viewer
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, gestalt},
    widgets::Widget,
};
use eframe::egui::{self, Color32, Galley, Key, Pos2, Rect, Sense};
use std::{cmp, io::Write, sync::Arc};

const TEXT_COLOR: Color32 = Color32::from_rgb(0xba, 0x9c, 0x7e);

struct Line {
    number: usize,
    text: String,
}
struct RenderedLine {
    height: f32,
    number: usize,
    text: Arc<Galley>,
}

pub struct Basic {
    id_salt: String,
    lines: Vec<Line>,
    wrapped: Vec<RenderedLine>,
    flown_width: f32,
    flown_height: f32,
    cursor: usize,
    jump_cursor: Option<usize>,
    search: Option<String>,
}

impl Basic {
    pub fn new(id_salt: &str) -> Self {
        Self {
            id_salt: id_salt.to_string(),
            lines: vec![],
            wrapped: vec![],
            flown_width: 0.0,
            flown_height: 0.0,
            cursor: 0,
            jump_cursor: None,
            search: None,
        }
    }

    pub fn add(&mut self, line_num: usize, line: String) {
        self.lines.push(Line {
            number: line_num,
            text: format!("{:-8} {}", line_num, line),
        });
    }

    fn reflow(&mut self, ui: &mut egui::Ui) {
        let font_id = egui::TextStyle::Monospace.resolve(ui.style());
        self.wrapped.clear();
        self.flown_height = 0.0;
        for line in self.lines.iter() {
            let text = ui.painter().layout(
                line.text.clone(),
                font_id.clone(),
                Color32::PLACEHOLDER,
                self.flown_width,
            );
            let height = text.size().y;
            self.flown_height += height;
            self.wrapped.push(RenderedLine {
                number: line.number,
                height,
                text,
            });
        }
    }
}

impl Widget for Basic {
    fn ui(&mut self, ui: &mut egui::Ui, _ctx: Context) {
        egui::ScrollArea::vertical()
            .id_salt(&self.id_salt)
            .show_viewport(ui, |ui, visible| {
                let baserect = ui.available_rect_before_wrap();
                if baserect.width() != self.flown_width {
                    self.flown_width = baserect.width();
                    self.reflow(ui);
                }
                let minrect = baserect.with_max_y(baserect.top() + self.flown_height);
                ui.allocate_rect(minrect, Sense::hover());

                ui.input(|i| {
                    if i.key_pressed(Key::ArrowUp) || i.key_pressed(Key::K) {
                        self.jump_cursor = Some(self.cursor.saturating_sub(1));
                    }
                    if i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::J) {
                        self.jump_cursor = Some(cmp::min(
                            self.cursor.saturating_add(1),
                            self.wrapped.len().saturating_sub(1),
                        ));
                    }
                    if i.key_pressed(Key::PageUp) || (i.key_pressed(Key::K) && i.modifiers.shift) {
                        self.jump_cursor = Some(self.cursor.saturating_sub(20));
                    }
                    if i.key_pressed(Key::PageDown) || (i.key_pressed(Key::J) && i.modifiers.shift)
                    {
                        self.jump_cursor = Some(cmp::min(
                            self.cursor.saturating_add(20),
                            self.wrapped.len().saturating_sub(1),
                        ));
                    }
                });

                let mut pos = baserect.left_top();
                for (idx, galley) in self.wrapped.iter().enumerate() {
                    let rect = Rect {
                        min: pos,
                        max: Pos2 {
                            x: baserect.right(),
                            y: pos.y + galley.height,
                        },
                    };

                    if self.jump_cursor.take_if(|cursor| *cursor == idx).is_some() {
                        self.cursor = idx;
                        ui.scroll_to_rect(rect, None);
                    }

                    if pos.y - baserect.top() < visible.bottom()
                        && pos.y + galley.height - baserect.top() > visible.top()
                    {
                        if self.cursor == idx {
                            ui.painter().rect_filled(rect, 0, TEXT_COLOR);
                            ui.painter()
                                .galley(pos, galley.text.clone(), Color32::WHITE);
                        } else {
                            ui.painter().galley(pos, galley.text.clone(), TEXT_COLOR);
                        }
                    }
                    pos.y += galley.height;
                }
            });
    }

    fn status(&self, ui: &mut egui::Ui) {
        if let Some(line) = self.lines.get(self.cursor) {
            ui.label(format!("Line {}", line.number));
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
        self.jump_cursor = Some(0);
        for (idx, line) in self.wrapped.iter().enumerate() {
            self.jump_cursor = Some(idx);
            if line.number >= to {
                return;
            }
        }
    }

    fn find(&mut self, key: &str, ctx: Context) {
        self.search = Some(key.to_string());
        self.find_next(ctx);
    }

    fn find_next(&mut self, _ctx: Context) {
        if let Some(search) = self.search.as_ref() {
            for cursor in self.cursor + 1..self.lines.len() {
                if let Some(row) = self.lines.get(cursor)
                    && row.text.contains(search)
                {
                    self.jump_cursor = Some(cursor);
                    return;
                }
            }
        }
        // not found
    }

    fn find_prev(&mut self, _ctx: Context) {
        if let Some(search) = self.search.as_ref() {
            for cursor in (0..self.cursor.saturating_sub(1)).rev() {
                if let Some(row) = self.lines.get(cursor)
                    && row.text.contains(search)
                {
                    self.jump_cursor = Some(cursor);
                    return;
                }
            }
        }
        // not found
    }

    fn save(&self, out: &mut std::fs::File, _ctx: Context) -> Result<(), std::io::Error> {
        for line in self.lines.iter() {
            let raw = line.text.as_bytes();
            out.write_all(raw)?;
            let nl = "\n".as_bytes();
            out.write_all(nl)?;
        }
        Ok(())
    }
}
