use crate::error::{CompressionError, Result};
use std::path::{Path, PathBuf};
use std::fs;

/// Maximum file size in bytes (100MB)
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Validate input file path for security and accessibility
pub fn validate_input_path(path: &Path) -> Result<()> {
    // Check if file exists
    if !path.exists() {
        return Err(CompressionError::FileNotFound(path.to_path_buf()));
    }
    
    // Check if it's a file (not a directory)
    if !path.is_file() {
        return Err(CompressionError::UnsupportedFormat(
            "Input path is not a file".to_string()
        ));
    }
    
    // Check file size
    let metadata = fs::metadata(path)
        .map_err(|_| CompressionError::FileNotFound(path.to_path_buf()))?;
    
    if metadata.len() > MAX_FILE_SIZE {
        return Err(CompressionError::UnsupportedFormat(
            format!("File size ({} bytes) exceeds maximum allowed size ({} bytes)", 
                    metadata.len(), MAX_FILE_SIZE)
        ));
    }
    
    // Basic path traversal protection
    let canonical_path = path.canonicalize()
        .map_err(|_| CompressionError::FileNotFound(path.to_path_buf()))?;
    
    // Check for suspicious path components
    for component in canonical_path.components() {
        if let std::path::Component::Normal(name) = component {
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') && name_str.len() > 1 && name_str.chars().nth(1) == Some('.') {
                return Err(CompressionError::UnsupportedFormat(
                    "Suspicious path component detected".to_string()
                ));
            }
        }
    }
    
    Ok(())
}

/// Validate output directory path and create if it doesn't exist
pub fn validate_output_path(path: &Path) -> Result<PathBuf> {
    let canonical_output = if let Some(parent) = path.parent() {
        // Create parent directories if they don't exist
        fs::create_dir_all(parent)
            .map_err(|_| CompressionError::DirectoryCreationFailed(parent.to_path_buf()))?;
        
        // Canonicalize the parent directory
        let canonical_parent = parent.canonicalize()
            .map_err(|_| CompressionError::DirectoryCreationFailed(parent.to_path_buf()))?;
        
        // Combine with the filename
        if let Some(filename) = path.file_name() {
            canonical_parent.join(filename)
        } else {
            return Err(CompressionError::UnsupportedFormat(
                "Invalid output filename".to_string()
            ));
        }
    } else {
        // No parent directory, use current directory
        std::env::current_dir()
            .map_err(|_| CompressionError::DirectoryCreationFailed(PathBuf::from(".")))?
            .join(path)
    };
    
    // Basic path traversal protection for output
    for component in canonical_output.components() {
        if let std::path::Component::Normal(name) = component {
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') && name_str.len() > 1 && name_str.chars().nth(1) == Some('.') {
                return Err(CompressionError::UnsupportedFormat(
                    "Suspicious output path component detected".to_string()
                ));
            }
        }
    }
    
    Ok(canonical_output)
}

/// Check if the file extension indicates it might be an image
pub fn is_potential_image_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            matches!(ext_str.to_lowercase().as_str(), 
                "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tiff" | "tif" | "gif" | "avif" | "heic")
        } else {
            false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_validate_input_path_not_found() {
        let path = Path::new("nonexistent.jpg");
        let result = validate_input_path(path);
        assert!(matches!(result, Err(CompressionError::FileNotFound(_))));
    }

    #[test]
    fn test_is_potential_image_file() {
        assert!(is_potential_image_file(Path::new("test.jpg")));
        assert!(is_potential_image_file(Path::new("test.PNG")));
        assert!(is_potential_image_file(Path::new("test.webp")));
        assert!(!is_potential_image_file(Path::new("test.txt")));
        assert!(!is_potential_image_file(Path::new("test")));
    }

    #[test]
    fn test_validate_input_path_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.jpg");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"fake image data").unwrap();

        let result = validate_input_path(&test_file);
        assert!(result.is_ok());
    }
}