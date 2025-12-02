use crate::types::{PathPrimitive, Shape};
use std::f64::consts::PI;

/// Bounding box
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Bounds {
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn expand(&mut self, other: &Bounds) {
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_y.min(other.min_y);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
    }

    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }
}

/// Calculate Bezier curve extrema (t values where derivative is zero)
fn bezier_extrema(p0: (f64, f64), c0: (f64, f64), c1: (f64, f64), p1: (f64, f64)) -> Vec<f64> {
    fn get_extrema(a: f64, b: f64, c: f64, d: f64) -> Vec<f64> {
        let mut res = Vec::new();
        let aa = -a + 3.0 * b - 3.0 * c + d;
        let bb = 2.0 * (a - 2.0 * b + c);
        let cc = b - a;

        if aa.abs() < 1e-8 {
            if bb.abs() > 1e-8 {
                let t = -cc / bb;
                if t > 0.0 && t < 1.0 {
                    res.push(t);
                }
            }
        } else {
            let disc = bb * bb - 4.0 * aa * cc;
            if disc >= 0.0 {
                let sqrt_d = disc.sqrt();
                let t1 = (-bb + sqrt_d) / (2.0 * aa);
                let t2 = (-bb - sqrt_d) / (2.0 * aa);
                if t1 > 0.0 && t1 < 1.0 {
                    res.push(t1);
                }
                if t2 > 0.0 && t2 < 1.0 {
                    res.push(t2);
                }
            }
        }
        res
    }

    let tx = get_extrema(p0.0, c0.0, c1.0, p1.0);
    let ty = get_extrema(p0.1, c0.1, c1.1, p1.1);

    let mut result: Vec<f64> = vec![0.0, 1.0];
    result.extend(tx);
    result.extend(ty);

    // Remove duplicates
    result.sort_by(|a, b| a.partial_cmp(b).unwrap());
    result.dedup_by(|a, b| (*a - *b).abs() < 1e-10);

    result
}

/// Evaluate a cubic Bezier curve at parameter t
fn bezier_point(
    t: f64,
    p0: (f64, f64),
    c0: (f64, f64),
    c1: (f64, f64),
    p1: (f64, f64),
) -> (f64, f64) {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    let t2 = t * t;
    let t3 = t2 * t;

    let x = mt3 * p0.0 + 3.0 * mt2 * t * c0.0 + 3.0 * mt * t2 * c1.0 + t3 * p1.0;
    let y = mt3 * p0.1 + 3.0 * mt2 * t * c0.1 + 3.0 * mt * t2 * c1.1 + t3 * p1.1;

    (x, y)
}

