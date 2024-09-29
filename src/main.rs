use egui::TextureHandle;
use image::DynamicImage;
use img_utils::{ShowResizedTexture, ToColorImage};
use libloading::Library;
use log::info;
use rfd::FileDialog;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Load the libcudaimg library
    let lib_path = Path::new("data/libcudaimg.dll");
    let libcudaimg = unsafe { Library::new(lib_path)? };

    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Image Processing Utility",
        options,
        Box::new(|_cc| {
            //egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MyApp::new(libcudaimg)))
        }),
    );

    Ok(())
}

#[derive(Default)]
struct TextureMap {
    pub original_image: Option<TextureHandle>,
    pub modified_image: Option<TextureHandle>,
}

#[allow(unused)]
struct MyApp {
    libcudaimg: Library,
    image_path: Option<String>,
    modified_image_path: Option<String>,
    image: Option<DynamicImage>,
    modified_image: Option<DynamicImage>,
    texture_map: TextureMap,
}

impl MyApp {
    fn new(libcudaimg: Library) -> Self {
        Self {
            libcudaimg,
            image_path: None,
            modified_image_path: None,
            image: None,
            modified_image: None,
            texture_map: TextureMap::default(),
        }
    }
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
                        self.image = None;
                        self.modified_image = None;
                        self.texture_map = TextureMap::default();
                    }

                    if self.image.is_none() {
                        if let Some(image_path) = &self.image_path {
                            let image = image::open(image_path).expect("Failed to open image");
                            self.image = Some(image);
                        }
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

            // Image processing tools
            ui.horizontal(|ui| {
                // Invert image button
                if ui.button("Invert image").clicked() {
                    self.texture_map.modified_image = None;

                    if let Some(image) = &self.image {
                        let start = std::time::Instant::now();
                        let modified_image =
                            img_utils::cudaimg::invert_image(&self.libcudaimg, image)
                                .expect("Failed to invert image");
                        let duration = start.elapsed();
                        info!("Invert image duration: {:?}", duration);

                        self.modified_image = Some(modified_image);
                    }
                }

                // Gamma transform image button
                if ui.button("Gamma transformation").clicked() {
                    self.texture_map.modified_image = None;

                    if let Some(image) = &self.image {
                        let start = std::time::Instant::now();
                        let modified_image =
                            img_utils::cudaimg::gamma_transform_image(&self.libcudaimg, image)
                                .expect("Failed to use Gamma transformation on image");
                        let duration = start.elapsed();
                        info!("Gamma transformation duration: {:?}", duration);

                        self.modified_image = Some(modified_image);
                    }
                }
                // ... other image processing tools
            });

            // Display the images side by side
            let available_height = ui.available_height();
            ui.horizontal(|ui| {
                ui.set_height(available_height);

                // Get the available width of the panel
                let available_width = ui.available_width();
                let half_width = available_width / 2.0;

                ui.vertical(|ui| {
                    ui.set_width(half_width - ui.spacing().window_margin.left);

                    if let Some(image) = &self.image {
                        let texture: &egui::TextureHandle =
                            self.texture_map.original_image.get_or_insert_with(|| {
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

                ui.add_space(ui.spacing().window_margin.right);

                ui.vertical(|ui| {
                    ui.set_width(half_width - ui.spacing().window_margin.right);

                    if let Some(modified_image) = &self.modified_image {
                        let texture: &egui::TextureHandle =
                            self.texture_map.modified_image.get_or_insert_with(|| {
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
