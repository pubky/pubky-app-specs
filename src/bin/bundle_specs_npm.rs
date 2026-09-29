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
    check("patch.mjs", patch(None));
    check("tsc", compile_migration());
    check("patch.mjs migration", patch(Some("migration")));
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

fn patch(mode: Option<&str>) -> io::Result<ExitStatus> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    match mode {
        None => println!(
            "🩹 Lazy-loading glue and the CommonJS entry from {manifest_dir}/src/bin/patch.mjs ..."
        ),
        Some(mode) => println!("🩹 CommonJS twins of {mode}/ ..."),
    }

    let output = Command::new("node")
        .arg(format!("{manifest_dir}/src/bin/patch.mjs"))
        .args(mode)
        .output()?;

    if !output.status.success() {
        eprintln!(
            "patch.mjs failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output.status)
}

/// The migration engine is TypeScript; tsc emits its ES modules and declarations next to the
/// sources. It reads the entry's declarations, so it runs after the glue is patched.
fn compile_migration() -> io::Result<ExitStatus> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    println!("🧩 Compiling the migration engine with tsc ...");

    // A module renamed or removed since the last build would otherwise ship stale
    remove_emitted(&Path::new(&manifest_dir).join("pkg/migration"))?;

    let output = Command::new("npx")
        .args(["--no", "--", "tsc", "-p", "migration"])
        .current_dir(Path::new(&manifest_dir).join("pkg"))
        .output()?;

    if !output.status.success() {
        eprintln!(
            "tsc failed (is `npm install` done in pkg/?): {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output.status)
}

/// Deletes what tsc and patch.mjs emit under `dir`, its subdirectories included.
fn remove_emitted(dir: &Path) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            remove_emitted(&path)?;
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        let emitted = [".js", ".cjs", ".d.ts"]
            .iter()
            .any(|ext| name.ends_with(ext));
        if emitted && name != "host.d.ts" {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
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
    let migration = serde_json::json!({
        "skipReasons": reasons,
        "transformRev": pubky_social_specs::migrate::TRANSFORM_REV,
    });
    write_data_asset(
        "migrationData",
        &migration,
        &[
            ("skipReasons", "data.skipReasons"),
            ("transformRev", "data.transformRev"),
        ],
        &migration_data_dts(&reasons),
    )
}

/// The union is spelled from the same list as the data, so the type and the values cannot
/// drift apart.
fn migration_data_dts(reasons: &[&str]) -> String {
    let union = reasons
        .iter()
        .map(|reason| format!("\"{reason}\""))
        .collect::<Vec<_>>()
        .join(" | ");
    format!(
        r#"/** Why one 0.x object did not migrate. */
export type SkipReason = {union};
/** Every reason, frozen, so a report counts categories without a table of its own. */
export declare const skipReasons: readonly SkipReason[];
/** The revision of the transforms. A run records it; a tree recorded under a lower one is walked again, which picks up what earlier revisions skipped, never rewriting a destination that exists. */
export declare const transformRev: number;
declare const data: {{
  readonly skipReasons: readonly SkipReason[];
  readonly transformRev: number;
}};
export default data;
"#
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
