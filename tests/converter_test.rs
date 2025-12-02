use laser_tools::lbrn2::{lbrn2_to_svg, parse_lbrn2};
use std::fs;
use std::path::Path;

/// Normalize numeric values in a string for comparison
fn normalize_numbers(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '-' || chars[i].is_ascii_digit() {
            // Parse number
            let mut num_str = String::new();
            if chars[i] == '-' {
                num_str.push(chars[i]);
                i += 1;
            }
            while i < chars.len()
                && (chars[i].is_ascii_digit()
                    || chars[i] == '.'
                    || chars[i] == 'e'
                    || chars[i] == 'E'
                    || (chars[i] == '-' && i > 0 && (chars[i - 1] == 'e' || chars[i - 1] == 'E')))
            {
                num_str.push(chars[i]);
                i += 1;
            }
            if let Ok(n) = num_str.parse::<f64>() {
                // Format to 3 decimal places and remove trailing zeros
                let formatted = format!("{:.3}", n);
                let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
                result.push_str(trimmed);
            } else {
                result.push_str(&num_str);
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

/// Normalize SVG for comparison - strip whitespace, normalize numbers
fn normalize_svg(svg: &str) -> String {
    // Remove XML declaration
    let svg = svg.trim();
    let svg = if svg.starts_with("<?xml") {
        if let Some(pos) = svg.find("?>") {
            svg[pos + 2..].trim()
        } else {
            svg
        }
    } else {
        svg
    };

    // Normalize whitespace
    let svg = svg.replace(['\n', '\r'], " ");
    let svg = svg.split_whitespace().collect::<Vec<_>>().join(" ");

    // Remove stroke-width from comparison (varies between versions)
    let svg = remove_stroke_width(&svg);

    // Normalize path d attribute - remove spaces around commands
    let svg = normalize_path_data(&svg);

    // Normalize numbers
    normalize_numbers(&svg)
}

/// Remove stroke-width attribute for comparison
fn remove_stroke_width(svg: &str) -> String {
    // Simple regex-like replacement for stroke-width
    let mut result = svg.to_string();
    while let Some(start) = result.find("stroke-width:") {
        if let Some(end) = result[start..].find(';') {
            result = format!("{}{}", &result[..start], &result[start + end + 1..]);
        } else {
            break;
        }
    }
    result
}

/// Normalize path d attribute - remove spaces around SVG path commands
fn normalize_path_data(svg: &str) -> String {
    let mut result = svg.to_string();

    // Find d=" and normalize the path data within
    let mut i = 0;
    while i < result.len() {
        if let Some(pos) = result[i..].find("d=\"") {
            let start = i + pos + 3;
            if let Some(end) = result[start..].find('"') {
                let path_data = &result[start..start + end];
                let normalized = path_data
                    .replace(" L", "L")
                    .replace(" M", "M")
                    .replace(" C", "C")
                    .replace(" Z", "Z")
                    .replace("L ", "L")
                    .replace("M ", "M")
                    .replace("C ", "C");
                result = format!(
                    "{}{}{}",
                    &result[..start],
                    normalized,
                    &result[start + end..]
                );
                i = start + normalized.len();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    result
}

/// Compare two SVGs structurally
fn svg_equal(a: &str, b: &str) -> bool {
    let a_norm = normalize_svg(a);
    let b_norm = normalize_svg(b);

    // For debugging - print normalized forms if different
    if a_norm != b_norm {
        eprintln!("Normalized SVG A:\n{}", a_norm);
        eprintln!("\nNormalized SVG B:\n{}", b_norm);
    }

    a_norm == b_norm
}

fn run_conversion_test(name: &str) {
    let artifacts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/artifacts");
    let lbrn2_path = artifacts_dir.join(format!("{}.lbrn2", name));
    let expected_svg_path = artifacts_dir.join(format!("{}.svg", name));

    let lbrn2_content =
        fs::read_to_string(&lbrn2_path).unwrap_or_else(|_| panic!("Failed to read {}.lbrn2", name));
    let expected_svg = fs::read_to_string(&expected_svg_path)
        .unwrap_or_else(|_| panic!("Failed to read {}.svg", name));

    let project =
        parse_lbrn2(&lbrn2_content).unwrap_or_else(|_| panic!("Failed to parse {}.lbrn2", name));
    let generated_svg = lbrn2_to_svg(&project);

    // Save generated SVG for debugging
    let temp_dir = artifacts_dir.join("temp");
    let _ = fs::create_dir_all(&temp_dir);
    let _ = fs::write(temp_dir.join(format!("{}.svg", name)), &generated_svg);

    assert!(
        svg_equal(&generated_svg, &expected_svg),
        "SVG mismatch for {}",
        name
    );
}

#[test]
fn test_circle() {
    run_conversion_test("circle");
}

#[test]
fn test_square() {
    run_conversion_test("square");
}

#[test]
fn test_line() {
    run_conversion_test("line");
}

#[test]
fn test_ellipse_stretched() {
    run_conversion_test("ellipse_stretched");
}

#[test]
fn test_bezier_missing_cp() {
    run_conversion_test("bezier_missing_cp");
}

#[test]
fn test_group_empty() {
    run_conversion_test("group_empty");
}

#[test]
fn test_group_single_child() {
    run_conversion_test("group_single_child");
}

#[test]
fn test_image() {
    run_conversion_test("image");
}

#[test]
fn test_butterfly_vectorized() {
    run_conversion_test("butterfly_vectorized");
}

#[test]
fn test_word() {
    run_conversion_test("word");
}

#[test]
fn test_crucifix() {
    run_conversion_test("crucifix");
}

#[test]
fn test_k() {
    run_conversion_test("k");
}

#[test]
fn test_rings() {
    run_conversion_test("rings");
}
