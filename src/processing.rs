use crate::constants::*;
use crate::error::{CompressionError, Result};
use crate::formats::{OutputFormat, determine_output_format};
use crate::utils::{create_progress_spinner, print_compression_result, validate_file_exists};
use image::{DynamicImage, GenericImageView, ImageReader};
use oxipng::{Deflaters, InFile, Options, OutFile};
use std::fs;
use std::num::NonZeroU8;
use std::path::{Path, PathBuf};

/// Configuration options for image compression
/// 
/// Contains all the parameters needed to control the compression process,
/// including quality settings, dimensions, and output format.
#[derive(Debug, Clone)]
pub struct CompressionOptions {
    /// Compression quality (1-100 scale)
    pub quality: u8,
    /// Maximum output width in pixels (optional)
    pub width: Option<u32>,
    /// Maximum output height in pixels (optional)  
    pub height: Option<u32>,
    /// Output format override (optional)
    pub format: Option<String>,
}

impl CompressionOptions {
    /// Create new compression options with validation
    /// 
    /// # Arguments
    /// * `quality` - Optional quality setting (1-100), defaults to 80
    /// * `width` - Optional maximum width in pixels
    /// * `height` - Optional maximum height in pixels
    /// * `format` - Optional format override string
    /// 
    /// # Returns
    /// * `Ok(CompressionOptions)` if valid, `Err(CompressionError)` if quality out of range
    pub fn new(
        quality: Option<u8>,
        width: Option<u32>,
        height: Option<u32>,
        format: Option<String>,
    ) -> Result<Self> {
        let quality = quality.unwrap_or(DEFAULT_QUALITY);
        if !(1..=100).contains(&quality) {
            return Err(CompressionError::InvalidQuality(quality));
        }

        Ok(Self {
            quality,
            width,
            height,
            format,
        })
    }
}

/// Load an image from disk with metadata
/// 
/// # Arguments
/// * `input_path` - Path to the image file
/// 
/// # Returns
/// * `Ok((DynamicImage, u64))` containing the image and file size, or `Err(CompressionError)`
pub fn load_image_with_metadata(input_path: &Path) -> Result<(DynamicImage, u64)> {
    validate_file_exists(input_path)?;

    let img = ImageReader::open(input_path)?.decode()?;
    let file_size = fs::metadata(input_path)?.len();

    Ok((img, file_size))
}

/// Resize an image according to the specified options
/// 
/// # Arguments
/// * `img` - Mutable reference to the image to resize
/// * `options` - Compression options containing width/height constraints
pub fn resize_image(img: &mut DynamicImage, options: &CompressionOptions) {
    if let Some(w) = options.width.filter(|&w| w > 0 && w != img.width()) {
        println!("🔄 Resizing width to {} pixels...", w);
        *img = img.resize_exact(w, img.height(), image::imageops::FilterType::Lanczos3);
        println!("✅ Resized to width: {}", w);
    }

    if let Some(h) = options.height.filter(|&h| h > 0 && h != img.height()) {
        println!("🔄 Resizing height to {} pixels...", h);
        *img = img.resize_exact(img.width(), h, image::imageops::FilterType::Lanczos3);
        println!("✅ Resized to height: {}", h);
    }
}

/// Process and save an image with the specified options
/// 
/// # Arguments
/// * `img` - The image to save
/// * `output_path` - Destination file path
/// * `options` - Compression options
/// 
/// # Returns
/// * `Ok(u64)` containing the compressed file size, or `Err(CompressionError)`
pub fn process_and_save_image(
    img: &DynamicImage,
    output_path: &Path,
    options: &CompressionOptions,
) -> Result<u64> {
    let output_format = determine_output_format(output_path, options.format.as_deref())?;
    save_image(img, output_path, output_format, options)?;

    let compressed_size = fs::metadata(output_path)?.len();
    Ok(compressed_size)
}

