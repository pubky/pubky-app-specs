use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus};

// If the process hangs, try `cargo clean` to remove all locks.

fn main() {
    println!("🏗️ Building wasm for pubky-social-specs...");

    // A failed step must fail the build, or a stale glue from an earlier run ships
    check("wasm-pack", build_wasm("nodejs"));
    write_data_assets().unwrap();
    check("patch.mjs", patch());
    println!("📦 Pubky-social-specs JS binding package built successfully!");
}

fn check(step: &str, status: io::Result<ExitStatus>) {
    match status {
        Ok(status) if status.success() => {}
        Ok(status) => {
            eprintln!("{step} failed: {status}");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("{step} did not run: {e}");
            std::process::exit(1);
        }
    }
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
            // The package is what the browser migrator runs, so it ships the transforms
            "--",
            "--features",
            "migrator",
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

    println!(
        "🩹 Lazy-loading glue and the CommonJS entry from {manifest_dir}/src/bin/patch.mjs ..."
    );

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

fn write_data_assets() -> io::Result<()> {
    let limits =
        serde_json::to_value(pubky_social_specs::VALIDATION_LIMITS).map_err(io::Error::other)?;
    write_data_asset(
        "validationLimits",
        &limits,
        &[("validationLimits", "data")],
        VALIDATION_LIMITS_DTS,
    )?;

    let table: BTreeMap<&str, &str> = pubky_social_specs::MIME_TO_EXT.iter().copied().collect();
    let mime = serde_json::json!({
        "validMimeTypes": pubky_social_specs::VALID_MIME_TYPES,
        "mimeToExtTable": table,
    });
    write_data_asset(
        "mimeTypes",
        &mime,
        &[
            ("validMimeTypes", "data.validMimeTypes"),
            ("mimeToExtTable", "data.mimeToExtTable"),
        ],
        MIME_TYPES_DTS,
    )?;

    let reasons: Vec<&str> = pubky_social_specs::migrate::Skip::ALL
        .iter()
        .map(|skip| skip.as_str())
        .collect();
    let dts = skip_reasons_dts(&reasons);
    write_data_asset(
        "skipReasons",
        &serde_json::json!(reasons),
        &[("skipReasons", "data")],
        &dts,
    )
}

/// The union is spelled from the same list as the data, so the type and the values cannot
/// drift apart.
fn skip_reasons_dts(reasons: &[&str]) -> String {
    let union = reasons
        .iter()
        .map(|reason| format!("\"{reason}\""))
        .collect::<Vec<_>>()
        .join(" | ");
    format!(
        "/** Why one 0.x object did not migrate. */\nexport type SkipReason = {union};\n/** Every reason, frozen, so a report counts categories without a table of its own. */\nexport declare const skipReasons: readonly SkipReason[];\ndeclare const data: readonly SkipReason[];\nexport default data;\n"
    )
}

/// One JSON source and its `.js`, `.cjs` and `.d.ts` twins, readable without the wasm.
/// `named` maps each export to an expression over `data`.
fn write_data_asset(
    name: &str,
    value: &serde_json::Value,
    named: &[(&str, &str)],
    dts: &str,
) -> io::Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let pkg_dir = Path::new(&manifest_dir).join("pkg");
    fs::create_dir_all(&pkg_dir)?;
    let json = serde_json::to_string_pretty(value).map_err(io::Error::other)?;

    let esm_exports: String = named
        .iter()
        .map(|(export, expr)| format!("export const {export} = {expr};\n"))
        .collect();
    let cjs_exports: String = named
        .iter()
        .map(|(export, expr)| format!("  {export}: {expr},\n"))
        .collect();

    fs::write(pkg_dir.join(format!("{name}.json")), format!("{json}\n"))?;
    fs::write(
        pkg_dir.join(format!("{name}.js")),
        format!("{FREEZE}\nconst data = freeze({json});\n\n{esm_exports}export default data;\n"),
    )?;
    fs::write(
        pkg_dir.join(format!("{name}.cjs")),
        format!(
            "{FREEZE}\nconst data = freeze({json});\n\nmodule.exports = {{\n{cjs_exports}  default: data,\n}};\n"
        ),
    )?;
    fs::write(pkg_dir.join(format!("{name}.d.ts")), dts)?;
    Ok(())
}

// Frozen all the way down: the object is shared by every importer, so one caller's edit
// would move every other caller's values.
const FREEZE: &str = r#"const freeze = (value) => {
  if (value !== null && typeof value === "object") {
    Object.values(value).forEach(freeze);
    Object.freeze(value);
  }
  return value;
};
"#;

const VALIDATION_LIMITS_DTS: &str = r#"import type { ValidationLimits } from "./pubky_social_specs.js";

type DeepReadonly<T> = {
  readonly [K in keyof T]: T[K] extends (infer U)[] ? readonly U[] : T[K];
};

/** Every cap by name, frozen. */
export declare const validationLimits: DeepReadonly<ValidationLimits>;
declare const data: DeepReadonly<ValidationLimits>;
export default data;
"#;

const MIME_TYPES_DTS: &str = r#"/** A file picker hint; it gates nothing. Frozen. */
export declare const validMimeTypes: readonly string[];
/** The frozen map from a declared type's essence to the path extension. */
export declare const mimeToExtTable: Readonly<Record<string, string>>;
declare const data: {
  readonly validMimeTypes: readonly string[];
  readonly mimeToExtTable: Readonly<Record<string, string>>;
};
export default data;
"#;
