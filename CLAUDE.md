# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**img-squeeze** is a Rust-based image compression tool that reduces file sizes while maintaining quality. It supports multiple image formats (JPEG, PNG, WebP, BMP, TIFF, GIF) with features like parallel processing, batch compression, and advanced PNG optimization using oxipng.

## Claude 工作流

本项目集成了简化的 Claude 工作流系统，用于自动化代码质量检查和构建流程。

### 🚀 工作流特性

- **简单实用**: 轻量级配置，易于维护和扩展
- **Rust 优化**: 针对 Rust 项目的专用工具链
- **自动化**: 一键执行完整的开发和测试流程
- **错误处理**: 遇到错误自动停止，确保质量

### 📋 使用方法

#### 运行完整工作流
```bash
# 执行完整的 Claude 工作流
./claude-workflow.sh
```

#### 手动执行各个阶段
```bash
# 代码检查
cargo check
cargo clippy
cargo fmt --check

# 运行测试
cargo test --lib

# 构建项目
cargo build --release

# 性能验证
./target/release/img-squeeze --help
./target/release/img-squeeze --version
```

### 🔧 工作流文件

- **.claude-workflow.yml**: YAML 工作流配置文件
- **claude-workflow.sh**: Bash 执行脚本

### 📊 工作流阶段

1. **代码检查** - 使用 cargo check、clippy、fmt 检查代码质量
2. **运行测试** - 执行 35 个单元测试确保功能正确
3. **构建项目** - 构建优化版本用于生产环境
4. **性能验证** - 验证构建结果和基本功能

### 🎯 质量标准

- 代码编译无错误
- 通过所有 clippy 检查
- 代码格式符合 Rust 标准
- 所有单元测试通过
- 构建的二进制文件正常工作

## Development Commands

### Building and Running

- `cargo build` - Build the project (development)
- `cargo build --release` - Build optimized release version
- `cargo run` - Run the application
- `cargo check` - Check for compilation errors
- `cargo clippy` - Run linter for code quality checks
- `cargo fmt` - Format code according to Rust standards

### Claude Workflow

- `./claude-workflow.sh` - Run complete workflow with all stages
- `.claude-workflow.yml` - Workflow configuration file

### Testing

- `cargo test` - Run all tests (unit, integration, and property tests)
- `cargo test <test_name>` - Run specific test
- `cargo test --lib` - Run only library unit tests
- `cargo test --test <test_file>` - Run specific test file (e.g., `cargo test --test integration_tests`)
- `cargo bench` - Run performance benchmarks
- `cargo test property_tests` - Run property-based tests

## Architecture

### Core Modules

- `main.rs` - Application entry point with command routing and thread pool setup
- `cli.rs` - Command-line interface definition using clap with four subcommands:
  - `compress` - Single image compression with quality, size, and format options
  - `batch` - Batch processing with directory traversal and glob patterns
  - `upload` - Upload images to Walrus decentralized storage network
  - `info` - Image analysis and compression suggestions
- `processing.rs` - Core image processing logic:
  - `CompressionOptions` struct for configuration
  - `compress_image()` for single file processing
  - `save_image()` with format-specific optimizations
  - PNG optimization using oxipng with Zopfli/libdeflater
- `batch.rs` - Parallel batch processing:
  - Rayon-based parallel processing
  - Directory traversal and glob pattern support
  - Progress tracking and performance statistics
- `info.rs` - Image analysis and metadata extraction
- `walrus.rs` - Walrus decentralized storage integration:
  - `WalrusClient` integration for blockchain-based storage
  - `WalrusOptions` for configuring aggregator/publisher URLs and epochs
  - Async upload functionality with proper error handling
- `error.rs` - Comprehensive error handling with thiserror

### Key Dependencies

- `image` - Core image processing and format support
- `oxipng` - Advanced PNG compression with Zopfli optimization
- `clap` - Command-line argument parsing
- `rayon` - Parallel processing for batch operations
- `indicatif` - Progress bars and user feedback
- `anyhow` and `thiserror` - Error handling
- `walkdir` and `glob` - File system traversal
- `walrus_rs` - Walrus decentralized storage client library
- `tokio` - Async runtime for Walrus operations

### Project Layout

```text
img-squeeze/
├── src/
│   ├── main.rs          # Application entry point
│   ├── cli.rs           # Command-line interface
│   ├── processing.rs   # Core compression logic
│   ├── batch.rs         # Batch processing
│   ├── info.rs          # Image analysis
│   ├── walrus.rs        # Walrus storage integration
│   └── error.rs         # Error types
├── Cargo.toml           # Project configuration
├── .claude-workflow.yml # Claude workflow configuration
├── claude-workflow.sh   # Claude workflow execution script
└── target/              # Build artifacts
```

## Development Guidelines

### Image Processing Pipeline

1. **Input validation** - Check file existence and format support
2. **Image loading** - Use `ImageReader::open()` and `decode()`
3. **Resize processing** - Apply Lanczos3 filtering if dimensions specified
4. **Format-specific optimization**:
   - PNG: Use oxipng with quality-based compression levels
   - JPEG: Standard quality-based compression
   - WebP: Native WebP encoding
5. **Output saving** - Create directories and save with statistics

### Thread Management

- Main thread handles CLI and setup
- Rayon thread pool for parallel batch processing
- Configurable thread count with auto-detection fallback
- Thread-safe progress tracking using `Arc<AtomicUsize>`

