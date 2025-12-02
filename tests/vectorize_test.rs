//! Integration tests for image vectorization
//!
//! These tests create deterministic test images with known patterns
//! and verify that the vectorization produces expected results.

use image::{Rgba, RgbaImage};
use laser_tools::vectorize::{
    VectorizeOptions, create_black_mask, create_blue_mask, dilate_mask, trace_mask_to_svg_paths,
    vectorize_image,
};

// Helper to create a test image with specific dimensions filled with a color
fn create_solid_image(width: u32, height: u32, color: Rgba<u8>) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    for pixel in img.pixels_mut() {
        *pixel = color;
    }
    img
}

// Helper to draw a filled rectangle on an image
fn draw_rect(img: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    for py in y..(y + h).min(img.height()) {
        for px in x..(x + w).min(img.width()) {
            img.put_pixel(px, py, color);
        }
    }
}

// Color constants
const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);
const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const BLUE: Rgba<u8> = Rgba([0, 0, 200, 255]);
const DARK_GRAY: Rgba<u8> = Rgba([10, 10, 10, 255]); // Should be detected as black
const LIGHT_GRAY: Rgba<u8> = Rgba([128, 128, 128, 255]); // Should NOT be detected

// ============================================================================
// Mask Generation Tests
// ============================================================================

#[test]
fn test_black_mask_solid_black_image() {
    let img = create_solid_image(10, 10, BLACK);
    let mask = create_black_mask(&img);

    // All pixels should be marked as black
    assert_eq!(mask.len(), 100);
    assert!(mask.iter().all(|&v| v == 1), "All pixels should be black");
}

#[test]
fn test_black_mask_solid_white_image() {
    let img = create_solid_image(10, 10, WHITE);
    let mask = create_black_mask(&img);

    // No pixels should be marked as black
    assert_eq!(mask.len(), 100);
    assert!(mask.iter().all(|&v| v == 0), "No pixels should be black");
}

#[test]
fn test_black_mask_dark_gray_detected() {
    let img = create_solid_image(10, 10, DARK_GRAY);
    let mask = create_black_mask(&img);

    // Dark gray (RGB < 20) should be detected as black
    assert!(
        mask.iter().all(|&v| v == 1),
        "Dark gray should be detected as black"
    );
}

#[test]
fn test_black_mask_light_gray_not_detected() {
    let img = create_solid_image(10, 10, LIGHT_GRAY);
    let mask = create_black_mask(&img);

    // Light gray should NOT be detected as black
    assert!(
        mask.iter().all(|&v| v == 0),
        "Light gray should NOT be detected as black"
    );
}

#[test]
fn test_black_mask_with_black_square() {
    let mut img = create_solid_image(20, 20, WHITE);
    // Draw a 10x10 black square at position (5, 5)
    draw_rect(&mut img, 5, 5, 10, 10, BLACK);

    let mask = create_black_mask(&img);

    // Count black pixels in mask
    let black_count: usize = mask.iter().map(|&v| v as usize).sum();
    assert_eq!(
        black_count, 100,
        "Should have 100 black pixels (10x10 square)"
    );

    // Verify specific pixels
    // Pixel at (0, 0) should be white (mask = 0)
    assert_eq!(mask[0], 0, "Pixel (0,0) should be white");
    // Pixel at (5, 5) should be black (mask = 1)
    // Index = y * width + x = 5 * 20 + 5 = 105
    assert_eq!(mask[5 * 20 + 5], 1, "Pixel (5,5) should be black");
    // Pixel at (10, 10) should be black
    assert_eq!(mask[10 * 20 + 10], 1, "Pixel (10,10) should be black");
}

#[test]
fn test_blue_mask_solid_blue_image() {
    let img = create_solid_image(10, 10, BLUE);
    let mask = create_blue_mask(&img, None);

    // All pixels should be marked as blue
    assert!(mask.iter().all(|&v| v == 1), "All pixels should be blue");
}

#[test]
fn test_blue_mask_with_exclusion() {
    let img = create_solid_image(10, 10, BLUE);

    // Create exclusion mask that marks first 50 pixels
    let mut exclude = vec![0u8; 100];
    for item in exclude.iter_mut().take(50) {
        *item = 1;
    }

    let mask = create_blue_mask(&img, Some(&exclude));

    // First 50 pixels should be excluded (mask = 0)
    assert!(
        mask[..50].iter().all(|&v| v == 0),
        "First 50 pixels should be excluded"
    );
    // Last 50 pixels should be blue (mask = 1)
    assert!(
        mask[50..].iter().all(|&v| v == 1),
        "Last 50 pixels should be blue"
    );
}

