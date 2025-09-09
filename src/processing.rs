use crate::error::{CompressionError, Result};
use image::{DynamicImage, ImageFormat, ImageReader, GenericImageView};
use std::num::NonZeroU8;
use std::path::{Path, PathBuf};
use std::fs;
use oxipng::{Options, Deflaters, InFile, OutFile};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Clone)]
pub struct CompressionOptions {
    pub quality: u8,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub exact_resize: bool,
}

impl CompressionOptions {
    pub fn new(quality: Option<u8>, width: Option<u32>, height: Option<u32>, format: Option<String>, exact_resize: bool) -> Result<Self> {
        let quality = quality.unwrap_or(80);
        if !(1..=100).contains(&quality) {
            return Err(CompressionError::InvalidQuality(quality));
        }
        
        Ok(Self {
            quality,
            width,
            height,
            format,
            exact_resize,
        })
    }
}

pub fn load_image_with_metadata(input_path: &Path) -> Result<(DynamicImage, u64)> {
    if !input_path.exists() {
        return Err(CompressionError::FileNotFound(input_path.to_path_buf()));
    }
    
    let img = ImageReader::open(input_path)?.decode()?;
    let file_size = fs::metadata(input_path)?.len();
    
    Ok((img, file_size))
}

pub fn resize_image(img: &mut DynamicImage, options: &CompressionOptions) {
    let (current_width, current_height) = img.dimensions();
    
    match (options.width, options.height) {
        (Some(target_width), Some(target_height)) => {
            // Both dimensions specified - resize to exact dimensions
            if target_width > 0 && target_height > 0 && 
               (target_width != current_width || target_height != current_height) {
                println!("🔄 Resizing to exact dimensions: {}x{}", target_width, target_height);
                *img = img.resize_exact(target_width, target_height, image::imageops::FilterType::Lanczos3);
                println!("✅ Resized to: {}x{}", target_width, target_height);
            }
        }
        (Some(target_width), None) => {
            // Only width specified
            if target_width > 0 && target_width != current_width {
                if options.exact_resize {
                    println!("🔄 Resizing width exactly (may distort aspect ratio)...");
                    *img = img.resize_exact(target_width, current_height, image::imageops::FilterType::Lanczos3);
                    println!("✅ Resized to width: {} (height: {})", target_width, current_height);
                } else {
                    println!("🔄 Resizing width while maintaining aspect ratio...");
                    *img = img.resize(target_width, u32::MAX, image::imageops::FilterType::Lanczos3);
                    println!("✅ Resized to width: {} (height: {})", target_width, img.height());
                }
            }
        }
        (None, Some(target_height)) => {
            // Only height specified
            if target_height > 0 && target_height != current_height {
                if options.exact_resize {
                    println!("🔄 Resizing height exactly (may distort aspect ratio)...");
                    *img = img.resize_exact(current_width, target_height, image::imageops::FilterType::Lanczos3);
                    println!("✅ Resized to height: {} (width: {})", target_height, current_width);
                } else {
                    println!("🔄 Resizing height while maintaining aspect ratio...");
                    *img = img.resize(u32::MAX, target_height, image::imageops::FilterType::Lanczos3);
                    println!("✅ Resized to height: {} (width: {})", target_height, img.width());
                }
            }
        }
        (None, None) => {
            // No resizing needed
        }
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

pub fn compress_image(
    input: PathBuf,
    output: PathBuf,
    options: CompressionOptions,
) -> Result<()> {
    println!("🗜️  Compressing image: {:?}", input);
    println!("📁 Output: {:?}", output);
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}").unwrap());
    pb.set_message("Loading image...");
    
    let (mut img, original_size) = load_image_with_metadata(&input)?;
    pb.finish_with_message("✅ Image loaded");
    
    println!("📊 Original size: {} bytes ({}x{})", original_size, img.width(), img.height());
    
    // Resize if needed
    resize_image(&mut img, &options);
    
    pb.set_message("Saving compressed image...");
    let compressed_size = process_and_save_image(&img, &output, &options)?;
    pb.finish_with_message("✅ Compression complete");
    let compression_ratio = ((original_size as f64 - compressed_size as f64) / original_size as f64) * 100.0;
    
    println!("📈 Compressed size: {} bytes", compressed_size);
    println!("🎯 Compression ratio: {:.1}%", compression_ratio);
    
    if compression_ratio > 0.0 {
        println!("✅ Successfully reduced file size by {:.1}%", compression_ratio);
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
            _ => Err(CompressionError::UnsupportedFormat(fmt.clone())),
        }
    } else if let Some(ext) = output.extension().and_then(|ext| ext.to_str()) {
        match ext {
            "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
            "png" => Ok(ImageFormat::Png),
            "webp" => Ok(ImageFormat::WebP),
            _ => Ok(ImageFormat::Jpeg),
        }
    } else {
        Ok(ImageFormat::Jpeg)
    }
}

