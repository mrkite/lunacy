/*

TreeGrid widget
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, gestalt},
    screen::{Color, Point, Rect, Screen, Size},
    widgets::{KeyMap, Label, Scroll, TreeRow, Widget},
};
use crossterm::event::{KeyCode, KeyModifiers};
use std::{cell::RefCell, cmp, rc::Rc};

const SELECTOR: char = '►';
const HEADER_COLOR: Color = Some(0x00_00_01);
const HEADER_BG: Color = Some(0xc4_ab_9a);
const FG: Color = Some(0xa6_d6_a1);
const BG: Color = None;

pub struct TreeGrid {
    bounds: Rect,
    header_bounds: Rect,
    headers: Vec<Label>,
    rows: Vec<TreeRow>,
    scroll: Scroll,
    max_row: i32,
    max_column: usize,
    cursor: i32,
    column: usize,
    selected: Option<Rc<RefCell<dyn Widget>>>,
    fg: Option<u32>,
    bg: Option<u32>,
}

impl Default for TreeGrid {
    fn default() -> Self {
        Self {
            bounds: Rect::default(),
            header_bounds: Rect::default(),
            headers: vec![],
            rows: vec![],
            scroll: Scroll::default().with_style(FG, BG),
            max_row: 0,
            max_column: 0,
            cursor: 0,
            column: 0,
            selected: None,
            fg: FG,
            bg: BG,
        }
    }
}

impl TreeGrid {
    pub fn new(headers: &[&str]) -> Self {
        Self {
            headers: headers
                .iter()
                .map(|s| {
                    Label::new(s)
                        .with_style(HEADER_COLOR, HEADER_BG)
                        .with_align(super::Align::Center)
                })
                .collect(),
            ..Default::default()
        }
    }

    pub fn clear(&mut self) {
        self.rows.clear();
        self.bounds.clear();
        self.selected = None;
    }

    pub fn add(&mut self, row: TreeRow, ctx: &Context) {
        self.rows.push(row);
        self.bounds.clear(); // force reflow
        self.find_selected(0, ctx);
    }

    fn find_row(&self, r: i32) -> Option<&TreeRow> {
        for row in &self.rows {
            let sel = row.find_row(r);
            if sel.is_some() {
                return sel;
            }
        }
        None
    }

    fn find_row_mut(&mut self, r: i32) -> Option<&mut TreeRow> {
        for row in &mut self.rows {
            let sel = row.find_row_mut(r);
            if sel.is_some() {
                return sel;
            }
        }
        None
    }

    fn find_selected(&mut self, dir: i32, ctx: &Context) {
        // unhilight old selection
        if let Some(sel) = self.selected.as_ref() {
            sel.borrow_mut().focus(false, ctx);
        }

        let mut column = self.column;
        self.selected = None;
        if let Some(row) = self.find_row(self.bounds.t + self.cursor) {
            let mut dir = dir;
            let old = column;
            let mut done = false;
            while !done {
                if dir < 0 {
                    column = cmp::min(column.wrapping_sub(1), self.max_column.saturating_sub(1));
                } else if dir > 0 && self.max_column > 0 {
                    column = (column + 1).rem_euclid(self.max_column);
                }
                if let Some(col) = row.get(column)
                    && (col.borrow().gestalt() & gestalt::FOCUS != 0 || (column == old && dir != 0))
                {
                    done = true;
                }
                if dir == 0 {
                    dir = 1;
                }
            }
            self.selected = row.get(column).cloned();
        }
        self.column = column;

        if let Some(sel) = self.selected.as_ref() {
            sel.borrow_mut().focus(true, ctx);
        }
    }

    fn scroll_to_cursor(&mut self) {
        self.cursor = self.cursor.clamp(0, self.max_row.saturating_sub(1));
        if self.cursor < self.scroll.get() {
            self.scroll.set(self.cursor);
        }
        let page = self.bounds.h().saturating_sub(1);
        if self.cursor >= self.scroll.get() + page {
            self.scroll.set(self.cursor.saturating_sub(page));
        }
    }

    fn layout_children(&mut self) {
        let mut widths = vec![];
        for (idx, col) in self.headers.iter().enumerate() {
            let mut width = col.min_size().w;
            for row in &self.rows {
                width = cmp::max(width, row.min_size(0, idx).w);
            }
            widths.push(width);
        }

        let total_width = self.bounds.w().saturating_sub(1); // minus 1 for the selector
        // don't divide by zero
        if total_width == 0 || widths.is_empty() {
            return;
        }

        // scale each column to fill the tree bounds
        // or remvoe columns if it's too wide
        let mut done = false;
        while !done {
            let mut maxw: i32 = widths.iter().sum();
            let items = i32::try_from(widths.len()).unwrap_or(i32::MAX);
            maxw += items - 1; // space between columns
            if maxw < total_width {
                let padding = (total_width - maxw) / items;
                widths = widths.iter().map(|v| v + padding).collect();
                // fill out the first element with remainder
                widths[0] += (total_width - maxw) - padding * items;
                done = true;
            } else if widths.len() == 1 {
                // we can't hide any more columns
                widths[0] = total_width;
                done = true;
            } else {
                // hide last column and try again
                widths.pop();
            }
        }

        self.max_column = widths.len();
        // now layout each row
        let mut x = self.bounds.l;
        for (idx, width) in widths.iter().enumerate() {
            let mut width = *width;
            // first header should cover the selector
            if idx == 0 {
                width += 1;
            }
            if let Some(header) = self.headers.get_mut(idx) {
                let c = Rect::new(x, self.header_bounds.t, x + width + 1, self.header_bounds.b);
                header.layout(&c);
                x += width + 1; // width + divider
            }
        }
        let mut y = self.bounds.t;
        let mut x = self.bounds.l + 1; // next to selector
        for row in &mut self.rows {
            row.layout(&mut x, &mut y, 0, &widths);
        }
        self.max_row = y - self.bounds.t;
        self.scroll.set_range(0, self.max_row, self.bounds.h());
    }
}

impl Widget for TreeGrid {
    fn min_size(&self) -> Size {
        let mut s = Size::new(0, 1);
        for (idx, col) in self.headers.iter().enumerate() {
            let mut col_size = col.min_size();
            for row in &self.rows {
                let child_col = row.min_size(0, idx);
                col_size.h += child_col.h;
                col_size.w = cmp::max(col_size.w, child_col.w);
            }
            if idx > 0 {
                s.w += 1; // column divider
            }
            s.h = cmp::max(s.h, col_size.h);
            s.w += col_size.w;
        }
        s
    }

    fn layout(&mut self, constraint: &Rect) {
        self.header_bounds = constraint.clone();
        self.header_bounds.b = self.header_bounds.t + 1;
        let mut body = constraint.clone();
        body.t += 1;

        let without_scroll = body.remove_scroll();
        if self.bounds == without_scroll {
            // don't reflow unless we need to
            return;
        }
        self.bounds = without_scroll;
        self.scroll.layout(&body.to_scroll());

        self.layout_children();
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        self.scroll.draw(s, origin, clip);
        let header_range = clip.intersect(&self.header_bounds.offset(origin));
        let range = clip.intersect(&self.bounds.offset(origin));
        for h in &self.headers {
            h.draw(s, origin, &header_range);
        }
        let cursor = self.cursor.wrapping_sub(self.scroll.get());
        if range.inside(self.bounds.l, cursor + self.bounds.t) {
            s.draw(
                self.bounds.l,
                cursor + self.bounds.t,
                SELECTOR,
                self.fg,
                self.bg,
            );
        }
        let origin = Point::new(origin.x, origin.y.saturating_sub(self.scroll.get()));
        for row in &self.rows {
            row.draw(s, &origin, &range);
        }
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if let Some(sel) = self.selected.as_ref()
            && sel.borrow_mut().key(code, modifiers, ctx)
        {
            return true;
        }

        let mut column_dir = 0;
        let page = self.bounds.h().saturating_sub(1);
        match code {
            KeyCode::Up | KeyCode::Char('k') => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => self.cursor = self.cursor.saturating_add(1),
            KeyCode::Left | KeyCode::Char('h') | KeyCode::BackTab => column_dir = -1,
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => column_dir = 1,
            KeyCode::PageUp | KeyCode::Char('K') => self.cursor = self.cursor.saturating_sub(page),
            KeyCode::PageDown | KeyCode::Char('J') => {
                self.cursor = self.cursor.saturating_add(page);
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.max_row,
            KeyCode::Char('=') => {
                if let Some(row) = self.find_row_mut(self.bounds.t + self.cursor) {
                    row.toggle();
                    self.layout_children();
                }
            }
            _ => return false,
        }
        self.scroll_to_cursor();
        self.find_selected(column_dir, ctx);
        true
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        let top = self.scroll.get();
        if self.bounds.inside(pt.x, pt.y) {
            let cursor = pt.y - self.bounds.t;
            self.cursor = cmp::min(cursor + self.scroll.get(), self.max_row.saturating_sub(1));
            if let Some(row) = self.find_row_mut(self.bounds.t + self.cursor) {
                let pt = Point::new(pt.x, pt.y.saturating_add(top));
                if row.mouse(&pt, is_down, ctx) {
                    return true;
                }
            }
            self.find_selected(0, ctx);
            false
        } else {
            self.scroll.mouse(pt, is_down, ctx)
        }
    }

    fn get_key_map(&self) -> KeyMap {
        let mut keys = self
            .selected
            .as_ref()
            .map_or_else(KeyMap::default, |sel| sel.borrow().get_key_map());
        keys.insert("a←↓↑→|hjkl", "move");
        if let Some(row) = self.find_row(self.bounds.t + self.cursor) {
            if row.can_expand() {
                keys.insert("b=", "expand");
            } else if row.can_collapse() {
                keys.insert("b=", "collapse");
            }
        }
        keys
    }
}