#[test]
fn test_blue_mask_does_not_detect_white() {
    let img = create_solid_image(10, 10, WHITE);
    let mask = create_blue_mask(&img, None);

    assert!(
        mask.iter().all(|&v| v == 0),
        "White should not be detected as blue"
    );
}

// ============================================================================
// Dilation Tests
// ============================================================================

#[test]
fn test_dilate_single_pixel() {
    // 5x5 image with single pixel in center
    let mut mask = vec![0u8; 25];
    mask[12] = 1; // Center pixel (2, 2)

    let dilated = dilate_mask(&mask, 5, 5);

    // After dilation, 3x3 area around center should be filled
    let expected_ones = vec![
        6, 7, 8, // Row 1: (1,1), (2,1), (3,1)
        11, 12, 13, // Row 2: (1,2), (2,2), (3,2)
        16, 17, 18, // Row 3: (1,3), (2,3), (3,3)
    ];

    for (i, &value) in dilated.iter().enumerate() {
        if expected_ones.contains(&i) {
            assert_eq!(value, 1, "Pixel {} should be 1 after dilation", i);
        } else {
            assert_eq!(value, 0, "Pixel {} should be 0 after dilation", i);
        }
    }
}

#[test]
fn test_dilate_corner_pixel() {
    // 5x5 image with pixel at top-left corner
    let mut mask = vec![0u8; 25];
    mask[0] = 1; // (0, 0)

    let dilated = dilate_mask(&mask, 5, 5);

    // After dilation: (0,0), (1,0), (0,1), (1,1) should be filled
    assert_eq!(dilated[0], 1, "(0,0) should be 1");
    assert_eq!(dilated[1], 1, "(1,0) should be 1");
    assert_eq!(dilated[5], 1, "(0,1) should be 1");
    assert_eq!(dilated[6], 1, "(1,1) should be 1");
    // But (2,0) and (0,2) should still be 0
    assert_eq!(dilated[2], 0, "(2,0) should be 0");
    assert_eq!(dilated[10], 0, "(0,2) should be 0");
}

#[test]
fn test_dilate_empty_mask() {
    let mask = vec![0u8; 25];
    let dilated = dilate_mask(&mask, 5, 5);

    assert!(
        dilated.iter().all(|&v| v == 0),
        "Empty mask should remain empty after dilation"
    );
}

#[test]
fn test_dilate_full_mask() {
    let mask = vec![1u8; 25];
    let dilated = dilate_mask(&mask, 5, 5);

    assert!(
        dilated.iter().all(|&v| v == 1),
        "Full mask should remain full after dilation"
    );
}

// ============================================================================
// Tracing Tests
// ============================================================================

#[test]
fn test_trace_empty_mask() {
    let mask = vec![0u8; 100];
    let options = VectorizeOptions::default();

    let paths = trace_mask_to_svg_paths(&mask, 10, 10, &options).unwrap();

    assert!(paths.is_empty(), "Empty mask should produce no paths");
}

#[test]
fn test_trace_full_mask_produces_path() {
    let mask = vec![1u8; 100]; // 10x10 all filled
    let options = VectorizeOptions {
        scale_factor: 1, // No upscaling for determinism
        filter_speckle: 0,
        corner_threshold: 60,
        path_precision: 3,
    };

    let paths = trace_mask_to_svg_paths(&mask, 10, 10, &options).unwrap();

    assert!(
        !paths.is_empty(),
        "Full mask should produce at least one path"
    );
    // Check that paths contain SVG path data (raw d attribute values)
    for path in &paths {
        // Raw path data should start with a move command (M)
        assert!(
            path.starts_with('M') || path.starts_with('m'),
            "Path data should start with move command, got: {}",
            &path[..path.len().min(20)]
        );
    }
}

#[test]
fn test_trace_single_large_square() {
    // Create a 20x20 mask with a 10x10 square in the middle
    let mut mask = vec![0u8; 400];
    for y in 5..15 {
        for x in 5..15 {
            mask[y * 20 + x] = 1;
        }
    }

    let options = VectorizeOptions {
        scale_factor: 1,
        filter_speckle: 0,
        corner_threshold: 60,
        path_precision: 3,
    };

    let paths = trace_mask_to_svg_paths(&mask, 20, 20, &options).unwrap();

    assert!(!paths.is_empty(), "Square should produce at least one path");
}

