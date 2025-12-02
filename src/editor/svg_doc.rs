use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[allow(dead_code)]
    pub fn distance(&self, other: &Point) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl std::ops::Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Point {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    MoveTo(Point),
    LineTo(Point),
    CurveTo {
        ctrl1: Point,
        ctrl2: Point,
        end: Point,
    },
    QuadTo {
        ctrl: Point,
        end: Point,
    },
    ClosePath,
}

#[derive(Debug, Clone)]
pub struct SvgPath {
    pub id: String,
    pub segments: Vec<PathSegment>,
    pub stroke: Option<egui::Color32>,
    pub fill: Option<egui::Color32>,
    pub stroke_width: f32,
}

impl SvgPath {
    pub fn bounds(&self) -> (Point, Point) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for seg in &self.segments {
            let points: Vec<Point> = match seg {
                PathSegment::MoveTo(p) | PathSegment::LineTo(p) => vec![*p],
                PathSegment::CurveTo { ctrl1, ctrl2, end } => vec![*ctrl1, *ctrl2, *end],
                PathSegment::QuadTo { ctrl, end } => vec![*ctrl, *end],
                PathSegment::ClosePath => vec![],
            };
            for p in points {
                min_x = min_x.min(p.x);
                min_y = min_y.min(p.y);
                max_x = max_x.max(p.x);
                max_y = max_y.max(p.y);
            }
        }

