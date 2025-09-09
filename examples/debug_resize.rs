use image::{DynamicImage, ImageFormat};
use img_squeeze::processing::CompressionOptions;

fn main() {
    let mut img = DynamicImage::new_rgb8(100, 100);
    println!("Original dimensions: {:?}", img.dimensions());
    
    let options = CompressionOptions::new(Some(80), Some(200), None, None, false, false).unwrap();
    img_squeeze::processing::resize_image(&mut img, &options);
    println!("After width resize: {:?}", img.dimensions());
}
