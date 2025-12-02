use crate::types::{Path, PathPrimitive, Vec2};

/// Format a number with 6 decimal places
fn f(n: f64) -> String {
    format!("{:.6}", n)
}

/// Generate SVG path data (d attribute) from a Path shape
pub fn generate_path_data(path: &Path, log: &mut Vec<String>) -> String {
    // Handle LineClosed explicitly
    if path.prim_list == "LineClosed" {
        return generate_line_closed_path(path, log);
    }

    // Existing logic for explicit primitives
    if path.parsed_primitives.is_empty() || path.parsed_verts.is_empty() {
        log.push(format!(
            "Path {} or parsedVerts/parsedPrimitives missing/empty, skipping.",
            if path.prim_list.is_empty() {
                "PrimList missing"
            } else {
                &path.prim_list
            }
        ));
        return String::new();
    }

    let mut d = String::new();
    let mut first_move_to_idx: Option<usize> = None;
    let mut current_last_idx: Option<usize> = None;

    for prim in &path.parsed_primitives {
        match prim {
            PathPrimitive::Line { start_idx, end_idx } => {
                let idx0 = *start_idx;
                let idx1 = *end_idx;

                if idx0 >= path.parsed_verts.len() || idx1 >= path.parsed_verts.len() {
                    log.push(format!("Invalid indices for Line: {}, {}", idx0, idx1));
                    continue;
                }

                let p0 = &path.parsed_verts[idx0];
                let p1 = &path.parsed_verts[idx1];

                if first_move_to_idx.is_none() {
                    d.push_str(&format!("M{},{}", f(p0.x), f(p0.y)));
                    first_move_to_idx = Some(idx0);
                } else if current_last_idx != Some(idx0) {
                    d.push_str(&format!(" M{},{}", f(p0.x), f(p0.y)));
                }
                d.push_str(&format!(" L{},{}", f(p1.x), f(p1.y)));
                current_last_idx = Some(idx1);
            }
            PathPrimitive::Bezier { start_idx, end_idx } => {
                let idx0 = *start_idx;
                let idx1 = *end_idx;

                if idx0 >= path.parsed_verts.len() || idx1 >= path.parsed_verts.len() {
                    log.push(format!("Invalid indices for Bezier: {}, {}", idx0, idx1));
                    continue;
                }

                let p0 = &path.parsed_verts[idx0];
                let p1 = &path.parsed_verts[idx1];

                // Check if control points exist
                if p0.c0x.is_none() || p0.c0y.is_none() || p1.c1x.is_none() || p1.c1y.is_none() {
                    log.push(format!(
                        "Bezier primitive {} {} missing control points. Falling back to Line.",
                        idx0, idx1
                    ));

                    // Fallback to line
                    if first_move_to_idx.is_none() {
                        d.push_str(&format!("M{},{}", f(p0.x), f(p0.y)));
                        first_move_to_idx = Some(idx0);
                    } else if current_last_idx != Some(idx0) {
                        d.push_str(&format!(" M{},{}", f(p0.x), f(p0.y)));
                    }
                    d.push_str(&format!(" L{},{}", f(p1.x), f(p1.y)));
                    current_last_idx = Some(idx1);
                    continue;
                }

                if first_move_to_idx.is_none() {
                    d.push_str(&format!("M{},{}", f(p0.x), f(p0.y)));
                    first_move_to_idx = Some(idx0);
                } else if current_last_idx != Some(idx0) {
                    d.push_str(&format!(" M{},{}", f(p0.x), f(p0.y)));
                }

                d.push_str(&format!(
                    " C{},{} {},{} {},{}",
                    f(p0.c0x.unwrap()),
                    f(p0.c0y.unwrap()),
                    f(p1.c1x.unwrap()),
                    f(p1.c1y.unwrap()),
                    f(p1.x),
                    f(p1.y)
                ));
                current_last_idx = Some(idx1);
            }
        }
    }

    // Close path if it ends where it started
    if let (Some(first), Some(last)) = (first_move_to_idx, current_last_idx)
        && first == last
        && !d.is_empty()
    {
        d.push('Z');
    }

    d
}

fn generate_line_closed_path(path: &Path, log: &mut Vec<String>) -> String {
    if path.parsed_verts.is_empty() {
        log.push(format!(
            "Path {} or parsedVerts/parsedPrimitives missing/empty, skipping.",
            if path.prim_list.is_empty() {
                "PrimList missing"
            } else {
                &path.prim_list
            }
        ));
        return String::new();
    }

    let verts = &path.parsed_verts;

    if verts.len() == 1 {
        return format!("M{},{}Z", f(verts[0].x), f(verts[0].y));
    }

    let mut d = format!("M{},{}", f(verts[0].x), f(verts[0].y));

    for v in verts.iter().skip(1) {
        d.push_str(&format!(" L{},{}", f(v.x), f(v.y)));
    }

    d.push('Z');
    d
}

