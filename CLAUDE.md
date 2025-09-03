# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**img-squeeze** is a Rust-based image compression tool designed to reduce image file sizes while maintaining quality. The project is currently in early development stages with a basic "Hello, world!" binary structure.

## Development Commands

### Building and Running

- `cargo build` - Build the project
- `cargo run` - Run the application
- `cargo check` - Check for compilation errors
- `cargo clippy` - Run linter (when added)
- `cargo fmt` - Format code

### Testing

- `cargo test` - Run all tests
- `cargo test <test_name>` - Run specific test

## Architecture

### Current Structure

- `src/main.rs` - Main application entry point (currently minimal)
- `Cargo.toml` - Project configuration and dependencies

### Project Layout

```
img-squeeze/
├── src/
│   └── main.rs          # Application entry point
├── Cargo.toml           # Project configuration
├── LICENSE              # MIT License
└── .gitignore           # Git ignore rules
```

## Development Guidelines

### Dependencies

Currently has no external dependencies. When adding image processing capabilities, consider:

- `image` crate for basic image operations
- `imageproc` for advanced image processing
- `clap` for command-line interface
- `criterion` for performance benchmarking

### License

MIT License - free and open source. Ensure all contributions comply with MIT terms.

### Build Targets

- Debug builds: `cargo build` (development)
- Release builds: `cargo build --release` (production)
