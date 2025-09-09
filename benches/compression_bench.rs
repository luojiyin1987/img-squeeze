use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use img_squeeze::processing::{
    load_image_with_metadata, process_and_save_image, resize_image, CompressionOptions,
};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_image(width: u32, height: u32) -> (PathBuf, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.jpg");

    // Create a simple RGB image buffer
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&[0xFF, 0xD8, 0xFF, 0xE0]); // JPEG header
    buffer.extend_from_slice(&[0x00, 0x10]); // Length
    buffer.extend_from_slice(b"JFIF"); // Signature
    buffer.extend_from_slice(&[0x00]); // Null terminator
    buffer.extend_from_slice(&[0x01, 0x01]); // Version
    buffer.extend_from_slice(&[0x00]); // Units (pixels)
    buffer.extend_from_slice(&[0x00, 0x48]); // X density (72)
    buffer.extend_from_slice(&[0x00, 0x48]); // Y density (72)
    buffer.extend_from_slice(&[0x00, 0x00]); // X thumbnail
    buffer.extend_from_slice(&[0x00, 0x00]); // Y thumbnail

    // Fill with fake image data
    for _ in 0..(width * height * 3) {
        buffer.push(0xFF);
    }

    let mut file = File::create(&test_file).unwrap();
    file.write_all(&buffer).unwrap();

    (test_file, temp_dir)
}

fn bench_compression_options_creation(c: &mut Criterion) {
    c.bench_function("compression_options_creation", |b| {
        b.iter(|| {
            CompressionOptions::new(
                black_box(Some(85)),
                black_box(Some(800)),
                black_box(Some(600)),
                black_box(Some("webp".to_string())),
                false,
                false,
            )
        })
    });
}

fn bench_image_loading(c: &mut Criterion) {
    let (test_file, _temp_dir) = create_test_image(1920, 1080);

    c.bench_function("image_loading", |b| {
        b.iter(|| load_image_with_metadata(black_box(&test_file)))
    });
}

fn bench_image_resizing(c: &mut Criterion) {
    let mut group = c.benchmark_group("image_resizing");

    for size in [Small, Medium, Large].iter() {
        let (width, height) = match size {
            Small => (800, 600),
            Medium => (1920, 1080),
            Large => (3840, 2160),
        };

        let (test_file, _temp_dir) = create_test_image(width, height);

        if let Ok((img, _)) = load_image_with_metadata(&test_file) {
            let options =
                CompressionOptions::new(Some(80), Some(width / 2), Some(height / 2), None, false, false).unwrap();

            group.bench_with_input(
                BenchmarkId::new("resize", format!("{}x{}", width, height)),
                &(img, options),
                |b, (img, options)| {
                    b.iter(|| {
                        let mut img = img.clone();
                        resize_image(black_box(&mut img), black_box(options));
                    })
                },
            );
        }
    }

    group.finish();
}

fn bench_image_processing(c: &mut Criterion) {
    let (test_file, _temp_dir) = create_test_image(1920, 1080);
    let output_dir = TempDir::new().unwrap();
    let output_file = output_dir.path().join("output.jpg");

    if let Ok((img, _)) = load_image_with_metadata(&test_file) {
        let options = CompressionOptions::new(Some(80), None, None, None, false, false).unwrap();

        c.bench_function("image_processing", |b| {
            b.iter(|| {
                process_and_save_image(
                    black_box(&img),
                    black_box(&output_file),
                    black_box(&options),
                )
            })
        });
    }
}

fn bench_batch_processing(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = TempDir::new().unwrap();

    // Create multiple test files
    for i in 0..10 {
        let test_file = temp_dir.path().join(format!("test_{}.jpg", i));
        let (file, _temp) = create_test_image(800, 600);
        std::fs::copy(&file, &test_file).unwrap();
    }

    c.bench_function("batch_processing", |b| {
        b.iter(|| {
            // This is a simplified benchmark - in reality, you'd benchmark the actual batch_compress_images function
            let files: Vec<_> = std::fs::read_dir(temp_dir.path())
                .unwrap()
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .collect();

            for file in files {
                if let Ok((img, _)) = load_image_with_metadata(&file) {
                    let options = CompressionOptions::new(Some(80), None, None, None, false, false).unwrap();
                    let output_file = output_dir.path().join(file.file_name().unwrap());
                    let _ = process_and_save_image(&img, &output_file, &options);
                }
            }
        })
    });
}

enum ImageSize {
    Small,
    Medium,
    Large,
}

use ImageSize::*;

criterion_group!(
    benches,
    bench_compression_options_creation,
    bench_image_loading,
    bench_image_resizing,
    bench_image_processing,
    bench_batch_processing
);
criterion_main!(benches);
