//! LBRN2 to SVG conversion module
//!
//! This module provides functionality to parse LightBurn LBRN2 project files
//! and convert them to SVG format.

pub mod bounds;
pub mod parser;
pub mod path;
pub mod style;
pub mod svg;
pub mod types;

// Re-export main public API
pub use parser::{
    parse_lbrn2_complete as parse_lbrn2, parse_prim_list, parse_vert_list, parse_xform,
};
pub use svg::lbrn2_to_svg;
pub use types::*;
