use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "img-squeeze",
    about = "A Rust-based image compression tool",
    version = "0.1.0"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Compress an image")]
    Compress {
        #[arg(help = "Input image file")]
        input: PathBuf,

        #[arg(help = "Output image file")]
        output: PathBuf,

        #[arg(short = 'q', long, help = "Quality (1-100), default is 80")]
        quality: Option<u8>,

        #[arg(short = 'w', long, help = "Maximum width in pixels")]
        width: Option<u32>,

        #[arg(short = 'H', long, help = "Maximum height in pixels")]
        height: Option<u32>,

        #[arg(
            short = 'f',
            long,
            value_parser = ["jpeg","jpg","png","webp","avif"],
            value_name = "FORMAT",
            help = "Output format (jpeg, jpg, png, webp, avif). Note: heic/heif/jxl are recognized as inputs only."
        )]
        format: Option<String>,

        #[arg(short = 'j', long, help = "Number of parallel threads (default: auto)")]
        threads: Option<usize>,
    },

    #[command(about = "Compress multiple images in parallel")]
    Batch {
        #[arg(help = "Input directory or file pattern")]
        input: String,

        #[arg(help = "Output directory")]
        output: PathBuf,

        #[arg(short = 'q', long, help = "Quality (1-100), default is 80")]
        quality: Option<u8>,

        #[arg(short = 'w', long, help = "Maximum width in pixels")]
        width: Option<u32>,

        #[arg(short = 'H', long, help = "Maximum height in pixels")]
        height: Option<u32>,

        #[arg(
            short = 'f',
            long,
            value_parser = ["jpeg","jpg","png","webp","avif"],
            value_name = "FORMAT",
            help = "Output format (jpeg, jpg, png, webp, avif). Note: heic/heif/jxl are recognized as inputs only."
        )]
        format: Option<String>,

        #[arg(short = 'j', long, help = "Number of parallel threads (default: auto)")]
        threads: Option<usize>,

        #[arg(short = 'r', long, help = "Recursive directory processing")]
        recursive: bool,
    },

    #[command(about = "Upload an image to Walrus storage")]
    Upload {
        #[arg(help = "Image file to upload")]
        input: PathBuf,

        #[arg(short = 'a', long, help = "Walrus aggregator URL")]
        aggregator_url: Option<String>,

        #[arg(short = 'p', long, help = "Walrus publisher URL")]
        publisher_url: Option<String>,

        #[arg(short = 'e', long, help = "Number of epochs for storage")]
        epochs: Option<u64>,

        #[arg(short = 't', long, help = "Upload as temporary file (1 epoch storage)")]
        temp: bool,
    },

    #[command(about = "Get information about an image")]
    Info {
        #[arg(help = "Image file to analyze")]
        input: PathBuf,
    },
}
