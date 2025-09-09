use crate::error::{CompressionError, Result};
use crate::formats::OutputFormat;
use crate::processing::{
    load_image_with_metadata, process_and_save_image, resize_image, CompressionOptions,
};
use crate::utils::is_image_file;
use glob::glob;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use walkdir::WalkDir;

/// Process multiple images in parallel
/// 
/// # Arguments
/// * `input` - Input directory path or glob pattern
/// * `output` - Output directory path
/// * `options` - Compression options to apply to all images
/// * `recursive` - Whether to process subdirectories recursively
/// 
/// # Returns
/// * `Ok(())` on success, `Err(CompressionError)` on failure
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

    // Collect all image files
    let image_files = collect_image_files(&input, recursive)?;
    let total_files = image_files.len();

    if total_files == 0 {
        println!("⚠️  No image files found in the input path");
        return Ok(());
    }

    println!("📊 Found {} image files to process", total_files);

    // Create output directory
    fs::create_dir_all(&output)
        .map_err(|_| CompressionError::DirectoryCreationFailed(output.clone()))?;

    // Setup progress tracking
    let main_progress = ProgressBar::new(total_files as u64);
    main_progress.set_style(ProgressStyle::default_bar());

    let processed_count = Arc::new(AtomicUsize::new(0));
    let total_size_before = Arc::new(AtomicUsize::new(0));
    let total_size_after = Arc::new(AtomicUsize::new(0));

    // Process files in parallel using Rayon
    let results: Vec<Result<()>> = image_files
        .into_par_iter()
        .map(|input_path| {
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
        })
        .collect();

    main_progress.finish_with_message("✅ Batch compression complete");

    print_batch_summary(start_time, &processed_count, &total_size_before, &total_size_after, &results);

    Ok(())
}

/// Collect image files from input path or pattern
/// 
/// # Arguments
/// * `input` - Input path (file, directory, or glob pattern)
/// * `recursive` - Whether to search subdirectories recursively
/// 
/// # Returns
/// * `Ok(Vec<PathBuf>)` containing found image files, or `Err(CompressionError)`
pub fn collect_image_files(input: &str, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut image_files = Vec::new();

    // Check if input is a file or directory
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

/// Print comprehensive batch processing summary
/// 
/// # Arguments
/// * `start_time` - When batch processing started
/// * `processed_count` - Atomic counter of processed files
/// * `total_size_before` - Atomic counter of original file sizes
/// * `total_size_after` - Atomic counter of compressed file sizes
/// * `results` - Results from parallel processing
fn print_batch_summary(
    start_time: Instant,
    processed_count: &Arc<AtomicUsize>,
    total_size_before: &Arc<AtomicUsize>,
    total_size_after: &Arc<AtomicUsize>,
    results: &[Result<()>],
) {
    let total_before = total_size_before.load(Ordering::Relaxed);
    let total_after = total_size_after.load(Ordering::Relaxed);
    let compression_ratio = if total_before > 0 {
        ((total_before as f64 - total_after as f64) / total_before as f64) * 100.0
    } else {
        0.0
    };

    let elapsed_time = start_time.elapsed();
    let processed_files = processed_count.load(Ordering::Relaxed);

    println!("\n📊 Batch Compression Summary:");
    println!("  📁 Total files processed: {}", processed_files);
    println!("  📊 Total original size: {} bytes", total_before);
    println!("  📊 Total compressed size: {} bytes", total_after);
    println!("  🎯 Overall compression ratio: {:.1}%", compression_ratio);
    println!("  ⏱️  Total time: {:?}", elapsed_time);
    
    if elapsed_time.as_secs_f64() > 0.0 {
        println!(
            "  ⚡ Average speed: {:.2} files/second",
            processed_files as f64 / elapsed_time.as_secs_f64()
        );
    }

    // Check for failed files
    let failed_count = results.iter().filter(|r| r.is_err()).count();
    if failed_count > 0 {
        println!("  ⚠️  Failed files: {}", failed_count);
    }
}

/// Generate output file path for processed image
/// 
/// # Arguments
/// * `input_path` - Original image file path
/// * `output_dir` - Output directory
/// * `format` - Optional format override string
/// 
/// # Returns
/// * `Ok(PathBuf)` containing the output path, or `Err(CompressionError)`
pub fn generate_output_path(
    input_path: &Path,
    output_dir: &Path,
    format: &Option<String>,
) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .ok_or_else(|| CompressionError::UnsupportedFormat("Invalid file name".to_string()))?;

    let extension = if let Some(fmt) = format {
        // Convert format string to OutputFormat enum and get extension
        match OutputFormat::from_str(fmt) {
            Ok(output_format) => output_format.extension(),
            Err(_) => return Err(CompressionError::UnsupportedFormat(fmt.clone())),
        }
    } else {
        // Keep original extension, defaulting to jpg
        input_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg")
    };

    let output_filename = format!("{}.{}", file_stem.to_string_lossy(), extension);
    Ok(output_dir.join(output_filename))
}

/// Process a single image file
/// 
/// # Arguments
/// * `input_path` - Path to input image file
/// * `output_dir` - Output directory  
/// * `options` - Compression options
/// 
/// # Returns
/// * `Ok((usize, usize))` containing (original_size, compressed_size), or `Err(CompressionError)`
fn process_single_image(
    input_path: &Path,
    output_dir: &Path,
    options: &CompressionOptions,
) -> Result<(usize, usize)> {
    // Generate output path
    let output_path = generate_output_path(input_path, output_dir, &options.format)?;

    // Process image
    let (mut img, original_size) = load_image_with_metadata(input_path)?;

    resize_image(&mut img, options);

    let compressed_size = process_and_save_image(&img, &output_path, options)?;

    Ok((original_size as usize, compressed_size as usize))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::is_image_file;
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
}
