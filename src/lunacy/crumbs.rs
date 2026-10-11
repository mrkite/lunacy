/*

Breadcrumbs at top of Lunacy
@copyright 2026 Sean Kasun

*/

use super::handle::Handle;
use crate::{
    lunacy::{Context, Message, view::View},
    screen::{Color, Point, Rect, Screen},
    widgets::{KeyMap, Widget},
};
use crossterm::event::{KeyCode, KeyModifiers};

const ARROW: char = '➤';
const FG: Color = Some(0x20_22_31);
const BG: Color = Some(0x8e_72_92);
const DIVIDER: Color = Some(0x50_3d_5a);
const TEXT_FG: Color = Some(0x88_88_88);
const TEXT_BG: Color = Some(0x00_00_01);

#[derive(Default)]
pub struct Crumbs {
    bounds: Rect,
    views: Vec<View>,
}

impl Widget for Crumbs {
    fn layout(&mut self, constraint: &Rect) {
        constraint.clone_into(&mut self.bounds);
        self.layout_child();
    }

    fn draw(&self, s: &mut Screen, origin: &Point, clip: &Rect) {
        s.hide_cursor();
        let mut range = clip.intersect(&self.bounds.offset(origin));
        let mut header = vec![];
        for v in &self.views {
            header.push(' ');
            header.push(ARROW);
            header.push(' ');
            let name = v.get_name();
            for ch in name.chars() {
                header.push(ch);
            }
        }
        if let Some(view) = self.views.last() {
            let hnd = view.get_plugin();
            let pad = usize::try_from(self.bounds.w()).unwrap_or(usize::MAX) - hnd.len();
            if header.len() < pad {
                header.resize(pad, ' ');
            } else {
                header.truncate(pad);
            }

            for ch in hnd.chars() {
                header.push(ch);
            }

            for y in range.t..range.b {
                for x in range.l..range.r {
                    let cx = usize::try_from(x - (self.bounds.l + origin.x)).unwrap_or(usize::MAX);
                    let (ch, fg, bg) = if y - (self.bounds.t + origin.y) == 0
                        && let Some(ch) = header.get(cx)
                    {
                        let fg = if *ch == ARROW { DIVIDER } else { FG };
                        (*ch, fg, BG)
                    } else {
                        (' ', TEXT_FG, TEXT_BG)
                    };
                    s.draw(x, y, ch, fg, bg);
                }
            }

            range.t += 1;
            view.draw(s, origin, &range);
        }
    }

    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers, ctx: &Context) -> bool {
        if let Some(view) = self.views.last_mut()
            && view.key(code, modifiers, ctx)
        {
            return true;
        }

        if code == KeyCode::Esc || code == KeyCode::Backspace {
            self.pop();
            if self.views.is_empty() {
                ctx.post(Message::Quit);
            }
            return true;
        }
        false
    }

    fn mouse(&mut self, pt: &Point, is_down: bool, ctx: &Context) -> bool {
        if let Some(view) = self.views.last_mut()
            && view.mouse(pt, is_down, ctx)
        {
            return true;
        }
        // TODO check if we clicked on crumb
        false
    }

    fn gestalt(&self) -> u8 {
        if let Some(view) = self.views.last() {
            return view.gestalt();
        }
        0
    }

    fn jump(&mut self, to: usize, ctx: &Context) {
        if let Some(view) = self.views.last_mut() {
            view.jump(to, ctx);
        }
    }

    fn find(&mut self, val: &str, ctx: &Context) {
        if let Some(view) = self.views.last_mut() {
            view.find(val, ctx);
        }
    }

    fn find_next(&mut self, ctx: &Context) {
        if let Some(view) = self.views.last_mut() {
            view.find_next(ctx);
        }
    }

    fn find_prev(&mut self, ctx: &Context) {
        if let Some(view) = self.views.last_mut() {
            view.find_prev(ctx);
        }
    }

    fn export(&self, out: &mut std::fs::File) -> Result<(), std::io::Error> {
        self.views.last().map_or_else(
            || {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "unsupported",
                ))
            },
            |view| view.export(out),
        )
    }

    fn get_key_map(&self) -> KeyMap {
        let mut m = self
            .views
            .last()
            .map_or_else(KeyMap::default, Widget::get_key_map);
        m.insert("x⎋|⌫", "close");
        m
    }
}

impl Crumbs {
    pub fn push(&mut self, handle: Handle, ctx: &Context) {
        self.views.push(View::new(handle, ctx));
        self.layout_child();
    }

    pub fn pop(&mut self) {
        self.views.pop();
        self.layout_child();
    }

    pub fn get_name(&self) -> &str {
        self.views.last().map_or("unknown", |view| view.get_name())
    }

    fn layout_child(&mut self) {
        let mut c = self.bounds.clone();
        c.t += 1;
        if let Some(view) = self.views.last_mut() {
            view.layout(&c);
        }
    }
}
