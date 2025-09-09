use crate::error::{CompressionError, Result};
use std::path::Path;
use std::fs::File;
use std::io::Read;
use walrus_rs::WalrusClient;

#[derive(Debug, Clone)]
pub struct WalrusOptions {
    pub aggregator_url: String,
    pub publisher_url: String,
    pub epochs: Option<u64>,
}

impl Default for WalrusOptions {
    fn default() -> Self {
        Self {
            aggregator_url: "https://aggregator.walrus-testnet.walrus.space".to_string(),
            publisher_url: "https://publisher.walrus-testnet.walrus.space".to_string(),
            epochs: Some(10),
        }
    }
}

impl WalrusOptions {
    pub fn new(
        aggregator_url: Option<String>,
        publisher_url: Option<String>,
        epochs: Option<u64>,
    ) -> Self {
        Self {
            aggregator_url: aggregator_url.unwrap_or_else(|| "https://aggregator.walrus-testnet.walrus.space".to_string()),
            publisher_url: publisher_url.unwrap_or_else(|| "https://publisher.walrus-testnet.walrus.space".to_string()),
            epochs,
        }
    }
}

pub async fn upload_to_walrus_async(
    file_path: &Path,
    options: &WalrusOptions,
) -> Result<String> {
    if !file_path.exists() {
        return Err(CompressionError::FileNotFound(file_path.to_path_buf()));
    }

    let mut file = File::open(file_path)
        .map_err(CompressionError::Io)?;
    
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(CompressionError::Io)?;

    let client = WalrusClient::new(&options.aggregator_url, &options.publisher_url)
        .map_err(|e| CompressionError::WalrusUpload(format!("Failed to create Walrus client: {}", e)))?;

    let store_result = client.store_blob(data, options.epochs, None, None, None)
        .await
        .map_err(|e| CompressionError::WalrusUpload(format!("Failed to store blob: {}", e)))?;

    if let Some(newly_created) = store_result.newly_created {
        Ok(newly_created.blob_object.blob_id)
    } else {
        Err(CompressionError::WalrusUpload("Failed to create new blob".to_string()))
    }
}

pub fn upload_to_walrus_sync(
    file_path: &Path,
    options: &WalrusOptions,
) -> Result<String> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| CompressionError::WalrusUpload(format!("Failed to create runtime: {}", e)))?;
    
    runtime.block_on(upload_to_walrus_async(file_path, options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walrus_options_default() {
        let options = WalrusOptions::default();
        assert_eq!(options.aggregator_url, "https://aggregator.walrus-testnet.walrus.space");
        assert_eq!(options.publisher_url, "https://publisher.walrus-testnet.walrus.space");
        assert_eq!(options.epochs, Some(10));
    }

    #[test]
    fn test_walrus_options_new() {
        let options = WalrusOptions::new(
            Some("https://custom.aggregator.com".to_string()),
            Some("https://custom.publisher.com".to_string()),
            Some(20),
        );
        
        assert_eq!(options.aggregator_url, "https://custom.aggregator.com");
        assert_eq!(options.publisher_url, "https://custom.publisher.com");
        assert_eq!(options.epochs, Some(20));
    }

    #[tokio::test]
    async fn test_upload_to_walrus_async_file_not_found() {
        let options = WalrusOptions::default();
        let result = upload_to_walrus_async(Path::new("nonexistent.jpg"), &options).await;
        assert!(matches!(result, Err(CompressionError::FileNotFound(_))));
    }
}