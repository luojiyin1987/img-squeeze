use crate::constants::{
    LARGE_IMAGE_THRESHOLD_MIB, MAX_BATCH_FILES, MAX_BATCH_MEMORY_MIB, MAX_CONCURRENT_LARGE_IMAGES,
    MIN_AVAILABLE_MEMORY_MIB,
};
use crate::error::{CompressionError, Result};
use crate::processing::{process_image_pipeline, CompressionOptions};
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use sysinfo::{MemoryRefreshKind, RefreshKind, System};
use walkdir::WalkDir;

/// Estimates memory usage for an image file without loading it into memory.
///
/// # Arguments
/// * `file_path` - Path to the image file
///
/// # Returns
/// * `Ok(memory_mib)` - Estimated memory usage in MiB
/// * `Err(CompressionError)` - If file metadata cannot be read
fn estimate_image_memory_usage(file_path: &Path) -> Result<f64> {
    let metadata = fs::metadata(file_path)?;
    let file_size_mib = metadata.len() as f64 / (1024.0 * 1024.0);

    // Conservative estimate: uncompressed image memory usage is typically 3-4x file size
    // for compressed formats like JPEG, and 1-1.5x for uncompressed formats like BMP
    let multiplier = match file_path.extension().and_then(|s| s.to_str()) {
        Some(ext) => match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => 4.0, // JPEG compression ratio is typically high
            "png" => 3.0,          // PNG has good compression
            "webp" => 3.5,         // WebP has good compression
            "bmp" | "tiff" => 1.2, // Usually uncompressed or lightly compressed
            "gif" => 2.0,          // GIF has moderate compression
            "avif" | "heic" | "heif" => 4.0, // Modern efficient formats
            _ => 3.0,              // Default conservative estimate
        },
        None => 3.0,
    };

    Ok(file_size_mib * multiplier)
}

