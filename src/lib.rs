use image::DynamicImage;
use libloading::{Library, Symbol};
use std::{env, path::Path};

pub fn invert_image(libcudaimg: &Library, image: &DynamicImage) -> anyhow::Result<DynamicImage> {
    // Define the signature for the processImage function
    type InvertImageFn =
        unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32);
    // Get the invertImage function from the library
    let process_image: Symbol<InvertImageFn> = unsafe { libcudaimg.get(b"invertImage\0")? };

    // Load the image using the image crate
    let img_rgb8 = image.to_rgb8();
    let mut img_asbytes = img_rgb8.as_raw().to_owned();
    let width: u32 = img_rgb8.width();
    let height: u32 = img_rgb8.height();

    println!("Image width: {}, height: {}", width, height);

    // Call the processImage function (invert the image)
    unsafe {
        // Note: the width * 3 is used because the image is in RGB format, which means that each pixel has 3 bytes (R, G, B)
        process_image(
            img_asbytes.as_mut_ptr(),
            img_asbytes.len() as u32,
            width * 3,
            height,
        );
    }

    // Create a new image from the modified bytes
    let inverted_image = image::DynamicImage::ImageRgb8(
        image::RgbImage::from_raw(width, height, img_asbytes)
            .expect("Failed to create the modified image from bytes"),
    );

    Ok(inverted_image)
}
