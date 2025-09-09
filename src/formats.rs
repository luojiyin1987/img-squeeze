/// Image format utilities and type-safe format handling
/// 
/// This module provides type-safe image format handling, replacing string-based
/// format operations with proper enums and validation.

use crate::error::{CompressionError, Result};
use image::ImageFormat;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

/// Supported output image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// JPEG format with lossy compression
    Jpeg,
    /// PNG format with lossless compression
    Png,
    /// WebP format with modern compression
    WebP,
}

impl OutputFormat {
    /// Returns the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Jpeg => "jpg",
            OutputFormat::Png => "png", 
            OutputFormat::WebP => "webp",
        }
    }

    /// Convert to the image crate's ImageFormat
    pub fn to_image_format(&self) -> ImageFormat {
        match self {
            OutputFormat::Jpeg => ImageFormat::Jpeg,
            OutputFormat::Png => ImageFormat::Png,
            OutputFormat::WebP => ImageFormat::WebP,
        }
    }

    /// Get all supported formats as a vector
    pub fn all_formats() -> Vec<OutputFormat> {
        vec![OutputFormat::Jpeg, OutputFormat::Png, OutputFormat::WebP]
    }

    /// Get format names for CLI help text
    pub fn format_names() -> Vec<&'static str> {
        vec!["jpeg", "png", "webp"]
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            OutputFormat::Jpeg => "JPEG",
            OutputFormat::Png => "PNG",
            OutputFormat::WebP => "WebP",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for OutputFormat {
    type Err = CompressionError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "jpeg" | "jpg" => Ok(OutputFormat::Jpeg),
            "png" => Ok(OutputFormat::Png),
            "webp" => Ok(OutputFormat::WebP),
            _ => Err(CompressionError::UnsupportedFormat(s.to_string())),
        }
    }
}

/// Determine output format from file path and optional format override
pub fn determine_output_format(
    output_path: &Path, 
    format_override: Option<&str>
) -> Result<OutputFormat> {
    if let Some(fmt_str) = format_override {
        return OutputFormat::from_str(fmt_str);
    }

    // Try to determine from file extension
    if let Some(ext) = output_path.extension().and_then(|ext| ext.to_str()) {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Ok(OutputFormat::Jpeg),
            "png" => Ok(OutputFormat::Png),
            "webp" => Ok(OutputFormat::WebP),
            _ => {
                // Default to JPEG for unknown extensions
                Ok(OutputFormat::Jpeg)
            }
        }
    } else {
        // Default to JPEG if no extension
        Ok(OutputFormat::Jpeg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("jpeg").unwrap(), OutputFormat::Jpeg);
        assert_eq!(OutputFormat::from_str("jpg").unwrap(), OutputFormat::Jpeg);
        assert_eq!(OutputFormat::from_str("PNG").unwrap(), OutputFormat::Png);
        assert_eq!(OutputFormat::from_str("webp").unwrap(), OutputFormat::WebP);
        
        assert!(OutputFormat::from_str("unsupported").is_err());
    }

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Jpeg.extension(), "jpg");
        assert_eq!(OutputFormat::Png.extension(), "png");
        assert_eq!(OutputFormat::WebP.extension(), "webp");
    }

    #[test]
    fn test_determine_output_format_from_path() {
        let path = Path::new("test.jpg");
        assert_eq!(determine_output_format(path, None).unwrap(), OutputFormat::Jpeg);

        let path = Path::new("test.PNG");
        assert_eq!(determine_output_format(path, None).unwrap(), OutputFormat::Png);

        let path = Path::new("test.webp");
        assert_eq!(determine_output_format(path, None).unwrap(), OutputFormat::WebP);

        let path = Path::new("test.unknown");
        assert_eq!(determine_output_format(path, None).unwrap(), OutputFormat::Jpeg);
    }

    #[test]
    fn test_determine_output_format_with_override() {
        let path = Path::new("test.jpg");
        assert_eq!(
            determine_output_format(path, Some("png")).unwrap(), 
            OutputFormat::Png
        );
    }

    #[test]
    fn test_output_format_display() {
        assert_eq!(format!("{}", OutputFormat::Jpeg), "JPEG");
        assert_eq!(format!("{}", OutputFormat::Png), "PNG"); 
        assert_eq!(format!("{}", OutputFormat::WebP), "WebP");
    }
}