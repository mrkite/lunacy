/*

Breadcrumbs at top of Lunacy
@copyright 2026 Sean Kasun

*/
use super::file::File;
use crate::{
    lunacy::{Context, view::View},
    widgets::{Tab, TabResponse},
};
use eframe::egui;

#[derive(Default)]
pub struct Crumbs {
    views: Vec<View>,
    selected: usize,
}

impl Crumbs {
    pub fn push(&mut self, file: File, ctx: Context) {
        self.selected = self.views.len();
        self.views.push(View::new(file, ctx));
    }
    pub fn get_name(&self) -> &str {
        if let Some(view) = self.views.get(self.selected) {
            view.get_name()
        } else {
            "unknown"
        }
    }

    pub fn status(&self, ui: &mut egui::Ui) {
        if let Some(view) = self.views.get(self.selected) {
            view.status(ui);
        }
    }

    pub fn gestalt(&self) -> u8 {
        if let Some(view) = self.views.get(self.selected) {
            view.gestalt()
        } else {
            0
        }
    }

    pub fn jump(&mut self, to: usize, ctx: Context) {
        if let Some(view) = self.views.get_mut(self.selected) {
            view.jump(to, ctx);
        }
    }

    pub fn find(&mut self, val: &str, ctx: Context) {
        if let Some(view) = self.views.get_mut(self.selected) {
            view.find(val, ctx);
        }
    }

    pub fn find_next(&mut self, ctx: Context) {
        if let Some(view) = self.views.get_mut(self.selected) {
            view.find_next(ctx);
        }
    }

    pub fn find_prev(&mut self, ctx: Context) {
        if let Some(view) = self.views.get_mut(self.selected) {
            view.find_prev(ctx);
        }
    }

    pub fn menu(&mut self, ui: &mut egui::Ui, ctx: Context) {
        if let Some(view) = self.views.get_mut(self.selected) {
            view.menu(ui, ctx);
        }
    }

    pub fn save(&self, out: &mut std::fs::File, ctx: Context) -> Result<(), std::io::Error> {
        if let Some(view) = self.views.get(self.selected) {
            view.save(out, ctx)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "unsupported",
            ))
        }
    }

    pub fn can_close(&self) -> bool {
        self.views.get(self.selected).is_some()
    }

    pub fn close(&mut self) {
        self.close_tab(self.selected);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: Context) {
        let mut to_remove = None;
        ui.vertical(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if let Some(view) = self.views.get_mut(self.selected) {
                    view.dropdown(ui, ctx.clone());
                }
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    egui::ScrollArea::horizontal()
                        .id_salt("tabbar")
                        .show(ui, |ui| {
                            for (idx, view) in self.views.iter().enumerate() {
                                let tab = Tab::default();
                                match tab.show(ui, view.get_name(), idx == self.selected) {
                                    TabResponse::None => {}
                                    TabResponse::Selected => {
                                        self.selected = idx;
                                    }
                                    TabResponse::Closed => {
                                        to_remove = Some(idx);
                                    }
                                }
                            }
                        });
                });
            });
            if let Some(view) = self.views.get_mut(self.selected) {
                view.ui(ui, ctx);
            }
        });
        if let Some(remove) = to_remove {
            self.close_tab(remove);
        }
    }

    fn close_tab(&mut self, idx: usize) {
        self.views.remove(idx);
        if self.selected >= idx {
            self.selected = self.selected.saturating_sub(1);
        }
    }
}
