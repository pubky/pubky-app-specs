use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus};

// If the process hangs, try `cargo clean` to remove all locks.

fn main() {
    println!("🏗️ Building wasm for pubky-app-specs...");

    build_wasm("nodejs").unwrap();
    write_validation_limits_assets().unwrap();
    patch().unwrap();
    println!("📦 Pubky-app-specs JS binding package built successfully!");
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

    let json = serde_json::to_string_pretty(&pubky_app_specs::VALIDATION_LIMITS)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::write(pkg_dir.join("validationLimits.json"), &json)?;

    let js = format!(
        "export const validationLimits = {json};\n\
export const getValidationLimits = () => ({{\n\
  ...validationLimits,\n\
  tagInvalidChars: [...validationLimits.tagInvalidChars],\n\
  postAllowedAttachmentProtocols: [...validationLimits.postAllowedAttachmentProtocols],\n\
}});\n"
    );
    fs::write(pkg_dir.join("validationLimits.js"), js)?;

    let cjs = format!(
        "const validationLimits = {json};\n\
const getValidationLimits = () => ({{\n\
  ...validationLimits,\n\
  tagInvalidChars: [...validationLimits.tagInvalidChars],\n\
  postAllowedAttachmentProtocols: [...validationLimits.postAllowedAttachmentProtocols],\n\
}});\n\
module.exports = {{ validationLimits, getValidationLimits }};\n"
    );
    fs::write(pkg_dir.join("validationLimits.cjs"), cjs)?;

    Ok(())
}
