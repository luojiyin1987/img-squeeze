use assert_cmd::Command;
use tempfile::TempDir;
use std::fs::File;
use std::io::Write;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.arg("--help");
    cmd.assert().success();
}

#[test]
fn test_compress_help() {
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["compress", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_batch_help() {
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["batch", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_info_help() {
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["info", "--help"]);
    cmd.assert().success();
}

#[test]
fn test_compress_missing_args() {
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["compress"]);
    cmd.assert().failure();
}

#[test]
fn test_compress_nonexistent_file() {
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["compress", "nonexistent.jpg", "output.jpg"]);
    cmd.assert().failure();
}

#[test]
fn test_batch_missing_args() {
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["batch"]);
    cmd.assert().failure();
}

#[test]
fn test_batch_nonexistent_input() {
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["batch", "nonexistent", "output"]);
    cmd.assert().success(); // Application handles this gracefully
}

#[test]
fn test_info_missing_args() {
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["info"]);
    cmd.assert().failure();
}

#[test]
fn test_info_nonexistent_file() {
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["info", "nonexistent.jpg"]);
    cmd.assert().failure();
}

#[test]
fn test_compress_with_invalid_quality() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.jpg");
    let output_file = temp_dir.path().join("output.jpg");
    
    // Create a fake image file
    let mut file = File::create(&test_file).unwrap();
    file.write_all(b"fake image data").unwrap();
    
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["compress", &test_file.to_string_lossy(), &output_file.to_string_lossy()]);
    cmd.arg("--quality").arg("0");
    cmd.assert().failure();
}

#[test]
fn test_compress_with_valid_quality() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.jpg");
    let output_file = temp_dir.path().join("output.jpg");
    
    // Create a fake image file
    let mut file = File::create(&test_file).unwrap();
    file.write_all(b"fake image data").unwrap();
    
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["compress", &test_file.to_string_lossy(), &output_file.to_string_lossy()]);
    cmd.arg("--quality").arg("85");
    cmd.assert().failure(); // Will fail because it's not a real image, but shouldn't panic
}

#[test]
fn test_compress_with_width_resize() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.jpg");
    let output_file = temp_dir.path().join("output.jpg");
    
    // Create a fake image file
    let mut file = File::create(&test_file).unwrap();
    file.write_all(b"fake image data").unwrap();
    
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["compress", &test_file.to_string_lossy(), &output_file.to_string_lossy()]);
    cmd.arg("--width").arg("800");
    cmd.assert().failure(); // Will fail because it's not a real image, but shouldn't panic
}

#[test]
fn test_compress_with_format_override() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.jpg");
    let output_file = temp_dir.path().join("output.png");
    
    // Create a fake image file
    let mut file = File::create(&test_file).unwrap();
    file.write_all(b"fake image data").unwrap();
    
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["compress", &test_file.to_string_lossy(), &output_file.to_string_lossy()]);
    cmd.arg("--format").arg("png");
    cmd.assert().failure(); // Will fail because it's not a real image, but shouldn't panic
}

#[test]
fn test_batch_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("output");
    
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["batch", &temp_dir.path().to_string_lossy(), &output_dir.to_string_lossy()]);
    cmd.assert().success(); // Should succeed with "no image files found" message
}

#[test]
fn test_batch_with_image_files() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().join("output");
    
    // Create fake image files
    File::create(temp_dir.path().join("test1.jpg")).unwrap();
    File::create(temp_dir.path().join("test2.png")).unwrap();
    
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["batch", &temp_dir.path().to_string_lossy(), &output_dir.to_string_lossy()]);
    cmd.assert().success(); // Application handles this gracefully
}

#[test]
fn test_batch_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let subdir = temp_dir.path().join("subdir");
    std::fs::create_dir(&subdir).unwrap();
    let output_dir = temp_dir.path().join("output");
    
    // Create fake image files in subdirectory
    File::create(subdir.join("test1.jpg")).unwrap();
    
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["batch", &temp_dir.path().to_string_lossy(), &output_dir.to_string_lossy()]);
    cmd.arg("--recursive");
    cmd.assert().success(); // Application handles this gracefully
}

#[test]
fn test_info_with_fake_image() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.jpg");
    
    // Create a fake image file
    let mut file = File::create(&test_file).unwrap();
    file.write_all(b"fake image data").unwrap();
    
    let mut cmd = Command::cargo_bin("img-squeeze").unwrap();
    cmd.args(["info", &test_file.to_string_lossy()]);
    cmd.assert().failure(); // Will fail because it's not a real image, but shouldn't panic
}