/// Generate path data for testing with arbitrary path-like data
pub fn generate_path_data_from_parts(
    prim_list: &str,
    parsed_verts: &[Option<Vec2>],
    parsed_primitives: &[PathPrimitive],
    log: &mut Vec<String>,
) -> String {
    // Handle LineClosed explicitly
    if prim_list == "LineClosed" {
        if parsed_verts.is_empty() {
            log.push(format!(
                "Path {} or parsedVerts/parsedPrimitives missing/empty, skipping.",
                prim_list
            ));
            return String::new();
        }

        // Check for nullish first vertex
        if parsed_verts[0].is_none() {
            log.push("Path with 'LineClosed' has nullish first vertex, skipping".to_string());
            return String::new();
        }

        let valid_verts: Vec<&Vec2> = parsed_verts.iter().filter_map(|v| v.as_ref()).collect();

        if valid_verts.is_empty() {
            log.push(format!(
                "Path {} or parsedVerts/parsedPrimitives missing/empty, skipping.",
                prim_list
            ));
            return String::new();
        }

        if valid_verts.len() == 1 {
            return format!("M{},{}Z", f(valid_verts[0].x), f(valid_verts[0].y));
        }

        let mut d = format!("M{},{}", f(valid_verts[0].x), f(valid_verts[0].y));

        for (i, v_opt) in parsed_verts.iter().enumerate().skip(1) {
            if let Some(v) = v_opt {
                d.push_str(&format!(" L{},{}", f(v.x), f(v.y)));
            } else {
                log.push(format!(
                    "Path with 'LineClosed' encountered a nullish vertex at index {}, stopping line generation for this path.",
                    i
                ));
                break;
            }
        }

        d.push('Z');
        return d;
    }

    // Non-LineClosed path
    let valid_verts: Vec<Option<&Vec2>> = parsed_verts.iter().map(|v| v.as_ref()).collect();

    if parsed_primitives.is_empty() || valid_verts.iter().all(|v| v.is_none()) {
        log.push(format!(
            "Path {} or parsedVerts/parsedPrimitives missing/empty, skipping.",
            if prim_list.is_empty() {
                "PrimList missing"
            } else {
                prim_list
            }
        ));
        return String::new();
    }

    let mut d = String::new();
    let mut first_move_to_idx: Option<usize> = None;
    let mut current_last_idx: Option<usize> = None;

    for prim in parsed_primitives {
        match prim {
            PathPrimitive::Line { start_idx, end_idx } => {
                let idx0 = *start_idx;
                let idx1 = *end_idx;

                // Check for negative indices (represented as very large usize)
                if idx0 > 1000000 || idx1 > 1000000 {
                    log.push(format!("Invalid indices for Line: {}, {}", idx0, idx1));
                    continue;
                }

                if idx0 >= valid_verts.len() || idx1 >= valid_verts.len() {
                    log.push(format!("Invalid vertex index for Line {} {}", idx0, idx1));
                    continue;
                }

                let p0 = match &valid_verts[idx0] {
                    Some(v) => *v,
                    None => {
                        log.push(format!("Invalid vertex index for Line {} {}", idx0, idx1));
                        continue;
                    }
                };

                let p1 = match &valid_verts[idx1] {
                    Some(v) => *v,
                    None => {
                        log.push(format!("Invalid vertex index for Line {} {}", idx0, idx1));
                        continue;
                    }
                };

                if first_move_to_idx.is_none() {
                    d.push_str(&format!("M{},{}", f(p0.x), f(p0.y)));
                    first_move_to_idx = Some(idx0);
                } else if current_last_idx != Some(idx0) {
                    d.push_str(&format!(" M{},{}", f(p0.x), f(p0.y)));
                }
                d.push_str(&format!(" L{},{}", f(p1.x), f(p1.y)));
                current_last_idx = Some(idx1);
            }
            PathPrimitive::Bezier { start_idx, end_idx } => {
                let idx0 = *start_idx;
                let idx1 = *end_idx;

                // Check for negative indices
                if idx0 > 1000000 || idx1 > 1000000 {
                    log.push(format!("Invalid indices for Bezier: {}, {}", idx0, idx1));
                    continue;
                }

                if idx0 >= valid_verts.len() || idx1 >= valid_verts.len() {
                    log.push(format!("Invalid vertex index for Bezier {} {}", idx0, idx1));
                    continue;
                }

                let p0 = match &valid_verts[idx0] {
                    Some(v) => *v,
                    None => {
                        log.push(format!("Invalid vertex index for Bezier {} {}", idx0, idx1));
                        continue;
                    }
                };

                let p1 = match &valid_verts[idx1] {
                    Some(v) => *v,
                    None => {
                        log.push(format!("Invalid vertex index for Bezier {} {}", idx0, idx1));
                        continue;
                    }
                };

                // Check if control points exist
                if p0.c0x.is_none() || p0.c0y.is_none() || p1.c1x.is_none() || p1.c1y.is_none() {
                    log.push(format!(
                        "Bezier primitive {} {} missing control points. Falling back to Line.",
                        idx0, idx1
                    ));

                    // Fallback to line
                    if first_move_to_idx.is_none() {
                        d.push_str(&format!("M{},{}", f(p0.x), f(p0.y)));
                        first_move_to_idx = Some(idx0);
                    } else if current_last_idx != Some(idx0) {
                        d.push_str(&format!(" M{},{}", f(p0.x), f(p0.y)));
                    }
                    d.push_str(&format!(" L{},{}", f(p1.x), f(p1.y)));
                    current_last_idx = Some(idx1);
                    continue;
                }

                if first_move_to_idx.is_none() {
                    d.push_str(&format!("M{},{}", f(p0.x), f(p0.y)));
                    first_move_to_idx = Some(idx0);
                } else if current_last_idx != Some(idx0) {
                    d.push_str(&format!(" M{},{}", f(p0.x), f(p0.y)));
                }

                d.push_str(&format!(
                    " C{},{} {},{} {},{}",
                    f(p0.c0x.unwrap()),
                    f(p0.c0y.unwrap()),
                    f(p1.c1x.unwrap()),
                    f(p1.c1y.unwrap()),
                    f(p1.x),
                    f(p1.y)
                ));
                current_last_idx = Some(idx1);
            }
        }
    }

    // Close path if it ends where it started
    if let (Some(first), Some(last)) = (first_move_to_idx, current_last_idx)
        && first == last
        && !d.is_empty()
    {
        d.push('Z');
    }

    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_closed_with_0_vertices() {
        let mut log = Vec::new();
        let result = generate_path_data_from_parts("LineClosed", &[], &[], &mut log);
        assert_eq!(result, "");
        assert!(log[0].contains("missing/empty"));
    }

    #[test]
    fn test_line_closed_with_1_vertex() {
        let mut log = Vec::new();
        let verts = vec![Some(Vec2::new(1.0, 2.0))];
        let result = generate_path_data_from_parts("LineClosed", &verts, &[], &mut log);
        assert_eq!(result, "M1.000000,2.000000Z");
    }

    #[test]
    fn test_line_closed_with_nullish_first_vertex() {
        let mut log = Vec::new();
        let verts = vec![None, Some(Vec2::new(2.0, 3.0))];
        let result = generate_path_data_from_parts("LineClosed", &verts, &[], &mut log);
        assert_eq!(result, "");
        assert!(log[0].contains("nullish first vertex"));
    }

    #[test]
    fn test_skips_if_verts_or_prims_missing() {
        let mut log = Vec::new();
        let result = generate_path_data_from_parts("X", &[], &[], &mut log);
        assert_eq!(result, "");
        assert!(log[0].contains("missing/empty"));
    }

    #[test]
    fn test_line_with_invalid_indices() {
        let mut log = Vec::new();
        let verts = vec![Some(Vec2::new(0.0, 0.0)), Some(Vec2::new(1.0, 1.0))];
        let prims = vec![PathPrimitive::Line {
            start_idx: usize::MAX, // Represents -1
            end_idx: 1,
        }];
        let result = generate_path_data_from_parts("", &verts, &prims, &mut log);
        assert_eq!(result, "");
        assert!(log[0].contains("Invalid indices"));
    }

    #[test]
    fn test_line_with_invalid_vertex_index() {
        let mut log = Vec::new();
        let verts = vec![Some(Vec2::new(0.0, 0.0))];
        let prims = vec![PathPrimitive::Line {
            start_idx: 0,
            end_idx: 1,
        }];
        let result = generate_path_data_from_parts("", &verts, &prims, &mut log);
        assert_eq!(result, "");
        assert!(log[0].contains("Invalid vertex index"));
    }

    #[test]
    fn test_bezier_missing_control_points_fallback() {
        let mut log = Vec::new();
        let verts = vec![Some(Vec2::new(0.0, 0.0)), Some(Vec2::new(1.0, 1.0))];
        let prims = vec![PathPrimitive::Bezier {
            start_idx: 0,
            end_idx: 1,
        }];
        let result = generate_path_data_from_parts("", &verts, &prims, &mut log);
        assert!(result.contains("M0.000000,0.000000 L1.000000,1.000000"));
        assert!(log[0].contains("missing control points"));
    }

    #[test]
    fn test_unknown_primitive_type() {
        // In Rust, we can only have Line or Bezier, so this test isn't directly applicable
        // The TypeScript version tests for "Unknown" type, but Rust's enum prevents that
        // We'll skip this test as it's not possible in Rust's type system
    }
}
