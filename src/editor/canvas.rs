use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};

use super::svg_doc::{PathSegment, Point, SvgDocument, SvgElement};

const POINT_RADIUS: f32 = 5.0;
const POINT_HIT_RADIUS: f32 = 10.0;
const CONTROL_POINT_COLOR: Color32 = Color32::from_rgb(100, 100, 255);
const ANCHOR_POINT_COLOR: Color32 = Color32::from_rgb(255, 100, 100);
const SELECTED_COLOR: Color32 = Color32::from_rgb(0, 150, 255);

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Tool {
    #[default]
    Select,
    Move,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointSelection {
    pub element_idx: usize,
    pub segment_idx: usize,
    pub point_idx: usize,
}

#[derive(Debug, Default)]
pub struct CanvasState {
    pub pan: Vec2,
    pub zoom: f32,
    pub selected_element: Option<usize>,
    pub selected_point: Option<PointSelection>,
    pub dragging: bool,
    pub drag_start: Option<Pos2>,
    pub current_tool: Tool,
}

impl CanvasState {
    pub fn new() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            selected_element: None,
            selected_point: None,
            dragging: false,
            drag_start: None,
            current_tool: Tool::Select,
        }
    }

    pub fn screen_to_canvas(&self, screen_pos: Pos2, canvas_rect: Rect) -> Point {
        let local = screen_pos - canvas_rect.min.to_vec2();
        Point::new(
            (local.x - self.pan.x) / self.zoom,
            (local.y - self.pan.y) / self.zoom,
        )
    }

    pub fn canvas_to_screen(&self, canvas_pos: Point, canvas_rect: Rect) -> Pos2 {
        Pos2::new(
            canvas_pos.x * self.zoom + self.pan.x + canvas_rect.min.x,
            canvas_pos.y * self.zoom + self.pan.y + canvas_rect.min.y,
        )
    }
}

pub fn render_canvas(ui: &mut egui::Ui, doc: &mut SvgDocument, state: &mut CanvasState) {
    let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
    let canvas_rect = response.rect;

    // Draw background
    painter.rect_filled(canvas_rect, 0.0, Color32::from_gray(40));

    // Draw document bounds
    let doc_min = state.canvas_to_screen(Point::new(0.0, 0.0), canvas_rect);
    let doc_max = state.canvas_to_screen(Point::new(doc.width, doc.height), canvas_rect);
    painter.rect_filled(Rect::from_min_max(doc_min, doc_max), 0.0, Color32::WHITE);
    painter.rect_stroke(
        Rect::from_min_max(doc_min, doc_max),
        0.0,
        Stroke::new(1.0, Color32::GRAY),
    );

    // Check if space is held for pan mode
    let space_held = ui.input(|i| i.key_down(egui::Key::Space));

    // Handle zoom with scroll wheel
    if response.hovered() {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta);

        if scroll_delta.y != 0.0 {
            let zoom_factor = 1.0 + scroll_delta.y * 0.002;
            let old_zoom = state.zoom;
            state.zoom = (state.zoom * zoom_factor).clamp(0.1, 10.0);

            // Zoom toward mouse position
            if let Some(mouse_pos) = response.hover_pos() {
                let local = mouse_pos - canvas_rect.min.to_vec2();
                state.pan.x = local.x - (local.x - state.pan.x) * (state.zoom / old_zoom);
                state.pan.y = local.y - (local.y - state.pan.y) * (state.zoom / old_zoom);
            }
        }
    }

    // Handle panning: middle mouse, right mouse, or space+drag
    let is_panning = response.dragged_by(egui::PointerButton::Middle)
        || response.dragged_by(egui::PointerButton::Secondary)
        || (space_held && response.dragged_by(egui::PointerButton::Primary));

    if is_panning {
        state.pan += response.drag_delta();
    }

    // Handle tool interactions (only when not panning)
    if !is_panning {
        handle_tool_interaction(ui, doc, state, &response, canvas_rect);
    }

    // Render all elements
    for (idx, element) in doc.elements.iter().enumerate() {
        let is_selected = state.selected_element == Some(idx);
        render_element(&painter, element, state, canvas_rect, is_selected);
    }

    // Render selection handles for selected element
    if let Some(idx) = state.selected_element
        && let Some(element) = doc.elements.get(idx)
    {
        render_selection_handles(&painter, element, state, canvas_rect);
    }
}

