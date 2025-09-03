use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::{PathBuf, Path};
use image::{DynamicImage, ImageFormat, ImageReader};
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use walkdir::WalkDir;
use glob::glob;
use std::time::Instant;

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
        
        #[arg(short = 'j', long, help = "Number of parallel threads (default: auto)")]
        threads: Option<usize>,
    },
    
    #[command(about = "Compress multiple images in parallel")]
    Batch {
        #[arg(help = "Input directory or file pattern")]
        input: String,
        
        #[arg(help = "Output directory")]
        output: PathBuf,
        
        #[arg(short = 'q', long, help = "Quality (1-100), default is 80")]
        quality: Option<u8>,
        
        #[arg(short = 'w', long, help = "Maximum width in pixels")]
        width: Option<u32>,
        
        #[arg(short = 'H', long, help = "Maximum height in pixels")]
        height: Option<u32>,
        
        #[arg(short = 'f', long, help = "Output format (jpeg, png, webp)")]
        format: Option<String>,
        
        #[arg(short = 'j', long, help = "Number of parallel threads (default: auto)")]
        threads: Option<usize>,
        
        #[arg(short = 'r', long, help = "Recursive directory processing")]
        recursive: bool,
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
        Commands::Compress { input, output, quality, width, height, format, threads } => {
            setup_thread_pool(threads);
            compress_image(input, output, quality, width, height, format)?;
        }
        Commands::Batch { input, output, quality, width, height, format, threads, recursive } => {
            setup_thread_pool(threads);
            batch_compress_images(input, output, quality, width, height, format, recursive)?;
        }
        Commands::Info { input } => {
            show_image_info(input)?;
        }
    }
    
    Ok(())
}

fn setup_thread_pool(threads: Option<usize>) {
    if let Some(num_threads) = threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to set thread pool size: {}", e);
            });
    }
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
    if !(1..=100).contains(&quality) {
        return Err(anyhow::anyhow!("❌ Quality must be between 1 and 100"));
    }
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.set_message("Loading image...");
    
    let mut img = ImageReader::open(&input)?.decode()?;
    
    let original_size = fs::metadata(&input)?.len();
    pb.finish_with_message("✅ Image loaded");
    
    println!("📊 Original size: {} bytes ({}x{})", original_size, img.width(), img.height());
    
    if let Some(w) = width
        && w > 0 && w != img.width() {
        println!("🔄 Resizing width...");
        img = img.resize(w, img.height(), image::imageops::FilterType::Lanczos3);
        println!("✅ Resized to width: {}", w);
    }
    
    if let Some(h) = height
        && h > 0 && h != img.height() {
        println!("🔄 Resizing height...");
        img = img.resize(img.width(), h, image::imageops::FilterType::Lanczos3);
        println!("✅ Resized to height: {}", h);
    }
    
    let output_format = determine_output_format(output.as_path(), &format)?;
    
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

