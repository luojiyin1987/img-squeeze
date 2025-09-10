# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**img-squeeze** is a Rust-based image compression tool that reduces file sizes while maintaining quality. It supports multiple image formats (JPEG, PNG, WebP, BMP, TIFF, GIF) with features like parallel processing, batch compression, and advanced PNG optimization using oxipng.

## Claude Code + Copilot PR工作流

这个项目集成了通用的Claude Code + Copilot PR审查工作流，提供智能化的代码质量保障。

### 🚀 工作流特性

- **项目类型自动检测**: 自动识别Rust项目并应用相应的分析工具
- **智能分析**: 使用cargo clippy、cargo fmt、cargo audit等Rust专用工具
- **Copilot集成**: 自动调用GitHub Copilot进行代码审查
- **质量门禁**: 包含代码质量、安全性、测试覆盖率等检查
- **自动化报告**: 生成详细的审查报告和改进建议

### 📋 使用方法

#### 自动设置（推荐）
```bash
# 运行项目适配脚本
./scripts/claude-workflow-adapter.sh
```

#### 手动运行
```bash
# 运行Claude Code分析
./scripts/run-claude-analysis.sh

# 手动触发Copilot审查
./scripts/trigger-copilot-review.sh <PR_NUMBER>
```

### 🔧 生成的文件

- **claude-workflow.yml**: 通用工作流配置
- **.claude-workflow/config.yml**: Rust项目特定配置
- **.github/workflows/claude-copilot-review.yml**: GitHub Actions工作流
- **scripts/run-claude-analysis.sh**: Rust分析执行脚本
- **docs/claude-workflow-setup.md**: 详细设置文档

### 🤖 Copilot集成

工作流会自动在PR评论中调用`@copilot`，重点关注：
- Rust所有权和借用检查
- 并发安全性
- 错误处理模式
- 性能优化
- 内存管理
- 零成本抽象

### 📊 分析工具

- **cargo clippy**: Rust代码检查和最佳实践
- **cargo fmt**: 代码格式化
- **cargo audit**: 安全漏洞扫描
- **cargo test**: 单元测试执行
- **cargo outdated**: 依赖更新检查

### 🎯 质量门禁

- 代码质量：无clippy警告
- 安全性：无已知安全漏洞
- 测试：所有单元测试通过
- 文档：代码文档完整性检查

### 📈 自动化触发

- PR创建时自动运行分析
- PR更新时重新执行
- 自动生成审查报告
- 智能调用Copilot审查

## Development Commands

### Building and Running

- `cargo build` - Build the project (development)
- `cargo build --release` - Build optimized release version
- `cargo run` - Run the application
- `cargo check` - Check for compilation errors
- `cargo clippy` - Run linter for code quality checks
- `cargo fmt` - Format code according to Rust standards

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