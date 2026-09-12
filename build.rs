//! Generates src/kitty_ss.rs's and src/title_ss.rs's spritesheet data from
//! their source PNGs (kitty-ss.png and kitty_title.png) via WASM-4's
//! `png2src` tool, so the compiled game always reflects whatever is
//! currently in those PNGs -- there's no separate "regenerate the source"
//! step to forget. See README.md's "Assets" section for the details this
//! automates.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// (source PNG, generated file written to OUT_DIR, filename png2src should
/// see). png2src derives its Rust constant names from the input filename,
/// and title_ss.rs's constants (OUTPUT_ONLINEPNGTOOLS_*) were first named
/// after a PNG called output_onlinepngtools.png, not kitty_title.png's
/// current name -- so we feed png2src a renamed copy to keep reproducing
/// those same names without having to touch every place they're used.
const SPRITESHEETS: &[(&str, &str, &str)] = &[
    ("kitty-ss.png", "kitty_ss.rs", "kitty-ss.png"),
    (
        "kitty_title.png",
        "title_ss.rs",
        "output_onlinepngtools.png",
    ),
];

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    for (png_name, out_name, png2src_input_name) in SPRITESHEETS {
        let png_path = manifest_dir.join(png_name);
        println!("cargo:rerun-if-changed={}", png_path.display());

        let source_png = if png2src_input_name == png_name {
            png_path.clone()
        } else {
            let renamed = out_dir.join(png2src_input_name);
            fs::copy(&png_path, &renamed).unwrap_or_else(|e| {
                panic!(
                    "copying {} to {}: {e}",
                    png_path.display(),
                    renamed.display()
                )
            });
            renamed
        };

        let dest = out_dir.join(out_name);
        let status = Command::new("npx")
            .args(["--yes", "-p", "wasm4", "w4", "png2src", "--rust"])
            .arg(&source_png)
            .arg("--output")
            .arg(&dest)
            .status()
            .unwrap_or_else(|e| panic!("running npx w4 png2src on {}: {e}", source_png.display()));
        assert!(
            status.success(),
            "png2src failed for {}",
            source_png.display()
        );

        // png2src emits private `const`s; the rest of the crate expects `pub`.
        let generated = fs::read_to_string(&dest).unwrap();
        let patched: String = generated
            .lines()
            .map(|line| {
                if let Some(rest) = line.strip_prefix("const ") {
                    format!("pub const {rest}")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&dest, patched).unwrap();
    }
}
