use image::DynamicImage;
use libloading::{Library, Symbol};
use log::info;

/// Trait to convert an image to CudaImageData
pub trait ToCudaImageData {
    fn to_cuda_image_data(&self) -> CudaImageData;
}

/// Struct to hold the image data for communication with libcudaimg
///
/// # Fields
///
/// * `bytes` - The image data as a vector of bytes
/// * `raw_len` - The length of the raw image data
/// * `width` - The width of the image
/// * `height` - The height of the image
/// * `pixel_size` - The size of each pixel in bytes
pub struct CudaImageData {
    pub bytes: Vec<u8>,
    pub raw_len: u32,
    pub width: u32,
    pub height: u32,
    pub pixel_size: u32,
}

impl ToCudaImageData for DynamicImage {
    /// Get the image data from a DynamicImage
    ///
    /// # Arguments
    ///
    /// * `image` - The DynamicImage to get the data from
    ///
    /// # Returns
    ///
    /// * An ImageData struct containing the image data
    fn to_cuda_image_data(&self) -> CudaImageData {
        let img_rgb8 = self.to_rgb8();
        let bytes = img_rgb8.as_raw().to_owned();
        let raw_len = bytes.len() as u32;

        CudaImageData {
            bytes,
            raw_len,
            width: img_rgb8.width(),
            height: img_rgb8.height(),
            pixel_size: 3, // RGB format (3 bytes per pixel)
        }
    }
}

/// Definition of the processImage function from libcudaimg
type InvertImageFn = unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32);

pub fn invert_image(libcudaimg: &Library, image: &DynamicImage) -> anyhow::Result<DynamicImage> {
    // Get the invertImage function from the library
    let process_image: Symbol<InvertImageFn> = unsafe { libcudaimg.get(b"invertImage\0")? };

    // Get the image data
    let mut img = image.to_cuda_image_data();

    info!("Image width: {}, height: {}", img.width, img.height);

    // Call the processImage function (invert the image)
    unsafe {
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

/// Definition of the gammaTransformImage function from libcudaimg
type GammaTransformImage =
    unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32, gamma: f32);

pub fn gamma_transform_image(
    libcudaimg: &Library,
    image: &DynamicImage,
    gamma: f32,
) -> anyhow::Result<DynamicImage> {
    // Get the invertImage function from the library
    let process_image: Symbol<GammaTransformImage> =
        unsafe { libcudaimg.get(b"gammaTransformImage\0")? };

    // Get the image data
    let mut img = image.to_cuda_image_data();

    info!("Image width: {}, height: {}", img.width, img.height);

    // Call the processImage function (invert the image)
    unsafe {
        process_image(
            img.bytes.as_mut_ptr(),
            img.raw_len,
            img.width * img.pixel_size,
            img.height,
            gamma,
        );
    }

    // Create a new image from the modified bytes
    let modified_image = image::DynamicImage::ImageRgb8(
        image::RgbImage::from_raw(img.width, img.height, img.bytes)
            .expect("Failed to create the modified image from bytes"),
    );

    Ok(modified_image)
}

/// Definition of the logarithmicTransformImage function from libcudaimg
type LogarithmicTransformImage =
    unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32, base: f32);

pub fn logarithmic_transform_image(
    libcudaimg: &Library,
    image: &DynamicImage,
    base: f32,
) -> anyhow::Result<DynamicImage> {
    // Get the invertImage function from the library
    let process_image: Symbol<LogarithmicTransformImage> =
        unsafe { libcudaimg.get(b"logarithmicTransformImage\0")? };

    // Get the image data
    let mut img = image.to_cuda_image_data();

    info!("Image width: {}, height: {}", img.width, img.height);

    // Call the processImage function (invert the image)
    unsafe {
        process_image(
            img.bytes.as_mut_ptr(),
            img.raw_len,
            img.width * img.pixel_size,
            img.height,
            base,
        );
    }

    // Create a new image from the modified bytes
    let modified_image = image::DynamicImage::ImageRgb8(
        image::RgbImage::from_raw(img.width, img.height, img.bytes)
            .expect("Failed to create the modified image from bytes"),
    );

    Ok(modified_image)
}

/// Definition of the processImage function from libcudaimg
type GrayscaleImageFn =
    unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32);

pub fn grayscale_image(libcudaimg: &Library, image: &DynamicImage) -> anyhow::Result<DynamicImage> {
    // Get the invertImage function from the library
    let process_image: Symbol<GrayscaleImageFn> = unsafe { libcudaimg.get(b"grayscaleImage\0")? };

    // Get the image data
    let mut img = image.to_cuda_image_data();

    info!("Image width: {}, height: {}", img.width, img.height);

    // Call the processImage function (invert the image)
    unsafe {
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
