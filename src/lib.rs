//! # laser-tools
//!
//! A Rust library for laser cutting file conversions and image vectorization.
//!
//! ## Features
//!
//! - **LBRN2 to SVG**: Convert LightBurn LBRN2 project files to SVG format
//! - **Image Vectorization**: Convert raster images to SVG with separate cut/engrave layers
//!
//! ## Example - LBRN2 Conversion
//!
//! ```rust,ignore
//! use laser_tools::lbrn2::{parse_lbrn2, lbrn2_to_svg};
//!
//! let lbrn2_content = std::fs::read_to_string("example.lbrn2").unwrap();
//! let project = parse_lbrn2(&lbrn2_content).unwrap();
//! let svg = lbrn2_to_svg(&project);
//! std::fs::write("output.svg", svg).unwrap();
//! ```
//!
//! ## Example - Image Vectorization
//!
//! ```rust,ignore
//! use laser_tools::vectorize::{vectorize_image_file, VectorizeOptions};
//!
//! let result = vectorize_image_file("input.png", None).unwrap();
//! std::fs::write("output.svg", result.svg).unwrap();
//! ```

pub mod lbrn2;
pub mod vectorize;

// Re-export commonly used items
pub use lbrn2::{LightBurnProject, lbrn2_to_svg, parse_lbrn2};
pub use vectorize::{VectorizeOptions, VectorizeResult, vectorize_image, vectorize_image_file};
