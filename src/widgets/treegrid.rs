/*

TreeGrid widget
@copyright 2026 Sean Kasun

*/

use crate::widgets::Widget;
use eframe::egui::{self, Color32, Rect, Sense, Vec2};

const INDENT: f32 = 16.0;
const HEADER_COLOR: Color32 = Color32::BLACK;
const HEADER_BG: Color32 = Color32::from_rgb(0xc4, 0xab, 0x9a);
const LABEL_COLOR: Color32 = Color32::from_rgb(0xb9, 0x90, 0x95);
const HOVER_COLOR: Color32 = Color32::from_rgb(0xde, 0xa3, 0xfc);
const LINES: Color32 = Color32::from_rgb(0x88, 0x77, 0x88);

struct Line {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl Line {
    fn draw(&self, ui: &mut egui::Ui, left: f32, top: f32) {
        ui.painter().vline(
            self.left + left,
            self.top + top..=self.bottom + top,
            egui::Stroke::new(1.0, LINES),
        );
        ui.painter().hline(
            self.left + left..=self.right + left,
            self.bottom + top,
            egui::Stroke::new(1.0, LINES),
        );
    }
}

pub struct TreeRow<E> {
    columns: Vec<String>,
    entry: E,
    children: Vec<TreeRow<E>>,
    collapsed: bool,
    lines: Vec<Line>,
    top: f32,
    height: f32,
}

impl<E> TreeRow<E> {
    pub fn new(entry: E) -> Self {
        Self {
            entry,
            columns: vec![],
            children: vec![],
            collapsed: false,
            lines: vec![],
            top: 0.0,
            height: 0.0,
        }
    }
    pub fn push_column(&mut self, col: &str) {
        self.columns.push(col.to_owned());
    }
    pub fn push_child(&mut self, row: TreeRow<E>) {
        self.children.push(row);
    }

    fn col_size(&self, indent: f32, idx: usize, ui: &mut egui::Ui) -> f32 {
        let font_id = egui::TextStyle::Body.resolve(ui.style());
        let mut x = 0.0;
        if let Some(cell) = self.columns.get(idx) {
            let galley = ui
                .painter()
                .layout_no_wrap(cell.to_owned(), font_id, Color32::WHITE);
            x = f32::max(x, galley.size().x);
        }
        let mut indent = indent;
        if idx == 0 {
            x += indent;
            indent += INDENT;
        }
        // only include children if it's expanded
        if !self.collapsed {
            for child in self.children.iter() {
                x = f32::max(x, child.col_size(indent, idx, ui));
            }
        }
        x
    }

    fn layout(&mut self, indent: f32, top: f32, ui: &mut egui::Ui) {
        let font_id = egui::TextStyle::Body.resolve(ui.style());
        self.lines.clear();
        self.top = top;
        let mut y = font_id.size;

        if !self.collapsed {
            let line_left = indent + 4.0;
            let indent = indent + INDENT;
            let line_right = indent - 4.0;
            let mut line_top = y;
            for child in self.children.iter_mut() {
                y += ui.spacing().item_spacing.y;
                child.layout(indent, top + y, ui);
                let line_bottom = y + font_id.size / 2.0;
                self.lines.push(Line {
                    left: line_left,
                    right: line_right,
                    top: line_top,
                    bottom: line_bottom,
                });
                line_top = line_bottom;
                y += child.height;
            }
        }
        self.height = y;
    }

