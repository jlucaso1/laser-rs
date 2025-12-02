use laser_tools::editor::{
    canvas::CanvasState,
    history::History,
    svg_doc::{PathSegment, Point, SvgDocument, SvgElement, SvgPath},
};

mod point_tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let p = Point::new(10.0, 20.0);
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
    }

    #[test]
    fn test_point_addition() {
        let p1 = Point::new(10.0, 20.0);
        let p2 = Point::new(5.0, 15.0);
        let result = p1 + p2;
        assert_eq!(result.x, 15.0);
        assert_eq!(result.y, 35.0);
    }

    #[test]
    fn test_point_subtraction() {
        let p1 = Point::new(10.0, 20.0);
        let p2 = Point::new(5.0, 15.0);
        let result = p1 - p2;
        assert_eq!(result.x, 5.0);
        assert_eq!(result.y, 5.0);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        let dist = p1.distance(&p2);
        assert!((dist - 5.0).abs() < 0.0001);
    }
}

mod path_tests {
    use super::*;

    fn create_test_path() -> SvgPath {
        SvgPath {
            id: "test_path".to_string(),
            segments: vec![
                PathSegment::MoveTo(Point::new(0.0, 0.0)),
                PathSegment::LineTo(Point::new(100.0, 0.0)),
                PathSegment::LineTo(Point::new(100.0, 100.0)),
                PathSegment::LineTo(Point::new(0.0, 100.0)),
                PathSegment::ClosePath,
            ],
            stroke: Some(egui::Color32::BLACK),
            fill: None,
            stroke_width: 1.0,
        }
    }

    #[test]
    fn test_path_bounds() {
        let path = create_test_path();
        let (min, max) = path.bounds();
        assert_eq!(min.x, 0.0);
        assert_eq!(min.y, 0.0);
        assert_eq!(max.x, 100.0);
        assert_eq!(max.y, 100.0);
    }

    #[test]
    fn test_path_center() {
        let path = create_test_path();
        let center = path.center();
        assert_eq!(center.x, 50.0);
        assert_eq!(center.y, 50.0);
    }

    #[test]
    fn test_path_translate() {
        let mut path = create_test_path();
        path.translate(Point::new(10.0, 20.0));

        let (min, max) = path.bounds();
        assert_eq!(min.x, 10.0);
        assert_eq!(min.y, 20.0);
        assert_eq!(max.x, 110.0);
        assert_eq!(max.y, 120.0);
    }

    #[test]
    fn test_path_get_all_points() {
        let path = create_test_path();
        let points = path.get_all_points();

        // MoveTo + 3 LineTo = 4 points (ClosePath has no points)
        assert_eq!(points.len(), 4);

        // Check first point (MoveTo)
        assert_eq!(points[0].0, 0); // segment_idx
        assert_eq!(points[0].1, 0); // point_idx
        assert_eq!(points[0].2.x, 0.0);
        assert_eq!(points[0].2.y, 0.0);
    }

    #[test]
    fn test_path_set_point() {
        let mut path = create_test_path();
        path.set_point(1, 0, Point::new(150.0, 50.0));

        let points = path.get_all_points();
        let modified_point = points.iter().find(|(s, p, _)| *s == 1 && *p == 0).unwrap();
        assert_eq!(modified_point.2.x, 150.0);
        assert_eq!(modified_point.2.y, 50.0);
    }

    #[test]
    fn test_path_with_bezier() {
        let path = SvgPath {
            id: "bezier_path".to_string(),
            segments: vec![
                PathSegment::MoveTo(Point::new(0.0, 0.0)),
                PathSegment::CurveTo {
                    ctrl1: Point::new(25.0, 50.0),
                    ctrl2: Point::new(75.0, 50.0),
                    end: Point::new(100.0, 0.0),
                },
            ],
            stroke: Some(egui::Color32::BLACK),
            fill: None,
            stroke_width: 1.0,
        };

        let points = path.get_all_points();
        // MoveTo (1 point) + CurveTo (3 points: ctrl1, ctrl2, end)
        assert_eq!(points.len(), 4);
    }
}

mod svg_element_tests {
    use super::*;

    #[test]
    fn test_element_contains_point() {
        let path = SvgPath {
            id: "test".to_string(),
            segments: vec![
                PathSegment::MoveTo(Point::new(0.0, 0.0)),
                PathSegment::LineTo(Point::new(100.0, 100.0)),
            ],
            stroke: Some(egui::Color32::BLACK),
            fill: None,
            stroke_width: 1.0,
        };
        let element = SvgElement::Path(path);

        // Point inside bounds
        assert!(element.contains_point(Point::new(50.0, 50.0), 5.0));

        // Point outside bounds
        assert!(!element.contains_point(Point::new(200.0, 200.0), 5.0));

        // Point near edge (within tolerance)
        assert!(element.contains_point(Point::new(-3.0, 0.0), 5.0));
    }
}

mod document_tests {
    use super::*;

    #[test]
    fn test_document_new() {
        let doc = SvgDocument::new();
        assert_eq!(doc.width, 800.0);
        assert_eq!(doc.height, 600.0);
        assert!(doc.elements.is_empty());
    }

