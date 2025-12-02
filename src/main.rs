use clap::{Parser, Subcommand};
use laser_tools::lbrn2::{lbrn2_to_svg, parse_lbrn2};
use laser_tools::vectorize::{VectorizeOptions, vectorize_image_file};
use std::fs;
use std::process;

#[derive(Parser)]
#[command(name = "laser-tools")]
#[command(author, version, about = "CLI tools for laser cutting file conversions", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert LightBurn LBRN2 files to SVG
    Lbrn2 {
        /// Input LBRN2 file path
        input: String,
        /// Output SVG file path
        output: String,
    },
    /// Convert raster images to SVG with cut/engrave layers
    #[command(name = "image")]
    Image {
        /// Input image file path (PNG, JPEG)
        input: String,
        /// Output SVG file path
        output: String,
        /// Scale factor for tracing quality (default: 2)
        #[arg(short, long, default_value = "2")]
        scale: u32,
        /// Filter speckle size - removes noise smaller than this (default: 4)
        #[arg(short, long, default_value = "4")]
        filter_speckle: usize,
        /// Corner threshold for path simplification (default: 60)
        #[arg(short, long, default_value = "60")]
        corner_threshold: i32,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lbrn2 { input, output } => {
            run_lbrn2_conversion(&input, &output);
        }
        Commands::Image {
            input,
            output,
            scale,
            filter_speckle,
            corner_threshold,
        } => {
            run_image_vectorization(&input, &output, scale, filter_speckle, corner_threshold);
        }
    }
}

fn run_lbrn2_conversion(input_path: &str, output_path: &str) {
    let lbrn2_content = match fs::read_to_string(input_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading input file '{}': {}", input_path, e);
            process::exit(2);
        }
    };

    let project = match parse_lbrn2(&lbrn2_content) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error parsing LBRN2 file: {}", e);
            process::exit(3);
        }
    };

    let svg = lbrn2_to_svg(&project);

    match fs::write(output_path, &svg) {
        Ok(_) => {
            println!(
                "Successfully converted '{}' to '{}'",
                input_path, output_path
            );
        }
        Err(e) => {
            eprintln!("Error writing output file '{}': {}", output_path, e);
            process::exit(4);
        }
    }
}

fn run_image_vectorization(
    input_path: &str,
    output_path: &str,
    scale: u32,
    filter_speckle: usize,
    corner_threshold: i32,
) {
    let options = VectorizeOptions {
        scale_factor: scale,
        filter_speckle,
        corner_threshold,
        path_precision: 3,
    };

    let result = match vectorize_image_file(input_path, Some(options)) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error vectorizing image '{}': {}", input_path, e);
            process::exit(3);
        }
    };

    match fs::write(output_path, &result.svg) {
        Ok(_) => {
            println!(
                "Successfully vectorized '{}' to '{}' ({}x{})",
                input_path, output_path, result.width, result.height
            );
        }
        Err(e) => {
            eprintln!("Error writing output file '{}': {}", output_path, e);
            process::exit(4);
        }
    }
}
