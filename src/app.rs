use crate::cudaimg::ImageProcessingFunction;
use crate::{ImageModifiers, ImageProcessingTask, ShowResizedTexture, TextureMap, ToColorImage};
use image::DynamicImage;
use libloading::Library;
use rfd::FileDialog;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::Mutex as TokioMutex;

#[allow(unused)]
pub struct MyApp {
    libcudaimg: Arc<TokioMutex<Library>>,
    image: Option<DynamicImage>,
    modified_image: Option<DynamicImage>,
    image_path_info: Option<PathBuf>,
    texture_map: TextureMap,
    image_modifiers: ImageModifiers,
    last_operation_duration: Option<std::time::Duration>,
    op_in_progress: Arc<Mutex<bool>>,
    tx: mpsc::Sender<ImageProcessingTask>,
    rx: mpsc::Receiver<ImageProcessingTask>,
}

impl MyApp {
    pub fn new(libcudaimg: Library) -> Self {
        let (tx, rx) = mpsc::channel(32);

        Self {
            libcudaimg: Arc::new(TokioMutex::new(libcudaimg)),
            image: None,
            modified_image: None,
            image_path_info: None,
            texture_map: TextureMap::default(),
            image_modifiers: ImageModifiers::default(),
            last_operation_duration: None,
            op_in_progress: Arc::new(Mutex::new(false)),
            tx,
            rx,
        }
    }
}

