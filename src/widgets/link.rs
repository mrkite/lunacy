/*

Hyperlink widget
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, Message, gestalt},
    screen::{Color, Point, Rect, Screen, Size},
    widgets::{KeyMap, Widget},
};
use crossterm::event::{KeyCode, KeyModifiers};

const LINK_COLOR: Color = Some(0x96_64_db);
const LINK_BG: Color = None;
const FOCUSED_COLOR: Color = Some(0xbe_a3_ec);
const FOCUSED_BG: Color = None;

pub struct Link<F>
where
    F: Fn(&Context),
{
    bounds: Rect,
    text: String,
    tooltip: String,
    selected: F,
    focused: bool,
}

impl<F> Widget for Link<F>
where
    F: Fn(&Context),
{
    fn min_size(&self) -> Size {
        Size::new(
            i32::try_from(self.text.chars().count()).unwrap_or(i32::MAX),
            1,
        )
    }

    fn layout(&mut self, constraint: &Rect) {
        self.bounds = constraint.clone();
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        let range = clip.intersect(&self.bounds.offset(origin));
        let (fg, bg) = if self.focused {
            (FOCUSED_COLOR, FOCUSED_BG)
        } else {
            (LINK_COLOR, LINK_BG)
        };

        for y in range.t..range.b {
            for x in range.l..range.r {
                s.draw(x, y, ' ', fg, bg);
            }
        }
        let y = self.bounds.t + origin.y + self.bounds.h() / 2;
        for (x, ch) in (self.bounds.offset(origin).l..).zip(self.text.chars()) {
            if range.inside(x, y) {
                s.draw(x, y, ch, fg, bg);
            }
        }
    }

    fn key(&mut self, code: KeyCode, _modifiers: KeyModifiers, ctx: &Context) -> bool {
        if code == KeyCode::Char(' ') || code == KeyCode::Enter {
            (self.selected)(ctx);
            true
        } else {
            false
        }
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        // follow links on mouse up
        if self.bounds.inside(pt.x, pt.y) && !is_down {
            (self.selected)(ctx);
            true
        } else {
            false
        }
    }

    fn gestalt(&self) -> u8 {
        gestalt::FOCUS
    }

    fn focus(&mut self, focus: bool, ctx: &Context) {
        self.focused = focus;
        if focus {
            ctx.post(Message::Status(self.tooltip.clone()));
        }
    }

    fn get_key_map(&self) -> KeyMap {
        let mut keys = KeyMap::default();
        keys.insert("g⏎|␣", "select");
        keys
    }
}

impl<F> Link<F>
where
    F: Fn(&Context),
{
    pub fn new(text: &str, tooltip: &str, selected: F) -> Self {
        Self {
            text: text.to_owned(),
            tooltip: tooltip.to_owned(),
            selected,
            bounds: Rect::default(),
            focused: false,
        }
    }
}
