//! Bitmap tracing using vtracer
//!
//! Converts binary masks into SVG path data using the vtracer library.

use super::{ColorMask, VectorizeOptions};
use vtracer::{ColorImage, Config, convert};

/// Bounding box for path coordinates
#[derive(Debug, Clone, Copy, Default)]
pub struct PathBounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl PathBounds {
    pub fn new() -> Self {
        Self {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }

    pub fn update(&mut self, x: f64, y: f64) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }

    pub fn merge(&mut self, other: &PathBounds) {
        if other.is_valid() {
            self.min_x = self.min_x.min(other.min_x);
            self.min_y = self.min_y.min(other.min_y);
            self.max_x = self.max_x.max(other.max_x);
            self.max_y = self.max_y.max(other.max_y);
        }
    }

    pub fn is_valid(&self) -> bool {
        self.min_x.is_finite() && self.min_y.is_finite()
    }
}

/// Trace a binary mask to SVG path data strings
/// Returns raw path data (d attribute values), not wrapped in <path> elements
pub fn trace_mask_to_svg_paths(
    mask: &ColorMask,
    width: u32,
    height: u32,
    options: &VectorizeOptions,
) -> Result<Vec<String>, String> {
    // Check if mask is empty (no pixels set)
    if !mask.contains(&1) {
        return Ok(Vec::new());
    }

    // Scale up the mask for better tracing quality
    let scaled_width = width * options.scale_factor;
    let scaled_height = height * options.scale_factor;

    // Create scaled grayscale image from mask
    let mut scaled_pixels = vec![0u8; (scaled_width * scaled_height * 4) as usize];

    for y in 0..scaled_height {
        for x in 0..scaled_width {
            let src_x = x / options.scale_factor;
            let src_y = y / options.scale_factor;
            let src_idx = (src_y * width + src_x) as usize;
            let dst_idx = ((y * scaled_width + x) * 4) as usize;

            // If mask pixel is set, make it black (for tracing)
            // Otherwise make it white (background)
            if mask[src_idx] == 1 {
                scaled_pixels[dst_idx] = 0; // R
                scaled_pixels[dst_idx + 1] = 0; // G
                scaled_pixels[dst_idx + 2] = 0; // B
                scaled_pixels[dst_idx + 3] = 255; // A
            } else {
                scaled_pixels[dst_idx] = 255; // R
                scaled_pixels[dst_idx + 1] = 255; // G
                scaled_pixels[dst_idx + 2] = 255; // B
                scaled_pixels[dst_idx + 3] = 255; // A
            }
        }
    }

    // Create ColorImage for vtracer
    let color_image = ColorImage {
        pixels: scaled_pixels,
        width: scaled_width as usize,
        height: scaled_height as usize,
    };

    // Configure vtracer
    let config = Config {
        filter_speckle: options.filter_speckle * options.scale_factor as usize,
        corner_threshold: options.corner_threshold,
        color_precision: 8,    // Binary image
        layer_difference: 128, // Binary threshold
        path_precision: Some(options.path_precision),
        ..Default::default()
    };

    // Convert to SVG
    let svg_file =
        convert(color_image, config).map_err(|e| format!("vtracer conversion failed: {}", e))?;

    // Extract and scale path data (returns raw d attribute strings)
    let paths = extract_and_scale_paths(&svg_file.to_string(), options.scale_factor);

    Ok(paths)
}

