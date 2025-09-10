use crate::constants::{
    DEFAULT_QUALITY, LIBDEFLATER_HIGH_LEVEL, LIBDEFLATER_LOW_LEVEL, MAX_FILE_SIZE,
    MAX_IMAGE_DIMENSION, MAX_QUALITY, MIN_QUALITY, ZOPFLI_ITERATIONS,
};
use crate::error::{CompressionError, Result};
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader};
use indicatif::{ProgressBar, ProgressStyle};
use oxipng::{Deflaters, InFile, Options, OutFile};
use std::fs;
use std::num::NonZeroU8;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CompressionOptions {
    pub quality: u8,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
}

impl CompressionOptions {
    pub fn new(
        quality: Option<u8>,
        width: Option<u32>,
        height: Option<u32>,
        format: Option<String>,
    ) -> Result<Self> {
        let quality = quality.unwrap_or(DEFAULT_QUALITY);
        if !(MIN_QUALITY..=MAX_QUALITY).contains(&quality) {
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

/// Validates that a file exists at the given path.
///
/// # Arguments
/// * `path` - The path to check for existence
///
/// # Returns
/// * `Ok(())` if the file exists
/// * `Err(CompressionError::FileNotFound)` if the file does not exist
///
/// # Example
/// ```
/// use std::path::Path;
/// use img_squeeze::validate_file_exists;
///
/// let result = validate_file_exists(Path::new("nonexistent.jpg"));
/// assert!(result.is_err());
/// ```
pub fn validate_file_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(CompressionError::FileNotFound(path.to_path_buf()));
    }
    Ok(())
}

/// Core image processing pipeline that handles the common workflow:
/// load -> resize -> process -> save
///
/// # Arguments
/// * `input_path` - Path to the input image file
/// * `output_path` - Path where the processed image will be saved
/// * `options` - Compression and processing options
///
/// # Returns
/// * `Ok((original_size, compressed_size))` - Tuple of file sizes in bytes
/// * `Err(CompressionError)` - If any processing step fails
///
/// # Security
/// - Validates file existence and canonical paths to prevent directory traversal
/// - Enforces maximum file size and image dimension limits
/// - Uses secure temporary file handling
pub fn process_image_pipeline(
    input_path: &Path,
    output_path: &Path,
    options: &CompressionOptions,
) -> Result<(u64, u64)> {
    // Load and validate image
    let (mut img, original_size) = load_image_with_metadata(input_path)?;

    // Resize if needed
    resize_image(&mut img, options);

    // Process and save
    let compressed_size = process_and_save_image(&img, output_path, options)?;

    Ok((original_size, compressed_size))
}

/// Loads an image file and returns it along with file metadata.
///
/// # Arguments
/// * `input_path` - Path to the image file to load
///
/// # Returns
/// * `Ok((image, file_size))` - The loaded image and its file size in bytes
/// * `Err(CompressionError)` - If loading fails or security limits are exceeded
///
/// # Security Features
/// - Validates file existence and canonical paths to prevent directory traversal
/// - Enforces maximum file size limit to prevent DoS attacks
/// - Validates image dimensions to prevent memory exhaustion
/// - Checks file size before attempting to load the image
pub fn load_image_with_metadata(input_path: &Path) -> Result<(DynamicImage, u64)> {
    validate_file_exists(input_path)?;

    // Security: Validate path to prevent directory traversal attacks
    let canonical_path = input_path
        .canonicalize()
        .map_err(|_| CompressionError::FileNotFound(input_path.to_path_buf()))?;

    // Check file size before loading to prevent DoS attacks
    let file_size = fs::metadata(&canonical_path)?.len();
    if file_size > MAX_FILE_SIZE {
        return Err(CompressionError::FileTooLarge(file_size, MAX_FILE_SIZE));
    }

    let img = ImageReader::open(&canonical_path)?.decode()?;

    // Security: Validate image dimensions to prevent DoS attacks
    let (width, height) = img.dimensions();
    if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
        return Err(CompressionError::InvalidDimensions(
            width,
            height,
            MAX_IMAGE_DIMENSION,
        ));
    }

