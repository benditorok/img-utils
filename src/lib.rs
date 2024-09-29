use egui::{ColorImage, ImageSource, ScrollArea, TextureHandle, TextureOptions};
use image::DynamicImage;
use libloading::{Library, Symbol};
use std::{borrow::Cow, sync::Arc};

pub mod cudaimg;

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

impl ToImageSource for &DynamicImage {
    fn to_image_source(&self, image_id: &str) -> egui::ImageSource {
        let image_buffer: Arc<[u8]> = Arc::from(self.to_rgba8().into_raw().into_boxed_slice());

        ImageSource::Bytes {
            uri: Cow::Owned(String::from(image_id)),
            bytes: egui::load::Bytes::Shared(image_buffer),
        }

        // let modified_image = self(image);
        // let color_image = modified_image.to_color_image();
        // ImageSource::Bytes(color_image)
    }
}

fn load_image_from_memory(image_data: &[u8]) -> Result<ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

pub fn display_image_in_ui(ui: &mut egui::Ui, image: &DynamicImage, image_id: &str) {
    // Create a unique IDs
    let texture_id = egui::Id::new(format!("texture_{}", image_id));
    let scroll_area_id = egui::Id::new(format!("scroll_area_{}", image_id));

    let texture = store_texture(ui, image, image_id);

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

    ScrollArea::both().id_source(scroll_area_id).show(ui, |ui| {
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

fn store_texture(ui: &mut egui::Ui, image: &image::DynamicImage, image_id: &str) -> TextureHandle {
    // Store the texture in the UI memory to avoid reloading it every frame
    let texture_id = egui::Id::new(format!("texture_{}", image_id));

    // Check if the texture is already in the cache
    if let Some(texture) = ui
        .ctx()
        .memory_mut(|mem| mem.data.get_persisted::<TextureHandle>(texture_id))
    {
        return texture.clone();
    }

    let image = image.to_color_image();

    // Load the image as a texture
    let texture = ui.ctx().load_texture(
        texture_id.short_debug_format(),
        image,
        TextureOptions::default(),
    );

    texture
}
