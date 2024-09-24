use libloading::Library;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    print!("Image to modify(./data/{{img_name.ext}}): ");

    // Read the image name from the user
    let mut image_name = String::new();
    io::stdout().flush()?;
    io::stdin().read_line(&mut image_name)?;

    // Split the image name into name and extension
    let image_path = Path::new(image_name.trim());
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

    // Load the libcudaimg library
    let lib_path = Path::new("data/libcudaimg.dll");
    let libcudaimg = unsafe { Library::new(lib_path)? };
    let base_path = std::env::current_dir()?;

    // Load the image from the 'data' directory
    let in_image_path = base_path.join(format!("data/{}.{}", &image_name, &image_ext));
    let out_image_path = base_path.join(format!("data/{}_inverted.{}", &image_name, &image_ext));

    // Load the image using the image crate
    let image = image::open(&in_image_path)?;

    // Invert the image using the CUDA library
    let inverted_image = cuda_imgproc::invert_image(&libcudaimg, &image)?;

    // Save the modified image
    inverted_image
        .save(&out_image_path)
        .expect("Failed to save the modified image");
    println!("Image inverted and saved to {:?}", &out_image_path);

    // The Library will be automatically unloaded when it goes out of scope
    Ok(())
}
