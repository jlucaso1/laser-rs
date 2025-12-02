use crate::types::*;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;

/// Parse XForm string "a b c d e f" into XForm struct
pub fn parse_xform(xform_str: &str) -> XForm {
    let parts: Vec<f64> = xform_str
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    if parts.len() == 6 {
        XForm {
            a: parts[0],
            b: parts[1],
            c: parts[2],
            d: parts[3],
            e: parts[4],
            f: parts[5],
        }
    } else {
        eprintln!("Invalid XForm string, using identity: {}", xform_str);
        XForm::identity()
    }
}

/// Parse control point data from a string like "c0x1c1x49c1y48"
fn parse_control_point_data(cp_str: &str) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    let mut c0x = None;
    let mut c0y = None;
    let mut c1x = None;
    let mut c1y = None;

    if !cp_str.starts_with('c') {
        return (c0x, c0y, c1x, c1y);
    }

    let mut i = 0;
    let chars: Vec<char> = cp_str.chars().collect();

    while i < chars.len() {
        let remaining: String = chars[i..].iter().collect();

        let key = if remaining.starts_with("c0x") {
            Some("c0x")
        } else if remaining.starts_with("c0y") {
            Some("c0y")
        } else if remaining.starts_with("c1x") {
            Some("c1x")
        } else if remaining.starts_with("c1y") {
            Some("c1y")
        } else {
            None
        };

        if let Some(k) = key {
            i += 3;
            let mut num_str = String::new();

            while i < chars.len() {
                let ch = chars[i];
                if ch == '-'
                    || ch == '+'
                    || ch.is_ascii_digit()
                    || ch == '.'
                    || ch == 'e'
                    || ch == 'E'
                {
                    num_str.push(ch);
                    i += 1;
                } else {
                    break;
                }
            }

            if let Ok(value) = num_str.parse::<f64>() {
                match k {
                    "c0x" => c0x = Some(value),
                    "c0y" => c0y = Some(value),
                    "c1x" => c1x = Some(value),
                    "c1y" => c1y = Some(value),
                    _ => {}
                }
            }
        } else {
            i += 1;
        }
    }

    (c0x, c0y, c1x, c1y)
}

/// Parse VertList string into Vec<Vec2>
pub fn parse_vert_list(vert_list_str: &str) -> Vec<Vec2> {
    let mut vertices = Vec::new();
    let chars: Vec<char> = vert_list_str.chars().collect();
    let mut i = 0;
    let len = chars.len();

    while i < len {
        // Skip whitespace
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }

        if i < len && chars[i] == 'V' {
            i += 1;

            // Skip whitespace after V
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }

            // Parse x coordinate
            let mut x_str = String::new();
            while i < len {
                let ch = chars[i];
                if ch == '-'
                    || ch == '+'
                    || ch.is_ascii_digit()
                    || ch == '.'
                    || ch == 'e'
                    || ch == 'E'
                {
                    x_str.push(ch);
                    i += 1;
                } else {
                    break;
                }
            }

            // Skip whitespace between x and y
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }

            // Parse y coordinate
            let mut y_str = String::new();
            while i < len {
                let ch = chars[i];
                if ch == '-'
                    || ch == '+'
                    || ch.is_ascii_digit()
                    || ch == '.'
                    || ch == 'e'
                    || ch == 'E'
                {
                    y_str.push(ch);
                    i += 1;
                } else {
                    break;
                }
            }

            // Collect control point string until next V or end
            let mut cp_str = String::new();
            while i < len && chars[i] != 'V' {
                cp_str.push(chars[i]);
                i += 1;
            }

            if x_str.is_empty() || y_str.is_empty() {
                eprintln!(
                    "Failed to parse vertex from X: \"{}\", Y: \"{}\" in VertList: \"{}\"",
                    x_str, y_str, vert_list_str
                );
                continue;
            }

            let x: f64 = x_str.parse().unwrap_or(0.0);
            let y: f64 = y_str.parse().unwrap_or(0.0);

            let (c0x, c0y, c1x, c1y) = parse_control_point_data(cp_str.trim());

            vertices.push(Vec2::with_control_points(x, y, c0x, c0y, c1x, c1y));
        } else {
            i += 1;
        }
    }

    vertices
}

