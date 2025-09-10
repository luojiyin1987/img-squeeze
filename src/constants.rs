pub const DEFAULT_QUALITY: u8 = 80;
pub const MIN_QUALITY: u8 = 1;
pub const MAX_QUALITY: u8 = 100;

pub const DEFAULT_EPOCHS: u64 = 10;
pub const TEMP_EPOCHS: u64 = 1;

pub const ZOPFLI_ITERATIONS: u8 = 15;
pub const LIBDEFLATER_HIGH_LEVEL: u8 = 12;
pub const LIBDEFLATER_LOW_LEVEL: u8 = 8;

pub const DEFAULT_WALRUS_AGGREGATOR: &str = "https://aggregator.walrus-testnet.walrus.space";
pub const DEFAULT_WALRUS_PUBLISHER: &str = "https://publisher.walrus-testnet.walrus.space";

pub const MAX_IMAGE_DIMENSION: u32 = 16384; // Maximum allowed image dimension
pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100 MiB maximum file size

// Batch processing memory limits (using MiB for consistency with sysinfo)
pub const MAX_BATCH_MEMORY_MIB: u64 = 2048; // 2 GiB maximum total batch memory usage
pub const MAX_BATCH_FILES: usize = 10000; // Maximum number of files in a batch
pub const MIN_AVAILABLE_MEMORY_MIB: u64 = 512; // Minimum memory to keep available (MiB)
pub const LARGE_IMAGE_THRESHOLD_MIB: f64 = 50.0; // Images above this size are considered large (MiB)
pub const MAX_CONCURRENT_LARGE_IMAGES: usize = 2; // Maximum concurrent large image processing