/// Extract path d attributes from SVG and scale coordinates back to original size
/// Only extracts paths with dark fill colors (not white background)
/// Returns raw path data strings (not wrapped in <path> elements)
/// Also applies any transform="translate(x,y)" from the path element
fn extract_and_scale_paths(svg_content: &str, scale_factor: u32) -> Vec<String> {
    let scale = scale_factor as f64;
    let mut scaled_paths: Vec<String> = Vec::new();

    for line in svg_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<path") && trimmed.contains(" d=\"") {
            // Skip white/light colored paths (background)
            if is_white_or_light_fill(trimmed) {
                continue;
            }

            // Extract the d attribute
            if let Some(d_start) = trimmed.find(" d=\"") {
                let d_content_start = d_start + 4;
                if let Some(d_end) = trimmed[d_content_start..].find('"') {
                    let d_attr = &trimmed[d_content_start..d_content_start + d_end];

                    // Scale the path data
                    let scaled_d = scale_path_data(d_attr, scale);

                    // Extract and apply transform="translate(x,y)" if present
                    // vtracer outputs shapes with their position as a transform
                    let final_d = if let Some((tx, ty)) = extract_translate_transform(trimmed) {
                        // Scale the transform values too
                        let scaled_tx = tx / scale;
                        let scaled_ty = ty / scale;
                        translate_path_data(&scaled_d, scaled_tx, scaled_ty)
                    } else {
                        scaled_d
                    };

                    scaled_paths.push(final_d);
                }
            }
        }
    }

    scaled_paths
}

/// Extract translate(x,y) values from a transform attribute
fn extract_translate_transform(path_element: &str) -> Option<(f64, f64)> {
    // Look for transform="translate(x,y)"
    let transform_start = path_element.find("transform=\"translate(")?;
    let values_start = transform_start + 21; // length of 'transform="translate('
    let values_end = path_element[values_start..].find(')')?;
    let values_str = &path_element[values_start..values_start + values_end];

    // Parse "x,y" or "x y"
    let parts: Vec<&str> = values_str.split([',', ' ']).collect();
    if parts.len() >= 2 {
        let x = parts[0].trim().parse::<f64>().ok()?;
        let y = parts[1].trim().parse::<f64>().ok()?;
        Some((x, y))
    } else {
        None
    }
}

/// Calculate combined bounds for a list of path data strings
pub fn calculate_paths_bounds(paths: &[String]) -> PathBounds {
    let mut combined = PathBounds::new();
    for path_d in paths {
        let bounds = calculate_path_bounds(path_d);
        combined.merge(&bounds);
    }
    combined
}

/// Translate a list of path data strings and wrap them in <path> elements
pub fn translate_and_wrap_paths(paths: &[String], offset_x: f64, offset_y: f64) -> Vec<String> {
    paths
        .iter()
        .map(|path_d| {
            let translated_d = if offset_x != 0.0 || offset_y != 0.0 {
                translate_path_data(path_d, offset_x, offset_y)
            } else {
                path_d.clone()
            };
            format!("<path d=\"{}\"/>", translated_d)
        })
        .collect()
}

/// Check if a path element has a white or light fill color (background)
fn is_white_or_light_fill(path_element: &str) -> bool {
    // Check for fill="rgb(R,G,B)" format
    if let Some(fill_start) = path_element.find("fill=\"rgb(") {
        let rgb_start = fill_start + 10;
        if let Some(rgb_end) = path_element[rgb_start..].find(')') {
            let rgb_str = &path_element[rgb_start..rgb_start + rgb_end];
            let parts: Vec<&str> = rgb_str.split(',').collect();
            if parts.len() == 3
                && let (Ok(r), Ok(g), Ok(b)) = (
                    parts[0].trim().parse::<u8>(),
                    parts[1].trim().parse::<u8>(),
                    parts[2].trim().parse::<u8>(),
                )
            {
                // Consider it "white/light" if all channels are > 200
                return r > 200 && g > 200 && b > 200;
            }
        }
    }

    // Check for fill="#RRGGBB" or fill="#RGB" format
    if let Some(fill_start) = path_element.find("fill=\"#") {
        let hex_start = fill_start + 7;
        if let Some(hex_end) = path_element[hex_start..].find('"') {
            let hex_str = &path_element[hex_start..hex_start + hex_end];
            if let Some((r, g, b)) = parse_hex_color(hex_str) {
                return r > 200 && g > 200 && b > 200;
            }
        }
    }

    // Check for fill="white"
    if path_element.contains("fill=\"white\"") {
        return true;
    }

    false
}

