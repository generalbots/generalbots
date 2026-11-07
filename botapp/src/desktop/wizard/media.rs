//! Media capability probe for the wizard finish screen.
//!
//! `/api/health` on the BotModels service is authoritative once it runs, but
//! provisioning deliberately leaves BotModels to install on first use. The
//! finish screen still has to say which optional media features this host will
//! have, so an unreachable service falls back to a local probe of the bundled
//! wheelhouse plus the system libraries pip cannot install.

use super::paths::stack_path;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const BOTMODELS_HEALTH: &str = "http://127.0.0.1:8085/api/health";
const LIBZBAR_REMEDIATION: &str = "apt-get install -y libzbar0";
const OCR_REMEDIATION: &str =
    "apt-get install -y tesseract-ocr (or reinstall the BotModels pythonbox)";

pub async fn capabilities() -> Value {
    match fetch_health().await {
        Ok(payload) => normalize(payload, true, "botmodels", None),
        Err(error) => {
            let mut payload = host_probe();
            payload["service_error"] = Value::String(error);
            payload
        }
    }
}

async fn fetch_health() -> Result<Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(4))
        .build()
        .map_err(|e| format!("HTTP client: {e}"))?;
    let response = client
        .get(BOTMODELS_HEALTH)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "the AI media service answered HTTP {}",
            response.status()
        ));
    }
    response
        .json::<Value>()
        .await
        .map_err(|e| format!("unreadable health payload: {e}"))
}

fn unavailable(detail: &str, remediation: &str) -> Value {
    json!({
        "available": false,
        "detail": detail,
        "remediation": remediation,
    })
}

fn normalize(payload: Value, service_running: bool, source: &str, error: Option<String>) -> Value {
    let capabilities = payload
        .get("media_capabilities")
        .cloned()
        .unwrap_or(Value::Null);
    let barcode = capabilities
        .get("barcode")
        .cloned()
        .unwrap_or_else(|| unavailable("the AI media service did not report barcode support", ""));
    let ocr = capabilities
        .get("ocr")
        .cloned()
        .unwrap_or_else(|| unavailable("the AI media service did not report OCR support", ""));
    let mut result = json!({
        "service_running": service_running,
        "source": source,
        "barcode": barcode,
        "ocr": ocr,
    });
    if let Some(error) = error {
        result["service_error"] = Value::String(error);
    }
    result
}

fn host_probe() -> Value {
    if !cfg!(target_os = "linux") {
        let detail = "The BotModels pythonbox is published for Linux x86_64 only.";
        return json!({
            "service_running": false,
            "source": "host",
            "barcode": unavailable(detail, ""),
            "ocr": unavailable(detail, ""),
        });
    }

    let pyzbar = bundle_contains("pyzbar");
    let libzbar = library_present("libzbar.so");
    let barcode = if !pyzbar {
        unavailable(
            "the barcode wheel is not part of this bundle",
            "reinstall the BotModels pythonbox or run ./install.sh --with barcode",
        )
    } else if !libzbar {
        unavailable(
            "libzbar0 is not installed on this host",
            LIBZBAR_REMEDIATION,
        )
    } else {
        json!({"available": true, "detail": Value::Null, "remediation": Value::Null})
    };

    let tesseract = executable_on_path("tesseract");
    let rapidocr =
        bundle_contains("rapidocr_onnxruntime") || bundle_contains("rapidocr-onnxruntime");
    let ocr = if tesseract {
        json!({"available": true, "detail": Value::Null, "remediation": Value::Null})
    } else if rapidocr {
        json!({
            "available": true,
            "detail": "RapidOCR will be used (tesseract is not installed).",
            "remediation": Value::Null,
        })
    } else {
        unavailable("no OCR backend was found", OCR_REMEDIATION)
    };

    json!({
        "service_running": false,
        "source": "host",
        "barcode": barcode,
        "ocr": ocr,
    })
}

fn botmodels_dir() -> PathBuf {
    stack_path().join("bin").join("botmodels")
}

/// Whether the module's wheel (or installed package) is present in the bundle.
fn bundle_contains(module: &str) -> bool {
    let root = botmodels_dir();
    let wheelhouse = root.join("wheelhouse");
    for candidate in [
        module.to_string(),
        module.replace('_', "-"),
        module.replace('-', "_"),
    ] {
        if has_prefix(&wheelhouse, &format!("{candidate}-")) {
            return true;
        }
    }
    venv_contains(&root.join(".venv"), module)
}

fn has_prefix(directory: &Path, prefix: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return false;
    };
    entries
        .flatten()
        .any(|entry| entry.file_name().to_string_lossy().starts_with(prefix))
}

fn venv_contains(venv: &Path, module: &str) -> bool {
    let Ok(versions) = std::fs::read_dir(venv.join("lib")) else {
        return false;
    };
    let prefixes = [
        format!("{module}-").to_ascii_lowercase(),
        format!("{module}_").to_ascii_lowercase(),
    ];
    for version in versions.flatten() {
        let site = version.path().join("site-packages");
        if site.join(module).exists() {
            return true;
        }
        let Ok(packages) = std::fs::read_dir(&site) else {
            continue;
        };
        for package in packages.flatten() {
            let name = package.file_name().to_string_lossy().to_ascii_lowercase();
            if prefixes.iter().any(|prefix| name.starts_with(prefix)) {
                return true;
            }
        }
    }
    false
}

fn library_present(prefix: &str) -> bool {
    const ROOTS: [&str; 5] = ["/usr/lib", "/usr/lib64", "/usr/local/lib", "/lib", "/lib64"];
    let mut roots: Vec<PathBuf> = ROOTS.iter().map(PathBuf::from).collect();
    if let Some(paths) = std::env::var_os("LD_LIBRARY_PATH") {
        roots.extend(std::env::split_paths(&paths));
    }
    roots.iter().any(|root| scan_prefix(root, prefix, 2))
}

fn scan_prefix(directory: &Path, prefix: &str, depth: usize) -> bool {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(prefix) {
            return true;
        }
        if depth > 0 && path.is_dir() && scan_prefix(&path, prefix, depth - 1) {
            return true;
        }
    }
    false
}

fn executable_on_path(name: &str) -> bool {
    let file = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|directory| directory.join(&file).is_file())
}

#[cfg(test)]
mod tests {
    use super::normalize;
    use serde_json::json;

    #[test]
    fn missing_health_capabilities_are_reported_unavailable() {
        let payload = normalize(json!({}), true, "botmodels", None);
        assert_eq!(payload["barcode"]["available"], json!(false));
        assert_eq!(payload["ocr"]["available"], json!(false));
        assert_eq!(payload["source"], json!("botmodels"));
    }

    #[test]
    fn health_payload_is_passed_through() {
        let payload = normalize(
            json!({
                "media_capabilities": {
                    "barcode": {"available": false, "detail": "no libzbar", "remediation": "apt-get install -y libzbar0"},
                    "ocr": {"available": true, "detail": null, "remediation": null}
                }
            }),
            true,
            "botmodels",
            None,
        );
        assert_eq!(payload["barcode"]["available"], json!(false));
        assert_eq!(payload["ocr"]["available"], json!(true));
    }
}