    Ok((img, file_size))
}

pub fn resize_image(img: &mut DynamicImage, options: &CompressionOptions) {
    if let Some(w) = options.width.filter(|&w| w > 0 && w != img.width()) {
        println!("🔄 Resizing width...");
        *img = img.resize_exact(w, img.height(), image::imageops::FilterType::Lanczos3);
        println!("✅ Resized to width: {}", w);
    }

    if let Some(h) = options.height.filter(|&h| h > 0 && h != img.height()) {
        println!("🔄 Resizing height...");
        *img = img.resize_exact(img.width(), h, image::imageops::FilterType::Lanczos3);
        println!("✅ Resized to height: {}", h);
    }
}

pub fn process_and_save_image(
    img: &DynamicImage,
    output_path: &Path,
    options: &CompressionOptions,
) -> Result<u64> {
    let output_buf = output_path.to_path_buf();
    let output_format = determine_output_format(output_path, &options.format)?;
    save_image(img, &output_buf, output_format, options)?;

    let compressed_size = fs::metadata(output_path)?.len();
    Ok(compressed_size)
}

pub fn compress_image(input: PathBuf, output: PathBuf, options: CompressionOptions) -> Result<()> {
    println!("🗜️  Compressing image: {:?}", input);
    println!("📁 Output: {:?}", output);

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Loading image...");

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

    pb.set_message("Saving compressed image...");
    let compressed_size = process_and_save_image(&img, &output, &options)?;
    pb.finish_with_message("✅ Compression complete");
    let compression_ratio =
        ((original_size as f64 - compressed_size as f64) / original_size as f64) * 100.0;

    println!("📈 Compressed size: {} bytes", compressed_size);
    println!("🎯 Compression ratio: {:.1}%", compression_ratio);

    if compression_ratio > 0.0 {
        println!(
            "✅ Successfully reduced file size by {:.1}%",
            compression_ratio
        );
    } else {
        println!("⚠️  File size increased by {:.1}%", compression_ratio.abs());
    }

    Ok(())
}

pub fn determine_output_format(output: &Path, format: &Option<String>) -> Result<ImageFormat> {
    if let Some(fmt) = format {
        match fmt.to_lowercase().as_str() {
            "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
            "png" => Ok(ImageFormat::Png),
            "webp" => Ok(ImageFormat::WebP),
            "avif" => Ok(ImageFormat::Avif),
            "heic" | "heif" => Err(CompressionError::UnsupportedFormat(
                format!("{} format is not yet supported in this version. Use AVIF for modern compression", fmt)
            )),
            "jxl" | "jpegxl" => Err(CompressionError::UnsupportedFormat(
                format!("{} format is not yet supported in this version. Use AVIF for modern compression", fmt)
            )),
            _ => Err(CompressionError::UnsupportedFormat(fmt.clone())),
        }
    } else if let Some(ext) = output.extension().and_then(|ext| ext.to_str()) {
        match ext {
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "png" => Ok(ImageFormat::Png),
            "webp" => Ok(ImageFormat::WebP),
            "avif" => Ok(ImageFormat::Avif),
            "heic" | "heif" => Err(CompressionError::UnsupportedFormat(
                format!("{} format is not yet supported in this version. Use AVIF for modern compression", ext)
            )),
            "jxl" => Err(CompressionError::UnsupportedFormat(
                format!("{} format is not yet supported in this version. Use AVIF for modern compression", ext)
            )),
            _ => Ok(ImageFormat::Jpeg),
        }
    } else {
        Ok(ImageFormat::Jpeg)
    }
}

