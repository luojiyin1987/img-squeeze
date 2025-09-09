pub const DEFAULT_QUALITY: u8 = 80;
pub const MIN_QUALITY: u8 = 1;
pub const MAX_QUALITY: u8 = 100;

pub const DEFAULT_EPOCHS: u64 = 10;
pub const TEMP_EPOCHS: u64 = 1;

pub const DEFAULT_PNG_COMPRESSION_LEVEL: u8 = 6;
pub const HIGH_QUALITY_PNG_COMPRESSION_LEVEL: u8 = 9;
pub const LOW_QUALITY_PNG_COMPRESSION_LEVEL: u8 = 3;

pub const ZOPFLI_ITERATIONS: u8 = 15;
pub const LIBDEFLATER_HIGH_LEVEL: u8 = 12;
pub const LIBDEFLATER_LOW_LEVEL: u8 = 8;

pub const PROGRESS_BAR_WIDTH: usize = 40;
pub const ESTIMATED_TIME_FORMAT: &str = "[{:.2?}]";

// Common output message prefixes
pub const SIZE_PREFIX: &str = "📊";
pub const ORIGINAL_SIZE_PREFIX: &str = "📊 Original size:";
pub const COMPRESSED_SIZE_PREFIX: &str = "📈 Compressed size:";
pub const COMPRESSION_RATIO_PREFIX: &str = "🎯 Compression ratio:";
pub const SUCCESS_PREFIX: &str = "✅";
pub const WARNING_PREFIX: &str = "⚠️";
pub const ERROR_PREFIX: &str = "❌";
pub const INFO_PREFIX: &str = "📋";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqueezeImageFormat {
    Jpeg,
    Png,
    WebP,
    Bmp,
    Tiff,
    Gif,
}

impl SqueezeImageFormat {
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(SqueezeImageFormat::Jpeg),
            "png" => Some(SqueezeImageFormat::Png),
            "webp" => Some(SqueezeImageFormat::WebP),
            "bmp" => Some(SqueezeImageFormat::Bmp),
            "tiff" => Some(SqueezeImageFormat::Tiff),
            "gif" => Some(SqueezeImageFormat::Gif),
            _ => None,
        }
    }
    
    pub fn extension(&self) -> &'static str {
        match self {
            SqueezeImageFormat::Jpeg => "jpg",
            SqueezeImageFormat::Png => "png",
            SqueezeImageFormat::WebP => "webp",
            SqueezeImageFormat::Bmp => "bmp",
            SqueezeImageFormat::Tiff => "tiff",
            SqueezeImageFormat::Gif => "gif",
        }
    }
    
    pub fn mime_type(&self) -> &'static str {
        match self {
            SqueezeImageFormat::Jpeg => "image/jpeg",
            SqueezeImageFormat::Png => "image/png",
            SqueezeImageFormat::WebP => "image/webp",
            SqueezeImageFormat::Bmp => "image/bmp",
            SqueezeImageFormat::Tiff => "image/tiff",
            SqueezeImageFormat::Gif => "image/gif",
        }
    }
}

pub const DEFAULT_WALRUS_AGGREGATOR: &str = "https://aggregator.walrus-testnet.walrus.space";
pub const DEFAULT_WALRUS_PUBLISHER: &str = "https://publisher.walrus-testnet.walrus.space";