/// Compress a single image file
/// 
/// # Arguments
/// * `input` - Input image file path
/// * `output` - Output image file path
/// * `options` - Compression options
/// 
/// # Returns
/// * `Ok(())` on success, `Err(CompressionError)` on failure
pub fn compress_image(input: PathBuf, output: PathBuf, options: CompressionOptions) -> Result<()> {
    println!("🗜️  Compressing image: {:?}", input);
    println!("📁 Output: {:?}", output);

    let pb = create_progress_spinner("Loading image...");

    let (mut img, original_size) = load_image_with_metadata(&input)?;
    pb.finish_with_message("✅ Image loaded");

    println!(
        "📊 Original size: {} bytes ({}x{})",
        original_size,
        img.width(),
        img.height()
    );

    // Resize if needed
    resize_image(&mut img, &options);

    let pb = create_progress_spinner("Saving compressed image...");
    let compressed_size = process_and_save_image(&img, &output, &options)?;
    pb.finish_with_message("✅ Compression complete");

    print_compression_result(original_size, compressed_size);

    Ok(())
}

/// Save an image with format-specific optimizations
/// 
/// # Arguments
/// * `img` - The image to save
/// * `output` - Output file path
/// * `format` - Target output format
/// * `options` - Compression options
/// 
/// # Returns
/// * `Ok(())` on success, `Err(CompressionError)` on failure
pub fn save_image(
    img: &DynamicImage,
    output: &Path,
    format: OutputFormat,
    options: &CompressionOptions,
) -> Result<()> {
    // Ensure output directory exists
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|_| CompressionError::DirectoryCreationFailed(parent.to_path_buf()))?;
    }

    match format {
        OutputFormat::Jpeg => {
            img.save_with_format(output, image::ImageFormat::Jpeg)?;
        }
        OutputFormat::Png => {
            optimize_png_image(img, output, options)?;
        }
        OutputFormat::WebP => {
            img.save_with_format(output, image::ImageFormat::WebP)?;
        }
    }

    Ok(())
}