        (Point::new(min_x, min_y), Point::new(max_x, max_y))
    }

    pub fn center(&self) -> Point {
        let (min, max) = self.bounds();
        Point::new((min.x + max.x) / 2.0, (min.y + max.y) / 2.0)
    }

    pub fn translate(&mut self, delta: Point) {
        for seg in &mut self.segments {
            match seg {
                PathSegment::MoveTo(p) | PathSegment::LineTo(p) => {
                    p.x += delta.x;
                    p.y += delta.y;
                }
                PathSegment::CurveTo { ctrl1, ctrl2, end } => {
                    ctrl1.x += delta.x;
                    ctrl1.y += delta.y;
                    ctrl2.x += delta.x;
                    ctrl2.y += delta.y;
                    end.x += delta.x;
                    end.y += delta.y;
                }
                PathSegment::QuadTo { ctrl, end } => {
                    ctrl.x += delta.x;
                    ctrl.y += delta.y;
                    end.x += delta.x;
                    end.y += delta.y;
                }
                PathSegment::ClosePath => {}
            }
        }
    }

    pub fn get_all_points(&self) -> Vec<(usize, usize, Point)> {
        let mut points = Vec::new();
        for (seg_idx, seg) in self.segments.iter().enumerate() {
            match seg {
                PathSegment::MoveTo(p) | PathSegment::LineTo(p) => {
                    points.push((seg_idx, 0, *p));
                }
                PathSegment::CurveTo { ctrl1, ctrl2, end } => {
                    points.push((seg_idx, 0, *ctrl1));
                    points.push((seg_idx, 1, *ctrl2));
                    points.push((seg_idx, 2, *end));
                }
                PathSegment::QuadTo { ctrl, end } => {
                    points.push((seg_idx, 0, *ctrl));
                    points.push((seg_idx, 1, *end));
                }
                PathSegment::ClosePath => {}
            }
        }
        points
    }

    pub fn set_point(&mut self, seg_idx: usize, point_idx: usize, new_pos: Point) {
        if let Some(seg) = self.segments.get_mut(seg_idx) {
            match seg {
                PathSegment::MoveTo(p) | PathSegment::LineTo(p) => {
                    if point_idx == 0 {
                        *p = new_pos;
                    }
                }
                PathSegment::CurveTo { ctrl1, ctrl2, end } => match point_idx {
                    0 => *ctrl1 = new_pos,
                    1 => *ctrl2 = new_pos,
                    2 => *end = new_pos,
                    _ => {}
                },
                PathSegment::QuadTo { ctrl, end } => match point_idx {
                    0 => *ctrl = new_pos,
                    1 => *end = new_pos,
                    _ => {}
                },
                PathSegment::ClosePath => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SvgRect {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub stroke: Option<egui::Color32>,
    pub fill: Option<egui::Color32>,
    pub stroke_width: f32,
}

impl SvgRect {
    pub fn bounds(&self) -> (Point, Point) {
        (
            Point::new(self.x, self.y),
            Point::new(self.x + self.width, self.y + self.height),
        )
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn translate(&mut self, delta: Point) {
        self.x += delta.x;
        self.y += delta.y;
    }
}

#[derive(Debug, Clone)]
pub struct SvgCircle {
    pub id: String,
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub stroke: Option<egui::Color32>,
    pub fill: Option<egui::Color32>,
    pub stroke_width: f32,
}

impl SvgCircle {
    pub fn bounds(&self) -> (Point, Point) {
        (
            Point::new(self.cx - self.r, self.cy - self.r),
            Point::new(self.cx + self.r, self.cy + self.r),
        )
    }

    pub fn center(&self) -> Point {
        Point::new(self.cx, self.cy)
    }

    pub fn translate(&mut self, delta: Point) {
        self.cx += delta.x;
        self.cy += delta.y;
    }
}

#[derive(Debug, Clone)]
pub struct SvgEllipse {
    pub id: String,
    pub cx: f32,
    pub cy: f32,
    pub rx: f32,
    pub ry: f32,
    pub stroke: Option<egui::Color32>,
    pub fill: Option<egui::Color32>,
    pub stroke_width: f32,
}

impl SvgEllipse {
    pub fn bounds(&self) -> (Point, Point) {
        (
            Point::new(self.cx - self.rx, self.cy - self.ry),
            Point::new(self.cx + self.rx, self.cy + self.ry),
        )
    }

    pub fn center(&self) -> Point {
        Point::new(self.cx, self.cy)
    }

    pub fn translate(&mut self, delta: Point) {
        self.cx += delta.x;
        self.cy += delta.y;
    }
}

#[derive(Debug, Clone)]
pub enum SvgElement {
    Path(SvgPath),
    #[allow(dead_code)]
    Rect(SvgRect),
    Circle(SvgCircle),
    Ellipse(SvgEllipse),
}

impl SvgElement {
    pub fn id(&self) -> &str {
        match self {
            SvgElement::Path(p) => &p.id,
            SvgElement::Rect(r) => &r.id,
            SvgElement::Circle(c) => &c.id,
            SvgElement::Ellipse(e) => &e.id,
        }
    }

    pub fn bounds(&self) -> (Point, Point) {
        match self {
            SvgElement::Path(p) => p.bounds(),
            SvgElement::Rect(r) => r.bounds(),
            SvgElement::Circle(c) => c.bounds(),
            SvgElement::Ellipse(e) => e.bounds(),
        }
    }

    pub fn center(&self) -> Point {
        match self {
            SvgElement::Path(p) => p.center(),
            SvgElement::Rect(r) => r.center(),
            SvgElement::Circle(c) => c.center(),
            SvgElement::Ellipse(e) => e.center(),
        }
    }

    pub fn translate(&mut self, delta: Point) {
        match self {
            SvgElement::Path(p) => p.translate(delta),
            SvgElement::Rect(r) => r.translate(delta),
            SvgElement::Circle(c) => c.translate(delta),
            SvgElement::Ellipse(e) => e.translate(delta),
        }
    }

    pub fn contains_point(&self, point: Point, tolerance: f32) -> bool {
        let (min, max) = self.bounds();
        point.x >= min.x - tolerance
            && point.x <= max.x + tolerance
            && point.y >= min.y - tolerance
            && point.y <= max.y + tolerance
    }
}

#[derive(Debug, Default, Clone)]
pub struct SvgDocument {
    pub width: f32,
    pub height: f32,
    pub elements: Vec<SvgElement>,
    #[allow(dead_code)]
    pub file_path: Option<String>,
}

impl SvgDocument {
    pub fn new() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            elements: Vec::new(),
            file_path: None,
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let path_ref = path.as_ref();
        let svg_data =
            std::fs::read_to_string(path_ref).map_err(|e| format!("Failed to read file: {}", e))?;

        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_str(&svg_data, &opt)
            .map_err(|e| format!("Failed to parse SVG: {}", e))?;

        let mut doc = SvgDocument {
            width: tree.size().width(),
            height: tree.size().height(),
            elements: Vec::new(),
            file_path: Some(path_ref.to_string_lossy().to_string()),
        };

        let mut id_counter = 0;
        parse_group(tree.root(), &mut doc.elements, &mut id_counter);

        Ok(doc)
    }
}

fn parse_group(group: &usvg::Group, elements: &mut Vec<SvgElement>, id_counter: &mut usize) {
    for child in group.children() {
        match child {
            usvg::Node::Group(g) => {
                parse_group(g, elements, id_counter);
            }
            usvg::Node::Path(path) => {
                if let Some(elem) = parse_path(path, id_counter) {
                    elements.push(elem);
                    *id_counter += 1;
                }
            }
            usvg::Node::Image(_) => {}
            usvg::Node::Text(_) => {}
        }
    }
}

fn parse_path(path: &usvg::Path, id_counter: &mut usize) -> Option<SvgElement> {
    let mut segments = Vec::new();

    for seg in path.data().segments() {
        match seg {
            usvg::tiny_skia_path::PathSegment::MoveTo(pt) => {
                segments.push(PathSegment::MoveTo(Point::new(pt.x, pt.y)));
            }
            usvg::tiny_skia_path::PathSegment::LineTo(pt) => {
                segments.push(PathSegment::LineTo(Point::new(pt.x, pt.y)));
            }
            usvg::tiny_skia_path::PathSegment::CubicTo(pt1, pt2, pt3) => {
                segments.push(PathSegment::CurveTo {
                    ctrl1: Point::new(pt1.x, pt1.y),
                    ctrl2: Point::new(pt2.x, pt2.y),
                    end: Point::new(pt3.x, pt3.y),
                });
            }
            usvg::tiny_skia_path::PathSegment::QuadTo(pt1, pt2) => {
                segments.push(PathSegment::QuadTo {
                    ctrl: Point::new(pt1.x, pt1.y),
                    end: Point::new(pt2.x, pt2.y),
                });
            }
            usvg::tiny_skia_path::PathSegment::Close => {
                segments.push(PathSegment::ClosePath);
            }
        }
    }

    if segments.is_empty() {
        return None;
    }

    let id = if path.id().is_empty() {
        format!("path_{}", id_counter)
    } else {
        path.id().to_string()
    };

    // Try to detect if this path is actually an ellipse or circle
    // usvg converts ellipse/circle to 4 cubic bezier curves + close
    if let Some(ellipse) = try_parse_ellipse(&segments, &id, path) {
        return Some(ellipse);
    }

    let stroke = path.stroke().map(|s| {
        let c = match s.paint() {
            usvg::Paint::Color(c) => *c,
            _ => usvg::Color::black(),
        };
        egui::Color32::from_rgb(c.red, c.green, c.blue)
    });

    let fill = path.fill().map(|f| {
        let c = match f.paint() {
            usvg::Paint::Color(c) => *c,
            _ => usvg::Color::black(),
        };
        egui::Color32::from_rgb(c.red, c.green, c.blue)
    });

    let stroke_width = path.stroke().map(|s| s.width().get()).unwrap_or(1.0);

    Some(SvgElement::Path(SvgPath {
        id,
        segments,
        stroke,
        fill,
        stroke_width,
    }))
}

fn try_parse_ellipse(segments: &[PathSegment], id: &str, path: &usvg::Path) -> Option<SvgElement> {
    // An ellipse converted by usvg has: MoveTo + 4 CurveTo + Close (6 segments)
    if segments.len() != 6 {
        return None;
    }

    // Check pattern: MoveTo, CurveTo, CurveTo, CurveTo, CurveTo, Close
    let start = match &segments[0] {
        PathSegment::MoveTo(p) => *p,
        _ => return None,
    };

    let mut curve_endpoints = Vec::new();
    for segment in segments.iter().take(5).skip(1) {
        match segment {
            PathSegment::CurveTo { end, .. } => curve_endpoints.push(*end),
            _ => return None,
        }
    }

    if !matches!(&segments[5], PathSegment::ClosePath) {
        return None;
    }

    // The 4 curve endpoints should be at the cardinal points of the ellipse
    // Calculate bounding box from all points
    let all_points = [
        start,
        curve_endpoints[0],
        curve_endpoints[1],
        curve_endpoints[2],
        curve_endpoints[3],
    ];
    let min_x = all_points.iter().map(|p| p.x).fold(f32::MAX, f32::min);
    let max_x = all_points.iter().map(|p| p.x).fold(f32::MIN, f32::max);
    let min_y = all_points.iter().map(|p| p.y).fold(f32::MAX, f32::min);
    let max_y = all_points.iter().map(|p| p.y).fold(f32::MIN, f32::max);

    let cx = (min_x + max_x) / 2.0;
    let cy = (min_y + max_y) / 2.0;
    let rx = (max_x - min_x) / 2.0;
    let ry = (max_y - min_y) / 2.0;

    // Verify this looks like an ellipse by checking endpoints are at cardinal positions
    let tolerance = (rx + ry) * 0.1; // 10% tolerance
    let is_ellipse = all_points.iter().all(|p| {
        let dx = (p.x - cx).abs();
        let dy = (p.y - cy).abs();
        // Point should be at one of the cardinal directions
        (dx < tolerance && (dy - ry).abs() < tolerance)
            || (dy < tolerance && (dx - rx).abs() < tolerance)
    });

    if !is_ellipse {
        return None;
    }

    let stroke = path.stroke().map(|s| {
        let c = match s.paint() {
            usvg::Paint::Color(c) => *c,
            _ => usvg::Color::black(),
        };
        egui::Color32::from_rgb(c.red, c.green, c.blue)
    });

    let fill = path.fill().map(|f| {
        let c = match f.paint() {
            usvg::Paint::Color(c) => *c,
            _ => usvg::Color::black(),
        };
        egui::Color32::from_rgb(c.red, c.green, c.blue)
    });

    let stroke_width = path.stroke().map(|s| s.width().get()).unwrap_or(1.0);

    // Check if it's a circle (rx == ry within tolerance)
    if (rx - ry).abs() < tolerance {
        Some(SvgElement::Circle(SvgCircle {
            id: id.to_string(),
            cx,
            cy,
            r: (rx + ry) / 2.0, // average for slight differences
            stroke,
            fill,
            stroke_width,
        }))
    } else {
        Some(SvgElement::Ellipse(SvgEllipse {
            id: id.to_string(),
            cx,
            cy,
            rx,
            ry,
            stroke,
            fill,
            stroke_width,
        }))
    }
}
