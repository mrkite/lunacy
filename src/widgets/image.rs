/*

General scrollable image view
@copyright 2026 Sean Kasun

*/

use crate::{
    lunacy::{Context, gestalt},
    widgets::Widget,
};
use eframe::egui::{
    self, ColorImage, ImageSource, TextureHandle, TextureOptions, load::SizedTexture, vec2,
};
use image::ExtendedColorType;

pub struct ImageView {
    image: ColorImage,
    handle: Option<TextureHandle>,
    texture: Option<SizedTexture>,
    scale: f32,
}

impl ImageView {
    pub fn new(pixels: &[u8], width: usize, height: usize) -> Self {
        let scale = 640.0 / width as f32;

        Self {
            image: ColorImage::from_rgba_unmultiplied([width, height], pixels),
            handle: None,
            texture: None,
            scale,
        }
    }
}

impl Widget for ImageView {
    fn ui(&mut self, ui: &mut egui::Ui, _ctx: Context) {
        if let Some(texture) = self.texture.or_else(|| {
            let handle = ui.load_texture("imageview", self.image.clone(), TextureOptions::NEAREST);
            self.handle = Some(handle.clone());
            self.texture = Some(SizedTexture::new(
                handle.id(),
                vec2(self.image.width() as f32, self.image.height() as f32),
            ));
            self.texture
        }) {
            ui.vertical(|ui| {
                ui.add(egui::Slider::new(&mut self.scale, 0.1..=10.0));

                egui::ScrollArea::both().show(ui, |ui| {
                    let image = egui::Image::new(ImageSource::Texture(texture));

                    let scale_x = self.image.width() as f32 * self.scale;
                    let scale_y = self.image.height() as f32 * self.scale;
                    ui.add(image.fit_to_exact_size(vec2(scale_x, scale_y)));
                });
            });
        }
    }

    fn status(&self, ui: &mut egui::Ui) {
        ui.label(format!("{} x {}", self.image.width(), self.image.height()));
    }

    fn gestalt(&self) -> u8 {
        gestalt::EXPORT
    }

    fn save(&self, out: &mut std::fs::File, _ctx: Context) -> Result<(), std::io::Error> {
        match image::write_buffer_with_format(
            out,
            self.image.as_raw(),
            self.image.width() as u32,
            self.image.height() as u32,
            ExtendedColorType::Rgba8,
            image::ImageFormat::Png,
        ) {
            Ok(_) => Ok(()),
            Err(image::ImageError::IoError(err)) => Err(err),
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "png creation error",
            )),
        }
    }
}
