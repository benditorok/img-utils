use egui::{ColorImage, ImageSource, TextureHandle};
use image::DynamicImage;
use std::{borrow::Cow, sync::Arc};

pub mod app;
pub mod cudaimg;

/// A struct to hold the original and modified images as texture handles.
/// This is useful to avoid loading the same image multiple times.
///
/// # Fields
///
/// * `original_image` - The original image as a texture handle.
/// * `modified_image` - The modified image as a texture handle.
#[derive(Default)]
struct TextureMap {
    pub original_image: Option<TextureHandle>,
    pub modified_image: Option<TextureHandle>,
}

/// A struct to hold the image modifiers.
///
/// # Fields
///
/// * `gamma` - The gamma value to use for gamma transformation.
/// * `log_base` - The base value to use for logarithmic transformation.
struct ImageModifiers {
    pub gamma: f32,
    pub log_base: f32,
    pub box_filter_size: u32,
}

impl Default for ImageModifiers {
    fn default() -> Self {
        Self {
            gamma: 2.2,
            log_base: 10.0,
            box_filter_size: 1,
        }
    }
}

pub trait ToColorImage {
    fn to_color_image(&self) -> ColorImage;
}

impl ToColorImage for DynamicImage {
    fn to_color_image(&self) -> ColorImage {
        let rgba_image = self.to_rgba8();
        let size = [self.width() as usize, self.height() as usize];
        let pixels: Vec<_> = rgba_image
            .pixels()
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        ColorImage { size, pixels }
    }
}

pub trait ToImageSource {
    fn to_image_source(&self, image_id: &str) -> egui::ImageSource;
}

impl ToImageSource for DynamicImage {
    fn to_image_source(&self, image_id: &str) -> egui::ImageSource {
        let image_buffer: Arc<[u8]> = Arc::from(self.to_rgba8().into_raw().into_boxed_slice());

        ImageSource::Bytes {
            uri: Cow::Owned(String::from(image_id)),
            bytes: egui::load::Bytes::Shared(image_buffer),
        }
    }
}

pub trait ShowResizedTexture {
    fn show_resized_texture(&mut self, texture: &TextureHandle);
}

impl ShowResizedTexture for egui::Ui {
    fn show_resized_texture(&mut self, texture: &TextureHandle) {
        let image_size = texture.size_vec2();
        let available_size = self.available_size();
        let aspect_ratio = image_size.x / image_size.y;
        let scaled_width = available_size.x.min(image_size.x);
        let scaled_height = (scaled_width / aspect_ratio).min(available_size.y);
        let desired_size = egui::Vec2::new(scaled_width, scaled_height);

        // Calculate the offset to center the image
        let offset_x = (available_size.x - desired_size.x) / 2.0;
        let offset_y = (available_size.y - desired_size.y) / 2.0;
        let desired_rect = egui::Rect::from_min_size(
            self.min_rect().min + egui::Vec2::new(offset_x, offset_y),
            desired_size,
        );

        // Paint the image
        self.painter().image(
            texture.id(),
            desired_rect,
            egui::Rect::from_min_max(
                egui::Pos2::new(0.0, 0.0),
                egui::Pos2::new(1.0, 1.0), // Use normalized texture coordinates
            ),
            egui::Color32::WHITE,
        );

        self.allocate_space(desired_size);
    }
}