pub fn save_image(img: &DynamicImage, output: &PathBuf, format: ImageFormat, options: &CompressionOptions) -> Result<()> {
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
            
            // 先保存为临时文件
            let temp_path = output.with_extension("temp.png");
            img.save_with_format(&temp_path, image::ImageFormat::Png)?;
            
            // 配置 oxipng 选项
            let mut oxipng_options = Options::from_preset(4); // 使用预设 4 (最高压缩)
            oxipng_options.force = true; // 强制覆盖
            
            // 根据质量设置调整压缩级别
            if options.quality >= 90 {
                oxipng_options.deflate = Deflaters::Zopfli { iterations: NonZeroU8::new(15).unwrap() };
            } else if options.quality >= 70 {
                oxipng_options.deflate = Deflaters::Libdeflater { compression: 12 };
            } else {
                oxipng_options.deflate = Deflaters::Libdeflater { compression: 8 };
            }
            
            // 使用 oxipng 优化文件
            let input = InFile::Path(temp_path.clone());
            let out = OutFile::Path { 
                path: Some(output.clone()), 
                preserve_attrs: false 
            };
            oxipng::optimize(&input, &out, &oxipng_options)
                .map_err(|e| CompressionError::PngOptimization(e.to_string()))?;
            
            // 删除临时文件
            fs::remove_file(temp_path)?;
        }
        ImageFormat::WebP => {
            img.save_with_format(output, image::ImageFormat::WebP)?;
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
        let options = CompressionOptions::new(Some(85), Some(800), Some(600), Some("webp".to_string()), false).unwrap();
        assert_eq!(options.quality, 85);
        assert_eq!(options.width, Some(800));
        assert_eq!(options.height, Some(600));
        assert_eq!(options.format, Some("webp".to_string()));
        assert_eq!(options.exact_resize, false);
    }

    #[test]
    fn test_compression_options_default() {
        let options = CompressionOptions::new(None, None, None, None, false).unwrap();
        assert_eq!(options.quality, 80);
        assert_eq!(options.width, None);
        assert_eq!(options.height, None);
        assert_eq!(options.format, None);
        assert_eq!(options.exact_resize, false);
    }

    #[test]
    fn test_compression_options_invalid_quality() {
        let result = CompressionOptions::new(Some(0), None, None, None, false);
        assert!(matches!(result, Err(CompressionError::InvalidQuality(0))));
        
        let result = CompressionOptions::new(Some(101), None, None, None, false);
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

        let path = Path::new("test.unknown");
        let format = determine_output_format(path, &None).unwrap();
        assert_eq!(format, ImageFormat::Jpeg);
    }

    #[test]
    fn test_determine_output_format_with_override() {
        let path = Path::new("test.jpg");
        let format = determine_output_format(path, &Some("png".to_string())).unwrap();
        assert_eq!(format, ImageFormat::Png);
    }

    #[test]
    fn test_determine_output_format_unsupported() {
        let path = Path::new("test.jpg");
        let result = determine_output_format(path, &Some("unsupported".to_string()));
        assert!(matches!(result, Err(CompressionError::UnsupportedFormat(_))));
    }

    #[test]
    fn test_resize_image_dimensions() {
        let mut img = DynamicImage::new_rgb8(2000, 1500);
        let options = CompressionOptions::new(Some(80), Some(1000), None, None, false).unwrap();

        resize_image(&mut img, &options);
        
        // With aspect ratio preserved, 2000x1500 resized to width 1000 becomes 1000x750
        assert_eq!(img.dimensions(), (1000, 750));
    }

    #[test]
    fn test_resize_image_height_only() {
        let mut img = DynamicImage::new_rgb8(2000, 1500);
        let options = CompressionOptions::new(Some(80), None, Some(750), None, false).unwrap();

        resize_image(&mut img, &options);
        
        // With aspect ratio preserved, 2000x1500 resized to height 750 becomes 1000x750
        assert_eq!(img.dimensions(), (1000, 750));
    }

    #[test]
    fn test_resize_image_both_dimensions() {
        let mut img = DynamicImage::new_rgb8(2000, 1500);
        let options = CompressionOptions::new(Some(80), Some(800), Some(600), None, false).unwrap();

        resize_image(&mut img, &options);
        
        assert_eq!(img.dimensions(), (800, 600));
    }

    #[test]
    fn test_resize_image_no_dimensions() {
        let mut img = DynamicImage::new_rgb8(2000, 1500);
        let options = CompressionOptions::new(Some(80), None, None, None, false).unwrap();

        resize_image(&mut img, &options);
        
        assert_eq!(img.dimensions(), (2000, 1500));
    }

    #[test]
    fn test_resize_image_same_dimensions() {
        let mut img = DynamicImage::new_rgb8(2000, 1500);
        let options = CompressionOptions::new(Some(80), Some(2000), Some(1500), None, false).unwrap();

        resize_image(&mut img, &options);
        
        assert_eq!(img.dimensions(), (2000, 1500));
    }

    #[test]
    fn test_resize_image_exact_width() {
        let mut img = DynamicImage::new_rgb8(2000, 1500);
        let options = CompressionOptions::new(Some(80), Some(1000), None, None, true).unwrap();

        resize_image(&mut img, &options);
        
        // With exact resize, 2000x1500 resized to width 1000 becomes 1000x1500 (distorted)
        assert_eq!(img.dimensions(), (1000, 1500));
    }

    #[test]
    fn test_resize_image_exact_height() {
        let mut img = DynamicImage::new_rgb8(2000, 1500);
        let options = CompressionOptions::new(Some(80), None, Some(750), None, true).unwrap();

        resize_image(&mut img, &options);
        
        // With exact resize, 2000x1500 resized to height 750 becomes 2000x750 (distorted)
        assert_eq!(img.dimensions(), (2000, 750));
    }

    #[test]
    fn test_load_image_with_metadata_not_found() {
        let path = Path::new("nonexistent.jpg");
        let result = load_image_with_metadata(path);
        assert!(matches!(result, Err(CompressionError::FileNotFound(_))));
    }
}