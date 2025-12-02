use super::types::CutSetting;

const DEFAULT_COLORS: [&str; 8] = [
    "#000000", "#FF0000", "#00AA00", "#0000FF", "#FF9900", "#9900FF", "#00AAAA", "#AAAA00",
];

/// Get the SVG style string for a given cut index
pub fn get_cut_setting_style(cut_index: i32, cut_settings: Option<&[CutSetting]>) -> String {
    let cut_settings = match cut_settings {
        Some(cs) if !cs.is_empty() => cs,
        _ => return "stroke:#000000;stroke-width:0.050000mm;fill:none".to_string(),
    };

    let cs = cut_settings.iter().find(|cs| cs.index == cut_index);

    let color = match cs {
        Some(cs) if cs.color.is_some() => cs.color.as_ref().unwrap().clone(),
        Some(cs) => {
            let palette_idx = if cs.index >= 0 {
                (cs.index as usize) % DEFAULT_COLORS.len()
            } else {
                0
            };
            DEFAULT_COLORS[palette_idx].to_string()
        }
        None => "#000000".to_string(),
    };

    let stroke_width = cs
        .and_then(|cs| cs.stroke_width.as_ref())
        .cloned()
        .unwrap_or_else(|| "0.050000mm".to_string());

    format!("stroke:{};stroke-width:{};fill:none", color, stroke_width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_style_undefined() {
        assert_eq!(
            get_cut_setting_style(0, None),
            "stroke:#000000;stroke-width:0.050000mm;fill:none"
        );
    }

    #[test]
    fn test_default_style_empty() {
        assert_eq!(
            get_cut_setting_style(0, Some(&[])),
            "stroke:#000000;stroke-width:0.050000mm;fill:none"
        );
    }

    #[test]
    fn test_style_with_color_and_stroke_width() {
        let cs = vec![CutSetting {
            index: 1,
            name: "cut1".to_string(),
            color: Some("#123456".to_string()),
            stroke_width: Some("0.2mm".to_string()),
        }];
        assert_eq!(
            get_cut_setting_style(1, Some(&cs)),
            "stroke:#123456;stroke-width:0.2mm;fill:none"
        );
    }

    #[test]
    fn test_style_with_color_default_stroke_width() {
        let cs = vec![CutSetting {
            index: 2,
            name: "cut2".to_string(),
            color: Some("#654321".to_string()),
            stroke_width: None,
        }];
        assert_eq!(
            get_cut_setting_style(2, Some(&cs)),
            "stroke:#654321;stroke-width:0.050000mm;fill:none"
        );
    }

    #[test]
    fn test_palette_color_if_color_missing() {
        let cs = vec![CutSetting {
            index: 3,
            name: "cut3".to_string(),
            color: None,
            stroke_width: None,
        }];
        // DEFAULT_COLORS[3] = "#0000FF"
        assert_eq!(
            get_cut_setting_style(3, Some(&cs)),
            "stroke:#0000FF;stroke-width:0.050000mm;fill:none"
        );
    }

    #[test]
    fn test_palette_color_custom_stroke_width() {
        let cs = vec![CutSetting {
            index: 4,
            name: "cut4".to_string(),
            color: None,
            stroke_width: Some("0.3mm".to_string()),
        }];
        // DEFAULT_COLORS[4] = "#FF9900"
        assert_eq!(
            get_cut_setting_style(4, Some(&cs)),
            "stroke:#FF9900;stroke-width:0.3mm;fill:none"
        );
    }

    #[test]
    fn test_negative_index() {
        let cs = vec![CutSetting {
            index: -1,
            name: "cut5".to_string(),
            color: None,
            stroke_width: None,
        }];
        // Should fallback to DEFAULT_COLORS[0]
        assert_eq!(
            get_cut_setting_style(-1, Some(&cs)),
            "stroke:#000000;stroke-width:0.050000mm;fill:none"
        );
    }

    #[test]
    fn test_no_matching_cut_setting() {
        let cs = vec![CutSetting {
            index: 0,
            name: "cut6".to_string(),
            color: Some("#111111".to_string()),
            stroke_width: None,
        }];
        assert_eq!(
            get_cut_setting_style(99, Some(&cs)),
            "stroke:#000000;stroke-width:0.050000mm;fill:none"
        );
    }
}
