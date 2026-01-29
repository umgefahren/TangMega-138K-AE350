//! Example build.rs for using sag_parser in an embedded Rust project
//!
//! Add to your Cargo.toml:
//! ```toml
//! [build-dependencies]
//! sag_parser = { path = "../tools/sag_parser" }
//! ```

use sag_parser::{LinkerScriptConfig, SagFile};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Tell Cargo to re-run if the SAG file changes
    println!("cargo:rerun-if-changed=memory.sag");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Parse SAG file
    let sag = SagFile::from_file("memory.sag").expect("Failed to parse SAG file");

    // Generate linker script
    let config = LinkerScriptConfig::ae350_ddr();
    let linker_script = sag.to_linker_script(&config);

    // Write to OUT_DIR
    let linker_path = out_dir.join("memory.x");
    fs::write(&linker_path, linker_script).expect("Failed to write linker script");

    // Tell the linker where to find the script
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Also copy to the project root for debugging
    fs::copy(&linker_path, "target/memory.x").ok();
}
