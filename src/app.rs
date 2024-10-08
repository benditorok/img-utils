use std::path::PathBuf;

use crate::{ImageModifiers, ShowResizedTexture, TextureMap, ToColorImage};
use egui::epaint::tessellator::Path;
use image::DynamicImage;
use libloading::Library;
use rfd::FileDialog;

#[allow(unused)]
pub struct MyApp {
    libcudaimg: Library,
    image: Option<DynamicImage>,
    image_path_info: Option<PathBuf>,
    modified_image: Option<DynamicImage>,
    texture_map: TextureMap,
    image_modifiers: ImageModifiers,
    last_operation_duration: Option<std::time::Duration>,
}

impl MyApp {
    pub fn new(libcudaimg: Library) -> Self {
        Self {
            libcudaimg,
            image: None,
            image_path_info: None,
            modified_image: None,
            texture_map: TextureMap::default(),
            image_modifiers: ImageModifiers::default(),
            last_operation_duration: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Menu bar
            egui::menu::bar(ui, |ui| {
                // File menu
                ui.menu_button("File", |ui| {
                    // Open image button
                    if ui.button("Open Image").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("Image Files", &["jpg", "jpeg", "png"])
                            .pick_file()
                        {
                            self.image = Some(image::open(&path).expect("Failed to open image"));
                            self.image_path_info = Some(path);
                            self.modified_image = None;
                            self.texture_map = TextureMap::default();
                        }

                        ui.close_menu();
                    }

                    // Save image button
                    if ui.button("Save image").clicked() {
                        if let Some(image) = &self.modified_image {
                            let mut exts = vec!["jpg", "jpeg", "png"];

                            if let Some(impath) = &self.image_path_info {
                                exts = vec![impath.extension().unwrap().to_str().unwrap()];
                            }

                            if let Some(path) = FileDialog::new()
                                .add_filter("Image Files", exts.as_slice())
                                .save_file()
                            {
                                image.save(&path).expect("Failed to save image");
                            }
                        }

                        ui.close_menu();
                    }
                });

                // Tools menu
                ui.menu_button("Tools", |ui| {
                    // Invert image
                    if ui.button("Invert image").clicked() {
                        self.texture_map.modified_image = None;

                        if let Some(image) = &self.image {
                            let start = std::time::Instant::now();
                            let modified_image =
                                crate::cudaimg::invert_image(&self.libcudaimg, image)
                                    .expect("Failed to invert image");
                            self.last_operation_duration = Some(start.elapsed());

                            self.modified_image = Some(modified_image);
                        }

                        ui.close_menu();
                    }

                    // Gamma transformation
                    ui.menu_button("Gamma transformation", |ui| {
                        if ui.button("Run").clicked() {
                            self.texture_map.modified_image = None;

                            if let Some(image) = &self.image {
                                let start = std::time::Instant::now();
                                let modified_image = crate::cudaimg::gamma_transform_image(
                                    &self.libcudaimg,
                                    image,
                                    self.image_modifiers.gamma,
                                )
                                .expect("Failed to use gamma transformation on image");
                                self.last_operation_duration = Some(start.elapsed());

                                self.modified_image = Some(modified_image);
                            }

                            ui.close_menu();
                        }

                        ui.label("Gamma value");

                        // Gamma slider
                        ui.add(egui::Slider::new(
                            &mut self.image_modifiers.gamma,
                            0.1..=5.0,
                        ));
                    });
                });
            });
        });

        // Main window contents
        egui::CentralPanel::default().show(ctx, |ui| {
            // Image processing tools
            ui.horizontal(|ui| {
                // Invert image
                if ui.button("Invert image").clicked() {
                    self.texture_map.modified_image = None;

                    if let Some(image) = &self.image {
                        let start = std::time::Instant::now();
                        let modified_image = crate::cudaimg::invert_image(&self.libcudaimg, image)
                            .expect("Failed to invert image");
                        self.last_operation_duration = Some(start.elapsed());

                        self.modified_image = Some(modified_image);
                    }
                }

                // Gamma transformation
                ui.vertical(|ui| {
                    if ui.button("Gamma transformation").clicked() {
                        self.texture_map.modified_image = None;

                        if let Some(image) = &self.image {
                            let start = std::time::Instant::now();
                            let modified_image = crate::cudaimg::gamma_transform_image(
                                &self.libcudaimg,
                                image,
                                self.image_modifiers.gamma,
                            )
                            .expect("Failed to use gamma transformation on image");
                            self.last_operation_duration = Some(start.elapsed());

                            self.modified_image = Some(modified_image);
                        }
                    }

                    // Gamma slider
                    ui.add(egui::Slider::new(
                        &mut self.image_modifiers.gamma,
                        0.1..=5.0,
                    ));
                });

                // Logarithmic transformation
                ui.vertical(|ui| {
                    if ui.button("Logarithmic transformation").clicked() {
                        self.texture_map.modified_image = None;

                        if let Some(image) = &self.image {
                            let start = std::time::Instant::now();
                            let modified_image = crate::cudaimg::logarithmic_transform_image(
                                &self.libcudaimg,
                                image,
                                self.image_modifiers.log_base,
                            )
                            .expect("Failed to use logarithmic transformation on image");
                            self.last_operation_duration = Some(start.elapsed());

                            self.modified_image = Some(modified_image);
                        }
                    }

                    // Logarithmic base slider
                    ui.add(egui::Slider::new(
                        &mut self.image_modifiers.log_base,
                        0.1..=100f32,
                    ));
                });

                // Convert to grayscale
                ui.vertical(|ui| {
                    if ui.button("Convert to grayscale").clicked() {
                        self.texture_map.modified_image = None;

                        if let Some(image) = &self.image {
                            let start = std::time::Instant::now();
                            let modified_image =
                                crate::cudaimg::grayscale_image(&self.libcudaimg, image)
                                    .expect("Failed to convert to grayscale");
                            self.last_operation_duration = Some(start.elapsed());

                            self.modified_image = Some(modified_image);
                        }
                    }
                });

                // Generate histogram
                ui.vertical(|ui| {
                    if ui.button("Generate histogram").clicked() {
                        self.texture_map.modified_image = None;

                        if let Some(image) = &self.image {
                            let start = std::time::Instant::now();
                            let histogram =
                                crate::cudaimg::compute_histogram(&self.libcudaimg, image)
                                    .expect("Failed to generate histogram");
                            self.last_operation_duration = Some(start.elapsed());

                            let histogram = crate::cudaimg::plot_histogram(&histogram)
                                .expect("Failed to plot histogram");

                            self.modified_image = Some(histogram);
                        }
                    }
                });

                // Balance histogram
                ui.vertical(|ui| {
                    if ui.button("Balance histogram").clicked() {
                        self.texture_map.modified_image = None;

                        if let Some(image) = &self.image {
                            let start = std::time::Instant::now();
                            let balanced_image =
                                crate::cudaimg::balance_image_histogram(&self.libcudaimg, image)
                                    .expect("Failed to balance histogram");
                            self.last_operation_duration = Some(start.elapsed());

                            self.modified_image = Some(balanced_image);
                        }
                    }
                });

                // Box filter
                ui.vertical(|ui| {
                    if ui.button("Box filter").clicked() {
                        self.texture_map.modified_image = None;

                        if let Some(image) = &self.image {
                            let start = std::time::Instant::now();
                            let modified_image = crate::cudaimg::box_filter(
                                &self.libcudaimg,
                                image,
                                self.image_modifiers.box_filter_size,
                            )
                            .expect("Failed to use Box filter on image");
                            self.last_operation_duration = Some(start.elapsed());

                            self.modified_image = Some(modified_image);
                        }
                    }

                    // Box filter size slider
                    ui.add(egui::Slider::new(
                        &mut self.image_modifiers.box_filter_size,
                        0u32..=80u32,
                    ));
                });

                // ... other image processing tools
            });

            // Display the images side by side
            let available_height = ui.available_height();
            ui.horizontal(|ui| {
                ui.set_height(available_height);

                // Get the available width of the panel
                let available_width = ui.available_width();
                let half_width = available_width / 2.0;

                // Display the original image
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

                        ui.show_resized_texture(texture);
                    }
                });

                ui.add_space(ui.spacing().window_margin.right);

                // Display the modified image
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

                        ui.show_resized_texture(texture);
                    }
                });

                egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                    // Image selection and other information
                    ui.horizontal(|ui| {
                        // Display the duration of the last operation
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(duration) = self.last_operation_duration {
                                ui.label(format!("Last operation duration: {:?}", duration));
                            } else {
                                ui.label("No operation performed yet");
                            }
                        });
                    });
                });
            });
        });
    }
}
