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
}

pub type Result<T> = std::result::Result<T, CompressionError>;
