use image::DynamicImage;
use libloading::{Library, Symbol};

/// Definition of the processImage function from libcudaimg
type InvertImageFn = unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32);

pub fn invert_image(libcudaimg: &Library, image: &DynamicImage) -> anyhow::Result<DynamicImage> {
    // Get the invertImage function from the library
    let process_image: Symbol<InvertImageFn> = unsafe { libcudaimg.get(b"invertImage\0")? };

    // Get the image data
    let mut img = crate::get_image_data(&image);

    println!("Image width: {}, height: {}", img.width, img.height);

    // Call the processImage function (invert the image)
    unsafe {
        // Note: the width * 3 is used because the image is in RGB format, which means that each pixel has 3 bytes (R, G, B)
        process_image(
            img.bytes.as_mut_ptr(),
            img.raw_len,
            img.width * img.pixel_size,
            img.height,
        );
    }

    // Create a new image from the modified bytes
    let inverted_image = image::DynamicImage::ImageRgb8(
        image::RgbImage::from_raw(img.width, img.height, img.bytes)
            .expect("Failed to create the modified image from bytes"),
    );

    Ok(inverted_image)
}
