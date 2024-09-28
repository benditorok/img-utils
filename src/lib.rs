use egui::{ColorImage, ScrollArea, TextureHandle, TextureOptions};
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

// pub fn display_image_in_ui(ui: &mut egui::Ui, texture: &TextureHandle) {
//     ScrollArea::both().show(ui, |ui| {
//         ui.image(texture);
//     });
// }

pub fn display_image_in_ui(ui: &mut egui::Ui, image: &DynamicImage, component_id: u32) {
    // Create a unique IDs
    let scroll_area_id = format!("scroll_area_{}", component_id);
    let texture_id = format!("texture_{}", component_id);

    ScrollArea::both().id_source(scroll_area_id).show(ui, |ui| {
        // Convert the DynamicImage to a ColorImage
        let rgba_image = image.to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let pixels: Vec<_> = rgba_image
            .pixels()
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        let image = ColorImage { size, pixels };

        // Load the image as a texture
        let texture: TextureHandle =
            ui.ctx()
                .load_texture(texture_id, image, TextureOptions::default());

        // Get available size of the UI element
        let available_size = ui.available_size();

        // Get the size of the texture
        let image_size = texture.size_vec2();
        let aspect_ratio = image_size.x / image_size.y;

        // Calculate the scaled dimensions to fit the available space
        let scaled_width = available_size.x.min(image_size.x);
        let scaled_height = (scaled_width / aspect_ratio).min(available_size.y);

        // Define a rectangle to draw the image in, ensuring it fits
        let desired_rect = ui.min_rect().intersect(egui::Rect::from_min_size(
            ui.min_rect().min,
            egui::Vec2::new(scaled_width, scaled_height),
        ));

        // Use the `Painter` to draw the image in the defined rectangle
        ui.painter().image(
            texture.id(),
            desired_rect,
            egui::Rect::from_min_max(
                egui::Pos2::new(0.0, 0.0),
                egui::Pos2::new(image_size.x, image_size.y),
            ),
            egui::Color32::WHITE,
        );

        // Allocate space for the image to ensure it fits into the ScrollArea
        ui.allocate_space(egui::Vec2::new(scaled_width, scaled_height));
    });
}
