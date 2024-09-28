use egui::{ColorImage, ScrollArea, TextureHandle};
use image::DynamicImage;
use libloading::{Library, Symbol};

pub mod cudaimg;

pub struct ImageData {
    pub bytes: Vec<u8>,
    pub raw_len: u32,
    pub width: u32,
    pub height: u32,
    pub pixel_size: u32,
}

/// Get the image data from a DynamicImage
///
/// # Arguments
///
/// * `image` - The DynamicImage to get the data from
///
/// # Returns
///
/// * An ImageData struct containing the image data
pub fn get_image_data(image: &DynamicImage) -> ImageData {
    let img_rgb8 = image.to_rgb8();
    let bytes = img_rgb8.as_raw().to_owned();
    let raw_len = bytes.len() as u32;

    ImageData {
        bytes,
        raw_len,
        width: img_rgb8.width(),
        height: img_rgb8.height(),
        pixel_size: 3, // RGB format (3 bytes per pixel)
    }
}

pub fn dynamic_image_to_color_image(img: &DynamicImage) -> ColorImage {
    let rgba_image = img.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let pixels: Vec<_> = rgba_image
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    ColorImage { size, pixels }
}

pub fn display_image_in_ui(ui: &mut egui::Ui, texture: &TextureHandle) {
    ScrollArea::both().show(ui, |ui| {
        ui.image(texture);
    });
}