fn handle_tool_interaction(
    _ui: &mut egui::Ui,
    doc: &mut SvgDocument,
    state: &mut CanvasState,
    response: &egui::Response,
    canvas_rect: Rect,
) {
    let pointer_pos = response.interact_pointer_pos();

    match state.current_tool {
        Tool::Select | Tool::Move => {
            if response.drag_started_by(egui::PointerButton::Primary)
                && let Some(pos) = pointer_pos
            {
                let canvas_pos = state.screen_to_canvas(pos, canvas_rect);
                state.drag_start = Some(pos);

                // Check if clicking on a point of the selected element
                if let Some(elem_idx) = state.selected_element
                    && let Some(SvgElement::Path(path)) = doc.elements.get(elem_idx)
                {
                    for (seg_idx, pt_idx, pt) in path.get_all_points() {
                        let screen_pt = state.canvas_to_screen(pt, canvas_rect);
                        if pos.distance(screen_pt) < POINT_HIT_RADIUS {
                            state.selected_point = Some(PointSelection {
                                element_idx: elem_idx,
                                segment_idx: seg_idx,
                                point_idx: pt_idx,
                            });
                            state.dragging = true;
                            return;
                        }
                    }
                }

                // Check if clicking on an element
                let mut clicked_element = None;
                for (idx, element) in doc.elements.iter().enumerate().rev() {
                    if element.contains_point(canvas_pos, 5.0 / state.zoom) {
                        clicked_element = Some(idx);
                        break;
                    }
                }

                state.selected_element = clicked_element;
                state.selected_point = None;
                state.dragging = clicked_element.is_some();
            }

            if response.dragged_by(egui::PointerButton::Primary) && state.dragging {
                let delta = response.drag_delta();
                let canvas_delta = Point::new(delta.x / state.zoom, delta.y / state.zoom);

                if let Some(point_sel) = state.selected_point
                    && let Some(SvgElement::Path(path)) =
                        doc.elements.get_mut(point_sel.element_idx)
                {
                    // Moving a specific point
                    let points = path.get_all_points();
                    if let Some((_, _, current_pos)) = points
                        .iter()
                        .find(|(s, p, _)| *s == point_sel.segment_idx && *p == point_sel.point_idx)
                    {
                        let new_pos = Point::new(
                            current_pos.x + canvas_delta.x,
                            current_pos.y + canvas_delta.y,
                        );
                        path.set_point(point_sel.segment_idx, point_sel.point_idx, new_pos);
                    }
                } else if let Some(elem_idx) = state.selected_element
                    && let Some(element) = doc.elements.get_mut(elem_idx)
                {
                    // Moving entire element
                    element.translate(canvas_delta);
                }
            }

            if response.drag_stopped() {
                state.dragging = false;
                state.drag_start = None;
            }
        }
    }
}

fn render_element(
    painter: &egui::Painter,
    element: &SvgElement,
    state: &CanvasState,
    canvas_rect: Rect,
    is_selected: bool,
) {
    match element {
        SvgElement::Path(path) => {
            render_path(painter, path, state, canvas_rect, is_selected);
        }
        SvgElement::Rect(rect) => {
            let min = state.canvas_to_screen(Point::new(rect.x, rect.y), canvas_rect);
            let max = state.canvas_to_screen(
                Point::new(rect.x + rect.width, rect.y + rect.height),
                canvas_rect,
            );
            let screen_rect = Rect::from_min_max(min, max);

            if let Some(fill) = rect.fill {
                painter.rect_filled(screen_rect, 0.0, fill);
            }
            let stroke_color = if is_selected {
                SELECTED_COLOR
            } else {
                rect.stroke.unwrap_or(Color32::BLACK)
            };
            painter.rect_stroke(
                screen_rect,
                0.0,
                Stroke::new(rect.stroke_width * state.zoom, stroke_color),
            );
        }
        SvgElement::Circle(circle) => {
            let center = state.canvas_to_screen(Point::new(circle.cx, circle.cy), canvas_rect);
            let radius = circle.r * state.zoom;

            if let Some(fill) = circle.fill {
                painter.circle_filled(center, radius, fill);
            }
            let stroke_color = if is_selected {
                SELECTED_COLOR
            } else {
                circle.stroke.unwrap_or(Color32::BLACK)
            };
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(circle.stroke_width * state.zoom, stroke_color),
            );
        }
        SvgElement::Ellipse(ellipse) => {
            let center = state.canvas_to_screen(Point::new(ellipse.cx, ellipse.cy), canvas_rect);
            let rx = ellipse.rx * state.zoom;
            let ry = ellipse.ry * state.zoom;

            let stroke_color = if is_selected {
                SELECTED_COLOR
            } else {
                ellipse.stroke.unwrap_or(Color32::BLACK)
            };

            // Draw ellipse using bezier approximation
            render_ellipse(
                painter,
                center,
                rx,
                ry,
                ellipse.fill,
                Stroke::new(ellipse.stroke_width * state.zoom, stroke_color),
            );
        }
    }
}