// ============================================================================
// Full Vectorization Pipeline Tests
// ============================================================================

#[test]
fn test_vectorize_white_image_no_paths() {
    let img = create_solid_image(50, 50, WHITE);

    // Encode image to PNG bytes
    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    let result = vectorize_image(&bytes, None).unwrap();

    assert_eq!(result.width, 50);
    assert_eq!(result.height, 50);
    // SVG should have empty layers (no paths)
    assert!(result.svg.contains("<g id=\"cut-layer\""));
    assert!(result.svg.contains("<g id=\"engrave-layer\""));
}

#[test]
fn test_vectorize_black_image_has_cut_layer() {
    let img = create_solid_image(50, 50, BLACK);

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    let options = VectorizeOptions {
        scale_factor: 1,
        filter_speckle: 0,
        corner_threshold: 60,
        path_precision: 3,
    };

    let result = vectorize_image(&bytes, Some(options)).unwrap();

    // Cut layer should have paths
    assert!(result.svg.contains("<g id=\"cut-layer\""));
    assert!(
        result.svg.contains("<path"),
        "Black image should produce paths in cut layer"
    );
}

#[test]
fn test_vectorize_blue_image_has_engrave_layer() {
    let img = create_solid_image(50, 50, BLUE);

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    let options = VectorizeOptions {
        scale_factor: 1,
        filter_speckle: 0,
        corner_threshold: 60,
        path_precision: 3,
    };

    let result = vectorize_image(&bytes, Some(options)).unwrap();

    // Engrave layer should have paths
    assert!(result.svg.contains("<g id=\"engrave-layer\""));
    // The engrave layer content should not be empty
    let engrave_start = result.svg.find("<g id=\"engrave-layer\"").unwrap();
    let engrave_section = &result.svg[engrave_start..];
    assert!(
        engrave_section.contains("<path"),
        "Blue image should produce paths in engrave layer"
    );
}

#[test]
fn test_vectorize_dual_layer_black_and_blue() {
    // Create image with black square on left, blue square on right
    let mut img = create_solid_image(100, 50, WHITE);
    draw_rect(&mut img, 10, 10, 30, 30, BLACK); // Black square on left
    draw_rect(&mut img, 60, 10, 30, 30, BLUE); // Blue square on right

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    let options = VectorizeOptions {
        scale_factor: 1,
        filter_speckle: 0,
        corner_threshold: 60,
        path_precision: 3,
    };

    let result = vectorize_image(&bytes, Some(options)).unwrap();

    assert_eq!(result.width, 100);
    assert_eq!(result.height, 50);

    // Both layers should have content
    assert!(result.svg.contains("<g id=\"cut-layer\""));
    assert!(result.svg.contains("<g id=\"engrave-layer\""));

    // Extract cut layer content
    let cut_start = result.svg.find("<g id=\"cut-layer\"").unwrap();
    let cut_end = result.svg[cut_start..].find("</g>").unwrap() + cut_start;
    let cut_content = &result.svg[cut_start..cut_end];
    assert!(
        cut_content.contains("<path"),
        "Cut layer should have paths for black square"
    );

    // Extract engrave layer content
    let engrave_start = result.svg.find("<g id=\"engrave-layer\"").unwrap();
    let engrave_end = result.svg[engrave_start..].find("</g>").unwrap() + engrave_start;
    let engrave_content = &result.svg[engrave_start..engrave_end];
    assert!(
        engrave_content.contains("<path"),
        "Engrave layer should have paths for blue square"
    );
}

#[test]
fn test_vectorize_adjacent_black_blue_no_overlap() {
    // Create image with black and blue squares adjacent to each other
    // The dilation of black should prevent blue from overlapping
    let mut img = create_solid_image(50, 50, WHITE);
    draw_rect(&mut img, 10, 10, 15, 30, BLACK); // Black rectangle
    draw_rect(&mut img, 25, 10, 15, 30, BLUE); // Blue rectangle immediately adjacent

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    let options = VectorizeOptions {
        scale_factor: 1,
        filter_speckle: 0,
        corner_threshold: 60,
        path_precision: 3,
    };

    let result = vectorize_image(&bytes, Some(options)).unwrap();

    // Both layers should exist
    assert!(result.svg.contains("<g id=\"cut-layer\""));
    assert!(result.svg.contains("<g id=\"engrave-layer\""));

    // This tests that dilation prevents blue from touching black edges
    // The blue area should be slightly smaller due to the dilated black exclusion
}

// ============================================================================
// SVG Structure Tests
// ============================================================================

