use crate::bounds::get_transformed_bounds;
use crate::path::generate_path_data;
use crate::style::get_cut_setting_style;
use crate::types::{CutSetting, LightBurnProject, Shape, XForm};

/// Format a number with 6 decimal places, treating -0 as 0
fn f(n: f64) -> String {
    // Handle -0.0 case
    let n = if n == 0.0 { 0.0 } else { n };
    format!("{:.6}", n)
}

/// Format the transformation matrix for SVG (with Y-axis flip)
fn format_matrix(xform: &XForm) -> String {
    format!(
        "matrix({} {} {} {} {} {})",
        f(xform.a),
        f(-xform.b),
        f(xform.c),
        f(-xform.d),
        f(xform.e),
        f(-xform.f)
    )
}

/// Convert a shape to an SVG element string
fn shape_to_svg_element(
    shape: &Shape,
    cut_settings: Option<&[CutSetting]>,
    log: &mut Vec<String>,
) -> String {
    let transform = format_matrix(shape.xform());
    let style = get_cut_setting_style(shape.cut_index(), cut_settings);

    match shape {
        Shape::Rect(rect) => {
            let x = -rect.w / 2.0;
            let y = -rect.h / 2.0;

            // Match TS attribute order: x, y, width, height, [rx, ry], style, transform
            let mut el = format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"",
                f(x),
                f(y),
                f(rect.w),
                f(rect.h)
            );

            if rect.cr > 0.0 {
                el.push_str(&format!(" rx=\"{}\" ry=\"{}\"", f(rect.cr), f(rect.cr)));
            }

            el.push_str(&format!(
                " style=\"{}\" transform=\"{}\"/>",
                style, transform
            ));
            el
        }
        Shape::Ellipse(ellipse) => {
            if (ellipse.rx - ellipse.ry).abs() < 1e-10 {
                // Circle - match TS attribute order: cx, cy, r, style, transform
                format!(
                    "<circle cx=\"0\" cy=\"0\" r=\"{}\" style=\"{}\" transform=\"{}\"/>",
                    f(ellipse.rx),
                    style,
                    transform
                )
            } else {
                // Ellipse - match TS attribute order: cx, cy, rx, ry, style, transform
                format!(
                    "<ellipse cx=\"0\" cy=\"0\" rx=\"{}\" ry=\"{}\" style=\"{}\" transform=\"{}\"/>",
                    f(ellipse.rx),
                    f(ellipse.ry),
                    style,
                    transform
                )
            }
        }
        Shape::Path(path) => {
            if path.parsed_verts.is_empty() {
                log.push("Path shape with no vertices".to_string());
                return String::new();
            }

            let d = generate_path_data(path, log);
            if d.is_empty() {
                log.push("Path shape with no valid primitives".to_string());
                return String::new();
            }

            // Match TS attribute order: d, style, transform
            format!(
                "<path d=\"{}\" style=\"{}\" transform=\"{}\"/>",
                d, style, transform
            )
        }
        Shape::Bitmap(bitmap) => {
            if bitmap.data.is_empty() {
                log.push("Bitmap shape missing Data".to_string());
                return String::new();
            }

            let href = format!("data:image/png;base64,{}", bitmap.data);
            // Match TS attribute order: x, y, width, height, xlink:href, transform
            format!(
                "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" xlink:href=\"{}\" transform=\"{}\"/>",
                f(-bitmap.w / 2.0),
                f(-bitmap.h / 2.0),
                f(bitmap.w),
                f(bitmap.h),
                href,
                transform
            )
        }
        Shape::Group(group) => {
            if group.children.is_empty() {
                log.push("Group shape with no children".to_string());
                return String::new();
            }

            // If only one child, flatten transform into the child
            if group.children.len() == 1 {
                let mut child = group.children[0].clone();
                let child_xform = child.xform();

                // Compose transforms: group.XForm * child.XForm
                let composed = group.xform.compose(child_xform);
                *child.xform_mut() = composed;

                return shape_to_svg_element(&child, cut_settings, log);
            }

            // Otherwise, wrap in <g>
            let group_content: Vec<String> = group
                .children
                .iter()
                .map(|child| shape_to_svg_element(child, cut_settings, log))
                .filter(|s| !s.is_empty())
                .collect();

            format!(
                "<g transform=\"{}\">\n    {}\n</g>",
                transform,
                group_content.join("\n    ")
            )
        }
    }
}

/// Convert a LightBurnProject to SVG string
pub fn lbrn2_to_svg(project: &LightBurnProject) -> String {
    if project.shapes.is_empty() {
        return r#"<svg xmlns="http://www.w3.org/2000/svg" width="100mm" height="100mm" viewBox="0 0 100 100"><text>No shapes found</text></svg>"#.to_string();
    }

    let cut_settings = if project.cut_settings.is_empty() {
        None
    } else {
        Some(project.cut_settings.as_slice())
    };

    let mut log: Vec<String> = Vec::new();

    let svg_elements: Vec<String> = project
        .shapes
        .iter()
        .map(|s| shape_to_svg_element(s, cut_settings, &mut log))
        .filter(|s| !s.is_empty())
        .collect();

    // Compute viewBox to encompass all shapes
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for shape in &project.shapes {
        if let Some(bounds) = get_transformed_bounds(shape) {
            min_x = min_x.min(bounds.min_x);
            min_y = min_y.min(bounds.min_y);
            max_x = max_x.max(bounds.max_x);
            max_y = max_y.max(bounds.max_y);
        }
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        min_x = 0.0;
        min_y = -100.0;
        max_x = 100.0;
        max_y = 0.0;
    }

    let w = max_x - min_x;
    let h = max_y - min_y;
    let svg_width = format!("{}mm", f(w));
    let svg_height = format!("{}mm", f(h));
    let view_box = format!("{} {} {} {}", f(min_x), f(min_y), f(w), f(h));

    if !log.is_empty() {
        eprintln!("SVG Conversion Warnings: {:?}", log);
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{}" height="{}" viewBox="{}">
    {}
</svg>"#,
        svg_width,
        svg_height,
        view_box,
        svg_elements.join("\n    ")
    )
}
