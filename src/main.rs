use cuda_imgproc::cudaimg;
use egui::{TextureHandle, TextureOptions};
use image::DynamicImage;
use libloading::Library;
use rfd::FileDialog;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Load the libcudaimg library
    let lib_path = Path::new("data/libcudaimg.dll");
    let libcudaimg = unsafe { Library::new(lib_path)? };

    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "License Plate Extractor",
        options,
        Box::new(|_cc| {
            //egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MyApp::new(libcudaimg)))
        }),
    );

    Ok(())

    // // Get the image path from the user
    // let image_path = get_image_path()?;

    // // Split the image path into name and extension
    // let (image_name, image_ext) = split_image_path(&image_path)?;

    // // Load the image from the 'data' directory
    // let in_image_path = base_path.join(format!("data/{}.{}", &image_name, &image_ext));
    // let out_image_path = base_path.join(format!("data/{}_inverted.{}", &image_name, &image_ext));

    // // Load the image using the image crate
    // let image = image::open(&in_image_path)?;

    // // Invert the image using the CUDA library
    // let inverted_image = cudaimg::invert_image(&libcudaimg, &image)?;

    // // Save the modified image
    // inverted_image
    //     .save(&out_image_path)
    //     .expect("Failed to save the modified image");
    // println!("Image inverted and saved to {:?}", &out_image_path);

    // The Library will be automatically unloaded when it goes out of scope
}

struct MyApp {
    libcudaimg: Library,
    image_path: Option<String>,
    modified_image_path: Option<String>,
    image: Option<DynamicImage>,
    modified_image: Option<DynamicImage>,
}

impl MyApp {
    fn new(libcudaimg: Library) -> Self {
        Self {
            libcudaimg,
            image_path: None,
            modified_image_path: None,
            image: None,
            modified_image: None,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Photoshop");

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

            ui.horizontal(|ui| {
                // Get the available width of the panel
                let available_width = ui.available_width();
                let half_width = available_width / 2.0;

                ui.vertical(|ui| {
                    ui.set_width(half_width);

                    if let Some(image) = &self.image {
                        cuda_imgproc::display_image_in_ui(ui, image, 1);
                    }
                });

                ui.vertical(|ui| {
                    ui.set_width(half_width);

                    if let Some(modified_image) = &self.modified_image {
                        cuda_imgproc::display_image_in_ui(ui, modified_image, 2);
                    }
                });
            });
        });
    }
}

fn get_image_path() -> anyhow::Result<PathBuf> {
    print!("Image to modify(./data/{{img_name.ext}}): ");

    // Read the image name from the user
    let mut image_name = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut image_name)?;

    Ok(Path::new(image_name.trim()).to_path_buf())
}

fn split_image_path(image_path: &Path) -> anyhow::Result<(String, String)> {
    let image_name = image_path
        .file_stem()
        .expect("Invalid image name")
        .to_str()
        .expect("Invalid image name")
        .to_owned();
    let image_ext = image_path
        .extension()
        .expect("Invalid image extension")
        .to_str()
        .expect("Invalid image extension")
        .to_owned();

    Ok((image_name, image_ext))
}