fn determine_output_format(output: &Path, format: &Option<String>) -> Result<ImageFormat> {
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

fn batch_compress_images(
    input: String,
    output: PathBuf,
    quality: Option<u8>,
    width: Option<u32>,
    height: Option<u32>,
    format: Option<String>,
    recursive: bool,
) -> Result<()> {
    println!("🚀 Starting batch compression...");
    println!("📁 Input: {}", input);
    println!("📁 Output: {:?}", output);
    
    let start_time = Instant::now();
    
    // 收集所有图片文件
    let image_files = collect_image_files(&input, recursive)?;
    let total_files = image_files.len();
    
    if total_files == 0 {
        println!("⚠️  No image files found in the input path");
        return Ok(());
    }
    
    println!("📊 Found {} image files to process", total_files);
    
    // 创建输出目录
    fs::create_dir_all(&output)?;
    
    // 设置进度条
    let main_progress = ProgressBar::new(total_files as u64);
    main_progress.set_style(ProgressStyle::default_bar());
    
    let processed_count = Arc::new(AtomicUsize::new(0));
    let total_size_before = Arc::new(AtomicUsize::new(0));
    let total_size_after = Arc::new(AtomicUsize::new(0));
    
    // 使用Rayon并行处理
    let results: Vec<Result<()>> = image_files.into_par_iter().map(|input_path| {
        let processed_count = processed_count.clone();
        let total_size_before = total_size_before.clone();
        let total_size_after = total_size_after.clone();
        
        match process_single_image(&input_path, &output, quality, width, height, &format) {
            Ok((before_size, after_size)) => {
                total_size_before.fetch_add(before_size, Ordering::Relaxed);
                total_size_after.fetch_add(after_size, Ordering::Relaxed);
                processed_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Failed to process {:?}: {}", input_path, e);
                Err(e)
            }
        }
    }).collect();
    
    main_progress.finish_with_message("✅ Batch compression complete");
    
    // 输出统计信息
    let total_before = total_size_before.load(Ordering::Relaxed);
    let total_after = total_size_after.load(Ordering::Relaxed);
    let compression_ratio = if total_before > 0 {
        ((total_before as f64 - total_after as f64) / total_before as f64) * 100.0
    } else {
        0.0
    };
    
    let elapsed_time = start_time.elapsed();
    
    println!("\n📊 Batch Compression Summary:");
    println!("  📁 Total files processed: {}", processed_count.load(Ordering::Relaxed));
    println!("  📊 Total original size: {} bytes", total_before);
    println!("  📊 Total compressed size: {} bytes", total_after);
    println!("  🎯 Overall compression ratio: {:.1}%", compression_ratio);
    println!("  ⏱️  Total time: {:?}", elapsed_time);
    println!("  ⚡ Average speed: {:.2} files/second", 
             processed_count.load(Ordering::Relaxed) as f64 / elapsed_time.as_secs_f64());
    
    // 检查是否有失败的文件
    let failed_count = results.iter().filter(|r| r.is_err()).count();
    if failed_count > 0 {
        println!("  ⚠️  Failed files: {}", failed_count);
    }
    
    Ok(())
}

fn collect_image_files(input: &str, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut image_files = Vec::new();
    
    // 检查输入是文件还是目录
    let input_path = Path::new(input);
    
    if input_path.exists() && input_path.is_file() {
        // 单个文件
        image_files.push(input_path.to_path_buf());
    } else if input_path.exists() && input_path.is_dir() {
        // 目录处理
        let walker = if recursive {
            WalkDir::new(input_path).into_iter()
        } else {
            WalkDir::new(input_path).max_depth(1).into_iter()
        };
        
        for entry in walker.filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.')) {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && is_image_file(path) {
                image_files.push(path.to_path_buf());
            }
        }
    } else {
        // 尝试使用glob模式
        if let Ok(glob_pattern) = glob(input) {
            for entry in glob_pattern.flatten() {
                if entry.is_file() && is_image_file(&entry) {
                    image_files.push(entry);
                }
            }
        } else {
            return Err(anyhow::anyhow!("Input path does not exist: {}", input));
        }
    }
    
    Ok(image_files)
}

fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tiff" | "gif")
    } else {
        false
    }
}

fn process_single_image(
    input_path: &Path,
    output_dir: &Path,
    quality: Option<u8>,
    width: Option<u32>,
    height: Option<u32>,
    format: &Option<String>,
) -> Result<(usize, usize)> {
    // 生成输出路径
    let output_path = generate_output_path(input_path, output_dir, format)?;
    
    // 读取原始文件大小
    let before_size = fs::metadata(input_path)?.len() as usize;
    
    // 处理图片
    let mut img = ImageReader::open(input_path)?.decode()?;
    
    if let Some(w) = width
        && w > 0 && w != img.width() {
        img = img.resize(w, img.height(), image::imageops::FilterType::Lanczos3);
    }
    
    if let Some(h) = height
        && h > 0 && h != img.height() {
        img = img.resize(img.width(), h, image::imageops::FilterType::Lanczos3);
    }
    
    let output_format = determine_output_format(output_path.as_path(), format)?;
    save_image(&img, &output_path, output_format, quality.unwrap_or(80))?;
    
    // 读取压缩后文件大小
    let after_size = fs::metadata(&output_path)?.len() as usize;
    
    Ok((before_size, after_size))
}

fn generate_output_path(input_path: &Path, output_dir: &Path, format: &Option<String>) -> Result<PathBuf> {
    let file_stem = input_path.file_stem()
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;
    
    let extension = if let Some(fmt) = format {
        match fmt.to_lowercase().as_str() {
            "jpeg" | "jpg" => "jpg",
            "png" => "png",
            "webp" => "webp",
            _ => return Err(anyhow::anyhow!("Unsupported format: {}", fmt)),
        }
    } else {
        input_path.extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg")
    };
    
    let output_filename = format!("{}.{}", file_stem.to_string_lossy(), extension);
    Ok(output_dir.join(output_filename))
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