/// Get transformed bounds for a shape
pub fn get_transformed_bounds(shape: &Shape) -> Option<Bounds> {
    let xform = shape.xform();

    // Transform a point with the shape's transform and flip Y for SVG
    let tx = |x: f64, y: f64| -> (f64, f64) {
        (
            xform.a * x + xform.c * y + xform.e,
            -(xform.b * x + xform.d * y + xform.f),
        )
    };

    let mut points_to_bound: Vec<(f64, f64)> = Vec::new();

    match shape {
        Shape::Rect(rect) => {
            let w = rect.w / 2.0;
            let h = rect.h / 2.0;
            points_to_bound.push((-w, -h));
            points_to_bound.push((w, -h));
            points_to_bound.push((w, h));
            points_to_bound.push((-w, h));
        }
        Shape::Ellipse(ellipse) => {
            // Add center
            points_to_bound.push((0.0, 0.0));

            // Add cardinal points
            points_to_bound.push((ellipse.rx, 0.0));
            points_to_bound.push((-ellipse.rx, 0.0));
            points_to_bound.push((0.0, ellipse.ry));
            points_to_bound.push((0.0, -ellipse.ry));

            // Sample 32 points around the ellipse
            let steps = 32;
            for i in 0..steps {
                let theta = 2.0 * PI * (i as f64) / (steps as f64);
                let x = ellipse.rx * theta.cos();
                let y = ellipse.ry * theta.sin();
                points_to_bound.push((x, y));
            }
        }
        Shape::Path(path) => {
            if path.parsed_verts.is_empty() {
                return None;
            }

            if path.prim_list == "LineClosed" {
                // Use all vertices
                for v in &path.parsed_verts {
                    points_to_bound.push((v.x, v.y));
                }
            } else if !path.parsed_primitives.is_empty() {
                for prim in &path.parsed_primitives {
                    match prim {
                        PathPrimitive::Line { start_idx, end_idx } => {
                            if *start_idx < path.parsed_verts.len() {
                                let p0 = &path.parsed_verts[*start_idx];
                                points_to_bound.push((p0.x, p0.y));
                            }
                            if *end_idx < path.parsed_verts.len() {
                                let p1 = &path.parsed_verts[*end_idx];
                                points_to_bound.push((p1.x, p1.y));
                            }
                        }
                        PathPrimitive::Bezier { start_idx, end_idx } => {
                            if *start_idx >= path.parsed_verts.len()
                                || *end_idx >= path.parsed_verts.len()
                            {
                                continue;
                            }

                            let p0 = &path.parsed_verts[*start_idx];
                            let p1 = &path.parsed_verts[*end_idx];

                            points_to_bound.push((p0.x, p0.y));
                            points_to_bound.push((p1.x, p1.y));

                            // Add control points
                            if let (Some(c0x), Some(c0y)) = (p0.c0x, p0.c0y) {
                                points_to_bound.push((c0x, c0y));
                            }
                            if let (Some(c1x), Some(c1y)) = (p1.c1x, p1.c1y) {
                                points_to_bound.push((c1x, c1y));
                            }

                            // Calculate Bezier extrema points
                            if let (Some(c0x), Some(c0y), Some(c1x), Some(c1y)) =
                                (p0.c0x, p0.c0y, p1.c1x, p1.c1y)
                            {
                                let c0 = (c0x, c0y);
                                let c1 = (c1x, c1y);
                                let ts = bezier_extrema((p0.x, p0.y), c0, c1, (p1.x, p1.y));

                                for t in ts {
                                    let pt = bezier_point(t, (p0.x, p0.y), c0, c1, (p1.x, p1.y));
                                    points_to_bound.push(pt);
                                }
                            }
                        }
                    }
                }
            } else {
                // Fallback: use all vertices
                for v in &path.parsed_verts {
                    points_to_bound.push((v.x, v.y));
                }
            }
        }
        Shape::Bitmap(bitmap) => {
            let w = bitmap.w / 2.0;
            let h = bitmap.h / 2.0;
            points_to_bound.push((-w, -h));
            points_to_bound.push((w, -h));
            points_to_bound.push((w, h));
            points_to_bound.push((-w, h));
        }
        Shape::Group(group) => {
            if group.children.is_empty() {
                return None;
            }

            let mut combined_bounds: Option<Bounds> = None;

            for child in &group.children {
                // Compose transforms
                let effective_child_xform = xform.compose(child.xform());
                let mut temp_child = child.clone();
                *temp_child.xform_mut() = effective_child_xform;

                if let Some(child_bounds) = get_transformed_bounds(&temp_child) {
                    match &mut combined_bounds {
                        None => combined_bounds = Some(child_bounds),
                        Some(cb) => cb.expand(&child_bounds),
                    }
                }
            }

            return combined_bounds;
        }
    }

    if points_to_bound.is_empty() {
        return None;
    }

    let transformed: Vec<(f64, f64)> = points_to_bound.into_iter().map(|(x, y)| tx(x, y)).collect();

    if transformed.is_empty() {
        return None;
    }

    let xs: Vec<f64> = transformed.iter().map(|(x, _)| *x).collect();
    let ys: Vec<f64> = transformed.iter().map(|(_, y)| *y).collect();

    Some(Bounds::new(
        xs.iter().cloned().fold(f64::INFINITY, f64::min),
        ys.iter().cloned().fold(f64::INFINITY, f64::min),
        xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
    ))
}
