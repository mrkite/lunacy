/*

A row in a TreeGrid
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::Context,
    screen::{Color, Point, Rect, Screen, Size},
    widgets::Widget,
};
use std::{cell::RefCell, cmp, rc::Rc};

const LINES: Color = Some(0x55_33_55);
const BG: Color = None;
const INDENT: i32 = 1;
const EXPAND: char = '⊞';
const COLLAPSE: char = '⊟';

struct Line {
    bounds: Rect,
    last: bool,
}

impl Line {
    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        let range = clip.intersect(&self.bounds.offset(origin));
        for y in range.t..range.b {
            let ch = if y == self.bounds.b.saturating_sub(1) + origin.y {
                if self.last { '└' } else { '├' }
            } else {
                '│'
            };
            if self.bounds.l + origin.x >= range.l {
                s.draw(self.bounds.l + origin.x, y, ch, LINES, BG);
            }
            if y == self.bounds.b.saturating_sub(1) + origin.y {
                for x in range.l.saturating_add(1)..=range.r {
                    s.draw(x, y, '─', LINES, BG);
                }
            }
        }
    }
}

#[derive(Default)]
pub struct TreeRow {
    columns: Vec<Rc<RefCell<dyn Widget>>>,
    children: Vec<Self>,
    collapsed: bool,
    lines: Vec<Line>,
    start: i32,
    left: i32,
    end: i32,
}

impl TreeRow {
    pub fn push_column(&mut self, widget: impl Widget + 'static) {
        self.columns.push(Rc::new(RefCell::new(widget)));
    }

    pub fn push_child(&mut self, row: Self) {
        self.children.push(row);
    }

    pub fn get(&self, column: usize) -> Option<&Rc<RefCell<dyn Widget>>> {
        self.columns.get(column)
    }

    pub const fn toggle(&mut self) {
        self.collapsed = !self.collapsed;
    }

    pub const fn can_expand(&self) -> bool {
        self.collapsed && !self.children.is_empty()
    }

    pub const fn can_collapse(&self) -> bool {
        !self.collapsed && !self.children.is_empty()
    }

    pub fn min_size(&self, indent: i32, idx: usize) -> Size {
        let mut s = Size::new(0, 0);
        if let Some(cell) = self.columns.get(idx) {
            let cell_size = cell.borrow().min_size();
            s.w = cmp::max(s.w, cell_size.w);
            s.h += cell_size.h;
        }
        let mut indent = indent;
        if idx == 0 {
            s.w += indent;
            indent += INDENT;
        }
        // only include children if it's expanded
        if !self.collapsed {
            for child in &self.children {
                let cs = child.min_size(indent, idx);
                s.w = cmp::max(s.w, cs.w);
                s.h += cs.h;
            }
        }
        s
    }

    pub fn layout(&mut self, x: &mut i32, y: &mut i32, indent: i32, widths: &[i32]) {
        self.start = *y;
        self.lines.clear();

        let mut row_height = 0;
        let mut dx = *x + indent;
        self.left = dx;
        let plus = if self.children.is_empty() { 0 } else { 2 };
        dx += plus;
        for (idx, col) in self.columns.iter_mut().enumerate() {
            if let Some(width) = widths.get(idx) {
                let width = if idx == 0 {
                    *width - (indent + plus)
                } else {
                    *width
                };
                let height = col.borrow().min_size().h;
                col.borrow_mut()
                    .layout(&Rect::new(dx, *y, dx + width, *y + height));
                dx += width + 1;
                row_height = cmp::max(row_height, height);
            } else {
                col.borrow_mut().layout(&Rect::EMPTY);
            }
        }
        *y += row_height;

        if !self.collapsed {
            let line_left = *x + indent;
            let indent = indent + INDENT;
            let line_right = line_left + INDENT;
            let mut rect = Rect::new(line_left, *y, line_right, *y);
            let last_idx = self.children.len().saturating_sub(1);
            for (idx, child) in self.children.iter_mut().enumerate() {
                let last = idx == last_idx;
                rect.b = *y + row_height;
                self.lines.push(Line {
                    bounds: rect.clone(),
                    last,
                });
                rect.t = rect.b;

                child.layout(x, y, indent + INDENT, widths);
            }
        }
        self.end = *y;
    }

    pub fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        if !self.children.is_empty() && clip.inside(self.left + origin.x, self.start + origin.y) {
            if self.collapsed {
                s.draw(
                    self.left + origin.x,
                    self.start + origin.y,
                    EXPAND,
                    LINES,
                    BG,
                );
            } else {
                s.draw(
                    self.left + origin.x,
                    self.start + origin.y,
                    COLLAPSE,
                    LINES,
                    BG,
                );
            }
        }
        for col in &self.columns {
            col.borrow().draw(s, origin, clip);
        }
        if !self.collapsed {
            for line in &self.lines {
                line.draw(s, origin, clip);
            }
            for child in &self.children {
                child.draw(s, origin, clip);
            }
        }
    }

    pub fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        for col in &mut self.columns {
            if col.borrow_mut().mouse(pt, is_down, ctx) {
                return true;
            }
        }
        false
    }

    pub fn find_row(&self, r: i32) -> Option<&Self> {
        if r == self.start {
            return Some(self);
        }
        if !self.collapsed && r > self.start && r < self.end {
            for child in &self.children {
                if r == child.start {
                    return Some(child);
                }
                if r > child.start && r < child.end {
                    return child.find_row(r);
                }
            }
        }
        None
    }

    pub fn find_row_mut(&mut self, r: i32) -> Option<&mut Self> {
        if r == self.start {
            return Some(self);
        }
        if !self.collapsed && r > self.start && r < self.end {
            for child in &mut self.children {
                if r == child.start {
                    return Some(child);
                }
                if r > child.start && r < child.end {
                    return child.find_row_mut(r);
                }
            }
        }
        None
    }
}
