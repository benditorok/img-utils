use libloading::Library;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    // Load the libcudaimg library
    let lib_path = Path::new("data/libcudaimg.dll");
    let libcudaimg = unsafe { Library::new(lib_path)? };
    let base_path = std::env::current_dir()?;

    // Load the image from the 'data' directory
    let in_image_path = base_path.join("data/ship.jpg");
    let out_image_path = base_path.join("data/ship_inverted.jpg");

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
