use crate::constants::TEMP_EPOCHS;
use crate::error::Result;
use crate::processing::validate_file_exists;
use crate::walrus::{upload_to_walrus_sync, WalrusOptions};
use std::path::Path;

/// Handles uploading an image to Walrus decentralized storage
/// 
/// # Arguments
/// * `input_path` - Path to the image file to upload
/// * `aggregator_url` - Optional custom aggregator URL (uses default if None)
/// * `publisher_url` - Optional custom publisher URL (uses default if None)
/// * `epochs` - Optional number of epochs for storage (uses default if None)
/// * `temp` - If true, uploads as temporary file with 1 epoch storage
/// 
/// # Returns
/// * `Ok(())` if upload succeeds
/// * `Err(CompressionError)` if upload fails
pub fn upload_image_to_walrus(
    input_path: &Path,
    aggregator_url: Option<String>,
    publisher_url: Option<String>,
    epochs: Option<u64>,
    temp: bool,
) -> Result<()> {
    println!("📤 Uploading to Walrus: {:?}", input_path);

    validate_file_exists(input_path)?;

    // 处理临时文件选项
    let final_epochs = if temp {
        Some(TEMP_EPOCHS) // 临时文件只存储 1 个 epoch
    } else {
        epochs
    };

    let options = WalrusOptions::new(aggregator_url, publisher_url, final_epochs);

    println!("🔗 Aggregator URL: {}", options.aggregator_url);
    println!("🔗 Publisher URL: {}", options.publisher_url);
    println!("⏰ Epochs: {:?}", options.epochs);

    let blob_id = upload_to_walrus_sync(input_path, &options)?;

    println!("✅ Upload successful!");
    println!("🆔 Blob ID: {}", blob_id);

    // 构建访问地址
    let access_url = build_walrus_access_url(&options.aggregator_url, &blob_id);
    println!("🌐 Access URL: {}", access_url);

    // 临时文件提示
    if temp {
        println!("⏰ Temporary file: Will expire after 1 epoch (~24 hours)");
        println!("🔄 Use without -t flag for longer storage");
    }

    // 显示文件信息
    if let Ok(metadata) = std::fs::metadata(input_path) {
        println!("📊 File size: {} bytes", metadata.len());
    }

    println!("💡 You can use the blob ID to retrieve the file later");

    Ok(())
}

/// Builds a Walrus network access URL from aggregator URL and blob ID
/// 
/// # Arguments
/// * `aggregator_url` - The base aggregator URL
/// * `blob_id` - The blob ID returned from storage
/// 
/// # Returns
/// * Complete URL for accessing the stored blob
fn build_walrus_access_url(aggregator_url: &str, blob_id: &str) -> String {
    // 构建 Walrus 网络的访问地址
    // 通常格式是 {aggregator_url}/v1/blobs/{blob_id}
    if aggregator_url.ends_with('/') {
        format!("{}v1/blobs/{}", aggregator_url, blob_id)
    } else {
        format!("{}/v1/blobs/{}", aggregator_url, blob_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_walrus_access_url_with_trailing_slash() {
        let aggregator = "https://example.com/";
        let blob_id = "test123";
        let result = build_walrus_access_url(aggregator, blob_id);
        assert_eq!(result, "https://example.com/v1/blobs/test123");
    }

    #[test]
    fn test_build_walrus_access_url_without_trailing_slash() {
        let aggregator = "https://example.com";
        let blob_id = "test123";
        let result = build_walrus_access_url(aggregator, blob_id);
        assert_eq!(result, "https://example.com/v1/blobs/test123");
    }
}