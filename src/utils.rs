/// Utility functions for common operations
/// 
/// This module contains helper functions that are used across multiple modules
/// to reduce code duplication and improve maintainability.

use crate::constants::{SUPPORTED_IMAGE_EXTENSIONS, WALRUS_BLOB_PATH};
use crate::error::{CompressionError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

/// Check if a file path represents a supported image file
/// 
/// # Arguments
/// * `path` - The file path to check
/// 
/// # Returns
/// * `true` if the file has a supported image extension, `false` otherwise
pub fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| {
            let ext_lower = ext.to_lowercase();
            SUPPORTED_IMAGE_EXTENSIONS.contains(&ext_lower.as_str())
        })
        .unwrap_or(false)
}

/// Validate that a file exists and return a descriptive error if not
/// 
/// # Arguments
/// * `path` - The file path to validate
/// 
/// # Returns
/// * `Ok(())` if file exists, `Err(CompressionError::FileNotFound)` otherwise
pub fn validate_file_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(CompressionError::FileNotFound(path.to_path_buf()));
    }
    Ok(())
}

/// Create a progress spinner with consistent styling
/// 
/// # Arguments
/// * `message` - Initial message to display
/// 
/// # Returns
/// * Configured `ProgressBar` instance
pub fn create_progress_spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template(crate::constants::PROGRESS_SPINNER_TEMPLATE)
            .expect("Invalid progress template"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Build a Walrus blob access URL from aggregator URL and blob ID
/// 
/// # Arguments
/// * `aggregator_url` - The Walrus aggregator base URL
/// * `blob_id` - The blob identifier
/// 
/// # Returns
/// * Complete URL for accessing the blob
pub fn build_walrus_access_url(aggregator_url: &str, blob_id: &str) -> String {
    if aggregator_url.ends_with('/') {
        format!("{}{}{}", aggregator_url.trim_end_matches('/'), WALRUS_BLOB_PATH, blob_id)
    } else {
        format!("{}{}{}", aggregator_url, WALRUS_BLOB_PATH, blob_id)
    }
}

/// Format file size in human-readable format
/// 
/// # Arguments
/// * `bytes` - Size in bytes
/// 
/// # Returns
/// * Human-readable size string (e.g., "1.2 MB", "512 KB")
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Calculate compression ratio as a percentage
/// 
/// # Arguments
/// * `original_size` - Original file size in bytes
/// * `compressed_size` - Compressed file size in bytes
/// 
/// # Returns
/// * Compression ratio as percentage (positive means reduction, negative means increase)
pub fn calculate_compression_ratio(original_size: u64, compressed_size: u64) -> f64 {
    if original_size == 0 {
        return 0.0;
    }
    ((original_size as f64 - compressed_size as f64) / original_size as f64) * 100.0
}

/// Print compression result with formatted output
/// 
/// # Arguments
/// * `original_size` - Original file size in bytes
/// * `compressed_size` - Compressed file size in bytes
pub fn print_compression_result(original_size: u64, compressed_size: u64) {
    let ratio = calculate_compression_ratio(original_size, compressed_size);
    
    println!("📈 Compressed size: {} ({})", 
             compressed_size, 
             format_file_size(compressed_size));
    println!("🎯 Compression ratio: {:.1}%", ratio);
    
    if ratio > 0.0 {
        println!("✅ Successfully reduced file size by {:.1}%", ratio);
    } else {
        println!("⚠️  File size increased by {:.1}%", ratio.abs());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_is_image_file() {
        assert!(is_image_file(Path::new("test.jpg")));
        assert!(is_image_file(Path::new("test.JPEG")));
        assert!(is_image_file(Path::new("test.png")));
        assert!(is_image_file(Path::new("test.webp")));
        assert!(is_image_file(Path::new("test.bmp")));
        assert!(is_image_file(Path::new("test.tiff")));
        assert!(is_image_file(Path::new("test.gif")));
        
        assert!(!is_image_file(Path::new("test.txt")));
        assert!(!is_image_file(Path::new("test")));
        assert!(!is_image_file(Path::new("test.doc")));
    }

    #[test]
    fn test_build_walrus_access_url() {
        let url1 = build_walrus_access_url("https://example.com", "blob123");
        assert_eq!(url1, "https://example.com/v1/blobs/blob123");
        
        let url2 = build_walrus_access_url("https://example.com/", "blob123");
        assert_eq!(url2, "https://example.com/v1/blobs/blob123");
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_calculate_compression_ratio() {
        assert_eq!(calculate_compression_ratio(1000, 800), 20.0);
        assert_eq!(calculate_compression_ratio(1000, 1200), -20.0);
        assert_eq!(calculate_compression_ratio(1000, 1000), 0.0);
        assert_eq!(calculate_compression_ratio(0, 500), 0.0);
    }

    #[test]
    fn test_validate_file_exists() {
        // Test with a file that definitely doesn't exist
        let result = validate_file_exists(Path::new("/nonexistent/file.jpg"));
        assert!(matches!(result, Err(CompressionError::FileNotFound(_))));
    }
}