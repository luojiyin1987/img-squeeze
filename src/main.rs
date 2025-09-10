mod batch;
mod cli;
mod constants;
mod error;
mod info;
mod processing;
mod upload;
mod walrus;

use batch::batch_compress_images;
use clap::Parser;
use cli::{Args, Commands};
use error::Result;
use info::{get_image_info, print_detailed_info};
use processing::{compress_image, CompressionOptions};
use rayon::ThreadPoolBuilder;
use std::path::Path;
use upload::upload_image_to_walrus;

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

/// Sets up the global thread pool for parallel processing
///
/// # Arguments
/// * `threads` - Optional number of threads to use. If None, uses default (CPU count)
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

/// Displays information about an image file
///
/// # Arguments
/// * `input_path` - Path to the image file to analyze
///
/// # Returns
/// * `Ok(())` if analysis succeeds
/// * `Err(CompressionError)` if file cannot be read or analyzed
fn show_image_info(input_path: &Path) -> Result<()> {
    println!("📋 Getting info for: {:?}", input_path);

    // 基本图片信息
    get_image_info(input_path)?;

    // 详细信息（可选）
    print_detailed_info(input_path)?;

    Ok(())
}
