use crate::error::{CompressionError, Result};
use crate::processing::{determine_output_format, save_image, resize_image, CompressionOptions};
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use walkdir::WalkDir;
use glob::glob;
use std::time::Instant;

pub fn batch_compress_images(
    input: String,
    output: PathBuf,
    options: CompressionOptions,
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
    fs::create_dir_all(&output)
        .map_err(|_| CompressionError::DirectoryCreationFailed(output.clone()))?;
    
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
        
        match process_single_image(&input_path, &output, &options) {
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
    } else if let Ok(glob_pattern) = glob(input) {
        // 尝试使用glob模式
        for entry in glob_pattern.flatten() {
            if entry.is_file() && is_image_file(&entry) {
                image_files.push(entry);
            }
        }
    } else {
        return Err(CompressionError::NoImageFilesFound(input.to_string()));
    }
    
    Ok(image_files)
}

fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tiff" | "gif"))
        .unwrap_or(false)
}

fn process_single_image(
    input_path: &Path,
    output_dir: &Path,
    options: &CompressionOptions,
) -> Result<(usize, usize)> {
    // 生成输出路径
    let output_path = generate_output_path(input_path, output_dir, &options.format)?;
    
    // 读取原始文件大小
    let before_size = fs::metadata(input_path)?.len() as usize;
    
    // 处理图片
    let mut img = image::ImageReader::open(input_path)?.decode()?;
    
    resize_image(&mut img, options);
    
    let output_format = determine_output_format(&output_path, &options.format)?;
    save_image(&img, &output_path, output_format, options)?;
    
    // 读取压缩后文件大小
    let after_size = fs::metadata(&output_path)?.len() as usize;
    
    Ok((before_size, after_size))
}

fn generate_output_path(input_path: &Path, output_dir: &Path, format: &Option<String>) -> Result<PathBuf> {
    let file_stem = input_path.file_stem()
        .ok_or_else(|| CompressionError::UnsupportedFormat("Invalid file name".to_string()))?;
    
    let extension = if let Some(fmt) = format {
        match fmt.to_lowercase().as_str() {
            "jpeg" | "jpg" => "jpg",
            "png" => "png",
            "webp" => "webp",
            _ => return Err(CompressionError::UnsupportedFormat(fmt.clone())),
        }
    } else if let Some(ext) = input_path.extension().and_then(|s| s.to_str()) {
        ext
    } else {
        "jpg"
    };
    
    let output_filename = format!("{}.{}", file_stem.to_string_lossy(), extension);
    Ok(output_dir.join(output_filename))
}

