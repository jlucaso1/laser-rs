//! Image vectorization module
//!
//! This module provides functionality to convert raster images (PNG, JPEG)
//! into vector SVG format suitable for laser cutting/engraving operations.
//!
//! The conversion process:
//! 1. Load image and extract metadata
//! 2. Create color masks (black for cutting, blue for engraving)
//! 3. Apply morphological dilation to prevent artifacts
//! 4. Trace bitmap masks to vector paths using vtracer
//! 5. Assemble final SVG with separate layers

mod mask;
mod trace;

use image::{DynamicImage, GenericImageView, ImageReader};
use std::io::Cursor;

pub use mask::{ColorMask, create_black_mask, create_blue_mask, dilate_mask};
pub use trace::{
    PathBounds, calculate_paths_bounds, trace_mask_to_svg_paths, translate_and_wrap_paths,
};

/// Options for image vectorization
#[derive(Debug, Clone)]
pub struct VectorizeOptions {
    /// Scale factor for upsampling before tracing (default: 2)
    pub scale_factor: u32,
    /// Filter speckle size (removes noise smaller than this)
    pub filter_speckle: usize,
    /// Corner threshold for path simplification
    pub corner_threshold: i32,
    /// Path precision (decimal places)
    pub path_precision: u32,
}

impl Default for VectorizeOptions {
    fn default() -> Self {
        Self {
            scale_factor: 2,
            filter_speckle: 4,
            corner_threshold: 60,
            path_precision: 3,
        }
    }
}

/// Result of vectorization containing SVG string
pub struct VectorizeResult {
    pub svg: String,
    pub width: u32,
    pub height: u32,
}

/// Vectorize an image from bytes into SVG with cut and engrave layers
pub fn vectorize_image(
    image_bytes: &[u8],
    options: Option<VectorizeOptions>,
) -> Result<VectorizeResult, String> {
    let options = options.unwrap_or_default();

    // Load image
    let img = ImageReader::new(Cursor::new(image_bytes))
        .with_guessed_format()
        .map_err(|e| format!("Failed to guess image format: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    vectorize_dynamic_image(&img, &options)
}

/// Vectorize a DynamicImage into SVG
pub fn vectorize_dynamic_image(
    img: &DynamicImage,
    options: &VectorizeOptions,
) -> Result<VectorizeResult, String> {
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();

    // Create black mask (for cutting) - pixels with RGB < 20
    let black_mask = create_black_mask(&rgba);

    // Dilate black mask by 1 pixel to prevent artifacts at edges
    let dilated_black = dilate_mask(&black_mask, width, height);

    // Create blue mask (for engraving) - excludes pixels already in dilated black mask
    let blue_mask = create_blue_mask(&rgba, Some(&dilated_black));

    // Trace masks to SVG path data (raw d attributes, not wrapped)
    let black_path_data = trace_mask_to_svg_paths(&black_mask, width, height, options)?;
    let blue_path_data = trace_mask_to_svg_paths(&blue_mask, width, height, options)?;

    // Calculate combined bounds across BOTH layers to preserve relative positions
    let mut combined_bounds = PathBounds::new();
    combined_bounds.merge(&calculate_paths_bounds(&black_path_data));
    combined_bounds.merge(&calculate_paths_bounds(&blue_path_data));

    // Calculate translation offset (same for both layers)
    let (offset_x, offset_y) = if combined_bounds.is_valid() {
        (
            if combined_bounds.min_x < 0.0 {
                -combined_bounds.min_x
            } else {
                0.0
            },
            if combined_bounds.min_y < 0.0 {
                -combined_bounds.min_y
            } else {
                0.0
            },
        )
    } else {
        (0.0, 0.0)
    };

    // Apply the same translation to both layers and wrap in <path> elements
    let black_paths = translate_and_wrap_paths(&black_path_data, offset_x, offset_y);
    let blue_paths = translate_and_wrap_paths(&blue_path_data, offset_x, offset_y);

    // Assemble final SVG
    let svg = assemble_svg(width, height, &black_paths, &blue_paths);

    Ok(VectorizeResult { svg, width, height })
}

/// Vectorize an image file into SVG
pub fn vectorize_image_file(
    path: &str,
    options: Option<VectorizeOptions>,
) -> Result<VectorizeResult, String> {
    let img = ImageReader::open(path)
        .map_err(|e| format!("Failed to open image file: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let options = options.unwrap_or_default();
    vectorize_dynamic_image(&img, &options)
}

/// Assemble final SVG with cut and engrave layers
fn assemble_svg(width: u32, height: u32, black_paths: &[String], blue_paths: &[String]) -> String {
    let black_content = black_paths.join("\n        ");
    let blue_content = blue_paths.join("\n        ");

    format!(
        r##"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
    <g id="cut-layer" fill="#000000" stroke="none">
        {black_content}
    </g>
    <g id="engrave-layer" fill="#0000FF" stroke="none">
        {blue_content}
    </g>
</svg>"##
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = VectorizeOptions::default();
        assert_eq!(opts.scale_factor, 2);
        assert_eq!(opts.filter_speckle, 4);
    }
}
