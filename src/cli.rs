use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "img-squeeze",
    about = "A fast and efficient image compression tool with Walrus storage support",
    long_about = "img-squeeze is a Rust-based image compression tool that reduces file sizes while maintaining quality. \
                  It supports multiple image formats (JPEG, PNG, WebP) with features like parallel processing, \
                  batch compression, advanced PNG optimization using oxipng, and decentralized storage via Walrus.",
    version = "0.1.0",
    after_help = "EXAMPLES:\n  \
    img-squeeze compress input.jpg output.jpg -q 85 -w 1920\n  \
    img-squeeze batch \"./images/*.jpg\" ./compressed -r -q 80 -f webp\n  \
    img-squeeze upload image.jpg -t\n  \
    img-squeeze info photo.png"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(
        about = "Compress a single image file",
        long_about = "Compress a single image file with customizable quality, size, and format options. \
                      Supports JPEG, PNG, and WebP output formats with advanced PNG optimization."
    )]
    Compress {
        #[arg(help = "Input image file path")]
        input: PathBuf,

        #[arg(help = "Output image file path")]
        output: PathBuf,

        #[arg(
            short = 'q', 
            long, 
            help = "Compression quality (1-100, default: 80)",
            long_help = "Compression quality from 1 (lowest) to 100 (highest). \
                         For PNG: >=90 uses Zopfli, >=70 uses high compression, <70 uses standard compression."
        )]
        quality: Option<u8>,

        #[arg(
            short = 'w', 
            long, 
            help = "Maximum width in pixels",
            long_help = "Resize image to maximum width while preserving aspect ratio. \
                         Image will only be resized if larger than specified width."
        )]
        width: Option<u32>,

        #[arg(
            short = 'H', 
            long, 
            help = "Maximum height in pixels", 
            long_help = "Resize image to maximum height while preserving aspect ratio. \
                         Image will only be resized if larger than specified height."
        )]
        height: Option<u32>,

        #[arg(
            short = 'f', 
            long, 
            help = "Output format (jpeg, png, webp)",
            long_help = "Force output format regardless of file extension. \
                         Supported formats: jpeg/jpg, png, webp"
        )]
        format: Option<String>,

        #[arg(
            short = 'j', 
            long, 
            help = "Number of parallel threads (default: auto)",
            long_help = "Number of threads for parallel processing. \
                         If not specified, uses number of CPU cores."
        )]
        threads: Option<usize>,
    },

    #[command(
        about = "Compress multiple images in parallel",
        long_about = "Process multiple images in parallel with batch operations. \
                      Supports directory traversal, glob patterns, and recursive processing."
    )]
    Batch {
        #[arg(
            help = "Input directory, file pattern, or glob",
            long_help = "Input can be a directory path, file pattern, or glob expression. \
                         Examples: './images', '*.jpg', '/path/to/images/*.{jpg,png}'"
        )]
        input: String,

        #[arg(help = "Output directory path")]
        output: PathBuf,

        #[arg(
            short = 'q', 
            long, 
            help = "Compression quality (1-100, default: 80)",
            long_help = "Compression quality applied to all images. \
                         Same quality rules as single compress apply."
        )]
        quality: Option<u8>,

        #[arg(
            short = 'w', 
            long, 
            help = "Maximum width in pixels",
            long_help = "Resize all images to maximum width while preserving aspect ratio."
        )]
        width: Option<u32>,

        #[arg(
            short = 'H', 
            long, 
            help = "Maximum height in pixels",
            long_help = "Resize all images to maximum height while preserving aspect ratio."
        )]
        height: Option<u32>,

        #[arg(
            short = 'f', 
            long, 
            help = "Output format (jpeg, png, webp)",
            long_help = "Convert all images to specified format. \
                         If not specified, preserves original format."
        )]
        format: Option<String>,

        #[arg(
            short = 'j', 
            long, 
            help = "Number of parallel threads (default: auto)",
            long_help = "Number of threads for parallel batch processing."
        )]
        threads: Option<usize>,

        #[arg(
            short = 'r', 
            long, 
            help = "Process subdirectories recursively",
            long_help = "Recursively process all subdirectories when input is a directory."
        )]
        recursive: bool,
    },

    #[command(
        about = "Upload an image to Walrus decentralized storage",
        long_about = "Upload images to the Walrus decentralized storage network. \
                      Supports both temporary and persistent storage with configurable endpoints."
    )]
    Upload {
        #[arg(help = "Image file path to upload")]
        input: PathBuf,

        #[arg(
            short = 'a', 
            long, 
            help = "Custom Walrus aggregator URL",
            long_help = "Override default aggregator URL. \
                         Default: https://aggregator.walrus-testnet.walrus.space"
        )]
        aggregator_url: Option<String>,

        #[arg(
            short = 'p', 
            long, 
            help = "Custom Walrus publisher URL",
            long_help = "Override default publisher URL. \
                         Default: https://publisher.walrus-testnet.walrus.space"
        )]
        publisher_url: Option<String>,

        #[arg(
            short = 'e', 
            long, 
            help = "Storage duration in epochs (default: 10)",
            long_help = "Number of epochs to store the file. Each epoch is approximately 24 hours."
        )]
        epochs: Option<u64>,

        #[arg(
            short = 't', 
            long, 
            help = "Upload as temporary file (1 epoch ≈ 24 hours)",
            long_help = "Upload file with temporary storage (1 epoch). \
                         Overrides --epochs setting. Useful for quick file sharing."
        )]
        temp: bool,
    },

    #[command(
        about = "Display comprehensive image information",
        long_about = "Analyze and display detailed information about image files including \
                      dimensions, format, file size, and compression recommendations."
    )]
    Info {
        #[arg(help = "Image file path to analyze")]
        input: PathBuf,
    },
}
