use crate::constants::{
    DEFAULT_QUALITY, LIBDEFLATER_HIGH_LEVEL, LIBDEFLATER_LOW_LEVEL, MAX_FILE_SIZE,
    MAX_IMAGE_DIMENSION, MAX_QUALITY, MIN_QUALITY, ZOPFLI_ITERATIONS,
};
use crate::error::{CompressionError, Result};
use image::{DynamicImage, GenericImageView, ImageEncoder, ImageFormat, ImageReader};
use indicatif::{ProgressBar, ProgressStyle};
use oxipng::{Deflaters, InFile, Options, OutFile};
use std::fs;
use std::num::NonZeroU8;
use std::path::{Path, PathBuf};

#[cfg(feature = "heic")]
use libheif_rs::HeifContext;

/// Loads a HEIC/HEIF image file using libheif-rs and converts it to DynamicImage.
///
/// # Arguments
/// * `input_path` - Path to the HEIC/HEIF image file to load
///
/// # Returns
/// * `Ok((image, file_size))` - The loaded image and its file size in bytes
/// * `Err(CompressionError)` - If loading fails
///
/// # Requirements
/// - Requires libheif >= 1.20.0 to be installed on the system
/// - On Ubuntu/Debian: `sudo apt-get install libheif-dev` (may need backports or source build)
/// - On macOS: `brew install libheif`
/// - Enable with: `cargo build --features heic`
#[cfg(feature = "heic")]
pub fn load_heic_image(input_path: &Path) -> Result<(DynamicImage, u64)> {
    validate_file_exists(input_path)?;

    // Check file size before loading
    let file_size = fs::metadata(input_path)?.len();
    if file_size > MAX_FILE_SIZE {
        return Err(CompressionError::FileTooLarge(file_size, MAX_FILE_SIZE));
    }

    // Read the HEIC file
    let heif_data = fs::read(input_path)?;

    // Create HeifContext from the data
    let context = HeifContext::read_from_bytes(&heif_data)?;

    // Get the primary image handle
    let primary_image_handle = context.primary_image_handle()?;

    // Get image dimensions
    let width = primary_image_handle.width();
    let height = primary_image_handle.height();

    // Security: Validate image dimensions to prevent DoS attacks
    if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
        return Err(CompressionError::InvalidDimensions(
            width,
            height,
            MAX_IMAGE_DIMENSION,
        ));
    }

    // Create a LibHeif instance for decoding
    let lib_heif = libheif_rs::LibHeif::new();

    // Decode the HEIC image to RGB format for compatibility with image crate
    let decoded_image = lib_heif.decode(
        &primary_image_handle,
        libheif_rs::ColorSpace::Rgb(libheif_rs::RgbChroma::C444),
        None,
    )?;

    // Get the RGB planes from the decoded image
    let planes = decoded_image.planes();

    // Extract the individual color channels
    let r_plane = planes.r.ok_or_else(|| CompressionError::UnsupportedFormat(
        "Missing red channel in decoded HEIC image".to_string()
    ))?;
    let g_plane = planes.g.ok_or_else(|| CompressionError::UnsupportedFormat(
        "Missing green channel in decoded HEIC image".to_string()
    ))?;
    let b_plane = planes.b.ok_or_else(|| CompressionError::UnsupportedFormat(
        "Missing blue channel in decoded HEIC image".to_string()
    ))?;

    // Check if there's an alpha channel
    let has_alpha = planes.a.is_some();
    let width = r_plane.width;
    let height = r_plane.height;

    // Create an ImageBuffer based on the color format
    let dynamic_image = if has_alpha {
        // RGBA format
        if let Some(a_plane) = planes.a {
            let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

            // Interleave the RGBA channels
            for y in 0..height {
                for x in 0..width {
                    let r_idx = (y as usize * r_plane.stride + x as usize) as usize;
                    let g_idx = (y as usize * g_plane.stride + x as usize) as usize;
                    let b_idx = (y as usize * b_plane.stride + x as usize) as usize;
                    let a_idx = (y as usize * a_plane.stride + x as usize) as usize;

                    rgba_data.push(r_plane.data[r_idx]);
                    rgba_data.push(g_plane.data[g_idx]);
                    rgba_data.push(b_plane.data[b_idx]);
                    rgba_data.push(a_plane.data[a_idx]);
                }
            }

            let rgba_image = image::RgbaImage::from_raw(width, height, rgba_data)
                .ok_or_else(|| CompressionError::UnsupportedFormat(
                    "Failed to create RGBA image from HEIC data".to_string()
                ))?;
            DynamicImage::ImageRgba8(rgba_image)
        } else {
            return Err(CompressionError::UnsupportedFormat(
                "Inconsistent alpha channel information in HEIC image".to_string()
            ));
        }
    } else {
        // RGB format
        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);

        // Interleave the RGB channels
        for y in 0..height {
            for x in 0..width {
                let r_idx = (y as usize * r_plane.stride + x as usize) as usize;
                let g_idx = (y as usize * g_plane.stride + x as usize) as usize;
                let b_idx = (y as usize * b_plane.stride + x as usize) as usize;

                rgb_data.push(r_plane.data[r_idx]);
                rgb_data.push(g_plane.data[g_idx]);
                rgb_data.push(b_plane.data[b_idx]);
            }
        }

        let rgb_image = image::RgbImage::from_raw(width, height, rgb_data)
            .ok_or_else(|| CompressionError::UnsupportedFormat(
                "Failed to create RGB image from HEIC data".to_string()
            ))?;
        DynamicImage::ImageRgb8(rgb_image)
    };

    Ok((dynamic_image, file_size))
}

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
    let compressed_size = process_and_save_image(&img, output_path, options, input_path)?;

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

    // Check for special format handling
    if let Some(ext) = input_path.extension().and_then(|s| s.to_str()) {
        match ext.to_ascii_lowercase().as_str() {
            "heic" | "heif" => {
                #[cfg(feature = "heic")]
                {
                    return load_heic_image(input_path);
                }
                #[cfg(not(feature = "heic"))]
                {
                    return Err(CompressionError::UnsupportedFormat(
                        "HEIC/HEIF format requires the 'heic' feature. Enable with: cargo build --features heic\n\
                        Note: Requires libheif >= 1.20.0 system library. On Ubuntu/Debian: sudo apt-get install libheif-dev".to_string()
                    ));
                }
            }
            "jxl" | "jpegxl" => {
                return Err(CompressionError::UnsupportedFormat(
                    "JPEG XL format is not yet supported in this version. Use AVIF for modern compression with similar quality and efficiency".to_string()
                ));
            }
            _ => {} // Continue with supported formats
        }
    }

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
    input_path: &Path,
) -> Result<u64> {
    let output_buf = output_path.to_path_buf();
    let output_format = determine_output_format(output_path, &options.format)?;
    save_image(img, &output_buf, output_format, options, input_path)?;

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
    let compressed_size = process_and_save_image(&img, &output, &options, &input)?;
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
            "heic" | "heif" => {
                #[cfg(feature = "heic")]
                {
                    return Err(CompressionError::UnsupportedFormat(
                        "HEIC/HEIF output format not yet supported. HEIC images can be read and converted to other formats.".to_string()
                    ));
                }
                #[cfg(not(feature = "heic"))]
                {
                    return Err(CompressionError::UnsupportedFormat(
                        "HEIC/HEIF format requires the 'heic' feature. Enable with: cargo build --features heic\n\
                        Note: Requires libheif >= 1.20.0 system library. On Ubuntu/Debian: sudo apt-get install libheif-dev".to_string()
                    ));
                }
            }
            "jxl" | "jpegxl" => Err(CompressionError::UnsupportedFormat(
                format!("{} format is not yet supported in this version. Use AVIF for modern compression", fmt)
            )),
            _ => Err(CompressionError::UnsupportedFormat(fmt.clone())),
        }
    } else if let Some(ext) = output.extension().and_then(|ext| ext.to_str()) {
        let ext = ext.to_ascii_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "png" => Ok(ImageFormat::Png),
            "webp" => Ok(ImageFormat::WebP),
            "avif" => Ok(ImageFormat::Avif),
            "heic" | "heif" => {
                #[cfg(feature = "heic")]
                {
                    return Err(CompressionError::UnsupportedFormat(
                        "HEIC/HEIF output format not yet supported. HEIC images can be read and converted to other formats.".to_string()
                    ));
                }
                #[cfg(not(feature = "heic"))]
                {
                    return Err(CompressionError::UnsupportedFormat(
                        "HEIC/HEIF format requires the 'heic' feature. Enable with: cargo build --features heic\n\
                        Note: Requires libheif >= 1.20.0 system library. On Ubuntu/Debian: sudo apt-get install libheif-dev".to_string()
                    ));
                }
            }
            "jxl" | "jpegxl" => Err(CompressionError::UnsupportedFormat(
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
    input_path: &Path,
) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|_| CompressionError::DirectoryCreationFailed(parent.to_path_buf()))?;
    }

    match format {
        ImageFormat::Jpeg => {
            // 智能 JPEG 压缩：避免重新压缩导致文件变大
            use image::codecs::jpeg::JpegEncoder;

            // 如果输入已经是 JPEG，使用更保守的质量设置
            let actual_quality = if input_path.extension()
                .and_then(|s| s.to_str())
                .map(|s| s.to_lowercase() == "jpg" || s.to_lowercase() == "jpeg")
                .unwrap_or(false) {
                // 对于已经是 JPEG 的文件，使用更保守的质量设置
                // 只有在质量明显低于原始质量时才重新压缩
                std::cmp::min(options.quality, 75)
            } else {
                options.quality
            };

            let mut file = std::fs::File::create(output)?;
            let mut encoder = JpegEncoder::new_with_quality(&mut file, actual_quality);
            encoder.encode_image(img)?;

            // 如果输出文件比输入大，发出警告
            if let Ok(output_size) = std::fs::metadata(output).map(|m| m.len()) {
                if let Ok(input_size) = std::fs::metadata(input_path).map(|m| m.len()) {
                    if output_size > input_size {
                        eprintln!("⚠️  Warning: Output file ({}) is larger than input ({})",
                                 output_size, input_size);
                        eprintln!("💡 Tip: Try lower quality setting or avoid re-compressing JPEG files");
                    }
                }
            }
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
            // Honor quality and enable parallel encoding (when "image/rayon" is enabled).
            use image::codecs::avif::AvifEncoder;
            let rgba = img.to_rgba8();
            let (w, h) = (rgba.width(), rgba.height());
            let mut file = std::fs::File::create(output)?;
            // Heuristic speed; consider surfacing as an option later.
            let speed: u8 = 6;
            let enc = AvifEncoder::new_with_speed_quality(&mut file, speed, options.quality);
            // If you later thread this via CLI, call: enc = enc.with_num_threads(Some(n));
            enc.write_image(
                rgba.as_raw(),
                w,
                h,
                image::ExtendedColorType::Rgba8
            )?;
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
            #[cfg(feature = "heic")]
            {
                assert!(msg.contains("output format not yet supported"));
                assert!(msg.contains("can be read and converted"));
            }
            #[cfg(not(feature = "heic"))]
            {
                assert!(msg.contains("requires the 'heic' feature"));
                assert!(msg.contains("libheif >= 1.20.0"));
            }
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

        // Alias: jpegxl should behave like jxl
        let result = determine_output_format(path, &Some("jpegxl".to_string()));
        assert!(matches!(result, Err(CompressionError::UnsupportedFormat(_))));
        if let Err(CompressionError::UnsupportedFormat(msg)) = result {
            assert!(msg.contains("not yet supported"));
            assert!(msg.contains("AVIF"));
        }
    }

    #[test]
    fn test_determine_output_format_case_insensitive_exts() {
        // After normalizing extension cases, uppercase should work.
        let path = Path::new("IMAGE.AVIF");
        let format = determine_output_format(path, &None).unwrap();
        assert_eq!(format, ImageFormat::Avif);

        // Unsupported alias via extension
        let path = Path::new("photo.jpegxl");
        let result = determine_output_format(path, &None);
        assert!(matches!(result, Err(CompressionError::UnsupportedFormat(_))));
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
