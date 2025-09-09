/// Configuration constants for img-squeeze
/// 
/// This module contains all the default values, magic numbers, and configuration
/// constants used throughout the application.
/// Default compression quality (1-100 scale)
pub const DEFAULT_QUALITY: u8 = 80;

/// Quality threshold for using Zopfli compression in PNG optimization
pub const PNG_ZOPFLI_QUALITY_THRESHOLD: u8 = 90;

/// Quality threshold for high compression level in PNG optimization
pub const PNG_HIGH_COMPRESSION_QUALITY_THRESHOLD: u8 = 70;

/// Zopfli iterations for highest quality PNG compression
pub const PNG_ZOPFLI_ITERATIONS: u8 = 15;

/// High compression level for libdeflater
pub const PNG_HIGH_COMPRESSION_LEVEL: u8 = 12;

/// Standard compression level for libdeflater
pub const PNG_STANDARD_COMPRESSION_LEVEL: u8 = 8;

/// Temporary file epochs for Walrus storage
pub const WALRUS_TEMP_EPOCHS: u64 = 1;

/// Default epochs for Walrus storage
pub const WALRUS_DEFAULT_EPOCHS: u64 = 10;

/// Default Walrus aggregator URL
pub const WALRUS_DEFAULT_AGGREGATOR_URL: &str = "https://aggregator.walrus-testnet.walrus.space";

/// Default Walrus publisher URL  
pub const WALRUS_DEFAULT_PUBLISHER_URL: &str = "https://publisher.walrus-testnet.walrus.space";

/// Walrus blob access URL path
pub const WALRUS_BLOB_PATH: &str = "/v1/blobs/";

/// Supported image file extensions (lowercase)
pub const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "bmp", "tiff", "gif"
];

/// Progress bar template for spinner
pub const PROGRESS_SPINNER_TEMPLATE: &str = "{spinner:.green} {msg}";