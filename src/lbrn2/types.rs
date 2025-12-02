/// 2D vertex with optional Bezier control points
#[derive(Debug, Clone, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
    /// Control point 0 x (for curve leaving this vertex)
    pub c0x: Option<f64>,
    /// Control point 0 y
    pub c0y: Option<f64>,
    /// Control point 1 x (for curve arriving at this vertex)
    pub c1x: Option<f64>,
    /// Control point 1 y
    pub c1y: Option<f64>,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            c0x: None,
            c0y: None,
            c1x: None,
            c1y: None,
        }
    }

    pub fn with_control_points(
        x: f64,
        y: f64,
        c0x: Option<f64>,
        c0y: Option<f64>,
        c1x: Option<f64>,
        c1y: Option<f64>,
    ) -> Self {
        Self {
            x,
            y,
            c0x,
            c0y,
            c1x,
            c1y,
        }
    }
}

/// 2D affine transformation matrix [a, b, c, d, e, f]
/// Represents: | a  c  e |
///             | b  d  f |
///             | 0  0  1 |
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XForm {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl XForm {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// Compose two transforms: self * other
    pub fn compose(&self, other: &XForm) -> XForm {
        XForm {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    /// Transform a point
    pub fn transform_point(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }
}

/// Cut setting for laser operations
#[derive(Debug, Clone)]
pub struct CutSetting {
    pub index: i32,
    pub name: String,
    pub color: Option<String>,
    pub stroke_width: Option<String>,
}

/// Path primitive intermediate representation
#[derive(Debug, Clone, PartialEq)]
pub enum PathPrimitive {
    Line { start_idx: usize, end_idx: usize },
    Bezier { start_idx: usize, end_idx: usize },
}

/// Rectangle shape
#[derive(Debug, Clone)]
pub struct Rect {
    pub cut_index: i32,
    pub xform: XForm,
    pub w: f64,
    pub h: f64,
    pub cr: f64, // corner radius
}

/// Ellipse shape
#[derive(Debug, Clone)]
pub struct Ellipse {
    pub cut_index: i32,
    pub xform: XForm,
    pub rx: f64,
    pub ry: f64,
}

/// Path shape with vertices and primitives
#[derive(Debug, Clone)]
pub struct Path {
    pub cut_index: i32,
    pub xform: XForm,
    pub vert_list: String,
    pub prim_list: String,
    pub parsed_verts: Vec<Vec2>,
    pub parsed_primitives: Vec<PathPrimitive>,
}

/// Bitmap/image shape
#[derive(Debug, Clone)]
pub struct Bitmap {
    pub cut_index: i32,
    pub xform: XForm,
    pub w: f64,
    pub h: f64,
    pub data: String, // Base64 encoded image data
}

/// Group of shapes
#[derive(Debug, Clone)]
pub struct Group {
    pub cut_index: i32,
    pub xform: XForm,
    pub children: Vec<Shape>,
}

/// All possible shape types
#[derive(Debug, Clone)]
pub enum Shape {
    Rect(Rect),
    Ellipse(Ellipse),
    Path(Path),
    Bitmap(Bitmap),
    Group(Group),
}

impl Shape {
    pub fn xform(&self) -> &XForm {
        match self {
            Shape::Rect(r) => &r.xform,
            Shape::Ellipse(e) => &e.xform,
            Shape::Path(p) => &p.xform,
            Shape::Bitmap(b) => &b.xform,
            Shape::Group(g) => &g.xform,
        }
    }

    pub fn xform_mut(&mut self) -> &mut XForm {
        match self {
            Shape::Rect(r) => &mut r.xform,
            Shape::Ellipse(e) => &mut e.xform,
            Shape::Path(p) => &mut p.xform,
            Shape::Bitmap(b) => &mut b.xform,
            Shape::Group(g) => &mut g.xform,
        }
    }

    pub fn cut_index(&self) -> i32 {
        match self {
            Shape::Rect(r) => r.cut_index,
            Shape::Ellipse(e) => e.cut_index,
            Shape::Path(p) => p.cut_index,
            Shape::Bitmap(b) => b.cut_index,
            Shape::Group(g) => g.cut_index,
        }
    }
}

/// Parsed LightBurn project file
#[derive(Debug, Clone)]
pub struct LightBurnProject {
    pub app_version: String,
    pub format_version: String,
    pub cut_settings: Vec<CutSetting>,
    pub shapes: Vec<Shape>,
}
