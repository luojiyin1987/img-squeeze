use proptest::prelude::*;
use img_squeeze::processing::{CompressionOptions, determine_output_format, resize_image};
use img_squeeze::batch::is_image_file;
use image::{DynamicImage, ImageFormat, GenericImageView};
use std::path::Path;

proptest! {
    #[test]
    fn compression_options_quality_in_range(quality in 1u8..=100u8) {
        let options = CompressionOptions::new(Some(quality), None, None, None, false);
        assert!(options.is_ok());
    }

    #[test]
    fn compression_options_invalid_quality(quality in 0u8..200u8) {
        // Test invalid quality values (0 and > 100)
        let result = CompressionOptions::new(Some(quality), None, None, None, false);
        if quality == 0 || quality > 100 {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn resize_image_width_only(
        width in 100u32..=1000u32,
        height in 100u32..=1000u32,
        new_width in 100u32..=1000u32
    ) {
        prop_assume!(width > 0 && height > 0 && new_width > 0);
        prop_assume!(new_width != width); // Only test if resize is needed

        let mut img = DynamicImage::new_rgb8(width, height);
        let options = CompressionOptions::new(Some(80), Some(new_width), None, None, false).unwrap();
        
        resize_image(&mut img, &options);
        
        let (resized_w, resized_h) = img.dimensions();
        
        // With aspect ratio preserved, width should be set and height should be proportional
        assert_eq!(resized_w, new_width);
        // Height should be proportional to maintain aspect ratio
        let expected_height = (height as f64 * new_width as f64 / width as f64).round() as u32;
        assert_eq!(resized_h, expected_height);
    }

    #[test]
    fn resize_image_height_only(
        width in 100u32..=1000u32,
        height in 100u32..=1000u32,
        new_height in 100u32..=1000u32
    ) {
        prop_assume!(width > 0 && height > 0 && new_height > 0);
        prop_assume!(new_height != height); // Only test if resize is needed

        let mut img = DynamicImage::new_rgb8(width, height);
        let options = CompressionOptions::new(Some(80), None, Some(new_height), None, false).unwrap();
        
        resize_image(&mut img, &options);
        
        let (resized_w, resized_h) = img.dimensions();
        
        // With aspect ratio preserved, height should be set and width should be proportional
        assert_eq!(resized_h, new_height);
        // Width should be proportional to maintain aspect ratio
        let expected_width = (width as f64 * new_height as f64 / height as f64).round() as u32;
        assert_eq!(resized_w, expected_width);
    }

    #[test]
    fn determine_output_format_property(
        filename in "[a-zA-Z0-9_-]+\\.[a-zA-Z]{3,4}",
        format_override in prop::option::weighted(0.3, "[a-zA-Z]{3,4}")
    ) {
        let path = Path::new(&filename);
        let format_opt = format_override.as_ref().map(|s| s.as_str());
        
        let result = determine_output_format(path, &format_opt.map(|s| s.to_string()));
        
        // Should always return a valid format or a proper error
        match result {
            Ok(format) => {
                // Should be one of the supported formats
                assert!(matches!(format, ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP));
            }
            Err(_) => {
                // Error is acceptable for unsupported formats
            }
        }
    }

    #[test]
    fn is_image_file_recognizes_extensions(
        extension in prop::sample::select(&["jpg", "jpeg", "png", "webp", "bmp", "tiff", "gif", "txt", "doc", "pdf"])
    ) {
        let filename = format!("test.{}", extension);
        let path = Path::new(&filename);
        
        let is_image = is_image_file(path);
        
        // Check that known image extensions are recognized
        let expected = matches!(extension.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tiff" | "gif");
        assert_eq!(is_image, expected);
    }

    #[test]
    fn compression_options_creation_properties(
        quality in prop::option::weighted(0.8, 1u8..=100u8),
        width in prop::option::weighted(0.5, 1u32..=10000u32),
        height in prop::option::weighted(0.5, 1u32..=10000u32),
        format in prop::option::weighted(0.3, prop::sample::select(&["jpg", "png", "webp"]))
    ) {
        let format_str = format.map(|s| s.to_string());
        let result = CompressionOptions::new(quality, width, height, format_str.clone(), false);
        
        match result {
            Ok(options) => {
                // Check that valid options are set correctly
                assert_eq!(options.quality, quality.unwrap_or(80));
                assert_eq!(options.width, width);
                assert_eq!(options.height, height);
                assert_eq!(options.format, format_str);
                assert_eq!(options.exact_resize, false);
            }
            Err(_) => {
                // Only error should be invalid quality
                assert!(quality.map_or(false, |q| q == 0 || q > 100));
            }
        }
    }

    #[test]
    fn resize_image_no_change_when_same_dimensions(
        width in 100u32..=5000u32,
        height in 100u32..=5000u32
    ) {
        prop_assume!(width > 0 && height > 0);

        let mut img = DynamicImage::new_rgb8(width, height);
        let options = CompressionOptions::new(Some(80), Some(width), Some(height), None, false).unwrap();
        
        resize_image(&mut img, &options);
        
        // Dimensions should remain unchanged
        assert_eq!(img.dimensions(), (width, height));
    }

    #[test]
    fn determine_output_format_fallback_to_jpeg(
        filename in "[a-zA-Z0-9_-]+\\.(unknown|xyz|abc|def)"
    ) {
        let path = Path::new(&filename);
        let result = determine_output_format(path, &None);
        
        // Should fallback to JPEG for unknown extensions
        assert_eq!(result.unwrap(), ImageFormat::Jpeg);
    }
}