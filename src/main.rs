use lbrn2_to_svg::{lbrn2_to_svg, parse_lbrn2};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input.lbrn2> <output.svg>", args[0]);
        process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

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