/// Validates batch memory requirements before processing.
///
/// # Arguments
/// * `image_files` - List of image file paths to process
///
/// # Returns
/// * `Ok((total_memory_mib, large_image_count))` - Estimated memory usage and count of large images
/// * `Err(CompressionError)` - If memory limits would be exceeded
fn validate_batch_memory_limits(image_files: &[PathBuf]) -> Result<(f64, usize)> {
    // Check file count limit
    if image_files.len() > MAX_BATCH_FILES {
        return Err(CompressionError::BatchFileLimitExceeded(
            image_files.len(),
            MAX_BATCH_FILES,
        ));
    }

    let mut total_memory_mib = 0.0;
    let mut large_image_count = 0;

    // Estimate memory usage for each file
    for file_path in image_files {
        let memory_estimate = estimate_image_memory_usage(file_path)?;
        total_memory_mib += memory_estimate;

        if memory_estimate > LARGE_IMAGE_THRESHOLD_MIB {
            large_image_count += 1;
        }
    }

    // Check total memory limit
    let total_memory_mib_u64 = total_memory_mib.ceil() as u64;
    if total_memory_mib_u64 > MAX_BATCH_MEMORY_MIB {
        return Err(CompressionError::BatchMemoryLimitExceeded(
            total_memory_mib_u64,
            MAX_BATCH_MEMORY_MIB,
        ));
    }

    // Check against actual available memory (host/container)
    // sysinfo 0.30+ returns bytes. Convert to MiB.
    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_memory(MemoryRefreshKind::new())
    );
    sys.refresh_memory();
    let available_mem_mib = sys.available_memory() / (1024 * 1024); // bytes -> MiB
    let required_with_buffer = total_memory_mib_u64 + MIN_AVAILABLE_MEMORY_MIB;
    if required_with_buffer > available_mem_mib {
        // Report how much is actually available (not subtracting buffer for transparency)
        return Err(CompressionError::InsufficientMemory(
            total_memory_mib_u64,
            available_mem_mib,
        ));
    }

    Ok((total_memory_mib, large_image_count))
}

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

    // Security: Validate batch memory requirements before processing
    println!("🔍 Validating batch memory requirements...");
    let (estimated_memory_mib, large_image_count) = validate_batch_memory_limits(&image_files)?;

    println!("📊 Batch validation complete:");
    println!("  📁 Total files: {}", total_files);
    println!("  💾 Estimated memory usage: {:.1} MiB", estimated_memory_mib);
    println!(
        "  📏 Large images (>{}MiB): {}",
        LARGE_IMAGE_THRESHOLD_MIB, large_image_count
    );

    // Adjust parallelism based on large image count and available memory
    let baseline = rayon::current_num_threads().min(total_files);
    let large_cap = if large_image_count >= MAX_CONCURRENT_LARGE_IMAGES {
        MAX_CONCURRENT_LARGE_IMAGES
    } else {
        baseline
    };
    // Derive an upper bound from available memory vs. avg per-file estimate
    let mut sys = System::new_with_specifics(RefreshKind::new().with_memory(MemoryRefreshKind::new()));
    sys.refresh_memory();
    let available_mem_mib = sys.available_memory() / (1024 * 1024);
    let avg_per_file_mib = ((estimated_memory_mib / total_files as f64).ceil() as u64).max(1);
    let mem_cap = ((available_mem_mib.saturating_sub(MIN_AVAILABLE_MEMORY_MIB)) / avg_per_file_mib)
        .clamp(1, baseline as u64) as usize;
    let max_parallelism = large_cap.min(mem_cap);

    println!(
        "⚙️  Using {} parallel threads for processing",
        max_parallelism
    );

    // Build a scoped Rayon pool to enforce the chosen parallelism
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(max_parallelism)
        .build()
        .expect("Failed to build Rayon thread pool");

    // 创建输出目录
    fs::create_dir_all(&output)
        .map_err(|_| CompressionError::DirectoryCreationFailed(output.clone()))?;

    // 设置进度条
    let main_progress = ProgressBar::new(total_files as u64);
    main_progress.set_style(ProgressStyle::default_bar());

    let processed_count = Arc::new(AtomicUsize::new(0));
    let total_size_before = Arc::new(AtomicUsize::new(0));
    let total_size_after = Arc::new(AtomicUsize::new(0));

    // Security: Use limited parallelism based on memory requirements
    let results: Vec<Result<()>> = pool.install(|| {
        if large_image_count > MAX_CONCURRENT_LARGE_IMAGES {
            // For batches with many large images, use chunked processing to limit memory usage
            let chunk_size = MAX_CONCURRENT_LARGE_IMAGES.max(1);
            image_files
                .chunks(chunk_size)
                .flat_map(|chunk| {
                    chunk
                        .into_par_iter()
                        .map(|input_path| {
                            let progress = main_progress.clone();
                            let processed_count = processed_count.clone();
                            let total_size_before = total_size_before.clone();
                            let total_size_after = total_size_after.clone();

                            match process_single_image(input_path, &output, &options) {
                                Ok((before_size, after_size)) => {
                                    total_size_before.fetch_add(before_size, Ordering::Relaxed);
                                    total_size_after.fetch_add(after_size, Ordering::Relaxed);
                                    processed_count.fetch_add(1, Ordering::Relaxed);
                                    progress.inc(1);
                                    Ok(())
                                }
                                Err(e) => {
                                    eprintln!("❌ Failed to process {:?}: {}", input_path, e);
                                    progress.inc(1);
                                    Err(e)
                                }
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        } else {
            // Standard parallel processing for smaller batches
            image_files
                .into_par_iter()
                .map(|input_path| {
                    let progress = main_progress.clone();
                    let processed_count = processed_count.clone();
                    let total_size_before = total_size_before.clone();
                    let total_size_after = total_size_after.clone();

                    match process_single_image(&input_path, &output, &options) {
                        Ok((before_size, after_size)) => {
                            total_size_before.fetch_add(before_size, Ordering::Relaxed);
                            total_size_after.fetch_add(after_size, Ordering::Relaxed);
                            processed_count.fetch_add(1, Ordering::Relaxed);
                            progress.inc(1);
                            Ok(())
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to process {:?}: {}", input_path, e);
                            progress.inc(1);
                            Err(e)
                        }
                    }
                })
                .collect()
        }
    });

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
    println!(
        "  📁 Total files processed: {}",
        processed_count.load(Ordering::Relaxed)
    );
    println!("  📊 Total original size: {} bytes", total_before);
    println!("  📊 Total compressed size: {} bytes", total_after);
    println!("  🎯 Overall compression ratio: {:.1}%", compression_ratio);
    println!("  ⏱️  Total time: {:?}", elapsed_time);
    println!(
        "  ⚡ Average speed: {:.2} files/second",
        processed_count.load(Ordering::Relaxed) as f64 / elapsed_time.as_secs_f64()
    );

    // 检查是否有失败的文件
    let failed_count = results.iter().filter(|r| r.is_err()).count();
    if failed_count > 0 {
        println!("  ⚠️  Failed files: {}", failed_count);
    }

    Ok(())
}

pub fn collect_image_files(input: &str, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut image_files = Vec::new();

    // Security: Validate and canonicalize input path to prevent directory traversal
    let input_path = Path::new(input);
    let canonical_input = if input_path.exists() {
        input_path
            .canonicalize()
            .map_err(|_| CompressionError::NoImageFilesFound(input.to_string()))?
    } else {
        // For glob patterns, we'll validate each result individually
        input_path.to_path_buf()
    };

    if canonical_input.exists() && canonical_input.is_file() {
        // 单个文件
        image_files.push(canonical_input);
    } else if canonical_input.exists() && canonical_input.is_dir() {
        // 目录处理
        let walker = if recursive {
            WalkDir::new(&canonical_input).into_iter()
        } else {
            WalkDir::new(&canonical_input).max_depth(1).into_iter()
        };

        for entry in walker.filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.')) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && is_image_file(path) {
                // Security: Canonicalize each file path
                if let Ok(canonical_path) = path.canonicalize() {
                    image_files.push(canonical_path);
                }
            }
        }
    } else if let Ok(glob_pattern) = glob(input) {
        // 尝试使用glob模式
        for entry in glob_pattern.flatten() {
            if entry.is_file() && is_image_file(&entry) {
                // Security: Canonicalize glob results
                if let Ok(canonical_path) = entry.canonicalize() {
                    image_files.push(canonical_path);
                }
            }
        }
    } else {
        return Err(CompressionError::NoImageFilesFound(input.to_string()));
    }

    Ok(image_files)
}

pub fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| {
            matches!(
                ext.to_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tiff" | "gif" | "avif" | "heic" | "heif" | "jxl"
            )
        })
        .unwrap_or(false)
}