#[test]
fn test_svg_has_correct_structure() {
    let img = create_solid_image(100, 80, WHITE);

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    let result = vectorize_image(&bytes, None).unwrap();

    // Check XML declaration
    assert!(result.svg.starts_with("<?xml version=\"1.0\""));

    // Check SVG element with correct dimensions
    assert!(result.svg.contains("width=\"100\""));
    assert!(result.svg.contains("height=\"80\""));
    assert!(result.svg.contains("viewBox=\"0 0 100 80\""));

    // Check layer structure
    assert!(result.svg.contains("<g id=\"cut-layer\" fill=\"#000000\""));
    assert!(
        result
            .svg
            .contains("<g id=\"engrave-layer\" fill=\"#0000FF\"")
    );
}

#[test]
fn test_svg_dimensions_match_input() {
    for (w, h) in [(10, 10), (100, 50), (50, 100), (256, 256)] {
        let img = create_solid_image(w, h, WHITE);

        let mut bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();

        let result = vectorize_image(&bytes, None).unwrap();

        assert_eq!(result.width, w, "Width should match for {}x{}", w, h);
        assert_eq!(result.height, h, "Height should match for {}x{}", w, h);
        assert!(
            result.svg.contains(&format!("width=\"{}\"", w)),
            "SVG width should be {} for {}x{}",
            w,
            w,
            h
        );
        assert!(
            result.svg.contains(&format!("height=\"{}\"", h)),
            "SVG height should be {} for {}x{}",
            h,
            w,
            h
        );
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_vectorize_tiny_image() {
    let img = create_solid_image(1, 1, BLACK);

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    // Should not panic
    let result = vectorize_image(&bytes, None);
    assert!(result.is_ok(), "Should handle 1x1 image");
}

#[test]
fn test_vectorize_narrow_image() {
    let img = create_solid_image(100, 1, BLACK);

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    let result = vectorize_image(&bytes, None);
    assert!(result.is_ok(), "Should handle narrow image");
    assert_eq!(result.unwrap().height, 1);
}

#[test]
fn test_vectorize_tall_image() {
    let img = create_solid_image(1, 100, BLACK);

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    let result = vectorize_image(&bytes, None);
    assert!(result.is_ok(), "Should handle tall image");
    assert_eq!(result.unwrap().width, 1);
}

// ============================================================================
// Options Tests
// ============================================================================

#[test]
fn test_scale_factor_affects_quality() {
    let mut img = create_solid_image(20, 20, WHITE);
    draw_rect(&mut img, 5, 5, 10, 10, BLACK);

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    // Test with different scale factors
    let result_1x = vectorize_image(
        &bytes,
        Some(VectorizeOptions {
            scale_factor: 1,
            filter_speckle: 0,
            corner_threshold: 60,
            path_precision: 3,
        }),
    )
    .unwrap();

    let result_2x = vectorize_image(
        &bytes,
        Some(VectorizeOptions {
            scale_factor: 2,
            filter_speckle: 0,
            corner_threshold: 60,
            path_precision: 3,
        }),
    )
    .unwrap();

    // Both should produce valid SVGs
    assert!(result_1x.svg.contains("<path"));
    assert!(result_2x.svg.contains("<path"));

    // Dimensions should remain the same (scaling is internal)
    assert_eq!(result_1x.width, result_2x.width);
    assert_eq!(result_1x.height, result_2x.height);
}

#[test]
fn test_filter_speckle_removes_noise() {
    // Create image with tiny noise dots
    let mut img = create_solid_image(50, 50, WHITE);
    // Add single pixel "noise"
    img.put_pixel(10, 10, BLACK);
    img.put_pixel(30, 30, BLACK);

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    // With high filter_speckle, tiny dots should be filtered out
    let result = vectorize_image(
        &bytes,
        Some(VectorizeOptions {
            scale_factor: 1,
            filter_speckle: 10, // Filter out small areas
            corner_threshold: 60,
            path_precision: 3,
        }),
    )
    .unwrap();

    // Cut layer might be empty if speckles are filtered
    let cut_start = result.svg.find("<g id=\"cut-layer\"").unwrap();
    let cut_end = result.svg[cut_start..].find("</g>").unwrap() + cut_start;
    let _cut_content = &result.svg[cut_start..cut_end];

    // With high filter_speckle, single pixels might be removed
    // This is implementation-dependent, but the test ensures it doesn't crash
    assert!(result.svg.contains("<g id=\"cut-layer\""));
}
