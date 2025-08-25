use std::env;
use std::io;
use std::process::{Command, ExitStatus};

// If the process hangs, try `cargo clean` to remove all locks.

fn main() {
    println!("🏗️ Building wasm for pubky-app-specs...");

    build_wasm("nodejs").unwrap();
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
            &format!("bindings/js/dist/{}", target),
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
