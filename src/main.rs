use image::EncodableLayout;
use libloading::{Library, Symbol};
use std::{env, ffi::c_void, path::Path};

// Define the signature for the processImage function
type InvertImageFn = unsafe extern "C" fn(image: *mut u8, width: u32, height: u32, len: u32);

fn main() -> anyhow::Result<()> {
    let lib_path = Path::new(env!("OUT_DIR")).join("data/libcudaimg.dll");
    let in_image_path = Path::new(env!("OUT_DIR")).join("data/ship.jpg");
    let out_image_path = Path::new(env!("OUT_DIR")).join("data/ship_inverted.jpg");

    // Load the shared library (DLL)
    let libcudaimg = unsafe { Library::new(lib_path)? };

    // Get the invertImage function from the library
    let process_image: Symbol<InvertImageFn> = unsafe { libcudaimg.get(b"invertImage\0")? };

    // Load the image using the image crate
    let img = image::open(in_image_path)?;
    let img_rgb8 = img.to_rgb8();
    let mut img_asbytes = img_rgb8.as_raw().to_owned();
    let width: u32 = img_rgb8.width();
    let height: u32 = img_rgb8.height();
    let raw_len: u32 = img_asbytes.len() as u32;

    println!("witdh: {}, height: {}", width, height);

    // Call the processImage function (invert the image)
    unsafe {
        // Note: the width * 3 is used because the image is in RGB format, which means that each pixel has 3 bytes (R, G, B)
        process_image(img_asbytes.as_mut_ptr(), width * 3, height, raw_len);
    }

    // Save the modified image
    let img = image::DynamicImage::ImageRgb8(
        image::RgbImage::from_raw(width, height, img_asbytes)
            .expect("Failed to create the modified image from bytes"),
    );

    img.save(&out_image_path)
        .expect("Failed to save the modified image");

    println!("Image inverted and saved to {:?}", out_image_path);
    // The Library will be automatically unloaded when it goes out of scope

    Ok(())
}
