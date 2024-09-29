use cuda_imgproc::{cudaimg, ShowResizedTexture, ToColorImage, ToImageSource};
use egui::{Response, ScrollArea, TextureHandle, TextureOptions};
use image::DynamicImage;
use libloading::Library;
use log::{debug, info};
use rfd::FileDialog;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Load the libcudaimg library
    let lib_path = Path::new("data/libcudaimg.dll");
    let libcudaimg = unsafe { Library::new(lib_path)? };

    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Photoshop",
        options,
        Box::new(|_cc| {
            //egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MyApp::new(libcudaimg)))
        }),
    );

    Ok(())
}

struct MyApp {
    libcudaimg: Library,
    image_loaded: AtomicBool,
    image_path: Option<String>,
    modified_image_path: Option<String>,
    image: Option<DynamicImage>,
    modified_image: Option<DynamicImage>,
    image_map: ImageMap,
}

impl MyApp {
    fn new(libcudaimg: Library) -> Self {
        Self {
            libcudaimg,
            image_loaded: AtomicBool::new(false),
            image_path: None,
            modified_image_path: None,
            image: None,
            modified_image: None,
            image_map: ImageMap::default(),
        }
    }
}

#[derive(Default)]
struct ImageMap {
    pub original_image: Option<TextureHandle>,
    pub modified_image: Option<TextureHandle>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Image selection and other information
            ui.horizontal(|ui| {
                // Select image button

                if ui.button("Select Image").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Image Files", &["jpg", "jpeg", "png"])
                        .pick_file()
                    {
                        self.image_path = Some(path.display().to_string());
                    }

                    if let Some(image_path) = &self.image_path {
                        let image = image::open(image_path).expect("Failed to open image");
                        self.image = Some(image);
                        ctx.request_repaint();
                    }
                }

                // Display the image path
                ui.label("Image Path:");

                if let Some(image_path) = &self.image_path {
                    ui.label(image_path);
                } else {
                    ui.label("No image selected");
                }
            });

            if ui.button("Invert image").clicked() {
                if let Some(image) = &self.image {
                    let modified_image = cudaimg::invert_image(&self.libcudaimg, image)
                        .expect("Failed to invert image");
                    self.modified_image = Some(modified_image);
                    ctx.request_repaint();
                }
            }

            let available_height = ui.available_height();

            // Display the images side by side
            ui.horizontal(|ui| {
                ui.set_height(available_height);

                // Get the available width of the panel
                let available_width = ui.available_width();
                let half_width = available_width / 2.0;

                ui.vertical(|ui| {
                    ui.set_width(half_width);

                    if let Some(image) = &self.image {
                        let texture: &egui::TextureHandle =
                            self.image_map.original_image.get_or_insert_with(|| {
                                // Load the texture only once.
                                ui.ctx().load_texture(
                                    "image_original",
                                    image.to_color_image(),
                                    Default::default(),
                                )
                            });

                        ui.show_resized_texture(texture, "original");

                        // ScrollArea::both()
                        //     .id_source("scroll_area_original")
                        //     .show(ui, |ui| {
                        //         ui.image(texture);
                        //     });
                    }
                });

                ui.vertical(|ui| {
                    ui.set_width(half_width);

                    if let Some(modified_image) = &self.modified_image {
                        let texture: &egui::TextureHandle =
                            self.image_map.modified_image.get_or_insert_with(|| {
                                // Load the texture only once.
                                ui.ctx().load_texture(
                                    "image_modified",
                                    modified_image.to_color_image(),
                                    Default::default(),
                                )
                            });

                        ui.show_resized_texture(texture, "modified");

                        // ScrollArea::both()
                        //     .id_source("scroll_area_modified")
                        //     .show(ui, |ui| {
                        //         ui.image(texture);
                        //     });
                    }
                });
            });
        });
    }
}
