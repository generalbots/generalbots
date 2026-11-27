use serde_json::{Map, Value};
use std::error::Error;

/// The edition config the release workflow passes to `cargo tauri build`.
const GBGOV_CONFIG: &str = "tauri.gblite.conf.json";

/// The keys of the edition config that every build of the edition carries,
/// with or without the Tauri CLI. The bundle section stays CLI-only: its
/// sidecar and resources exist only in the release workflow.
const IDENTITY_KEYS: [&str; 2] = ["productName", "identifier"];

fn main() -> Result<(), Box<dyn Error>> {
    // The suite and the first-run wizard are embedded from botui/ui at build
    // time, so a change there must relink the application.
    println!("cargo:rerun-if-changed=../botui/ui");
    if std::env::var_os("CARGO_FEATURE_GBGOV").is_some() {
        apply_identity(GBGOV_CONFIG)?;
    }
    tauri_build::build();
    Ok(())
}

/// Merges the edition's identity into `TAURI_CONFIG`, the JSON merge patch
/// that tauri-build reads here and `generate_context!` reads in rustc. A
/// patch the CLI already set (`--config`) is kept and the identity is laid
/// over it; the values are the same file's, so applying them twice is a no-op.
fn apply_identity(path: &str) -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed={path}");
    let edition: Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    let mut patch = match std::env::var("TAURI_CONFIG") {
        Ok(raw) if !raw.trim().is_empty() => match serde_json::from_str(&raw)? {
            Value::Object(map) => map,
            _ => return Err("TAURI_CONFIG is not a JSON object".into()),
        },
        _ => Map::new(),
    };
    for key in IDENTITY_KEYS {
        let value = edition
            .get(key)
            .ok_or_else(|| format!("{path} has no {key}"))?;
        patch.insert(key.to_string(), value.clone());
    }
    let merged = serde_json::to_string(&Value::Object(patch))?;
    std::env::set_var("TAURI_CONFIG", &merged);
    println!("cargo:rustc-env=TAURI_CONFIG={merged}");
    Ok(())
}
