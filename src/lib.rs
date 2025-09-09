pub mod batch;
pub mod cli;
pub mod constants;
pub mod error;
pub mod formats;
pub mod info;
pub mod processing;
pub mod utils;
pub mod walrus;

pub use batch::{batch_compress_images, collect_image_files, generate_output_path};
pub use constants::*;
pub use error::{CompressionError, Result};
pub use formats::{OutputFormat, determine_output_format};
pub use info::{get_image_info, print_detailed_info};
pub use processing::{
    compress_image, load_image_with_metadata, process_and_save_image,
    resize_image, CompressionOptions,
};
pub use utils::{
    build_walrus_access_url, calculate_compression_ratio, create_progress_spinner,
    format_file_size, is_image_file, print_compression_result, validate_file_exists,
};
pub use walrus::{upload_to_walrus_async, upload_to_walrus_sync, WalrusOptions};