/// Parse PrimList string into Vec<PathPrimitive>
pub fn parse_prim_list(prim_list_str: &str) -> Vec<PathPrimitive> {
    let mut primitives = Vec::new();
    let chars: Vec<char> = prim_list_str.chars().collect();
    let mut i = 0;
    let len = chars.len();

    fn parse_next_int(chars: &[char], i: &mut usize, len: usize) -> Option<usize> {
        // Skip whitespace
        while *i < len && chars[*i].is_whitespace() {
            *i += 1;
        }

        let mut num_str = String::new();
        while *i < len && chars[*i].is_ascii_digit() {
            num_str.push(chars[*i]);
            *i += 1;
        }

        if !num_str.is_empty() {
            num_str.parse().ok()
        } else {
            None
        }
    }

    while i < len {
        // Skip whitespace
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }

        if i >= len {
            break;
        }

        let prim_type = chars[i];
        if !prim_type.is_alphabetic() {
            i += 1;
            continue;
        }

        i += 1;

        let mut args = Vec::new();
        for _ in 0..4 {
            if let Some(num) = parse_next_int(&chars, &mut i, len) {
                args.push(num);
            } else {
                break;
            }
        }

        if prim_type == 'L' && args.len() >= 2 {
            primitives.push(PathPrimitive::Line {
                start_idx: args[0],
                end_idx: args[1],
            });
        } else if prim_type == 'B' && args.len() >= 2 {
            primitives.push(PathPrimitive::Bezier {
                start_idx: args[0],
                end_idx: args[1],
            });
        }
    }

    primitives
}

/// Parse an LBRN2 XML string into a LightBurnProject
pub fn parse_lbrn2_complete(xml_string: &str) -> Result<LightBurnProject, String> {
    let mut reader = Reader::from_str(xml_string);
    reader.config_mut().trim_text(true);

    let mut project = LightBurnProject {
        app_version: String::new(),
        format_version: String::new(),
        cut_settings: Vec::new(),
        shapes: Vec::new(),
    };

    let mut vertex_cache: HashMap<i32, (String, Vec<Vec2>)> = HashMap::new();
    let mut primitive_cache: HashMap<i32, (String, Vec<PathPrimitive>)> = HashMap::new();

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_bytes = e.name();
                let name = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("");

                if name == "LightBurnProject" {
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let value = std::str::from_utf8(&attr.value).unwrap_or("");
                        match key {
                            "AppVersion" => project.app_version = value.to_string(),
                            "FormatVersion" => project.format_version = value.to_string(),
                            _ => {}
                        }
                    }
                } else if name == "CutSetting" {
                    let cs = parse_cut_setting_inner(&mut reader)?;
                    project.cut_settings.push(cs);
                } else if name == "Shape" {
                    // Collect attributes first
                    let mut shape_type = String::new();
                    let mut cut_index: i32 = 0;
                    let mut w: f64 = 0.0;
                    let mut h: f64 = 0.0;
                    let mut cr: f64 = 0.0;
                    let mut rx: f64 = 0.0;
                    let mut ry: f64 = 0.0;
                    let mut vert_id: Option<i32> = None;
                    let mut prim_id: Option<i32> = None;
                    let mut has_backup_path = false;
                    let mut data_attr = String::new();

                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let value = std::str::from_utf8(&attr.value).unwrap_or("");
                        match key {
                            "Type" => shape_type = value.to_string(),
                            "CutIndex" => cut_index = value.parse().unwrap_or(0),
                            "W" => w = value.parse().unwrap_or(0.0),
                            "H" => h = value.parse().unwrap_or(0.0),
                            "Cr" => cr = value.parse().unwrap_or(0.0),
                            "Rx" => rx = value.parse().unwrap_or(0.0),
                            "Ry" => ry = value.parse().unwrap_or(0.0),
                            "VertID" => vert_id = value.parse().ok(),
                            "PrimID" => prim_id = value.parse().ok(),
                            "HasBackupPath" => has_backup_path = value == "1",
                            "Data" => data_attr = value.to_string(),
                            _ => {}
                        }
                    }

                    if let Some(shape) = parse_shape_inner(
                        &mut reader,
                        shape_type,
                        cut_index,
                        w,
                        h,
                        cr,
                        rx,
                        ry,
                        vert_id,
                        prim_id,
                        has_backup_path,
                        data_attr,
                        &mut vertex_cache,
                        &mut primitive_cache,
                    )? {
                        project.shapes.push(shape);
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let name_bytes = e.name();
                let name = std::str::from_utf8(name_bytes.as_ref()).unwrap_or("");
                if name == "Shape"
                    && let Some(shape) = parse_shape_from_empty_element(e)?
                {
                    project.shapes.push(shape);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parsing error: {:?}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(project)
}

fn parse_cut_setting_inner(reader: &mut Reader<&[u8]>) -> Result<CutSetting, String> {
    let mut index: i32 = 0;
    let name = String::new();
    let mut buf = Vec::new();
    let mut depth = 1;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                depth += 1;
                let tag_bytes = e.name();
                let tag = std::str::from_utf8(tag_bytes.as_ref()).unwrap_or("");
                if tag == "index" {
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let value = std::str::from_utf8(&attr.value).unwrap_or("");
                        if key == "Value" {
                            index = value.parse().unwrap_or(0);
                        }
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag_bytes = e.name();
                let tag = std::str::from_utf8(tag_bytes.as_ref()).unwrap_or("");
                if tag == "index" {
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let value = std::str::from_utf8(&attr.value).unwrap_or("");
                        if key == "Value" {
                            index = value.parse().unwrap_or(0);
                        }
                    }
                }
            }
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing CutSetting: {:?}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(CutSetting {
        index,
        name,
        color: None,
        stroke_width: None,
    })
}