    fn draw(
        &mut self,
        ui: &mut egui::Ui,
        visible: Rect,
        indent: f32,
        left: f32,
        top: f32,
        widths: &[f32],
    ) -> Option<&E> {
        let font_id = egui::TextStyle::Body.resolve(ui.style());
        let mut pos = egui::Pos2::new(left, top);
        let full_width: f32 = widths.iter().sum();
        let rect = Rect {
            min: pos,
            max: egui::Pos2 {
                x: pos.x + full_width,
                y: pos.y + font_id.size,
            },
        };
        let response = ui.allocate_rect(rect, Sense::click());
        let mut clicked = if response.clicked() {
            self.collapsed = !self.collapsed;
            Some(&self.entry)
        } else {
            None
        };
        for (idx, (col, width)) in self.columns.iter().zip(widths).enumerate() {
            let label = if idx == 0 {
                pos.x += indent;
                if !self.children.is_empty() {
                    if self.collapsed {
                        format!("\u{229e} {}", col)
                    } else {
                        format!("\u{229f} {}", col)
                    }
                } else {
                    col.to_string()
                }
            } else {
                col.to_string()
            };
            let galley = ui.painter().layout_no_wrap(
                label,
                font_id.clone(),
                if response.hovered() {
                    HOVER_COLOR
                } else {
                    LABEL_COLOR
                },
            );
            ui.painter().galley(pos, galley, Color32::WHITE);
            if idx == 0 {
                pos.x -= indent;
            }
            pos.x += width;
        }
        if !self.collapsed {
            for line in self.lines.iter() {
                line.draw(ui, left, pos.y);
            }
            pos.y += font_id.size + ui.spacing().item_spacing.y;
            for child in self.children.iter_mut() {
                let height = child.height;
                if child.top < visible.bottom() && child.top + child.height > visible.top() {
                    let cc = child.draw(ui, visible, indent + INDENT, left, pos.y, widths);
                    clicked = clicked.or(cc);
                }
                pos.y += height + ui.spacing().item_spacing.y;
            }
        }
        clicked
    }
}

pub struct TreeGrid<E, F>
where
    F: Fn(&E),
{
    id_salt: String,
    headers: Vec<String>,
    rows: Vec<TreeRow<E>>,
    clicked: F,
    widths: Vec<f32>,
    size: Vec2,
    cached_width: f32,
}

impl<E, F> TreeGrid<E, F>
where
    F: Fn(&E),
{
    pub fn new(headers: &[&str], id_salt: &str, clicked: F) -> Self {
        Self {
            id_salt: id_salt.to_string(),
            headers: headers.iter().map(|s| s.to_string()).collect(),
            rows: vec![],
            clicked,
            widths: vec![],
            size: Vec2::default(),
            cached_width: 0.0,
        }
    }

    pub fn add(&mut self, row: TreeRow<E>) {
        self.rows.push(row);
        self.widths.clear();
    }

    fn reflow(&mut self, ui: &mut eframe::egui::Ui) {
        self.widths.clear();
        let header_font_id = egui::TextStyle::Heading.resolve(ui.style());
        // calculate column widths
        for (idx, col) in self.headers.iter().enumerate() {
            let galley = ui.painter().layout_no_wrap(
                col.to_owned(),
                header_font_id.clone(),
                egui::Color32::WHITE,
            );
            let mut width = galley.size().x;
            for row in self.rows.iter_mut() {
                width = f32::max(width, row.col_size(0.0, idx, ui));
            }
            self.widths.push(width);
        }
        // try to stretch to fill the available area
        let baserect = ui.available_rect_before_wrap();
        let total_width = baserect.width();
        // don't divide by zero
        if total_width == 0.0 {
            return;
        }
        // scale each column to fill the tree bounds
        let mut maxw: f32 = self.widths.iter().sum();
        let items = self.widths.len() as f32;
        maxw += ui.spacing().item_spacing.x * (items - 1.0);
        // should we expand to fill?
        if maxw < total_width {
            let padding = (total_width - maxw) / items;
            self.widths = self.widths.iter().map(|v| v + padding).collect();
            // fill out the first element with any remainder
            self.widths[0] += (total_width - maxw) - padding * items;
            maxw = total_width;
        }
        // now layout each row
        let mut y = 0.0;
        for row in self.rows.iter_mut() {
            row.layout(0.0, y, ui);
            y += ui.spacing().item_spacing.y + row.height;
        }
        self.size = Vec2 { x: maxw, y };
    }
}

impl<E, F> Widget for TreeGrid<E, F>
where
    F: Fn(&E),
{
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _ctx: crate::lunacy::Context) {
        // always draw header
        let header_font_id = egui::TextStyle::Heading.resolve(ui.style());
        let rect = ui.available_rect_before_wrap();
        let header_rect =
            rect.with_max_y(rect.top() + header_font_id.size + ui.spacing().item_spacing.y * 2.);
        ui.allocate_rect(header_rect, Sense::hover());
        let mut left = 0.0;

        egui::ScrollArea::both()
            .id_salt(&self.id_salt)
            .show_viewport(ui, |ui, visible| {
                left = visible.left();
                let baserect = ui.available_rect_before_wrap();
                if baserect.width() != self.cached_width || self.widths.is_empty() {
                    self.reflow(ui);
                    self.cached_width = baserect.width();
                }
                let minrect = baserect
                    .with_max_x(baserect.left() + self.size.x)
                    .with_max_y(baserect.top() + self.size.y);

                ui.allocate_rect(minrect, Sense::hover());
                let mut pos = baserect.left_top();
                for row in self.rows.iter_mut() {
                    if row.top < visible.bottom()
                        && row.top + row.height > visible.top()
                        && let Some(clicked) =
                            row.draw(ui, visible, 0.0, pos.x, pos.y, &self.widths)
                    {
                        (self.clicked)(clicked);
                        self.cached_width = 0.0; // force reflow
                    }
                    pos.y += row.height + ui.spacing().item_spacing.y;
                }
            });
        // draw header after the scrollview so we can scroll left/right
        ui.painter().rect_filled(header_rect, 0, HEADER_BG);
        let mut pos = header_rect.left_top();
        pos.x -= left;
        for (h, width) in self.headers.iter().zip(self.widths.iter()) {
            let galley =
                ui.painter()
                    .layout_no_wrap(h.to_string(), header_font_id.clone(), HEADER_COLOR);
            ui.painter().galley(pos, galley, egui::Color32::WHITE);
            pos.x += width;
        }
    }
}