fn render_path(
    painter: &egui::Painter,
    path: &super::svg_doc::SvgPath,
    state: &CanvasState,
    canvas_rect: Rect,
    is_selected: bool,
) {
    let stroke_color = if is_selected {
        SELECTED_COLOR
    } else {
        path.stroke.unwrap_or(Color32::BLACK)
    };
    let stroke = Stroke::new(path.stroke_width * state.zoom, stroke_color);

    let mut current_pos = Point::new(0.0, 0.0);
    let mut path_start = Point::new(0.0, 0.0);

    for segment in &path.segments {
        match segment {
            PathSegment::MoveTo(pt) => {
                current_pos = *pt;
                path_start = *pt;
            }
            PathSegment::LineTo(pt) => {
                let from = state.canvas_to_screen(current_pos, canvas_rect);
                let to = state.canvas_to_screen(*pt, canvas_rect);
                painter.line_segment([from, to], stroke);
                current_pos = *pt;
            }
            PathSegment::CurveTo { ctrl1, ctrl2, end } => {
                // Approximate cubic bezier with line segments
                let steps = 20;
                let mut prev = state.canvas_to_screen(current_pos, canvas_rect);
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let p = cubic_bezier(current_pos, *ctrl1, *ctrl2, *end, t);
                    let screen_p = state.canvas_to_screen(p, canvas_rect);
                    painter.line_segment([prev, screen_p], stroke);
                    prev = screen_p;
                }
                current_pos = *end;
            }
            PathSegment::QuadTo { ctrl, end } => {
                // Approximate quadratic bezier with line segments
                let steps = 20;
                let mut prev = state.canvas_to_screen(current_pos, canvas_rect);
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let p = quad_bezier(current_pos, *ctrl, *end, t);
                    let screen_p = state.canvas_to_screen(p, canvas_rect);
                    painter.line_segment([prev, screen_p], stroke);
                    prev = screen_p;
                }
                current_pos = *end;
            }
            PathSegment::ClosePath => {
                let from = state.canvas_to_screen(current_pos, canvas_rect);
                let to = state.canvas_to_screen(path_start, canvas_rect);
                painter.line_segment([from, to], stroke);
                current_pos = path_start;
            }
        }
    }

    // Draw fill if present (simplified - just draw a semi-transparent overlay on bounds)
    if let Some(fill) = path.fill {
        let (min, max) = path.bounds();
        let screen_min = state.canvas_to_screen(min, canvas_rect);
        let screen_max = state.canvas_to_screen(max, canvas_rect);
        let fill_color = Color32::from_rgba_unmultiplied(fill.r(), fill.g(), fill.b(), 50);
        painter.rect_filled(Rect::from_min_max(screen_min, screen_max), 0.0, fill_color);
    }
}