    #[test]
    fn test_document_load_valid_svg() {
        let svg_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
  <rect x="10" y="10" width="80" height="80" fill="blue"/>
  <circle cx="100" cy="100" r="40" fill="red"/>
</svg>"#;

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_editor.svg");
        std::fs::write(&temp_file, svg_content).unwrap();

        let doc = SvgDocument::load(&temp_file).unwrap();
        assert_eq!(doc.width, 200.0);
        assert_eq!(doc.height, 200.0);
        // usvg converts rect and circle to paths, then we detect circle
        assert!(!doc.elements.is_empty());

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_document_load_invalid_file() {
        let result = SvgDocument::load("/nonexistent/path/file.svg");
        assert!(result.is_err());
    }

    #[test]
    fn test_document_load_invalid_svg() {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("invalid_editor.svg");
        std::fs::write(&temp_file, "not valid svg content").unwrap();

        let result = SvgDocument::load(&temp_file);
        assert!(result.is_err());

        std::fs::remove_file(temp_file).ok();
    }
}

mod history_tests {
    use super::*;

    fn create_test_document(width: f32) -> SvgDocument {
        SvgDocument {
            width,
            height: 600.0,
            elements: vec![],
            file_path: None,
        }
    }

    #[test]
    fn test_history_new() {
        let history = History::new();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_history_save_state() {
        let mut history = History::new();
        let doc = create_test_document(800.0);

        history.save_state(&doc);
        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 1);
    }

    #[test]
    fn test_history_undo() {
        let mut history = History::new();
        let doc1 = create_test_document(800.0);
        let doc2 = create_test_document(1000.0);

        history.save_state(&doc1);

        let restored = history.undo(&doc2);
        assert!(restored.is_some());
        assert_eq!(restored.unwrap().width, 800.0);
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_history_redo() {
        let mut history = History::new();
        let doc1 = create_test_document(800.0);
        let doc2 = create_test_document(1000.0);

        history.save_state(&doc1);
        history.undo(&doc2);

        let restored = history.redo(&doc1);
        assert!(restored.is_some());
        assert_eq!(restored.unwrap().width, 1000.0);
    }

    #[test]
    fn test_history_undo_empty() {
        let mut history = History::new();
        let doc = create_test_document(800.0);

        let result = history.undo(&doc);
        assert!(result.is_none());
    }

    #[test]
    fn test_history_redo_empty() {
        let mut history = History::new();
        let doc = create_test_document(800.0);

        let result = history.redo(&doc);
        assert!(result.is_none());
    }

    #[test]
    fn test_history_clear() {
        let mut history = History::new();
        let doc = create_test_document(800.0);

        history.save_state(&doc);
        history.save_state(&doc);
        assert_eq!(history.undo_count(), 2);

        history.clear();
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
    }

    #[test]
    fn test_history_new_action_clears_redo() {
        let mut history = History::new();
        let doc1 = create_test_document(800.0);
        let doc2 = create_test_document(1000.0);
        let doc3 = create_test_document(1200.0);

        history.save_state(&doc1);
        history.undo(&doc2);
        assert!(history.can_redo());

        // New action should clear redo stack
        history.save_state(&doc3);
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_multiple_undo_redo() {
        let mut history = History::new();
        let doc1 = create_test_document(100.0);
        let doc2 = create_test_document(200.0);
        let doc3 = create_test_document(300.0);

        history.save_state(&doc1);
        history.save_state(&doc2);

        // Current state is doc3 (300), stack has [doc1, doc2]
        let restored1 = history.undo(&doc3).unwrap();
        assert_eq!(restored1.width, 200.0);

        let restored2 = history.undo(&restored1).unwrap();
        assert_eq!(restored2.width, 100.0);

        // Redo back
        let redo1 = history.redo(&restored2).unwrap();
        assert_eq!(redo1.width, 200.0);

        let redo2 = history.redo(&redo1).unwrap();
        assert_eq!(redo2.width, 300.0);
    }
}

mod canvas_state_tests {
    use super::*;

    #[test]
    fn test_canvas_state_new() {
        let state = CanvasState::new();
        assert_eq!(state.zoom, 1.0);
        assert_eq!(state.pan.x, 0.0);
        assert_eq!(state.pan.y, 0.0);
        assert!(state.selected_element.is_none());
        assert!(state.selected_point.is_none());
        assert!(!state.dragging);
    }

    #[test]
    fn test_canvas_state_default() {
        // Default derive sets zoom to 0.0, use new() for proper initialization
        let state = CanvasState::default();
        assert_eq!(state.zoom, 0.0); // Default f32 is 0.0
        assert!(state.selected_element.is_none());
    }
}

mod circle_ellipse_detection_tests {
    use super::*;

    #[test]
    fn test_load_svg_with_circle() {
        let svg_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
  <circle cx="100" cy="100" r="50" stroke="black" fill="none"/>
</svg>"#;

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_circle.svg");
        std::fs::write(&temp_file, svg_content).unwrap();

        let doc = SvgDocument::load(&temp_file).unwrap();

        // Should detect circle
        let has_circle = doc
            .elements
            .iter()
            .any(|e| matches!(e, SvgElement::Circle(_)));
        assert!(has_circle, "Circle should be detected from SVG");

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_load_svg_with_ellipse() {
        let svg_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="200" height="200">
  <ellipse cx="100" cy="100" rx="80" ry="40" stroke="black" fill="none"/>
</svg>"#;

        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_ellipse.svg");
        std::fs::write(&temp_file, svg_content).unwrap();

        let doc = SvgDocument::load(&temp_file).unwrap();

        // Should detect ellipse
        let has_ellipse = doc
            .elements
            .iter()
            .any(|e| matches!(e, SvgElement::Ellipse(_)));
        assert!(has_ellipse, "Ellipse should be detected from SVG");

        std::fs::remove_file(temp_file).ok();
    }
}
