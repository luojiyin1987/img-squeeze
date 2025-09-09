mod batch;
mod cli;
mod error;
mod info;
mod processing;
mod walrus;

use batch::batch_compress_images;
use clap::Parser;
use cli::{Args, Commands};
use error::Result;
use info::{get_image_info, print_detailed_info};
use processing::{compress_image, CompressionOptions};
use rayon::ThreadPoolBuilder;
use std::path::Path;
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

fn upload_image_to_walrus(
    input_path: &Path,
    aggregator_url: Option<String>,
    publisher_url: Option<String>,
    epochs: Option<u64>,
    temp: bool,
) -> Result<()> {
    println!("📤 Uploading to Walrus: {:?}", input_path);

    validate_file_exists(input_path)?;

    // 处理临时文件选项
    let final_epochs = if temp {
        Some(1) // 临时文件只存储 1 个 epoch
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

    // 构建访问地址
    let access_url = build_walrus_access_url(&options.aggregator_url, &blob_id);
    println!("🌐 Access URL: {}", access_url);

    // 临时文件提示
    if temp {
        println!("⏰ Temporary file: Will expire after 1 epoch (~24 hours)");
        println!("🔄 Use without -t flag for longer storage");
    }

    // 显示文件信息
    if let Ok(metadata) = std::fs::metadata(input_path) {
        println!("📊 File size: {} bytes", metadata.len());
    }

    println!("💡 You can use the blob ID to retrieve the file later");

    Ok(())
}

fn build_walrus_access_url(aggregator_url: &str, blob_id: &str) -> String {
    // 构建 Walrus 网络的访问地址
    // 通常格式是 {aggregator_url}/v1/blobs/{blob_id}
    if aggregator_url.ends_with('/') {
        format!("{}v1/blobs/{}", aggregator_url, blob_id)
    } else {
        format!("{}/v1/blobs/{}", aggregator_url, blob_id)
    }
}

fn show_image_info(input_path: &Path) -> Result<()> {
    println!("📋 Getting info for: {:?}", input_path);

    validate_file_exists(input_path)?;

    // 基本图片信息
    get_image_info(input_path)?;

    // 详细信息（可选）
    print_detailed_info(input_path)?;

    Ok(())
}
