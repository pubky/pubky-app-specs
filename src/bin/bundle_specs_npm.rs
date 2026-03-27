use serde_json::{json, Value};
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

    fs::write(pkg_dir.join("validationLimits.json"), format!("{json}\n"))?;
    fs::write(pkg_dir.join("validationLimits.js"), validation_limits_esm())?;
    fs::write(
        pkg_dir.join("validationLimits.cjs"),
        validation_limits_cjs(),
    )?;

    update_package_json(&pkg_dir)?;

    Ok(())
}

fn validation_limits_esm() -> &'static str {
    "import limits from \"./validationLimits.json\" assert { type: \"json\" };\n\
\n\
export const validationLimits = limits;\n\
export const getValidationLimits = () => JSON.parse(JSON.stringify(limits));\n\
export default limits;\n"
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

fn update_package_json(pkg_dir: &Path) -> io::Result<()> {
    let package_json_path = pkg_dir.join("package.json");
    let package_json = fs::read_to_string(&package_json_path)?;
    let mut package: Value = serde_json::from_str(&package_json)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

    ensure_files(&mut package);
    ensure_exports(&mut package);

    let updated = serde_json::to_string_pretty(&package)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    fs::write(package_json_path, format!("{updated}\n"))?;
    Ok(())
}

fn ensure_files(package: &mut Value) {
    if !package.get("files").is_some() {
        package["files"] = Value::Array(Vec::new());
    }

    let files = package
        .get_mut("files")
        .and_then(Value::as_array_mut)
        .expect("files array");

    for entry in [
        "validationLimits.json",
        "validationLimits.js",
        "validationLimits.cjs",
    ] {
        if !files.iter().any(|value| value.as_str() == Some(entry)) {
            files.push(Value::String(entry.to_string()));
        }
    }
}

fn ensure_exports(package: &mut Value) {
    if !package.get("exports").is_some() {
        package["exports"] = Value::Object(serde_json::Map::new());
    }

    let exports = package
        .get_mut("exports")
        .and_then(Value::as_object_mut)
        .expect("exports object");

    exports
        .entry("./validationLimits".to_string())
        .or_insert_with(|| {
            json!({
                "import": "./validationLimits.js",
                "require": "./validationLimits.cjs"
            })
        });

    exports
        .entry("./validationLimits.json".to_string())
        .or_insert_with(|| Value::String("./validationLimits.json".to_string()));
}
