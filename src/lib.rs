pub mod batch;
pub mod cli;
pub mod error;
pub mod info;
pub mod processing;
pub mod walrus;

pub use batch::{batch_compress_images, collect_image_files, generate_output_path, is_image_file};
pub use error::{CompressionError, Result};
pub use info::{get_image_info, print_detailed_info};
pub use processing::{
    compress_image, determine_output_format, load_image_with_metadata, process_and_save_image,
    process_image_pipeline, resize_image, validate_file_exists, CompressionOptions,
};
pub use walrus::{upload_to_walrus_async, upload_to_walrus_sync, WalrusOptions};
