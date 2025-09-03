use crate::error::{CompressionError, Result};
use std::path::Path;
use image::{DynamicImage, ImageReader, GenericImageView};
use std::fs;

pub fn get_image_info(input_path: &Path) -> Result<()> {
    if !input_path.exists() {
        return Err(CompressionError::FileNotFound(input_path.to_path_buf()));
    }

    println!("📊 Analyzing image: {:?}", input_path);
    
    // 读取图片信息
    let img = ImageReader::open(input_path)?.decode()?;
    let metadata = fs::metadata(input_path)?;
    
    // 基本信息
    println!("📋 Basic Information:");
    println!("  📁 File: {:?}", input_path);
    println!("  📏 Dimensions: {}x{} pixels", img.width(), img.height());
    println!("  📦 File size: {} bytes", metadata.len());
    println!("  🎨 Color type: {:?}", img.color());
    println!("  🎭 Image format: {:?}", ImageReader::open(input_path)?.format());
    
    // 计算文件大小信息
    let size_kb = metadata.len() as f64 / 1024.0;
    let size_mb = size_kb / 1024.0;
    
    if size_mb >= 1.0 {
        println!("  📊 Size: {:.2} MB ({:.2} KB)", size_mb, size_kb);
    } else {
        println!("  📊 Size: {:.2} KB", size_kb);
    }
    
    // 像素总数
    let total_pixels = img.width() * img.height();
    println!("  🔢 Total pixels: {}", total_pixels);
    
    // 宽高比
    let aspect_ratio = img.width() as f64 / img.height() as f64;
    println!("  📐 Aspect ratio: {:.2}:1", aspect_ratio);
    
    // 压缩建议
    println!("\n💡 Compression Suggestions:");
    
    if metadata.len() > 5 * 1024 * 1024 {
        println!("  🎯 Large file (>5MB): Consider high compression (quality 60-80)");
    } else if metadata.len() > 1 * 1024 * 1024 {
        println!("  🎯 Medium file (1-5MB): Consider medium compression (quality 70-85)");
    } else {
        println!("  🎯 Small file (<1MB): Consider light compression (quality 85-95)");
    }
    
    // 根据图片尺寸提供建议
    if img.width() > 1920 || img.height() > 1080 {
        println!("  📏 Large dimensions: Consider resizing to 1920x1080 or smaller");
    } else if img.width() > 1280 || img.height() > 720 {
        println!("  📏 HD dimensions: Consider resizing to 1280x720 for web use");
    }
    
    // 根据图片格式提供建议
    if let Some(format) = ImageReader::open(input_path)?.format() {
        match format {
            image::ImageFormat::Png => {
                println!("  🎭 PNG format: Use oxipng optimization for better compression");
            }
            image::ImageFormat::Jpeg => {
                println!("  🎭 JPEG format: Adjust quality setting for size/quality balance");
            }
            image::ImageFormat::WebP => {
                println!("  🎭 WebP format: Already well compressed, consider quality adjustment");
            }
            _ => {
                println!("  🎭 Other format: Consider converting to JPEG/WebP for better compression");
            }
        }
    }
    
    Ok(())
}

pub fn print_detailed_info(input_path: &Path) -> Result<()> {
    if !input_path.exists() {
        return Err(CompressionError::FileNotFound(input_path.to_path_buf()));
    }

    let img = ImageReader::open(input_path)?.decode()?;
    let metadata = fs::metadata(input_path)?;
    
    println!("🔍 Detailed Image Analysis:");
    println!("═");
    for _ in 0..60 {
        print!("═");
    }
    println!();
    
    // 文件信息
    println!("📁 File Information:");
    println!("  Path: {:?}", input_path);
    println!("  Size: {} bytes ({:.2} KB)", metadata.len(), metadata.len() as f64 / 1024.0);
    println!("  Modified: {:?}", metadata.modified());
    println!("  Permissions: {:?}", metadata.permissions());
    
    // 图片信息
    println!("\n🎨 Image Properties:");
    println!("  Dimensions: {}x{} pixels", img.width(), img.height());
    println!("  Color type: {:?}", img.color());
    println!("  Image format: {:?}", ImageReader::open(input_path)?.format());
    
    // 计算信息
    let total_pixels = img.width() * img.height();
    let aspect_ratio = img.width() as f64 / img.height() as f64;
    let megapixels = total_pixels as f64 / 1_000_000.0;
    
    println!("\n📊 Calculated Metrics:");
    println!("  Total pixels: {}", total_pixels);
    println!("  Megapixels: {:.2} MP", megapixels);
    println!("  Aspect ratio: {:.2}:1", aspect_ratio);
    
    // 像素密度信息（如果可能）
    if let Some(dpi) = get_dpi_info(&img) {
        println!("  DPI: {}", dpi);
    }
    
    // 内存使用估算
    let estimated_memory = estimate_memory_usage(&img);
    println!("  Estimated memory usage: {:.2} MB", estimated_memory);
    
    println!("═");
    for _ in 0..60 {
        print!("═");
    }
    println!();
    
    Ok(())
}

fn get_dpi_info(_img: &DynamicImage) -> Option<u32> {
    // 尝试从图片中获取DPI信息
    // 注意：image库可能不支持所有格式的DPI读取
    None // 暂时返回None，实际实现可能需要使用其他库
}

fn estimate_memory_usage(img: &DynamicImage) -> f64 {
    // 估算图片在内存中的使用量（以MB为单位）
    let (width, height) = img.dimensions();
    let bytes_per_pixel = match img.color() {
        image::ColorType::Rgb8 => 3,
        image::ColorType::Rgba8 => 4,
        image::ColorType::L8 => 1,
        image::ColorType::La8 => 2,
        image::ColorType::Rgb16 => 6,
        image::ColorType::Rgba16 => 8,
        image::ColorType::L16 => 2,
        image::ColorType::La16 => 4,
        _ => 4, // 默认假设4字节每像素
    };
    
    let total_bytes = (width as u64 * height as u64 * bytes_per_pixel as u64) as f64;
    total_bytes / (1024.0 * 1024.0) // 转换为MB
}