/// Parse a hex color string to RGB values
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim();
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some((r, g, b))
        }
        _ => None,
    }
}

/// Scale path data coordinates by dividing by scale factor
fn scale_path_data(d: &str, scale: f64) -> String {
    transform_path_data(d, |n| n / scale)
}

/// Translate path data coordinates by adding offsets
fn translate_path_data(d: &str, offset_x: f64, offset_y: f64) -> String {
    let mut result = String::new();
    let mut chars = d.chars().peekable();
    let mut is_x = true; // Track whether next number is X or Y coordinate

    while let Some(c) = chars.next() {
        if c.is_alphabetic() {
            result.push(c);
            // Reset coordinate tracking based on command
            // Most commands alternate X,Y pairs
            is_x = true;
        } else if c == '-' || c == '.' || c.is_ascii_digit() {
            // Parse number
            let mut num_str = String::new();
            num_str.push(c);

            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit()
                    || next == '.'
                    || next == 'e'
                    || next == 'E'
                    || (next == '-' && num_str.ends_with(['e', 'E']))
                {
                    num_str.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            // Translate the number
            if let Ok(num) = num_str.parse::<f64>() {
                let offset = if is_x { offset_x } else { offset_y };
                let translated = num + offset;
                result.push_str(&format!("{:.3}", translated));
            } else {
                result.push_str(&num_str);
            }
            is_x = !is_x; // Alternate between X and Y
        } else if c == ',' || c.is_whitespace() {
            result.push(c);
        }
    }

    result
}

/// Generic path data transformation
fn transform_path_data<F>(d: &str, transform: F) -> String
where
    F: Fn(f64) -> f64,
{
    let mut result = String::new();
    let mut chars = d.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_alphabetic() {
            result.push(c);
        } else if c == '-' || c == '.' || c.is_ascii_digit() {
            // Parse number
            let mut num_str = String::new();
            num_str.push(c);

            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit()
                    || next == '.'
                    || next == 'e'
                    || next == 'E'
                    || (next == '-' && num_str.ends_with(['e', 'E']))
                {
                    num_str.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            // Transform the number
            if let Ok(num) = num_str.parse::<f64>() {
                let transformed = transform(num);
                result.push_str(&format!("{:.3}", transformed));
            } else {
                result.push_str(&num_str);
            }
        } else if c == ',' || c.is_whitespace() {
            result.push(c);
        }
    }

    result
}

/// Calculate bounding box from path data
fn calculate_path_bounds(d: &str) -> PathBounds {
    let mut bounds = PathBounds::new();
    let mut chars = d.chars().peekable();
    let mut is_x = true;
    let mut current_x = 0.0;

    while let Some(c) = chars.next() {
        if c.is_alphabetic() {
            is_x = true;
        } else if c == '-' || c == '.' || c.is_ascii_digit() {
            // Parse number
            let mut num_str = String::new();
            num_str.push(c);

            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit()
                    || next == '.'
                    || next == 'e'
                    || next == 'E'
                    || (next == '-' && num_str.ends_with(['e', 'E']))
                {
                    num_str.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            if let Ok(num) = num_str.parse::<f64>() {
                if is_x {
                    current_x = num;
                } else {
                    bounds.update(current_x, num);
                }
                is_x = !is_x;
            }
        }
    }

    bounds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_path_data() {
        let d = "M100,200 L300,400";
        let scaled = scale_path_data(d, 2.0);
        assert_eq!(scaled, "M50.000,100.000 L150.000,200.000");
    }

    #[test]
    fn test_empty_mask() {
        let mask = vec![0u8; 100];
        let options = VectorizeOptions::default();
        let result = trace_mask_to_svg_paths(&mask, 10, 10, &options).unwrap();
        assert!(result.is_empty());
    }
}