#[allow(clippy::too_many_arguments)]
fn parse_shape_inner(
    reader: &mut Reader<&[u8]>,
    shape_type: String,
    cut_index: i32,
    w: f64,
    h: f64,
    cr: f64,
    rx: f64,
    ry: f64,
    vert_id: Option<i32>,
    prim_id: Option<i32>,
    has_backup_path: bool,
    data_attr: String,
    vertex_cache: &mut HashMap<i32, (String, Vec<Vec2>)>,
    primitive_cache: &mut HashMap<i32, (String, Vec<PathPrimitive>)>,
) -> Result<Option<Shape>, String> {
    let mut xform = XForm::identity();
    let mut vert_list = String::new();
    let mut prim_list = String::new();
    let mut data = data_attr;
    let mut children: Vec<Shape> = Vec::new();
    let mut backup_path_shape: Option<Shape> = None;

    let mut buf = Vec::new();
    let mut depth = 1;
    let mut current_tag = String::new();
    let mut in_children = false;
    let mut in_backup_path = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                depth += 1;
                let tag_bytes = e.name();
                let tag = std::str::from_utf8(tag_bytes.as_ref()).unwrap_or("");
                current_tag = tag.to_string();

                if tag == "Children" {
                    in_children = true;
                } else if tag == "BackupPath" {
                    // BackupPath element has shape attributes directly on it
                    let mut bp_shape_type = String::new();
                    let mut bp_cut_index: i32 = 0;
                    let mut bp_w: f64 = 0.0;
                    let mut bp_h: f64 = 0.0;
                    let mut bp_cr: f64 = 0.0;
                    let mut bp_rx: f64 = 0.0;
                    let mut bp_ry: f64 = 0.0;
                    let mut bp_vert_id: Option<i32> = None;
                    let mut bp_prim_id: Option<i32> = None;
                    let mut bp_has_backup_path = false;
                    let mut bp_data_attr = String::new();

                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let value = std::str::from_utf8(&attr.value).unwrap_or("");
                        match key {
                            "Type" => bp_shape_type = value.to_string(),
                            "CutIndex" => bp_cut_index = value.parse().unwrap_or(0),
                            "W" => bp_w = value.parse().unwrap_or(0.0),
                            "H" => bp_h = value.parse().unwrap_or(0.0),
                            "Cr" => bp_cr = value.parse().unwrap_or(0.0),
                            "Rx" => bp_rx = value.parse().unwrap_or(0.0),
                            "Ry" => bp_ry = value.parse().unwrap_or(0.0),
                            "VertID" => bp_vert_id = value.parse().ok(),
                            "PrimID" => bp_prim_id = value.parse().ok(),
                            "HasBackupPath" => bp_has_backup_path = value == "1",
                            "Data" => bp_data_attr = value.to_string(),
                            _ => {}
                        }
                    }

                    if let Some(bp) = parse_shape_inner(
                        reader,
                        bp_shape_type,
                        bp_cut_index,
                        bp_w,
                        bp_h,
                        bp_cr,
                        bp_rx,
                        bp_ry,
                        bp_vert_id,
                        bp_prim_id,
                        bp_has_backup_path,
                        bp_data_attr,
                        vertex_cache,
                        primitive_cache,
                    )? {
                        backup_path_shape = Some(bp);
                    }
                    in_backup_path = false;
                    depth -= 1; // BackupPath is handled, adjust depth
                } else if tag == "Shape" {
                    // Collect child shape attributes
                    let mut child_shape_type = String::new();
                    let mut child_cut_index: i32 = 0;
                    let mut child_w: f64 = 0.0;
                    let mut child_h: f64 = 0.0;
                    let mut child_cr: f64 = 0.0;
                    let mut child_rx: f64 = 0.0;
                    let mut child_ry: f64 = 0.0;
                    let mut child_vert_id: Option<i32> = None;
                    let mut child_prim_id: Option<i32> = None;
                    let mut child_has_backup_path = false;
                    let mut child_data_attr = String::new();

                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                        let value = std::str::from_utf8(&attr.value).unwrap_or("");
                        match key {
                            "Type" => child_shape_type = value.to_string(),
                            "CutIndex" => child_cut_index = value.parse().unwrap_or(0),
                            "W" => child_w = value.parse().unwrap_or(0.0),
                            "H" => child_h = value.parse().unwrap_or(0.0),
                            "Cr" => child_cr = value.parse().unwrap_or(0.0),
                            "Rx" => child_rx = value.parse().unwrap_or(0.0),
                            "Ry" => child_ry = value.parse().unwrap_or(0.0),
                            "VertID" => child_vert_id = value.parse().ok(),
                            "PrimID" => child_prim_id = value.parse().ok(),
                            "HasBackupPath" => child_has_backup_path = value == "1",
                            "Data" => child_data_attr = value.to_string(),
                            _ => {}
                        }
                    }

                    if let Some(child) = parse_shape_inner(
                        reader,
                        child_shape_type,
                        child_cut_index,
                        child_w,
                        child_h,
                        child_cr,
                        child_rx,
                        child_ry,
                        child_vert_id,
                        child_prim_id,
                        child_has_backup_path,
                        child_data_attr,
                        vertex_cache,
                        primitive_cache,
                    )? {
                        if in_backup_path {
                            backup_path_shape = Some(child);
                        } else if in_children {
                            children.push(child);
                        }
                    }
                    depth -= 1;
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag_bytes = e.name();
                let tag = std::str::from_utf8(tag_bytes.as_ref()).unwrap_or("");
                if tag == "Shape"
                    && let Some(child) = parse_shape_from_empty_element(e)?
                {
                    if in_backup_path {
                        backup_path_shape = Some(child);
                    } else if in_children {
                        children.push(child);
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                match current_tag.as_str() {
                    "XForm" => xform = parse_xform(&text),
                    "VertList" => vert_list = text,
                    "PrimList" => prim_list = text,
                    "Data" => data = text,
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                depth -= 1;
                let tag_bytes = e.name();
                let tag = std::str::from_utf8(tag_bytes.as_ref()).unwrap_or("");
                if tag == "Children" {
                    in_children = false;
                } else if tag == "BackupPath" {
                    in_backup_path = false;
                }
                current_tag.clear();
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing Shape: {:?}", e)),
            _ => {}
        }
        buf.clear();
    }

    // If this is a Text with BackupPath, use the backup path shape instead
    if shape_type == "Text"
        && has_backup_path
        && let Some(bp) = backup_path_shape
    {
        return Ok(Some(bp));
    }

    // Handle VertID/PrimID caching
    let resolved_verts: Vec<Vec2>;
    let resolved_vert_list: String;
    let resolved_prims: Vec<PathPrimitive>;
    let resolved_prim_list: String;

    if !vert_list.is_empty() {
        resolved_verts = parse_vert_list(&vert_list);
        resolved_vert_list = vert_list.clone();
        if let Some(vid) = vert_id {
            vertex_cache.insert(vid, (vert_list, resolved_verts.clone()));
        }
    } else if let Some(vid) = vert_id {
        if let Some((vl, verts)) = vertex_cache.get(&vid) {
            resolved_vert_list = vl.clone();
            resolved_verts = verts.clone();
        } else {
            eprintln!("Vertex data for VertID={} not found in cache", vid);
            resolved_verts = Vec::new();
            resolved_vert_list = String::new();
        }
    } else {
        resolved_verts = Vec::new();
        resolved_vert_list = String::new();
    }

    if !prim_list.is_empty() {
        resolved_prims = if prim_list == "LineClosed" {
            Vec::new()
        } else {
            parse_prim_list(&prim_list)
        };
        resolved_prim_list = prim_list.clone();
        if let Some(pid) = prim_id {
            primitive_cache.insert(pid, (prim_list, resolved_prims.clone()));
        }
    } else if let Some(pid) = prim_id {
        if let Some((pl, prims)) = primitive_cache.get(&pid) {
            resolved_prim_list = pl.clone();
            resolved_prims = prims.clone();
        } else {
            eprintln!("Primitive data for PrimID={} not found in cache", pid);
            resolved_prims = Vec::new();
            resolved_prim_list = String::new();
        }
    } else {
        resolved_prims = Vec::new();
        resolved_prim_list = String::new();
    }

    // Create the shape based on type
    match shape_type.as_str() {
        "Rect" => Ok(Some(Shape::Rect(Rect {
            cut_index,
            xform,
            w,
            h,
            cr,
        }))),
        "Ellipse" => Ok(Some(Shape::Ellipse(Ellipse {
            cut_index,
            xform,
            rx,
            ry,
        }))),
        "Path" => {
            if resolved_verts.is_empty() {
                eprintln!("Path shape has no vertices after resolution, skipping");
                return Ok(None);
            }
            Ok(Some(Shape::Path(Path {
                cut_index,
                xform,
                vert_list: resolved_vert_list,
                prim_list: resolved_prim_list,
                parsed_verts: resolved_verts,
                parsed_primitives: resolved_prims,
            })))
        }
        "Bitmap" => Ok(Some(Shape::Bitmap(Bitmap {
            cut_index,
            xform,
            w,
            h,
            data,
        }))),
        "Group" => {
            if children.is_empty() {
                return Ok(None);
            }
            Ok(Some(Shape::Group(Group {
                cut_index,
                xform,
                children,
            })))
        }
        _ => Ok(None),
    }
}