fn render_selection_handles(
    painter: &egui::Painter,
    element: &SvgElement,
    state: &CanvasState,
    canvas_rect: Rect,
) {
    // Draw bounding box
    let (min, max) = element.bounds();
    let screen_min = state.canvas_to_screen(min, canvas_rect);
    let screen_max = state.canvas_to_screen(max, canvas_rect);
    painter.rect_stroke(
        Rect::from_min_max(screen_min, screen_max),
        0.0,
        Stroke::new(1.0, SELECTED_COLOR.gamma_multiply(0.5)),
    );

    // Draw points for paths
    if let SvgElement::Path(path) = element {
        let mut current_pos = Point::new(0.0, 0.0);

        for segment in &path.segments {
            match segment {
                PathSegment::MoveTo(pt) | PathSegment::LineTo(pt) => {
                    let screen_pt = state.canvas_to_screen(*pt, canvas_rect);
                    painter.circle_filled(screen_pt, POINT_RADIUS, ANCHOR_POINT_COLOR);
                    painter.circle_stroke(
                        screen_pt,
                        POINT_RADIUS,
                        Stroke::new(1.0, Color32::WHITE),
                    );
                    current_pos = *pt;
                }
                PathSegment::CurveTo { ctrl1, ctrl2, end } => {
                    // Draw control point lines
                    let screen_current = state.canvas_to_screen(current_pos, canvas_rect);
                    let screen_ctrl1 = state.canvas_to_screen(*ctrl1, canvas_rect);
                    let screen_ctrl2 = state.canvas_to_screen(*ctrl2, canvas_rect);
                    let screen_end = state.canvas_to_screen(*end, canvas_rect);

                    painter.line_segment(
                        [screen_current, screen_ctrl1],
                        Stroke::new(1.0, CONTROL_POINT_COLOR.gamma_multiply(0.5)),
                    );
                    painter.line_segment(
                        [screen_end, screen_ctrl2],
                        Stroke::new(1.0, CONTROL_POINT_COLOR.gamma_multiply(0.5)),
                    );

                    // Draw control points
                    painter.circle_filled(screen_ctrl1, POINT_RADIUS - 1.0, CONTROL_POINT_COLOR);
                    painter.circle_filled(screen_ctrl2, POINT_RADIUS - 1.0, CONTROL_POINT_COLOR);

                    // Draw end point
                    painter.circle_filled(screen_end, POINT_RADIUS, ANCHOR_POINT_COLOR);
                    painter.circle_stroke(
                        screen_end,
                        POINT_RADIUS,
                        Stroke::new(1.0, Color32::WHITE),
                    );

                    current_pos = *end;
                }
                PathSegment::QuadTo { ctrl, end } => {
                    let screen_current = state.canvas_to_screen(current_pos, canvas_rect);
                    let screen_ctrl = state.canvas_to_screen(*ctrl, canvas_rect);
                    let screen_end = state.canvas_to_screen(*end, canvas_rect);

                    painter.line_segment(
                        [screen_current, screen_ctrl],
                        Stroke::new(1.0, CONTROL_POINT_COLOR.gamma_multiply(0.5)),
                    );
                    painter.line_segment(
                        [screen_ctrl, screen_end],
                        Stroke::new(1.0, CONTROL_POINT_COLOR.gamma_multiply(0.5)),
                    );

                    painter.circle_filled(screen_ctrl, POINT_RADIUS - 1.0, CONTROL_POINT_COLOR);
                    painter.circle_filled(screen_end, POINT_RADIUS, ANCHOR_POINT_COLOR);
                    painter.circle_stroke(
                        screen_end,
                        POINT_RADIUS,
                        Stroke::new(1.0, Color32::WHITE),
                    );

                    current_pos = *end;
                }
                PathSegment::ClosePath => {}
            }
        }
    }
}

fn cubic_bezier(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    Point::new(
        mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    )
}

fn quad_bezier(p0: Point, p1: Point, p2: Point, t: f32) -> Point {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;

    Point::new(
        mt2 * p0.x + 2.0 * mt * t * p1.x + t2 * p2.x,
        mt2 * p0.y + 2.0 * mt * t * p1.y + t2 * p2.y,
    )
}

fn render_ellipse(
    painter: &egui::Painter,
    center: Pos2,
    rx: f32,
    ry: f32,
    fill: Option<Color32>,
    stroke: Stroke,
) {
    // Draw ellipse by sampling points around the perimeter
    let segments = 64;
    let mut points = Vec::with_capacity(segments);

    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let x = center.x + rx * angle.cos();
        let y = center.y + ry * angle.sin();
        points.push(Pos2::new(x, y));
    }

    // Draw fill
    if let Some(fill_color) = fill {
        let shape = egui::Shape::convex_polygon(points.clone(), fill_color, Stroke::NONE);
        painter.add(shape);
    }

    // Draw stroke
    if stroke.width > 0.0 {
        points.push(points[0]); // Close the path
        let shape = egui::Shape::line(points, stroke);
        painter.add(shape);
    }
}
