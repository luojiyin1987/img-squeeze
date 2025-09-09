use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub fn create_test_image_files(temp_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();

    // Create test image files
    let jpg_file = temp_dir.join("test.jpg");
    let png_file = temp_dir.join("test.png");
    let webp_file = temp_dir.join("test.webp");
    let txt_file = temp_dir.join("test.txt");

    File::create(&jpg_file)
        .unwrap()
        .write_all(b"fake jpg data")
        .unwrap();
    File::create(&png_file)
        .unwrap()
        .write_all(b"fake png data")
        .unwrap();
    File::create(&webp_file)
        .unwrap()
        .write_all(b"fake webp data")
        .unwrap();
    File::create(&txt_file)
        .unwrap()
        .write_all(b"not an image")
        .unwrap();

    files.push(jpg_file);
    files.push(png_file);
    files.push(webp_file);
    files.push(txt_file);

    files
}

pub fn create_nested_directory_structure(temp_dir: &Path) -> PathBuf {
    let subdir = temp_dir.join("subdir");
    std::fs::create_dir(&subdir).unwrap();

    // Create files in subdirectory
    File::create(subdir.join("nested.jpg"))
        .unwrap()
        .write_all(b"nested image")
        .unwrap();
    File::create(subdir.join("nested.txt"))
        .unwrap()
        .write_all(b"nested text")
        .unwrap();

    subdir
}

pub fn create_temp_directory() -> TempDir {
    TempDir::new().unwrap()
}

pub fn create_test_output_directory(temp_dir: &Path) -> PathBuf {
    let output_dir = temp_dir.join("output");
    std::fs::create_dir(&output_dir).unwrap();
    output_dir
}