/// Optimize PNG image using oxipng with quality-based settings
/// 
/// # Arguments
/// * `img` - The image to save
/// * `output` - Output file path
/// * `options` - Compression options containing quality setting
/// 
/// # Returns
/// * `Ok(())` on success, `Err(CompressionError)` on failure
fn optimize_png_image(
    img: &DynamicImage,
    output: &Path,
    options: &CompressionOptions,
) -> Result<()> {
    // Save to temporary file first
    let temp_path = output.with_extension("temp.png");
    img.save_with_format(&temp_path, image::ImageFormat::Png)?;

    // Configure oxipng options based on quality
    let mut oxipng_options = Options::from_preset(4); // Use highest compression preset
    oxipng_options.force = true; // Allow overwriting existing files

    // Configure compression algorithm based on quality
    if options.quality >= PNG_ZOPFLI_QUALITY_THRESHOLD {
        oxipng_options.deflate = Deflaters::Zopfli {
            iterations: NonZeroU8::new(PNG_ZOPFLI_ITERATIONS)
                .expect("PNG_ZOPFLI_ITERATIONS must be non-zero"),
        };
    } else if options.quality >= PNG_HIGH_COMPRESSION_QUALITY_THRESHOLD {
        oxipng_options.deflate = Deflaters::Libdeflater { 
            compression: PNG_HIGH_COMPRESSION_LEVEL 
        };
    } else {
        oxipng_options.deflate = Deflaters::Libdeflater { 
            compression: PNG_STANDARD_COMPRESSION_LEVEL 
        };
    }

    // Optimize the PNG file
    let input = InFile::Path(temp_path.clone());
    let out = OutFile::Path {
        path: Some(output.to_path_buf()),
        preserve_attrs: false,
    };
    
    oxipng::optimize(&input, &out, &oxipng_options)
        .map_err(|e| CompressionError::PngOptimization(e.to_string()))?;

    // Clean up temporary file
    if let Err(e) = fs::remove_file(&temp_path) {
        eprintln!("Warning: Failed to remove temporary file {:?}: {}", temp_path, e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_options_creation() {
        let options =
            CompressionOptions::new(Some(85), Some(800), Some(600), Some("webp".to_string()))
                .unwrap();
        assert_eq!(options.quality, 85);
        assert_eq!(options.width, Some(800));
        assert_eq!(options.height, Some(600));
        assert_eq!(options.format, Some("webp".to_string()));
    }

    #[test]
    fn test_compression_options_default() {
        let options = CompressionOptions::new(None, None, None, None).unwrap();
        assert_eq!(options.quality, 80);
        assert_eq!(options.width, None);
        assert_eq!(options.height, None);
        assert_eq!(options.format, None);
    }

    #[test]
    fn test_compression_options_invalid_quality() {
        let result = CompressionOptions::new(Some(0), None, None, None);
        assert!(matches!(result, Err(CompressionError::InvalidQuality(0))));

        let result = CompressionOptions::new(Some(101), None, None, None);
        assert!(matches!(result, Err(CompressionError::InvalidQuality(101))));
    }

    #[test]
    fn test_determine_output_format() {
        let path = Path::new("test.jpg");
        let format = determine_output_format(path, None).unwrap();
        assert_eq!(format, OutputFormat::Jpeg);

        let path = Path::new("test.png");
        let format = determine_output_format(path, None).unwrap();
        assert_eq!(format, OutputFormat::Png);

        let path = Path::new("test.webp");
        let format = determine_output_format(path, None).unwrap();
        assert_eq!(format, OutputFormat::WebP);

        let path = Path::new("test.unknown");
        let format = determine_output_format(path, None).unwrap();
        assert_eq!(format, OutputFormat::Jpeg);
    }

    #[test]
    fn test_determine_output_format_with_override() {
        let path = Path::new("test.jpg");
        let format = determine_output_format(path, Some("png")).unwrap();
        assert_eq!(format, OutputFormat::Png);
    }

    #[test]
    fn test_determine_output_format_unsupported() {
        let path = Path::new("test.jpg");
        let result = determine_output_format(path, Some("unsupported"));
        assert!(matches!(
            result,
            Err(CompressionError::UnsupportedFormat(_))
        ));
    }

    #[test]
    fn test_resize_image_dimensions() {
        let mut img = DynamicImage::new_rgb8(2000, 1500);
        let options = CompressionOptions::new(Some(80), Some(1000), None, None).unwrap();

        resize_image(&mut img, &options);

        assert_eq!(img.dimensions(), (1000, 1500));
    }

    #[test]
    fn test_resize_image_height_only() {
        let mut img = DynamicImage::new_rgb8(2000, 1500);
        let options = CompressionOptions::new(Some(80), None, Some(750), None).unwrap();

        resize_image(&mut img, &options);

        assert_eq!(img.dimensions(), (2000, 750));
    }

    #[test]
    fn test_resize_image_both_dimensions() {
        let mut img = DynamicImage::new_rgb8(2000, 1500);
        let options = CompressionOptions::new(Some(80), Some(800), Some(600), None).unwrap();

        resize_image(&mut img, &options);

        assert_eq!(img.dimensions(), (800, 600));
    }

    #[test]
    fn test_resize_image_no_dimensions() {
        let mut img = DynamicImage::new_rgb8(2000, 1500);
        let options = CompressionOptions::new(Some(80), None, None, None).unwrap();

        resize_image(&mut img, &options);

        assert_eq!(img.dimensions(), (2000, 1500));
    }

    #[test]
    fn test_resize_image_same_dimensions() {
        let mut img = DynamicImage::new_rgb8(2000, 1500);
        let options = CompressionOptions::new(Some(80), Some(2000), Some(1500), None).unwrap();

        resize_image(&mut img, &options);

        assert_eq!(img.dimensions(), (2000, 1500));
    }

    #[test]
    fn test_load_image_with_metadata_not_found() {
        let path = Path::new("nonexistent.jpg");
        let result = load_image_with_metadata(path);
        assert!(matches!(result, Err(CompressionError::FileNotFound(_))));
    }
}
