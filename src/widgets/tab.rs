/*

Simple closeable tab
@copyright 2026 Sean Kasun

*/

use eframe::egui::{self, Color32, Pos2, Sense};

const CLOSE_COLOR: Color32 = Color32::from_rgb(0x88, 0x44, 0x44);

#[derive(Default)]
pub struct Tab {}

pub enum TabResponse {
    None,
    Selected,
    Closed,
}

impl Tab {
    pub fn show(&self, ui: &mut egui::Ui, name: &str, selected: bool) -> TabResponse {
        let space = ui.available_rect_before_wrap();
        let font_id = egui::TextStyle::Button.resolve(ui.style());

        let galley =
            ui.painter()
                .layout_no_wrap(name.to_string(), font_id.clone(), Color32::PLACEHOLDER);

        let close_galley =
            ui.painter()
                .layout_no_wrap("\u{2716}".to_string(), font_id, CLOSE_COLOR);
        let button_rect = space
            .with_max_x(
                space.left()
                    + ui.spacing().button_padding.x
                    + galley.size().x
                    + ui.spacing().item_spacing.x,
            )
            .with_max_y(
                space.top()
                    + ui.spacing().button_padding.y
                    + galley.size().y
                    + ui.spacing().button_padding.y,
            );

        let close_rect = space
            .with_min_x(button_rect.right())
            .with_max_x(
                button_rect.right()
                    + ui.spacing().button_padding.x
                    + close_galley.size().x
                    + ui.spacing().button_padding.x,
            )
            .with_max_y(
                space.top()
                    + ui.spacing().button_padding.y
                    + close_galley.size().y
                    + ui.spacing().button_padding.y,
            );

        let response = ui.allocate_rect(button_rect, Sense::click());
        // check close button first:
        let close_response = ui.allocate_rect(close_rect, Sense::click());

        let mut can_switch = false;
        let close_visuals = if close_response.hovered() {
            ui.visuals().widgets.hovered
        } else if close_response.clicked() {
            ui.visuals().widgets.active
        } else {
            can_switch = true;
            ui.visuals().widgets.inactive
        };

        let visuals = if response.hovered() && can_switch {
            ui.visuals().widgets.hovered
        } else if response.clicked() && can_switch || selected {
            ui.visuals().widgets.active
        } else {
            ui.visuals().widgets.inactive
        };

        if let Some(cursor) = ui.visuals().interact_cursor
            && response.hovered()
        {
            ui.ctx().set_cursor_icon(cursor);
        }

        let full_rect = button_rect.with_max_x(close_rect.right());

        ui.painter()
            .rect_filled(full_rect, visuals.corner_radius, visuals.bg_fill);
        ui.painter().rect_stroke(
            full_rect,
            visuals.corner_radius,
            visuals.bg_stroke,
            egui::StrokeKind::Outside,
        );
        let textpos = Pos2::new(
            button_rect.left() + ui.spacing().button_padding.x,
            button_rect.top() + ui.spacing().button_padding.y,
        );
        ui.painter().galley(textpos, galley, visuals.text_color());
        ui.painter().rect_filled(
            close_rect,
            close_visuals.corner_radius,
            close_visuals.bg_fill,
        );
        ui.painter().rect_stroke(
            close_rect,
            close_visuals.corner_radius,
            close_visuals.bg_stroke,
            egui::StrokeKind::Outside,
        );
        let closepos = Pos2::new(
            close_rect.left() + ui.spacing().button_padding.x,
            close_rect.top() + ui.spacing().button_padding.y,
        );
        ui.painter()
            .galley(closepos, close_galley, close_visuals.text_color());
        if close_response.clicked() {
            TabResponse::Closed
        } else if response.clicked() {
            TabResponse::Selected
        } else {
            TabResponse::None
        }
    }
}
