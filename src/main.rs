use libloading::Library;
use std::{env, path::Path};

fn main() -> anyhow::Result<()> {
    // Load the libcudaimg library
    let lib_path = Path::new(env!("OUT_DIR")).join("data/libcudaimg.dll");
    let libcudaimg = unsafe { Library::new(lib_path)? };

    let in_image_path = Path::new(env!("OUT_DIR")).join("data/ship.jpg");
    let out_image_path = Path::new(env!("OUT_DIR")).join("data/ship_inverted.jpg");

    let image = image::open(&in_image_path)?;

    let inverted_image = callcuda_rs::invert_image(&libcudaimg, &image)?;

    // Save the modified image
    inverted_image
        .save(&out_image_path)
        .expect("Failed to save the modified image");
    println!("Image inverted and saved to {:?}", out_image_path);

    // The Library will be automatically unloaded when it goes out of scope
    Ok(())
}
