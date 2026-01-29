//! Build script that converts SAG to linker script for Rust

use sag_parser::{LinkerScriptConfig, SagFile};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Path to the SAG file in the Andes project
    let sag_path = manifest_dir.join("../Andes/src/bsp/sag/ae350-ddr.sag");

    println!("cargo:rerun-if-changed={}", sag_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=link.x");

    // Parse SAG and generate linker script
    let sag = match SagFile::from_file(&sag_path) {
        Ok(s) => s,
        Err(e) => {
            // Fall back to a basic memory.x if SAG parsing fails
            eprintln!("Warning: Could not parse SAG file: {}", e);
            eprintln!("Using fallback memory layout");
            write_fallback_memory_x(&out_dir);
            return;
        }
    };

    let config = LinkerScriptConfig::ae350_ddr();
    let linker_script = sag.to_linker_script(&config);

    // Write memory.x
    let memory_x_path = out_dir.join("memory.x");
    fs::write(&memory_x_path, &linker_script).expect("Failed to write memory.x");

    // Tell Cargo where to find linker scripts
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Also look in the project root for link.x
    println!("cargo:rustc-link-search={}", manifest_dir.display());
}

fn write_fallback_memory_x(out_dir: &PathBuf) {
    let fallback = r#"
/* Fallback memory layout for AE350 DDR mode */
MEMORY
{
    FLASH (rx)  : ORIGIN = 0x80000000, LENGTH = 256M
    RAM (rwx)   : ORIGIN = 0x00000000, LENGTH = 128M
}

REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);

_stack_start = ORIGIN(RAM) + LENGTH(RAM);
"#;
    fs::write(out_dir.join("memory.x"), fallback).expect("Failed to write fallback memory.x");
    println!("cargo:rustc-link-search={}", out_dir.display());
}
