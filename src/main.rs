mod batch;
mod cli;
mod constants;
mod error;
mod formats;
mod info;
mod processing;
mod utils;
mod walrus;

use batch::batch_compress_images;
use clap::Parser;
use cli::{Args, Commands};
use constants::WALRUS_TEMP_EPOCHS;
use error::Result;
use info::{get_image_info, print_detailed_info};
use processing::{compress_image, CompressionOptions};
use rayon::ThreadPoolBuilder;
use std::path::Path;
use utils::{build_walrus_access_url, validate_file_exists};
use walrus::{upload_to_walrus_sync, WalrusOptions};

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Compress {
            input,
            output,
            quality,
            width,
            height,
            format,
            threads,
        } => {
            setup_thread_pool(threads);
            let options = CompressionOptions::new(quality, width, height, format)?;
            compress_image(input, output, options)?;
        }
        Commands::Batch {
            input,
            output,
            quality,
            width,
            height,
            format,
            threads,
            recursive,
        } => {
            setup_thread_pool(threads);
            let options = CompressionOptions::new(quality, width, height, format)?;
            batch_compress_images(input, output, options, recursive)?;
        }
        Commands::Upload {
            input,
            aggregator_url,
            publisher_url,
            epochs,
            temp,
        } => {
            upload_image_to_walrus(&input, aggregator_url, publisher_url, epochs, temp)?;
        }
        Commands::Info { input } => {
            show_image_info(&input)?;
        }
    }

    Ok(())
}

fn setup_thread_pool(threads: Option<usize>) {
    if let Some(num_threads) = threads {
        ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to set thread pool size: {}", e);
            });
    }
}

/// Upload an image to Walrus decentralized storage
/// 
/// # Arguments
/// * `input_path` - Path to the image file to upload
/// * `aggregator_url` - Optional custom aggregator URL
/// * `publisher_url` - Optional custom publisher URL  
/// * `epochs` - Optional number of epochs for storage
/// * `temp` - Whether to upload as a temporary file (1 epoch)
/// 
/// # Returns
/// * `Ok(())` on successful upload, `Err(CompressionError)` on failure
fn upload_image_to_walrus(
    input_path: &Path,
    aggregator_url: Option<String>,
    publisher_url: Option<String>,
    epochs: Option<u64>,
    temp: bool,
) -> Result<()> {
    println!("📤 Uploading to Walrus: {:?}", input_path);

    // Validate file exists with helpful error message
    validate_file_exists(input_path)?;

    // Handle temporary file option
    let final_epochs = if temp {
        Some(WALRUS_TEMP_EPOCHS)
    } else {
        epochs
    };

    let options = WalrusOptions::new(aggregator_url, publisher_url, final_epochs);

    println!("🔗 Aggregator URL: {}", options.aggregator_url);
    println!("🔗 Publisher URL: {}", options.publisher_url);
    println!("⏰ Epochs: {:?}", options.epochs);

    let blob_id = upload_to_walrus_sync(input_path, &options)?;

    println!("✅ Upload successful!");
    println!("🆔 Blob ID: {}", blob_id);

    // Build and display access URL
    let access_url = build_walrus_access_url(&options.aggregator_url, &blob_id);
    println!("🌐 Access URL: {}", access_url);

    // Show temporary file warning
    if temp {
        println!("⏰ Temporary file: Will expire after 1 epoch (~24 hours)");
        println!("🔄 Use without -t flag for longer storage");
    }

    // Display file information
    if let Ok(metadata) = std::fs::metadata(input_path) {
        println!("📊 File size: {} bytes", metadata.len());
    }

    println!("💡 You can use the blob ID to retrieve the file later");

    Ok(())
}

/// Remove the old build_walrus_access_url function since it's now in utils

/// Display comprehensive information about an image file
/// 
/// # Arguments
/// * `input_path` - Path to the image file to analyze
/// 
/// # Returns
/// * `Ok(())` on success, `Err(CompressionError)` if file not found or invalid
fn show_image_info(input_path: &Path) -> Result<()> {
    println!("📋 Getting info for: {:?}", input_path);

    // Validate file exists with helpful error message
    validate_file_exists(input_path)?;

    // Get basic image information
    get_image_info(input_path)?;

    // Display detailed information
    print_detailed_info(input_path)?;

    Ok(())
}
