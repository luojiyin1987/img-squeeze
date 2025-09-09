#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;

    #[test]
    fn debug_resize_issue() {
        let mut img = DynamicImage::new_rgb8(100, 100);
        println!("Original dimensions: {:?}", img.dimensions());
        
        let options = CompressionOptions::new(Some(80), Some(101), None, None).unwrap();
        println!("Options: width={:?}, height={:?}", options.width, options.height);
        
        resize_image(&mut img, &options);
        println!("Final dimensions: {:?}", img.dimensions());
        
        // This should pass
        assert_eq!(img.dimensions(), (101, 100));
    }
}