### Error Handling

- Use `CompressionError` enum for all error cases
- Propagate errors with `?` operator
- Provide user-friendly error messages with context
- Handle file I/O, image processing, and validation errors separately

### Performance Considerations

- Batch processing uses Rayon for parallel execution
- Memory-efficient image processing with streaming where possible
- Progress bars for user feedback during long operations
- Detailed performance statistics for batch operations

## Common Usage Patterns

### Single Image Compression

```bash
img-squeeze compress input.jpg output.jpg -q 85 -w 1920 -H 1080 -j 4
```

### Batch Processing

```bash
img-squeeze batch "./images/*.jpg" ./compressed -r -q 80 -f webp
```

### Image Analysis

```bash
img-squeeze info image.jpg
```

### Walrus Upload

```bash
# Upload with default settings
img-squeeze upload image.jpg

# Upload with custom aggregator and publisher
img-squeeze upload image.jpg -a https://aggregator.walrus-testnet.walrus.space -p https://publisher.walrus-testnet.walrus.space

# Upload with custom epochs
img-squeeze upload image.jpg -e 20
```

**Upload Output:**
The upload command provides comprehensive feedback including:
- Upload progress and status
- Network endpoints used
- Blob ID for future reference
- Direct access URL for the uploaded file
- File size and storage information

Example output:
```
📤 Uploading to Walrus: "image.jpg"
🔗 Aggregator URL: https://aggregator.walrus-testnet.walrus.space
🔗 Publisher URL: https://publisher.walrus-testnet.walrus.space
⏰ Epochs: Some(10)
✅ Upload successful!
🆔 Blob ID: 3xAm...V7n9
🌐 Access URL: https://aggregator.walrus-testnet.walrus.space/v1/blobs/3xAm...V7n9
📊 File size: 1024 bytes
💡 You can use the blob ID to retrieve the file later
```

## Advanced Architecture Details

### Compression Flow

1. **CLI Parsing** (`main.rs`) → Routes to appropriate command handler
2. **Thread Pool Setup** (`main.rs:38-47`) → Configures Rayon for parallel processing
3. **Options Validation** (`processing.rs:17-40`) → Creates `CompressionOptions` with validation
4. **Image Processing** (`processing.rs:42-103`) → Core compression logic with progress tracking
5. **Format Handling** (`processing.rs:105-121`) → Determines output format based on extension or explicit option
6. **Format-Specific Saving** (`processing.rs:123-175`) → Specialized saving logic per format

### PNG Optimization Strategy

The tool uses sophisticated PNG optimization through oxipng:

- **Quality-based compression levels**: Higher quality (90+) uses Zopfli with 15 iterations
- **Medium quality (70-89)**: Uses libdeflater with compression level 12
- **Lower quality (<70)**: Uses libdeflater with compression level 8
- **Temporary file handling**: Creates temporary PNG files that are optimized then cleaned up

### Batch Processing Architecture

1. **File Collection** (`batch.rs:99-138`) → Traverses directories or expands glob patterns
2. **Parallel Processing** (`batch.rs:48-66`) → Uses Rayon's `par_iter()` for concurrent processing
3. **Progress Tracking** (`batch.rs:44-46`) → Thread-safe atomic counters for statistics
4. **Error Handling** → Individual file failures don't stop batch processing
5. **Performance Reporting** (`batch.rs:70-96`) → Comprehensive timing and compression statistics

### Error Handling Pattern

The project uses a centralized error handling approach:

- `CompressionError` enum covers all error scenarios
- Errors are propagated using `?` operator throughout the call stack
- User-friendly error messages with context (file paths, operation details)
- Separate error categories for I/O, image processing, validation, and optimization

### Walrus Storage Integration

The tool integrates with the Walrus decentralized storage network for blockchain-based image storage:

- **Real API Integration**: Uses `walrus_rs` library (v0.1.2) for actual network operations
- **Configurable Endpoints**: Supports custom aggregator and publisher URLs
- **Epoch Management**: Configurable storage duration through epochs parameter
- **Async Operations**: Full async/await support for non-blocking uploads
- **Error Handling**: Comprehensive error handling for network and storage failures

**Default Configuration:**
- **Aggregator URL**: `https://aggregator.walrus-testnet.walrus.space`
- **Publisher URL**: `https://publisher.walrus-testnet.walrus.space`
- **Epochs**: 10 (configurable)

**Upload Process:**
1. **File Validation** - Check file existence and readability
2. **Client Creation** - Initialize `WalrusClient` with configured URLs
3. **Data Reading** - Read file content into memory
4. **Blob Storage** - Use `client.store_blob()` to upload to Walrus network
5. **Result Handling** - Extract and return blob ID from storage result

## Thread Safety Considerations

- **Atomic counters**: `Arc<AtomicUsize>` for thread-safe progress tracking
- **Immutable options**: `CompressionOptions` is cloned for each parallel task
- **File operations**: Each thread processes independent files
- **Progress bars**: Main thread handles UI updates, workers update atomic counters

## Memory Management

- **Efficient image loading**: Uses `image` crate's streaming where possible
- **Temporary file cleanup**: PNG optimization creates and removes temporary files
- **Batch processing**: Processes files sequentially to avoid memory exhaustion
- **Format conversion**: Handles in-memory format conversion before saving

## License

MIT License - ensure all contributions comply with MIT terms.