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
type GaussianBlurFn =
    unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32, sigma: f32);

/// Definition of the sobelEdgeDetection function from libcudaimg.
type SobelEdgeDetectionFn =
    unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32);

/// Definition of the laplaceEdgeDetection function from libcudaimg.
type LaplaceEdgeDetectionFn =
    unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32);

/// Definition of the harrisCornerDetection function from libcudaimg.
type HarrisCornerDetectionFn =
    unsafe extern "C" fn(image: *mut u8, image_len: u32, width: u32, height: u32);

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

/// Enum to represent the image processing functions.
///
/// * `Invert` - Invert the image.
/// * `GammaTransform` - Apply a gamma transformation to the image.
/// * `LogarithmicTransform` - Apply a logarithmic transformation to the image.
/// * `Grayscale` - Convert the image to grayscale.
/// * `ComputeHistogram` - Compute the histogram of the image.
/// * `BalanceHistogram` - Balance the histogram of the image.
/// * `BoxFilter` - Apply a box filter to the image.
/// * `GaussianBlur` - Apply a Gaussian blur to the image.
/// * `SobelEdgeDetection` - Apply Sobel edge detection to the image.
pub enum ImageProcessingFunction {
    Invert,
    GammaTransform(f32),
    LogarithmicTransform(f32),
    Grayscale,
    ComputeHistogram,
    BalanceHistogram,
    BoxFilter(u32),
    GaussianBlur(f32),
    SobelEdgeDetection,
    LaplaceEdgeDetection,
    HarrisCornerDetection,
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

/// Process an image using a specified image processing function.
/// The image is modified in place using the CUDA kernels.
/// The modified image is returned as a DynamicImage.
///
/// # Arguments
///
/// * `libcudaimg` - The libcudaimg library to use for image processing.
/// * `image` - The image to process.
/// * `function` - The image processing function to apply.
pub fn process_image(
    libcudaimg: &Library,
    image: &DynamicImage,
    function: ImageProcessingFunction,
) -> anyhow::Result<DynamicImage> {
    // Get the image data
    let mut img = image.to_cuda_image_data();

    info!("Image width: {}, height: {}", img.width, img.height);

    match function {
        ImageProcessingFunction::Invert => {
            let process_image: Symbol<InvertImageFn> = unsafe { libcudaimg.get(b"invertImage\0")? };
            unsafe {
                process_image(
                    img.bytes.as_mut_ptr(),
                    img.raw_len,
                    img.width * img.pixel_size,
                    img.height,
                );
            }
        }
        ImageProcessingFunction::GammaTransform(gamma) => {
            let process_image: Symbol<GammaTransformImage> =
                unsafe { libcudaimg.get(b"gammaTransformImage\0")? };

            unsafe {
                process_image(
                    img.bytes.as_mut_ptr(),
                    img.raw_len,
                    img.width * img.pixel_size,
                    img.height,
                    gamma,
                );
            }
        }
        ImageProcessingFunction::LogarithmicTransform(base) => {
            let process_image: Symbol<LogarithmicTransformImage> =
                unsafe { libcudaimg.get(b"logarithmicTransformImage\0")? };

            unsafe {
                process_image(
                    img.bytes.as_mut_ptr(),
                    img.raw_len,
                    img.width * img.pixel_size,
                    img.height,
                    base,
                );
            }
        }
        ImageProcessingFunction::Grayscale => {
            let process_image: Symbol<GrayscaleImageFn> =
                unsafe { libcudaimg.get(b"grayscaleImage\0")? };

            unsafe {
                process_image(
                    img.bytes.as_mut_ptr(),
                    img.raw_len,
                    img.width * img.pixel_size,
                    img.height,
                );
            }
        }
        ImageProcessingFunction::ComputeHistogram => {
            let process_image: Symbol<ComputeHistogramFn> =
                unsafe { libcudaimg.get(b"computeHistogram\0")? };

            let mut histogram = CudaHistogramData::default();

            unsafe {
                process_image(
                    img.bytes.as_mut_ptr(),
                    img.raw_len,
                    histogram.data.as_mut_ptr(),
                    img.width * img.pixel_size,
                    img.height,
                );
            }

            // Return explicitly to avoid creating a new image from the modified bytes
            return plot_histogram(&histogram);
        }
        ImageProcessingFunction::BalanceHistogram => {
            let process_image: Symbol<BalanceHistogramFn> =
                unsafe { libcudaimg.get(b"balanceHistogram\0")? };

            unsafe {
                process_image(
                    img.bytes.as_mut_ptr(),
                    img.raw_len,
                    img.width * img.pixel_size,
                    img.height,
                );
            }
        }
        ImageProcessingFunction::BoxFilter(filter_size) => {
            let process_image: Symbol<BoxFilterFn> = unsafe { libcudaimg.get(b"boxFilter\0")? };

            unsafe {
                process_image(
                    img.bytes.as_mut_ptr(),
                    img.raw_len,
                    img.width * img.pixel_size,
                    img.height,
                    filter_size,
                );
            }
        }
        ImageProcessingFunction::GaussianBlur(sigma) => {
            let process_image: Symbol<GaussianBlurFn> =
                unsafe { libcudaimg.get(b"gaussianBlur\0")? };

            unsafe {
                process_image(
                    img.bytes.as_mut_ptr(),
                    img.raw_len,
                    img.width * img.pixel_size,
                    img.height,
                    sigma,
                );
            }
        }
        ImageProcessingFunction::SobelEdgeDetection => {
            let process_image: Symbol<SobelEdgeDetectionFn> =
                unsafe { libcudaimg.get(b"sobelEdgeDetection\0")? };

            unsafe {
                process_image(
                    img.bytes.as_mut_ptr(),
                    img.raw_len,
                    img.width * img.pixel_size,
                    img.height,
                );
            }
        }
        ImageProcessingFunction::LaplaceEdgeDetection => {
            let process_image: Symbol<LaplaceEdgeDetectionFn> =
                unsafe { libcudaimg.get(b"laplaceEdgeDetection\0")? };

            unsafe {
                process_image(
                    img.bytes.as_mut_ptr(),
                    img.raw_len,
                    img.width * img.pixel_size,
                    img.height,
                );
            }
        }
        ImageProcessingFunction::HarrisCornerDetection => {
            let process_image: Symbol<HarrisCornerDetectionFn> =
                unsafe { libcudaimg.get(b"harrisCornerDetection\0")? };

            unsafe {
                process_image(
                    img.bytes.as_mut_ptr(),
                    img.raw_len,
                    img.width * img.pixel_size,
                    img.height,
                );
            }
        }
    };

    // Create a new image from the modified bytes
    let modified_image = image::DynamicImage::ImageRgb8(
        image::RgbImage::from_raw(img.width, img.height, img.bytes)
            .expect("Failed to create the modified image from bytes"),
    );

    Ok(modified_image)
}
