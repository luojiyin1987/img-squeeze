use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;
use image::{DynamicImage, ImageFormat, ImageReader};
use std::fs;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser)]
#[command(
    name = "img-squeeze",
    about = "A Rust-based image compression tool",
    version = "0.1.0"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Compress an image")]
    Compress {
        #[arg(help = "Input image file")]
        input: PathBuf,
        
        #[arg(help = "Output image file")]
        output: PathBuf,
        
        #[arg(short = 'q', long, help = "Quality (1-100), default is 80")]
        quality: Option<u8>,
        
        #[arg(short = 'w', long, help = "Maximum width in pixels")]
        width: Option<u32>,
        
        #[arg(short = 'H', long, help = "Maximum height in pixels")]
        height: Option<u32>,
        
        #[arg(short = 'f', long, help = "Output format (jpeg, png, webp)")]
        format: Option<String>,
    },
    
    #[command(about = "Get information about an image")]
    Info {
        #[arg(help = "Image file to analyze")]
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Commands::Compress { input, output, quality, width, height, format } => {
            compress_image(input, output, quality, width, height, format)?;
        }
        Commands::Info { input } => {
            show_image_info(input)?;
        }
    }
    
    Ok(())
}

fn compress_image(
    input: PathBuf,
    output: PathBuf,
    quality: Option<u8>,
    width: Option<u32>,
    height: Option<u32>,
    format: Option<String>,
) -> Result<()> {
    println!("🗜️  Compressing image: {:?}", input);
    println!("📁 Output: {:?}", output);
    
    if !input.exists() {
        return Err(anyhow::anyhow!("❌ Input file does not exist: {:?}", input));
    }
    
    let quality = quality.unwrap_or(80);
    if quality < 1 || quality > 100 {
        return Err(anyhow::anyhow!("❌ Quality must be between 1 and 100"));
    }
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Loading image...");
    
    let mut img = ImageReader::open(&input)?.decode()?;
    
    let original_size = fs::metadata(&input)?.len();
    pb.finish_with_message("✅ Image loaded");
    
    println!("📊 Original size: {} bytes ({}x{})", original_size, img.width(), img.height());
    
    if let Some(w) = width {
        if w > 0 && w != img.width() {
            println!("🔄 Resizing width...");
            img = img.resize(w, img.height(), image::imageops::FilterType::Lanczos3);
            println!("✅ Resized to width: {}", w);
        }
    }
    
    if let Some(h) = height {
        if h > 0 && h != img.height() {
            println!("🔄 Resizing height...");
            img = img.resize(img.width(), h, image::imageops::FilterType::Lanczos3);
            println!("✅ Resized to height: {}", h);
        }
    }
    
    let output_format = determine_output_format(&output, &format)?;
    
    pb.set_message("Saving compressed image...");
    save_image(&img, &output, output_format, quality)?;
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

fn determine_output_format(output: &PathBuf, format: &Option<String>) -> Result<ImageFormat> {
    if let Some(fmt) = format {
        match fmt.to_lowercase().as_str() {
            "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
            "png" => Ok(ImageFormat::Png),
            "webp" => Ok(ImageFormat::WebP),
            _ => Err(anyhow::anyhow!("Unsupported format: {}", fmt)),
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

fn save_image(img: &DynamicImage, output: &PathBuf, format: ImageFormat, _quality: u8) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    
    match format {
        ImageFormat::Jpeg => {
            img.save_with_format(output, image::ImageFormat::Jpeg)?;
        }
        ImageFormat::Png => {
            img.save_with_format(output, image::ImageFormat::Png)?;
        }
        ImageFormat::WebP => {
            img.save_with_format(output, image::ImageFormat::WebP)?;
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported output format"));
        }
    }
    
    Ok(())
}

fn show_image_info(input: PathBuf) -> Result<()> {
    println!("📋 Getting info for: {:?}", input);
    
    if !input.exists() {
        return Err(anyhow::anyhow!("❌ File does not exist: {:?}", input));
    }
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Loading image info...");
    
    let img = ImageReader::open(&input)?.decode()?;
    let format = ImageReader::open(&input)?.with_guessed_format()?.format();
    let metadata = fs::metadata(&input)?;
    
    pb.finish_with_message("✅ Image info loaded");
    
    println!("📸 Image Information:");
    println!("  📏 Dimensions: {}x{}", img.width(), img.height());
    println!("  🎨 Color type: {:?}", img.color());
    println!("  💾 Format: {:?}", format);
    println!("  📊 File size: {} bytes", metadata.len());
    
    let megapixels = (img.width() * img.height()) as f64 / 1_000_000.0;
    println!("  📈 Megapixels: {:.1}", megapixels);
    
    Ok(())
}