fn parse_shape_from_empty_element(
    e: &quick_xml::events::BytesStart,
) -> Result<Option<Shape>, String> {
    let mut shape_type = String::new();
    let mut cut_index: i32 = 0;
    let mut w: f64 = 0.0;
    let mut h: f64 = 0.0;
    let mut cr: f64 = 0.0;
    let mut rx: f64 = 0.0;
    let mut ry: f64 = 0.0;

    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
        let value = std::str::from_utf8(&attr.value).unwrap_or("");
        match key {
            "Type" => shape_type = value.to_string(),
            "CutIndex" => cut_index = value.parse().unwrap_or(0),
            "W" => w = value.parse().unwrap_or(0.0),
            "H" => h = value.parse().unwrap_or(0.0),
            "Cr" => cr = value.parse().unwrap_or(0.0),
            "Rx" => rx = value.parse().unwrap_or(0.0),
            "Ry" => ry = value.parse().unwrap_or(0.0),
            _ => {}
        }
    }

    let xform = XForm::identity();

    match shape_type.as_str() {
        "Rect" => Ok(Some(Shape::Rect(Rect {
            cut_index,
            xform,
            w,
            h,
            cr,
        }))),
        "Ellipse" => Ok(Some(Shape::Ellipse(Ellipse {
            cut_index,
            xform,
            rx,
            ry,
        }))),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xform() {
        let xform = parse_xform("1 0 0 1 55 55");
        assert_eq!(xform.a, 1.0);
        assert_eq!(xform.b, 0.0);
        assert_eq!(xform.c, 0.0);
        assert_eq!(xform.d, 1.0);
        assert_eq!(xform.e, 55.0);
        assert_eq!(xform.f, 55.0);
    }

    #[test]
    fn test_parse_vert_list_simple() {
        let verts = parse_vert_list("V49 48V62 63");
        assert_eq!(verts.len(), 2);
        assert_eq!(verts[0].x, 49.0);
        assert_eq!(verts[0].y, 48.0);
        assert_eq!(verts[1].x, 62.0);
        assert_eq!(verts[1].y, 63.0);
    }

    #[test]
    fn test_parse_vert_list_with_control_points() {
        let verts = parse_vert_list("V49 48c0x1c1x49c1y48V62 63c0x62c0y63c1x1");
        assert_eq!(verts.len(), 2);
        assert_eq!(verts[0].x, 49.0);
        assert_eq!(verts[0].y, 48.0);
        assert_eq!(verts[0].c0x, Some(1.0));
        assert_eq!(verts[0].c1x, Some(49.0));
        assert_eq!(verts[0].c1y, Some(48.0));
        assert_eq!(verts[1].x, 62.0);
        assert_eq!(verts[1].y, 63.0);
        assert_eq!(verts[1].c0x, Some(62.0));
        assert_eq!(verts[1].c0y, Some(63.0));
        assert_eq!(verts[1].c1x, Some(1.0));
    }

    #[test]
    fn test_parse_prim_list() {
        let prims = parse_prim_list("L0 1");
        assert_eq!(prims.len(), 1);
        assert!(matches!(
            prims[0],
            PathPrimitive::Line {
                start_idx: 0,
                end_idx: 1
            }
        ));
    }

    #[test]
    fn test_parse_prim_list_bezier() {
        let prims = parse_prim_list("B0 1 L1 2");
        assert_eq!(prims.len(), 2);
        assert!(matches!(
            prims[0],
            PathPrimitive::Bezier {
                start_idx: 0,
                end_idx: 1
            }
        ));
        assert!(matches!(
            prims[1],
            PathPrimitive::Line {
                start_idx: 1,
                end_idx: 2
            }
        ));
    }

    #[test]
    fn test_parse_circle() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<LightBurnProject AppVersion="1.7.08" FormatVersion="1">
  <Shape Type="Ellipse" CutIndex="0" Rx="5" Ry="5">
    <XForm>1 0 0 1 55 55</XForm>
  </Shape>
</LightBurnProject>"#;

        let project = parse_lbrn2_complete(xml).unwrap();
        assert_eq!(project.shapes.len(), 1);
        match &project.shapes[0] {
            Shape::Ellipse(e) => {
                assert_eq!(e.rx, 5.0);
                assert_eq!(e.ry, 5.0);
                assert_eq!(e.xform.e, 55.0);
                assert_eq!(e.xform.f, 55.0);
            }
            _ => panic!("Expected Ellipse"),
        }
    }
}