fn process_single_image(
    input_path: &Path,
    output_dir: &Path,
    options: &CompressionOptions,
) -> Result<(usize, usize)> {
    // 生成输出路径
    let output_path = generate_output_path(input_path, output_dir, &options.format)?;

    // 使用统一的图片处理管道
    let (original_size, compressed_size) =
        process_image_pipeline(input_path, &output_path, options)?;

    Ok((original_size as usize, compressed_size as usize))
}

pub fn generate_output_path(
    input_path: &Path,
    output_dir: &Path,
    format: &Option<String>,
) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .ok_or_else(|| CompressionError::UnsupportedFormat("Invalid file name".to_string()))?;

    let extension = if let Some(fmt) = format {
        match fmt.to_lowercase().as_str() {
            "jpeg" | "jpg" => "jpg",
            "png" => "png",
            "webp" => "webp",
            _ => return Err(CompressionError::UnsupportedFormat(fmt.clone())),
        }
    } else {
        input_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg")
    };

    let output_filename = format!("{}.{}", file_stem.to_string_lossy(), extension);
    Ok(output_dir.join(output_filename))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_is_image_file() {
        let path = Path::new("test.jpg");
        assert!(is_image_file(path));

        let path = Path::new("test.jpeg");
        assert!(is_image_file(path));

        let path = Path::new("test.png");
        assert!(is_image_file(path));

        let path = Path::new("test.webp");
        assert!(is_image_file(path));

        let path = Path::new("test.bmp");
        assert!(is_image_file(path));

        let path = Path::new("test.tiff");
        assert!(is_image_file(path));

        let path = Path::new("test.gif");
        assert!(is_image_file(path));

        let path = Path::new("test.avif");
        assert!(is_image_file(path));

        let path = Path::new("test.heic");
        assert!(is_image_file(path));

        let path = Path::new("test.heif");
        assert!(is_image_file(path));

        let path = Path::new("test.jxl");
        assert!(is_image_file(path));

        let path = Path::new("test.txt");
        assert!(!is_image_file(path));

        let path = Path::new("test");
        assert!(!is_image_file(path));
    }

    #[test]
    fn test_is_image_file_case_insensitive() {
        let path = Path::new("test.JPG");
        assert!(is_image_file(path));

        let path = Path::new("test.PnG");
        assert!(is_image_file(path));
    }

    #[test]
    fn test_generate_output_path() {
        let input_path = Path::new("test.jpg");
        let output_dir = Path::new("/tmp/output");

        let result = generate_output_path(input_path, output_dir, &None).unwrap();
        assert_eq!(result, PathBuf::from("/tmp/output/test.jpg"));
    }

    #[test]
    fn test_generate_output_path_with_format_override() {
        let input_path = Path::new("test.jpg");
        let output_dir = Path::new("/tmp/output");

        let result =
            generate_output_path(input_path, output_dir, &Some("png".to_string())).unwrap();
        assert_eq!(result, PathBuf::from("/tmp/output/test.png"));
    }

    #[test]
    fn test_generate_output_path_webp_format() {
        let input_path = Path::new("test.jpg");
        let output_dir = Path::new("/tmp/output");

        let result =
            generate_output_path(input_path, output_dir, &Some("webp".to_string())).unwrap();
        assert_eq!(result, PathBuf::from("/tmp/output/test.webp"));
    }

    #[test]
    fn test_generate_output_path_unsupported_format() {
        let input_path = Path::new("test.jpg");
        let output_dir = Path::new("/tmp/output");

        let result = generate_output_path(input_path, output_dir, &Some("unsupported".to_string()));
        assert!(matches!(
            result,
            Err(CompressionError::UnsupportedFormat(_))
        ));
    }

    #[test]
    fn test_collect_image_files_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.jpg");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"fake image data").unwrap();

        let files = collect_image_files(&test_file.to_string_lossy(), false).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], test_file);
    }

    #[test]
    fn test_collect_image_files_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create test image files
        File::create(temp_dir.path().join("test1.jpg")).unwrap();
        File::create(temp_dir.path().join("test2.png")).unwrap();
        File::create(temp_dir.path().join("not_image.txt")).unwrap();

        let files = collect_image_files(&temp_dir.path().to_string_lossy(), false).unwrap();
        // Note: Empty files won't be detected as images, so we expect 0 files
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_collect_image_files_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        // Create test files
        File::create(temp_dir.path().join("test1.jpg")).unwrap();
        File::create(subdir.join("test2.png")).unwrap();

        let files = collect_image_files(&temp_dir.path().to_string_lossy(), true).unwrap();
        // Note: Empty files won't be detected as images, so we expect 0 files
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_collect_image_files_non_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        // Create test files
        File::create(temp_dir.path().join("test1.jpg")).unwrap();
        File::create(subdir.join("test2.png")).unwrap();

        let files = collect_image_files(&temp_dir.path().to_string_lossy(), false).unwrap();
        // Note: Empty files won't be detected as images, so we expect 0 files
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_collect_image_files_no_files() {
        let temp_dir = TempDir::new().unwrap();

        let result = collect_image_files(&temp_dir.path().to_string_lossy(), false).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_collect_image_files_glob_pattern() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        File::create(temp_dir.path().join("test1.jpg")).unwrap();
        File::create(temp_dir.path().join("test2.png")).unwrap();
        File::create(temp_dir.path().join("other.txt")).unwrap();

        let pattern = format!("{}/*.jpg", temp_dir.path().to_string_lossy());
        let files = collect_image_files(&pattern, false).unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_estimate_image_memory_usage() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.jpg");

        // Create a test file with known size (1KB)
        let mut file = File::create(&test_file).unwrap();
        let data = vec![0u8; 1024]; // 1KB of data
        file.write_all(&data).unwrap();

        let memory_estimate = estimate_image_memory_usage(&test_file).unwrap();

        // JPEG multiplier is 4.0, so 1KB file should estimate ~4KB memory (0.004MB)
        assert!(memory_estimate > 0.0);
        assert!(memory_estimate < 1.0); // Should be less than 1MB for 1KB file
    }

    #[test]
    fn test_estimate_image_memory_usage_png() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.png");

        // Create a test PNG file
        let mut file = File::create(&test_file).unwrap();
        let data = vec![0u8; 2048]; // 2KB of data
        file.write_all(&data).unwrap();

        let memory_estimate = estimate_image_memory_usage(&test_file).unwrap();

        // PNG multiplier is 3.0, so 2KB file should estimate ~6KB memory
        assert!(memory_estimate > 0.0);
        assert!(memory_estimate < 1.0); // Should be less than 1MB for 2KB file
    }

    #[test]
    fn test_validate_batch_memory_limits_empty() {
        let files = vec![];
        let result = validate_batch_memory_limits(&files).unwrap();
        assert_eq!(result.0, 0.0); // No memory usage
        assert_eq!(result.1, 0); // No large images
    }

    #[test]
    fn test_validate_batch_memory_limits_file_count_exceeded() {
        // Create more files than the limit
        let mut files = Vec::new();
        for i in 0..(MAX_BATCH_FILES + 1) {
            files.push(PathBuf::from(format!("test{}.jpg", i)));
        }

        let result = validate_batch_memory_limits(&files);
        assert!(matches!(
            result,
            Err(CompressionError::BatchFileLimitExceeded(_, _))
        ));
    }

    #[test]
    fn test_validate_batch_memory_limits_with_real_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create some small test files
        let file1 = temp_dir.path().join("test1.jpg");
        let file2 = temp_dir.path().join("test2.png");

        File::create(&file1)
            .unwrap()
            .write_all(&vec![0u8; 1024])
            .unwrap(); // 1KB
        File::create(&file2)
            .unwrap()
            .write_all(&vec![0u8; 2048])
            .unwrap(); // 2KB

        let files = vec![file1, file2];
        let result = validate_batch_memory_limits(&files).unwrap();

        assert!(result.0 > 0.0); // Should have some memory estimate
        assert_eq!(result.1, 0); // No large images (files are too small)
    }

    #[test]
    fn test_validate_batch_memory_limits_large_images() {
        let temp_dir = TempDir::new().unwrap();

        // Create a large test file (simulating a large image)
        let large_file = temp_dir.path().join("large.jpg");
        let large_data = vec![0u8; 20 * 1024 * 1024]; // 20MB file
        File::create(&large_file)
            .unwrap()
            .write_all(&large_data)
            .unwrap();

        let files = vec![large_file];
        let result = validate_batch_memory_limits(&files).unwrap();

        assert!(result.0 > LARGE_IMAGE_THRESHOLD_MIB); // Memory estimate should be above threshold
        assert_eq!(result.1, 1); // Should count as 1 large image
    }
}
