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
}

impl CompressionOptions {
    pub fn new(quality: Option<u8>, width: Option<u32>, height: Option<u32>, format: Option<String>) -> Result<Self> {
        let quality = quality.unwrap_or(80);
        if quality < 1 || quality > 100 {
            return Err(CompressionError::InvalidQuality(quality));
        }
        
        Ok(Self {
            quality,
            width,
            height,
            format,
        })
    }
    
    pub fn default() -> Self {
        Self {
            quality: 80,
            width: None,
            height: None,
            format: None,
        }
    }
}

pub fn compress_image(
    input: PathBuf,
    output: PathBuf,
    options: CompressionOptions,
) -> Result<()> {
    println!("🗜️  Compressing image: {:?}", input);
    println!("📁 Output: {:?}", output);
    
    if !input.exists() {
        return Err(CompressionError::FileNotFound(input));
    }
    
    // Quality validation is now handled in CompressionOptions::new()
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}").unwrap());
    pb.set_message("Loading image...");
    
    let mut img = ImageReader::open(&input)?.decode()?;
    
    let original_size = fs::metadata(&input)?.len();
    pb.finish_with_message("✅ Image loaded");
    
    println!("📊 Original size: {} bytes ({}x{})", original_size, img.width(), img.height());
    
    // Resize if needed
    if let Some(w) = options.width {
        if w > 0 && w != img.width() {
            println!("🔄 Resizing width...");
            img = img.resize(w, img.height(), image::imageops::FilterType::Lanczos3);
            println!("✅ Resized to width: {}", w);
        }
    }
    
    if let Some(h) = options.height {
        if h > 0 && h != img.height() {
            println!("🔄 Resizing height...");
            img = img.resize(img.width(), h, image::imageops::FilterType::Lanczos3);
            println!("✅ Resized to height: {}", h);
        }
    }
    
    let output_format = determine_output_format(&output, &options.format)?;
    
    pb.set_message("Saving compressed image...");
    save_image(&img, &output, output_format, &options)?;
    pb.finish_with_message("✅ Compression complete");
    
    let compressed_size = fs::metadata(&output)?.len();
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
    } else {
        match output.extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => Ok(ImageFormat::Jpeg),
            Some("png") => Ok(ImageFormat::Png),
            Some("webp") => Ok(ImageFormat::WebP),
            _ => Ok(ImageFormat::Jpeg),
        }
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