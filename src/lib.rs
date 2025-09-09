pub mod cli;
pub mod error;
pub mod processing;
pub mod batch;
pub mod info;
pub mod walrus;
pub mod logger;
pub mod validation;

pub use error::{CompressionError, Result};
pub use processing::{CompressionOptions, compress_image, determine_output_format, resize_image, load_image_with_metadata, process_and_save_image};
pub use batch::{batch_compress_images, collect_image_files, is_image_file, generate_output_path};
pub use info::{get_image_info, print_detailed_info};
pub use walrus::{WalrusOptions, upload_to_walrus_async, upload_to_walrus_sync};
pub use validation::{validate_input_path, validate_output_path, is_potential_image_file};