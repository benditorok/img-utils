use image::DynamicImage;
use libloading::{Library, Symbol};
use log::info;
use plotters::prelude::*;

/// Definition of the invertImage function from libcudaimg.
type InvertImageFn = unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32);

/// Definition of the gammaTransformImage function from libcudaimg.
type GammaTransformImage =
    unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32, gamma: f32);

/// Definition of the logarithmicTransformImage function from libcudaimg.
type LogarithmicTransformImage =
    unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32, base: f32);

/// Definition of the grayscaleImage function from libcudaimg.
type GrayscaleImageFn =
    unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32);

/// Definition of the computeHistogram function from libcudaimg.
type ComputeHistogramFn = unsafe extern "C" fn(
    image: *mut u8,
    image_len: u32,
    histogram: *mut u32,
    width: u32,
    height: u32,
);

/// Definition of the balanceHistogram function from libcudaimg.
type BalanceHistogramFn =
    unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32);

/// Definition of the boxFilter function from libcudaimg.
type BoxFilterFn =
    unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32, filter_size: u32);

// Definition of the gaussFilter function from libcudaimg.
type GaussFilterFn = unsafe extern "C" fn(
    image: *mut u8,
    image_len: u32,
    width: u32,
    height: u32,
    filter_size: u32,
    sigma: f32,
);

/// Trait to convert an image to CudaImageData.
pub trait ToCudaImageData {
    fn to_cuda_image_data(&self) -> CudaImageData;
}

/// Struct to hold the image data for communication with libcudaimg.
///
/// # Fields
///
/// * `bytes` - The image data as a vector of bytes.
/// * `raw_len` - The length of the raw image data.
/// * `width` - The width of the image.
/// * `height` - The height of the image.
/// * `pixel_size` - The size of each pixel in bytes.
pub struct CudaImageData {
    pub bytes: Vec<u8>,
    pub raw_len: u32,
    pub width: u32,
    pub height: u32,
    pub pixel_size: u32,
}