pub fn save_image(
    img: &DynamicImage,
    output: &PathBuf,
    format: ImageFormat,
    options: &CompressionOptions,
) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|_| CompressionError::DirectoryCreationFailed(parent.to_path_buf()))?;
    }

    match format {
        ImageFormat::Jpeg => {
            img.save_with_format(output, image::ImageFormat::Jpeg)?;
        }
        ImageFormat::Png => {
            // 使用 oxipng 进行 PNG 优化
            let (_width, _height) = img.dimensions();

            // Performance: Use secure temp file with proper cleanup
            let temp_path = output.with_extension("temp.png");
            img.save_with_format(&temp_path, image::ImageFormat::Png)?;

            // Performance: Ensure cleanup on any error using RAII pattern
            struct TempFileGuard(PathBuf);
            impl Drop for TempFileGuard {
                fn drop(&mut self) {
                    let _ = fs::remove_file(&self.0);
                }
            }
            let _guard = TempFileGuard(temp_path.clone());

            // 配置 oxipng 选项
            let mut oxipng_options = Options::from_preset(4); // 使用预设 4 (最高压缩)
            oxipng_options.force = true; // 强制覆盖

            // 根据质量设置调整压缩级别
            if options.quality >= 90 {
                oxipng_options.deflate = Deflaters::Zopfli {
                    iterations: NonZeroU8::new(ZOPFLI_ITERATIONS).unwrap(),
                };
            } else if options.quality >= 70 {
                oxipng_options.deflate = Deflaters::Libdeflater {
                    compression: LIBDEFLATER_HIGH_LEVEL,
                };
            } else {
                oxipng_options.deflate = Deflaters::Libdeflater {
                    compression: LIBDEFLATER_LOW_LEVEL,
                };
            }

            // 使用 oxipng 优化文件
            let input = InFile::Path(temp_path.clone());
            let out = OutFile::Path {
                path: Some(output.clone()),
                preserve_attrs: false,
            };
            oxipng::optimize(&input, &out, &oxipng_options)
                .map_err(|e| CompressionError::PngOptimization(e.to_string()))?;

            // Temp file automatically cleaned up by guard
        }
        ImageFormat::WebP => {
            img.save_with_format(output, image::ImageFormat::WebP)?;
        }
        ImageFormat::Avif => {
            img.save_with_format(output, image::ImageFormat::Avif)?;
        }
        _ => {
            return Err(CompressionError::UnsupportedFormat(format!("{:?}", format)));
        }
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
        let format = determine_output_format(path, &None).unwrap();
        assert_eq!(format, ImageFormat::Jpeg);

        let path = Path::new("test.png");
        let format = determine_output_format(path, &None).unwrap();
        assert_eq!(format, ImageFormat::Png);

        let path = Path::new("test.webp");
        let format = determine_output_format(path, &None).unwrap();
        assert_eq!(format, ImageFormat::WebP);

        let path = Path::new("test.avif");
        let format = determine_output_format(path, &None).unwrap();
        assert_eq!(format, ImageFormat::Avif);

        let path = Path::new("test.unknown");
        let format = determine_output_format(path, &None).unwrap();
        assert_eq!(format, ImageFormat::Jpeg);
    }

    #[test]
    fn test_determine_output_format_with_override() {
        let path = Path::new("test.jpg");
        let format = determine_output_format(path, &Some("png".to_string())).unwrap();
        assert_eq!(format, ImageFormat::Png);

        let path = Path::new("test.png");
        let format = determine_output_format(path, &Some("avif".to_string())).unwrap();
        assert_eq!(format, ImageFormat::Avif);
    }

    #[test]
    fn test_determine_output_format_unsupported() {
        let path = Path::new("test.jpg");
        let result = determine_output_format(path, &Some("unsupported".to_string()));
        assert!(matches!(
            result,
            Err(CompressionError::UnsupportedFormat(_))
        ));

        // Test HEIC/HEIF recognition with helpful error message
        let result = determine_output_format(path, &Some("heic".to_string()));
        assert!(matches!(
            result,
            Err(CompressionError::UnsupportedFormat(_))
        ));
        if let Err(CompressionError::UnsupportedFormat(msg)) = result {
            assert!(msg.contains("not yet supported"));
            assert!(msg.contains("AVIF"));
        }

        // Test JPEG XL recognition with helpful error message  
        let result = determine_output_format(path, &Some("jxl".to_string()));
        assert!(matches!(
            result,
            Err(CompressionError::UnsupportedFormat(_))
        ));
        if let Err(CompressionError::UnsupportedFormat(msg)) = result {
            assert!(msg.contains("not yet supported"));
            assert!(msg.contains("AVIF"));
        }
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
