//! # lbrn2-to-svg
//!
//! A Rust library for converting LightBurn LBRN2 project files to SVG format.
//!
//! This library provides functionality to parse LBRN2 XML files (used by LightBurn
//! laser cutting software) and convert them to standard SVG format.
//!
//! ## Example
//!
//! ```rust,ignore
//! use lbrn2_to_svg::{parse_lbrn2, lbrn2_to_svg};
//!
//! let lbrn2_content = std::fs::read_to_string("example.lbrn2").unwrap();
//! let project = parse_lbrn2(&lbrn2_content).unwrap();
//! let svg = lbrn2_to_svg(&project);
//! std::fs::write("output.svg", svg).unwrap();
//! ```

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