impl ToCudaImageData for DynamicImage {
    /// Get the image data from a DynamicImage.
    ///
    /// # Arguments
    ///
    /// * `self` - The DynamicImage to get the data from.
    ///
    /// # Returns
    ///
    /// * An ImageData struct containing the image data.
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

/// Struct to hold the histogram data for communication with libcudaimg.
///
/// # Fields
///
/// * `data` - The histogram data as a vector of u32 values.
pub struct CudaHistogramData {
    pub data: Vec<u32>,
}

impl Default for CudaHistogramData {
    fn default() -> Self {
        CudaHistogramData {
            data: vec![0u32; 256],
        }
    }
}

/// Invert an image using libcudaimg.
///
/// # Arguments
///
/// * `libcudaimg` - The libcudaimg library.
/// * `image` - The image to invert.
///
/// # Returns
///
/// * The inverted image.
pub fn invert_image(libcudaimg: &Library, image: &DynamicImage) -> anyhow::Result<DynamicImage> {
    // Get the invertImage function from the library
    let process_image: Symbol<InvertImageFn> = unsafe { libcudaimg.get(b"invertImage\0")? };

    // Get the image data
    let mut img = image.to_cuda_image_data();

    info!("Image width: {}, height: {}", img.width, img.height);

    // Call the invertImage function
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

/// Apply a gamma transformation to an image using libcudaimg.
///
/// # Arguments
///
/// * `libcudaimg` - The libcudaimg library.
/// * `image` - The image to transform.
/// * `gamma` - The gamma value to use.
///
/// # Returns
///
/// * The transformed image.
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

    // Call the gammaTransformImage function
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

/// Apply a logarithmic transformation to an image using libcudaimg.
///
/// # Arguments
///
/// * `libcudaimg` - The libcudaimg library.
/// * `image` - The image to transform.
/// * `base` - The base value to use.
///
/// # Returns
///
/// * The transformed image.
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

    // Call the logarithmicTransformImage function
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

/// Convert an image to grayscale using libcudaimg.
///     
/// # Arguments
///
/// * `libcudaimg` - The libcudaimg library.
/// * `image` - The image to convert.
///
/// # Returns
///
/// * The grayscale image.
pub fn grayscale_image(libcudaimg: &Library, image: &DynamicImage) -> anyhow::Result<DynamicImage> {
    // Get the invertImage function from the library
    let process_image: Symbol<GrayscaleImageFn> = unsafe { libcudaimg.get(b"grayscaleImage\0")? };

    // Get the image data
    let mut img = image.to_cuda_image_data();

    info!("Image width: {}, height: {}", img.width, img.height);

    // Call the grayscaleImage function
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

/// Compute the histogram of an image using libcudaimg.
///
/// # Arguments
///
/// * `libcudaimg` - The libcudaimg library.
/// * `image` - The image to compute the histogram of.
///
/// # Returns
///
/// * The histogram data.
pub fn compute_histogram(
    libcudaimg: &Library,
    image: &DynamicImage,
) -> anyhow::Result<CudaHistogramData> {
    // Get the invertImage function from the library
    let process_image: Symbol<ComputeHistogramFn> =
        unsafe { libcudaimg.get(b"computeHistogram\0")? };

    let mut histogram = CudaHistogramData::default();

    // Get the image data
    let mut img = image.to_cuda_image_data();

    info!("Image width: {}, height: {}", img.width, img.height);

    // Call the computeHistogram function
    unsafe {
        process_image(
            img.bytes.as_mut_ptr(),
            img.raw_len,
            histogram.data.as_mut_ptr(),
            img.width * img.pixel_size,
            img.height,
        );
    }

    Ok(histogram)
}

/// Plot a histogram using plotters.
///
/// # Arguments
///
/// * `histogram` - The histogram data to plot.
///
/// # Returns
///
/// * The plotted histogram as a DynamicImage.
pub fn plot_histogram(histogram: &CudaHistogramData) -> anyhow::Result<DynamicImage> {
    let root = BitMapBackend::new("data/histogram.png", (600, 400)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Histogram", ("sans-serif", 30).into_font())
        .margin(5)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(
            (0u32..256u32).into_segmented(),
            0u32..*histogram.data.iter().max().unwrap_or(&0),
        )?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Count")
        .x_desc("Pixel value")
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(
                histogram
                    .data
                    .iter()
                    .enumerate()
                    .map(|(i, &count)| (i as u32, count)),
            ),
    )?;

    root.present()?;
    let img = image::open("data/histogram.png")?;
    Ok(img)
}

/// Balance the histogram of an image using libcudaimg.
///
/// # Arguments
///
/// * `libcudaimg` - The libcudaimg library.
/// * `image` - The image to balance the histogram of.
///
/// # Returns
///
/// * The image with a balanced histogram.
pub fn balance_image_histogram(
    libcudaimg: &Library,
    image: &DynamicImage,
) -> anyhow::Result<DynamicImage> {
    // Get the invertImage function from the library
    let process_image: Symbol<BalanceHistogramFn> =
        unsafe { libcudaimg.get(b"balanceHistogram\0")? };

    // Get the image data
    let mut img = image.to_cuda_image_data();

    info!("Image width: {}, height: {}", img.width, img.height);

    // Call the balanceHistogram function
    unsafe {
        process_image(
            img.bytes.as_mut_ptr(),
            img.raw_len,
            img.width * img.pixel_size,
            img.height,
        );
    }

    // Create a new image from the modified bytes
    let modified_image = image::DynamicImage::ImageRgb8(
        image::RgbImage::from_raw(img.width, img.height, img.bytes)
            .expect("Failed to create the modified image from bytes"),
    );

    Ok(modified_image)
}

pub fn box_filter(
    libcudaimg: &Library,
    image: &DynamicImage,
    filter_size: u32,
) -> anyhow::Result<DynamicImage> {
    // Get the invertImage function from the library
    let process_image: Symbol<BoxFilterFn> = unsafe { libcudaimg.get(b"boxFilter\0")? };

    // Get the image data
    let mut img = image.to_cuda_image_data();

    info!("Image width: {}, height: {}", img.width, img.height);

    // Call the boxFilter function
    unsafe {
        process_image(
            img.bytes.as_mut_ptr(),
            img.raw_len,
            img.width * img.pixel_size,
            img.height,
            filter_size,
        );
    }

    // Create a new image from the modified bytes
    let modified_image = image::DynamicImage::ImageRgb8(
        image::RgbImage::from_raw(img.width, img.height, img.bytes)
            .expect("Failed to create the modified image from bytes"),
    );

    Ok(modified_image)
}

pub fn gauss_filter(
    libcudaimg: &Library,
    image: &DynamicImage,
    filter_size: u32,
    sigma: f32,
) -> anyhow::Result<DynamicImage> {
    // Get the invertImage function from the library
    let process_image: Symbol<GaussFilterFn> = unsafe { libcudaimg.get(b"gaussFilter\0")? };

    // Get the image data
    let mut img = image.to_cuda_image_data();

    info!("Image width: {}, height: {}", img.width, img.height);

    // Call the gaussFilter function
    unsafe {
        process_image(
            img.bytes.as_mut_ptr(),
            img.raw_len,
            img.width * img.pixel_size,
            img.height,
            filter_size,
            sigma,
        );
    }

    // Create a new image from the modified bytes
    let modified_image = image::DynamicImage::ImageRgb8(
        image::RgbImage::from_raw(img.width, img.height, img.bytes)
            .expect("Failed to create the modified image from bytes"),
    );

    Ok(modified_image)
}
