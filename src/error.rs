use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompressionError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    ImageProcessing(#[from] image::ImageError),

    #[error("PNG optimization error: {0}")]
    PngOptimization(String),

    #[error("Invalid quality value: {0}. Must be between 1 and 100")]
    InvalidQuality(u8),

    #[error("Invalid image dimensions: {0}x{1}. Maximum allowed: {2}x{2}")]
    InvalidDimensions(u32, u32, u32),

    #[error("File too large: {0} bytes. Maximum allowed: {1} bytes")]
    FileTooLarge(u64, u64),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Failed to create output directory: {0}")]
    DirectoryCreationFailed(PathBuf),

    #[error("No image files found in input path: {0}")]
    NoImageFilesFound(String),

    #[error("Walkdir error: {0}")]
    WalkdirError(#[from] walkdir::Error),

    #[error("Walrus upload error: {0}")]
    WalrusUpload(String),

    #[error("Batch memory limit exceeded: estimated {0}MB, maximum allowed {1}MB")]
    BatchMemoryLimitExceeded(u64, u64),

    #[error("Batch file count limit exceeded: {0} files, maximum allowed {1}")]
    BatchFileLimitExceeded(usize, usize),

    #[error(
        "Insufficient available memory: estimated batch requires {0}MB, but only {1}MB available"
    )]
    InsufficientMemory(u64, u64),
}

pub type Result<T> = std::result::Result<T, CompressionError>;
