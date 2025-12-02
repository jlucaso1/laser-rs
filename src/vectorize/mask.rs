//! Color mask generation for image vectorization
//!
//! Creates binary masks for different colors that will be converted
//! to separate laser cutting/engraving layers.

use image::{Rgba, RgbaImage};

/// A binary mask representing pixels that match a color criteria
pub type ColorMask = Vec<u8>;

/// Create a black mask from an RGBA image
/// Pixels with R < 20, G < 20, B < 20 are considered black
pub fn create_black_mask(img: &RgbaImage) -> ColorMask {
    let (width, height) = img.dimensions();
    let pixel_count = (width * height) as usize;
    let mut mask = vec![0u8; pixel_count];

    for (i, pixel) in img.pixels().enumerate() {
        let Rgba([r, g, b, _]) = *pixel;
        if r < 20 && g < 20 && b < 20 {
            mask[i] = 1;
        }
    }

    mask
}

/// Create a blue mask from an RGBA image
/// Blue pixels: B > 150 && R < 140 && G < 140
/// Also includes very dark pixels: R < 80 && G < 80 && B < 80
/// Optionally excludes pixels already in an exclusion mask
pub fn create_blue_mask(img: &RgbaImage, exclude: Option<&ColorMask>) -> ColorMask {
    let (width, height) = img.dimensions();
    let pixel_count = (width * height) as usize;
    let mut mask = vec![0u8; pixel_count];

    for (i, pixel) in img.pixels().enumerate() {
        // Skip if excluded
        if let Some(ex) = exclude
            && ex[i] == 1
        {
            continue;
        }

        let Rgba([r, g, b, _]) = *pixel;

        // Blue detection: bright blue or very dark pixels
        let is_blue = (b > 150 && r < 140 && g < 140) || (r < 80 && g < 80 && b < 80);

        if is_blue {
            mask[i] = 1;
        }
    }

    mask
}

/// Dilate a binary mask by 1 pixel using a 3x3 kernel
/// This expands all marked regions by 1 pixel in each direction
pub fn dilate_mask(mask: &ColorMask, width: u32, height: u32) -> ColorMask {
    let w = width as usize;
    let h = height as usize;
    let mut dilated = vec![0u8; mask.len()];

    for y in 0..h {
        for x in 0..w {
            // Check 3x3 neighborhood
            let mut found = false;
            for oy in -1i32..=1 {
                for ox in -1i32..=1 {
                    let ny = y as i32 + oy;
                    let nx = x as i32 + ox;

                    if ny >= 0 && ny < h as i32 && nx >= 0 && nx < w as i32 {
                        let idx = ny as usize * w + nx as usize;
                        if mask[idx] == 1 {
                            found = true;
                            break;
                        }
                    }
                }
                if found {
                    break;
                }
            }

            if found {
                dilated[y * w + x] = 1;
            }
        }
    }

    dilated
}

/// Create a custom color mask with a predicate function
#[allow(dead_code)]
pub fn create_custom_mask<F>(
    img: &RgbaImage,
    predicate: F,
    exclude: Option<&ColorMask>,
) -> ColorMask
where
    F: Fn(u8, u8, u8) -> bool,
{
    let (width, height) = img.dimensions();
    let pixel_count = (width * height) as usize;
    let mut mask = vec![0u8; pixel_count];

    for (i, pixel) in img.pixels().enumerate() {
        // Skip if excluded
        if let Some(ex) = exclude
            && ex[i] == 1
        {
            continue;
        }

        let Rgba([r, g, b, _]) = *pixel;
        if predicate(r, g, b) {
            mask[i] = 1;
        }
    }

    mask
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn test_black_mask() {
        let mut img = RgbaImage::new(3, 1);
        img.put_pixel(0, 0, Rgba([0, 0, 0, 255])); // black
        img.put_pixel(1, 0, Rgba([255, 255, 255, 255])); // white
        img.put_pixel(2, 0, Rgba([10, 10, 10, 255])); // dark

        let mask = create_black_mask(&img);
        assert_eq!(mask, vec![1, 0, 1]);
    }

    #[test]
    fn test_blue_mask() {
        let mut img = RgbaImage::new(3, 1);
        img.put_pixel(0, 0, Rgba([0, 0, 200, 255])); // blue
        img.put_pixel(1, 0, Rgba([255, 255, 255, 255])); // white
        img.put_pixel(2, 0, Rgba([50, 50, 50, 255])); // dark (should match)

        let mask = create_blue_mask(&img, None);
        assert_eq!(mask, vec![1, 0, 1]);
    }

    #[test]
    fn test_blue_mask_with_exclusion() {
        let mut img = RgbaImage::new(3, 1);
        img.put_pixel(0, 0, Rgba([0, 0, 200, 255])); // blue
        img.put_pixel(1, 0, Rgba([0, 0, 200, 255])); // blue (will be excluded)
        img.put_pixel(2, 0, Rgba([255, 255, 255, 255])); // white

        let exclude = vec![0, 1, 0]; // exclude middle pixel
        let mask = create_blue_mask(&img, Some(&exclude));
        assert_eq!(mask, vec![1, 0, 0]);
    }

    #[test]
    fn test_dilate_mask() {
        // 3x3 image with single pixel in center
        let mask = vec![0, 0, 0, 0, 1, 0, 0, 0, 0];
        let dilated = dilate_mask(&mask, 3, 3);
        // All pixels should be 1 after dilation
        assert_eq!(dilated, vec![1, 1, 1, 1, 1, 1, 1, 1, 1]);
    }
}
