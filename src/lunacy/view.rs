/*

File view for Lunacy
@copyright 2026 Sean Kasun

*/
use super::file::File;
use crate::{
    lunacy::Context,
    plugins::foreach_plugin,
    widgets::{ErrorView, Widget},
};
use eframe::egui;

struct OpenFile {
    name: String,
    widget: Box<dyn Widget>,
}

pub struct View {
    supported: Vec<OpenFile>,
    selected: usize,
    file: File,
}

impl View {
    pub fn new(file: File, ctx: Context) -> Self {
        let mut supported = vec![];
        let mut selected = 0;
        let mut maxval = 0;
        foreach_plugin(|plugin| {
            if let Some(val) = plugin.supports(&file) {
                if val > maxval {
                    maxval = val;
                    selected = supported.len();
                }
                supported.push(OpenFile {
                    name: plugin.get_name().to_string(),
                    widget: match plugin.create(&file, ctx.clone()) {
                        Ok(widget) => widget,
                        Err(err) => Box::new(ErrorView::new(err.into())),
                    },
                });
            }
        });
        Self {
            supported,
            selected,
            file,
        }
    }

    pub fn get_name(&self) -> &str {
        self.file.get_name()
    }

    pub fn get_plugin(&self) -> &str {
        if let Some(sel) = self.supported.get(self.selected) {
            &sel.name
        } else {
            "---"
        }
    }

    pub fn status(&self, ui: &mut egui::Ui) {
        if let Some(sel) = self.supported.get(self.selected) {
            sel.widget.status(ui);
        }
    }

    pub fn gestalt(&self) -> u8 {
        if let Some(sel) = self.supported.get(self.selected) {
            sel.widget.gestalt()
        } else {
            0
        }
    }

    pub fn jump(&mut self, to: usize, ctx: Context) {
        if let Some(sel) = self.supported.get_mut(self.selected) {
            sel.widget.jump(to, ctx);
        }
    }

    pub fn find(&mut self, val: &str, ctx: Context) {
        if let Some(sel) = self.supported.get_mut(self.selected) {
            sel.widget.find(val, ctx);
        }
    }

    pub fn find_next(&mut self, ctx: Context) {
        if let Some(sel) = self.supported.get_mut(self.selected) {
            sel.widget.find_next(ctx);
        }
    }

    pub fn find_prev(&mut self, ctx: Context) {
        if let Some(sel) = self.supported.get_mut(self.selected) {
            sel.widget.find_prev(ctx);
        }
    }

    pub fn menu(&mut self, ui: &mut egui::Ui, ctx: Context) {
        if let Some(sel) = self.supported.get_mut(self.selected) {
            sel.widget.menu(ui, ctx);
        }
    }

    pub fn save(&self, out: &mut std::fs::File, ctx: Context) -> Result<(), std::io::Error> {
        if let Some(sel) = self.supported.get(self.selected) {
            sel.widget.save(out, ctx)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "unsupported",
            ))
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, ctx: Context) {
        if let Some(sel) = self.supported.get_mut(self.selected) {
            sel.widget.ui(ui, ctx);
        }
    }

    pub fn dropdown(&mut self, ui: &mut egui::Ui, _ctx: Context) {
        let mut sel = self.selected;
        egui::ComboBox::from_label("")
            .selected_text(self.get_plugin())
            .show_ui(ui, |ui| {
                for (idx, plugin) in self.supported.iter().enumerate() {
                    ui.selectable_value(&mut sel, idx, &plugin.name);
                }
            });
        self.selected = sel;
    }
}
