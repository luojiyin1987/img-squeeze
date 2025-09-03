mod cli;
mod error;
mod processing;
mod batch;
mod info;

use clap::Parser;
use cli::{Args, Commands};
use error::Result;
use processing::compress_image;
use batch::batch_compress_images;
use info::{get_image_info, print_detailed_info};
use rayon::ThreadPoolBuilder;
use std::path::Path;

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Commands::Compress { input, output, quality, width, height, format, threads } => {
            setup_thread_pool(threads);
            compress_image(input, output, quality, width, height, format)?;
        }
        Commands::Batch { input, output, quality, width, height, format, threads, recursive } => {
            setup_thread_pool(threads);
            batch_compress_images(input, output, quality, width, height, format, recursive)?;
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

fn show_image_info(input_path: &Path) -> Result<()> {
    println!("📋 Getting info for: {:?}", input_path);
    
    if !input_path.exists() {
        return Err(error::CompressionError::FileNotFound(input_path.to_path_buf()));
    }
    
    // 基本图片信息
    get_image_info(input_path)?;
    
    // 详细信息（可选）
    print_detailed_info(input_path)?;
    
    Ok(())
}