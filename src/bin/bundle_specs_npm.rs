use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus};

// If the process hangs, try `cargo clean` to remove all locks.

fn main() {
    println!("🏗️ Building wasm for pubky-social-specs...");

    build_wasm("nodejs").unwrap();
    write_validation_limits_assets().unwrap();
    patch().unwrap();
    println!("📦 Pubky-social-specs JS binding package built successfully!");
}

fn build_wasm(target: &str) -> io::Result<ExitStatus> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let output = Command::new("wasm-pack")
        .args([
            "build",
            &manifest_dir,
            "--release",
            "--target",
            target,
            "--out-dir",
            &format!("pkg/{}", target),
        ])
        .output()?;

    if !output.status.success() {
        eprintln!(
            "wasm-pack failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output.status)
}

fn patch() -> io::Result<ExitStatus> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    println!("🩹 Applying patch to generate isomorphic code for web and nodejs from {manifest_dir}/src/bin/patch.mjs ...");

    let output = Command::new("node")
        .args([format!("{manifest_dir}/src/bin/patch.mjs")])
        .output()?;

    if !output.status.success() {
        eprintln!(
            "patch.mjs failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output.status)
}

fn write_validation_limits_assets() -> io::Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let pkg_dir = Path::new(&manifest_dir).join("pkg");
    fs::create_dir_all(&pkg_dir)?;

    let json = serde_json::to_string_pretty(&pubky_social_specs::VALIDATION_LIMITS)
        .map_err(io::Error::other)?;

    fs::write(pkg_dir.join("validationLimits.json"), format!("{json}\n"))?;
    fs::write(
        pkg_dir.join("validationLimits.js"),
        validation_limits_esm(&json),
    )?;
    fs::write(
        pkg_dir.join("validationLimits.cjs"),
        validation_limits_cjs(),
    )?;

    Ok(())
}

fn validation_limits_esm(json: &str) -> String {
    format!(
        "const limits = {json};\n\
\n\
export const validationLimits = limits;\n\
export const getValidationLimits = () => JSON.parse(JSON.stringify(limits));\n\
export default limits;\n"
    )
}

fn validation_limits_cjs() -> &'static str {
    "const limits = require(\"./validationLimits.json\");\n\
\n\
const clone = () => JSON.parse(JSON.stringify(limits));\n\
\n\
module.exports = {\n\
  validationLimits: limits,\n\
  getValidationLimits: clone,\n\
  default: limits,\n\
};\n"
}