impl MyApp {
    fn draw_top_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Menu bar
            egui::menu::bar(ui, |ui| {
                // File menu
                ui.menu_button("File", |ui| {
                    // Open image button
                    if ui.button("Open Image").clicked() {
                        self.image = None;
                        self.modified_image = None;
                        self.image_path_info = None;
                        self.texture_map = TextureMap::default();

                        let tx = self.tx.clone();
                        let op_in_progress = Arc::clone(&self.op_in_progress);

                        tokio::spawn(async move {
                            // Wait for the previous operation to finish
                            while *op_in_progress.lock().unwrap() {
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            }

                            *op_in_progress.lock().unwrap() = true;

                            if let Some(path) = FileDialog::new()
                                .add_filter("Image Files", &["jpg", "jpeg", "png"])
                                .pick_file()
                            {
                                let image = image::open(&path).expect("Failed to open image");
                                tx.send(ImageProcessingTask::OpenImage { image, path })
                                    .await
                                    .unwrap();
                            }

                            *op_in_progress.lock().unwrap() = false;
                        });

                        ui.close_menu();
                    }

                    // Save image button
                    if ui.button("Save image").clicked() {
                        if self.modified_image.is_some() {
                            let op_in_progress = Arc::clone(&self.op_in_progress);

                            let modified_image = self.modified_image.clone(); // TODO: avoid clone
                            let image_path_info = self.image_path_info.clone(); // TODO: avoid clone

                            tokio::spawn(async move {
                                // Wait for the previous operation to finish
                                while *op_in_progress.lock().unwrap() {
                                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                }

                                *op_in_progress.lock().unwrap() = true;

                                if let Some(image) = modified_image {
                                    let exts = if let Some(impath) = &image_path_info {
                                        vec![impath
                                            .extension()
                                            .unwrap()
                                            .to_str()
                                            .unwrap()
                                            .to_string()]
                                    } else {
                                        vec![
                                            "jpg".to_string(),
                                            "jpeg".to_string(),
                                            "png".to_string(),
                                        ]
                                    };

                                    if let Some(path) = FileDialog::new()
                                        .add_filter("Image Files", exts.as_slice())
                                        .save_file()
                                    {
                                        image.save(&path).expect("Failed to save image");
                                    }
                                }

                                *op_in_progress.lock().unwrap() = false;
                            });
                        }

                        ui.close_menu();
                    }
                });

                // Tools menu
                ui.menu_button("Tools", |ui| {
                    // Invert image
                    if ui.button("Invert image").clicked() {
                        self.texture_map.modified_image = None;

                        let tx = self.tx.clone();
                        let op_in_progress = Arc::clone(&self.op_in_progress);

                        let image = self.image.clone(); // TODO: avoid clone
                        let library = Arc::clone(&self.libcudaimg);

                        tokio::spawn(async move {
                            // Wait for the previous operation to finish
                            while *op_in_progress.lock().unwrap() {
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            }

                            *op_in_progress.lock().unwrap() = true;

                            if let Some(image) = image {
                                let library = library.lock().await;

                                let start = std::time::Instant::now();

                                let modified_image = crate::cudaimg::process_image(
                                    &library,
                                    &image,
                                    ImageProcessingFunction::Invert,
                                )
                                .expect("Failed to invert image");

                                let duration = start.elapsed();
                                tx.send(ImageProcessingTask::OperationFinished {
                                    image: modified_image,
                                    duration,
                                })
                                .await
                                .unwrap();
                            }

                            *op_in_progress.lock().unwrap() = false;
                        });

                        ui.close_menu();
                    }

                    // Gamma transformation
                    ui.menu_button("Gamma transformation", |ui| {
                        if ui.button("Run").clicked() {
                            self.texture_map.modified_image = None;

                            let tx = self.tx.clone();
                            let op_in_progress = Arc::clone(&self.op_in_progress);

                            let image = self.image.clone(); // TODO: avoid clone
                            let library = Arc::clone(&self.libcudaimg);
                            let gamma = self.image_modifiers.gamma;

                            tokio::spawn(async move {
                                // Wait for the previous operation to finish
                                while *op_in_progress.lock().unwrap() {
                                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                }

                                *op_in_progress.lock().unwrap() = true;

                                if let Some(image) = image {
                                    let library = library.lock().await;

                                    let start = std::time::Instant::now();

                                    let modified_image = crate::cudaimg::process_image(
                                        &library,
                                        &image,
                                        ImageProcessingFunction::GammaTransform(gamma),
                                    )
                                    .expect("Failed to use gamma transformation on image");

                                    let duration = start.elapsed();
                                    tx.send(ImageProcessingTask::OperationFinished {
                                        image: modified_image,
                                        duration,
                                    })
                                    .await
                                    .unwrap();
                                }

                                *op_in_progress.lock().unwrap() = false;
                            });

                            ui.close_menu();
                        }

                        // Gamma slider
                        ui.label("Gamma");
                        ui.add(egui::Slider::new(
                            &mut self.image_modifiers.gamma,
                            0.1..=5.0,
                        ));
                    });

                    // Logarithmic transformation
                    ui.menu_button("Logarithmic transformation", |ui| {
                        if ui.button("Run").clicked() {
                            self.texture_map.modified_image = None;

                            let tx = self.tx.clone();
                            let op_in_progress = Arc::clone(&self.op_in_progress);

                            let image = self.image.clone(); // TODO: avoid clone
                            let library = Arc::clone(&self.libcudaimg);
                            let log_base = self.image_modifiers.log_base;

                            tokio::spawn(async move {
                                // Wait for the previous operation to finish
                                while *op_in_progress.lock().unwrap() {
                                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                }

                                *op_in_progress.lock().unwrap() = true;

                                if let Some(image) = image {
                                    let library = library.lock().await;

                                    let start = std::time::Instant::now();

                                    let modified_image = crate::cudaimg::process_image(
                                        &library,
                                        &image,
                                        ImageProcessingFunction::LogarithmicTransform(log_base),
                                    )
                                    .expect("Failed to use Logarithmic transformation on image");

                                    let duration = start.elapsed();
                                    tx.send(ImageProcessingTask::OperationFinished {
                                        image: modified_image,
                                        duration,
                                    })
                                    .await
                                    .unwrap();
                                }

                                *op_in_progress.lock().unwrap() = false;
                            });

                            ui.close_menu();
                        }

                        // Logarithmic base slider
                        ui.label("Base");
                        ui.add(egui::Slider::new(
                            &mut self.image_modifiers.log_base,
                            0.1..=100f32,
                        ));
                    });

                    // Grayscale conversion
                    if ui.button("Grayscale conversion").clicked() {
                        self.texture_map.modified_image = None;

                        let tx = self.tx.clone();
                        let op_in_progress = Arc::clone(&self.op_in_progress);

                        let image = self.image.clone(); // TODO: avoid clone
                        let library = Arc::clone(&self.libcudaimg);

                        tokio::spawn(async move {
                            // Wait for the previous operation to finish
                            while *op_in_progress.lock().unwrap() {
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            }

                            *op_in_progress.lock().unwrap() = true;

                            if let Some(image) = image {
                                let library = library.lock().await;

                                let start = std::time::Instant::now();

                                let modified_image = crate::cudaimg::process_image(
                                    &library,
                                    &image,
                                    ImageProcessingFunction::Grayscale,
                                )
                                .expect("Failed to convert to grayscale");

                                let duration = start.elapsed();
                                tx.send(ImageProcessingTask::OperationFinished {
                                    image: modified_image,
                                    duration,
                                })
                                .await
                                .unwrap();
                            }

                            *op_in_progress.lock().unwrap() = false;
                        });

                        ui.close_menu();
                    }

                    // Generate histogram
                    if ui.button("Generate histogram").clicked() {
                        self.texture_map.modified_image = None;

                        let tx = self.tx.clone();
                        let op_in_progress = Arc::clone(&self.op_in_progress);

                        let image = self.image.clone(); // TODO: avoid clone
                        let library = Arc::clone(&self.libcudaimg);

                        tokio::spawn(async move {
                            // Wait for the previous operation to finish
                            while *op_in_progress.lock().unwrap() {
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            }

                            *op_in_progress.lock().unwrap() = true;

                            if let Some(image) = image {
                                let library = library.lock().await;

                                let start = std::time::Instant::now();

                                let histogram = crate::cudaimg::process_image(
                                    &library,
                                    &image,
                                    ImageProcessingFunction::ComputeHistogram,
                                )
                                .expect("Failed to generate histogram");

                                let duration = start.elapsed();
                                tx.send(ImageProcessingTask::OperationFinished {
                                    image: histogram,
                                    duration,
                                })
                                .await
                                .unwrap();
                            }

                            *op_in_progress.lock().unwrap() = false;
                        });

                        ui.close_menu();
                    }

                    // Balance histogram
                    if ui.button("Balance histogram").clicked() {
                        self.texture_map.modified_image = None;

                        let tx = self.tx.clone();
                        let op_in_progress = Arc::clone(&self.op_in_progress);

                        let image = self.image.clone(); // TODO: avoid clone
                        let library = Arc::clone(&self.libcudaimg);

                        tokio::spawn(async move {
                            // Wait for the previous operation to finish
                            while *op_in_progress.lock().unwrap() {
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            }

                            *op_in_progress.lock().unwrap() = true;

                            if let Some(image) = image {
                                let library = library.lock().await;

                                let start = std::time::Instant::now();

                                let modified_image = crate::cudaimg::process_image(
                                    &library,
                                    &image,
                                    ImageProcessingFunction::BalanceHistogram,
                                )
                                .expect("Failed to balance histogram");

                                let duration = start.elapsed();
                                tx.send(ImageProcessingTask::OperationFinished {
                                    image: modified_image,
                                    duration,
                                })
                                .await
                                .unwrap();
                            }

                            *op_in_progress.lock().unwrap() = false;
                        });

                        ui.close_menu();
                    }

                    // Box filter
                    ui.menu_button("Box filter", |ui| {
                        if ui.button("Run").clicked() {
                            self.texture_map.modified_image = None;

                            let tx = self.tx.clone();
                            let op_in_progress = Arc::clone(&self.op_in_progress);

                            let image = self.image.clone(); // TODO: avoid clone
                            let library = Arc::clone(&self.libcudaimg);
                            let filter_size = self.image_modifiers.box_filter_size;

                            tokio::spawn(async move {
                                // Wait for the previous operation to finish
                                while *op_in_progress.lock().unwrap() {
                                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                }

                                *op_in_progress.lock().unwrap() = true;

                                if let Some(image) = image {
                                    let library = library.lock().await;

                                    let start = std::time::Instant::now();

                                    let modified_image = crate::cudaimg::process_image(
                                        &library,
                                        &image,
                                        ImageProcessingFunction::BoxFilter(filter_size),
                                    )
                                    .expect("Failed to use Box filter on image");

                                    let duration = start.elapsed();
                                    tx.send(ImageProcessingTask::OperationFinished {
                                        image: modified_image,
                                        duration,
                                    })
                                    .await
                                    .unwrap();
                                }

                                *op_in_progress.lock().unwrap() = false;
                            });

                            ui.close_menu();
                        }

                        // Box filter size slider
                        ui.label("Filter size");
                        ui.add(egui::Slider::new(
                            &mut self.image_modifiers.box_filter_size,
                            0u32..=80u32,
                        ));
                    });

                    // Gaussian blur
                    ui.menu_button("Gaussian blur", |ui| {
                        if ui.button("Run").clicked() {
                            self.texture_map.modified_image = None;

                            let tx = self.tx.clone();
                            let op_in_progress = Arc::clone(&self.op_in_progress);

                            let image = self.image.clone(); // TODO: avoid clone
                            let library = Arc::clone(&self.libcudaimg);
                            let sigma = self.image_modifiers.gauss_sigma;

                            tokio::spawn(async move {
                                // Wait for the previous operation to finish
                                while *op_in_progress.lock().unwrap() {
                                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                }

                                *op_in_progress.lock().unwrap() = true;

                                if let Some(image) = image {
                                    let library = library.lock().await;

                                    let start = std::time::Instant::now();

                                    let modified_image = crate::cudaimg::process_image(
                                        &library,
                                        &image,
                                        ImageProcessingFunction::GaussianBlur(sigma),
                                    )
                                    .expect("Failed to use Gaussian blur on image");

                                    let duration = start.elapsed();
                                    tx.send(ImageProcessingTask::OperationFinished {
                                        image: modified_image,
                                        duration,
                                    })
                                    .await
                                    .unwrap();
                                }

                                *op_in_progress.lock().unwrap() = false;
                            });

                            ui.close_menu();
                        }

                        // Gauss sigma slider
                        ui.label("Sigma");
                        ui.add(egui::Slider::new(
                            &mut self.image_modifiers.gauss_sigma,
                            0.1..=5.0,
                        ));
                    });

                    // Sobel edge detection
                    if ui.button("Sobel edge detection").clicked() {
                        self.texture_map.modified_image = None;

                        let tx = self.tx.clone();
                        let op_in_progress = Arc::clone(&self.op_in_progress);

                        let image = self.image.clone(); // TODO: avoid clone
                        let library = Arc::clone(&self.libcudaimg);

                        tokio::spawn(async move {
                            // Wait for the previous operation to finish
                            while *op_in_progress.lock().unwrap() {
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            }

                            *op_in_progress.lock().unwrap() = true;

                            if let Some(image) = image {
                                let library = library.lock().await;

                                let start = std::time::Instant::now();
                                let modified_image = crate::cudaimg::process_image(
                                    &library,
                                    &image,
                                    ImageProcessingFunction::SobelEdgeDetection,
                                )
                                .expect("Failed to use Sobel edge detection on image");

                                // TODO do not panic on fail but show a message and set the op_in_progress to false on tokio tasks

                                let duration = start.elapsed();
                                tx.send(ImageProcessingTask::OperationFinished {
                                    image: modified_image,
                                    duration,
                                })
                                .await
                                .unwrap();
                            }

                            *op_in_progress.lock().unwrap() = false;
                        });

                        ui.close_menu();
                    }
                });

                // Display the duration of the last operation
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Apply modification and replace the image with the modified one
                    if ui.button("Apply current").clicked() {
                        if let Some(modified_image) = self.modified_image.take() {
                            self.image = Some(modified_image);
                            self.texture_map = TextureMap::default();
                        }
                    }

                    // Remove the current modification
                    if ui.button("Remove current").clicked() {
                        let _ = self.modified_image.take();
                        self.texture_map = TextureMap::default();
                    }
                });
            });
        });
    }

    fn draw_central_panel(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Main window contents
        egui::CentralPanel::default().show(ctx, |ui| {
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
                        if *self.op_in_progress.lock().unwrap() {
                            ui.label("Operation in progress...");
                        } else if let Some(path) = &self.image_path_info {
                            ui.label(format!("Image: {}", path.display()));
                        }

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

    fn post_update(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle results from async tasks
        while let Ok(result) = self.rx.try_recv() {
            match result {
                ImageProcessingTask::OpenImage { image, path } => {
                    self.image = Some(image);
                    self.image_path_info = Some(path);
                }
                ImageProcessingTask::OperationFinished { image, duration } => {
                    self.modified_image = Some(image);
                    self.texture_map = TextureMap::default(); // TODO: reset only the modified image texture
                    self.last_operation_duration = Some(duration);
                }
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update the menu bar
        self.draw_top_panel(ctx, _frame);

        // Update the main panel
        self.draw_central_panel(ctx, _frame);

        // Post update
        self.post_update(ctx, _frame);

        // Important: tell the app to repaint after the update
        ctx.request_repaint();
    }
}
