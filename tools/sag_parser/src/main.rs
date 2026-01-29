//! CLI tool to convert SAG files to GNU LD linker scripts
//!
//! Usage:
//!   sag2ld input.sag -o output.ld [--config ddr|ilm|xip]

use sag_parser::{LinkerScriptConfig, SagFile};
use std::env;
use std::fs;
use std::process;

fn print_usage() {
    eprintln!(
        r#"sag2ld - Convert Andes SAG files to GNU LD linker scripts

USAGE:
    sag2ld <input.sag> [OPTIONS]

OPTIONS:
    -o, --output <file>     Output linker script path (default: stdout)
    -c, --config <name>     Memory config preset: ddr, ilm (default: ddr)
    -p, --print-ast         Print parsed AST instead of linker script
    -h, --help              Show this help message

EXAMPLES:
    sag2ld ae350-ddr.sag -o memory.x
    sag2ld ae350-ilm.sag --config ilm -o memory.x
    sag2ld ae350-ddr.sag --print-ast
"#
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let mut input_path: Option<&str> = None;
    let mut output_path: Option<&str> = None;
    let mut config_name = "ddr";
    let mut print_ast = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_usage();
                process::exit(0);
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --output requires a path");
                    process::exit(1);
                }
                output_path = Some(&args[i]);
            }
            "-c" | "--config" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --config requires a name (ddr, ilm)");
                    process::exit(1);
                }
                config_name = &args[i];
            }
            "-p" | "--print-ast" => {
                print_ast = true;
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: Unknown option: {}", arg);
                print_usage();
                process::exit(1);
            }
            arg => {
                input_path = Some(arg);
            }
        }
        i += 1;
    }

    let input_path = match input_path {
        Some(p) => p,
        None => {
            eprintln!("Error: No input file specified");
            print_usage();
            process::exit(1);
        }
    };

    // Parse the SAG file
    let sag = match SagFile::from_file(input_path) {
        Ok(sag) => sag,
        Err(e) => {
            eprintln!("Error parsing {}: {}", input_path, e);
            process::exit(1);
        }
    };

    if print_ast {
        println!("{:#?}", sag);
        return;
    }

    // Select config
    let config = match config_name {
        "ddr" => LinkerScriptConfig::ae350_ddr(),
        "ilm" => LinkerScriptConfig::ae350_ilm(),
        other => {
            eprintln!("Error: Unknown config '{}'. Use 'ddr' or 'ilm'.", other);
            process::exit(1);
        }
    };

    // Generate linker script
    let linker_script = sag.to_linker_script(&config);

    // Output
    match output_path {
        Some(path) => {
            if let Err(e) = fs::write(path, &linker_script) {
                eprintln!("Error writing {}: {}", path, e);
                process::exit(1);
            }
            eprintln!("Wrote linker script to {}", path);
        }
        None => {
            print!("{}", linker_script);
        }
    }
}
