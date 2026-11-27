//! Agent-runtime manifest parser and validator (schema_version 1).
//!
//! The manifest is a small JSON file published alongside each release's
//! rootfs and workspace image.  It carries file names, sizes, SHA-256
//! digests and the Incus image fingerprint so the app can verify every
//! download before importing it.

use serde::Deserialize;

/// Parsed manifest (only the fields the app needs).
#[derive(Debug, Clone)]
pub(crate) struct Manifest {
    pub schema_version: u32,
    pub runtime_version: String,
    pub release_tag: String,
    pub rootfs: AssetInfo,
    pub workspace_image: AssetInfo,
}

#[derive(Debug, Clone)]
pub(crate) struct AssetInfo {
    pub file: String,
    pub size: u64,
    pub sha256: String,
    pub incus_fingerprint: Option<String>,
}

/// Parse and validate the manifest JSON.
pub(crate) fn parse(data: &[u8]) -> Result<Manifest, String> {
    let raw: RawManifest = serde_json::from_slice(data)
        .map_err(|e| format!("manifest JSON invalid: {e}"))?;

    if raw.schema_version != 1 {
        return Err(format!(
            "unsupported manifest schema_version {}; this app supports 1",
            raw.schema_version
        ));
    }

    if raw.runtime_version.is_empty() {
        return Err("manifest runtime_version is empty".into());
    }

    let rootfs = raw
        .assets
        .get("wsl_rootfs")
        .ok_or("manifest missing assets.wsl_rootfs")?;
    let workspace = raw
        .assets
        .get("workspace_image")
        .ok_or("manifest missing assets.workspace_image")?;

    validate_asset(rootfs, "wsl_rootfs")?;
    validate_asset(workspace, "workspace_image")?;

    if rootfs.distro_name.as_deref() != Some(super::state::DISTRO_NAME) {
        return Err(format!(
            "manifest rootfs distro_name is {:?}, expected {:?}",
            rootfs.distro_name,
            super::state::DISTRO_NAME,
        ));
    }

    let fingerprint = workspace
        .incus_fingerprint
        .clone()
        .ok_or("manifest workspace_image missing incus_fingerprint")?;
    if fingerprint != workspace.sha256 {
        return Err(
            "manifest workspace_image incus_fingerprint does not match sha256".into(),
        );
    }

    Ok(Manifest {
        schema_version: raw.schema_version,
        runtime_version: raw.runtime_version,
        release_tag: raw.release_tag,
        rootfs: AssetInfo {
            file: rootfs.file.clone(),
            size: rootfs.size,
            sha256: rootfs.sha256.clone(),
            incus_fingerprint: None,
        },
        workspace_image: AssetInfo {
            file: workspace.file.clone(),
            size: workspace.size,
            sha256: workspace.sha256.clone(),
            incus_fingerprint: Some(fingerprint),
        },
    })
}

fn validate_asset(asset: &RawAsset, role: &str) -> Result<(), String> {
    if asset.file.is_empty() {
        return Err(format!("manifest {role}.file is empty"));
    }
    if !asset
        .file
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'+' | b'-'))
    {
        return Err(format!(
            "manifest {role}.file contains invalid characters: {}",
            asset.file
        ));
    }
    if asset.size == 0 {
        return Err(format!("manifest {role}.size is zero"));
    }
    if asset.sha256.len() != 64
        || !asset.sha256.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(format!("manifest {role}.sha256 is not 64 hex chars"));
    }
    // Ensure lowercase hex.
    if asset.sha256 != asset.sha256.to_ascii_lowercase() {
        return Err(format!("manifest {role}.sha256 must be lowercase hex"));
    }
    Ok(())
}

// ── Raw serde types ─────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RawManifest {
    schema_version: u32,
    #[serde(default)]
    runtime_version: String,
    #[serde(default)]
    release_tag: String,
    #[serde(default)]
    assets: std::collections::HashMap<String, RawAsset>,
}

#[derive(Deserialize)]
struct RawAsset {
    #[serde(default)]
    file: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    sha256: String,
    #[serde(default)]
    distro_name: Option<String>,
    #[serde(default)]
    incus_fingerprint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::parse;

    fn valid_manifest_json() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "runtime_version": "1.0.0",
            "release_tag": "gbgov-agent-runtime-v1.0.0",
            "commit": "a".repeat(40),
            "built_at": "2026-01-01T00:00:00Z",
            "debian_snapshot": "20261120T000000Z",
            "assets": {
                "wsl_rootfs": {
                    "kind": "wsl-rootfs",
                    "file": "GeneralBotsLite-agent-rootfs-1.0.0-x86_64.tar.gz",
                    "size": 100_000_000,
                    "sha256": "a".repeat(64),
                    "arch": "x86_64",
                    "distro_name": "GeneralBotsLite",
                    "compression": "gzip",
                    "debian_suite": "trixie",
                    "debian_snapshot": "20261120T000000Z",
                    "incus_version": "6.0.4",
                    "systemd_version": "256"
                },
                "workspace_image": {
                    "kind": "incus-image",
                    "file": "GeneralBotsLite-agent-workspace-office-1.0.0-x86_64.tar.xz",
                    "size": 200_000_000,
                    "sha256": "b".repeat(64),
                    "arch": "x86_64",
                    "format": "unified",
                    "compression": "xz",
                    "incus_fingerprint": "b".repeat(64),
                    "alias": "gb-workspace-office",
                    "profile": "office",
                    "provisioned_marker": "v7-office"
                }
            }
        }))
        .expect("json")
    }

    #[test]
    fn valid_manifest_parses() {
        let m = parse(&valid_manifest_json()).expect("parse");
        assert_eq!(m.schema_version, 1);
        assert_eq!(m.runtime_version, "1.0.0");
        assert_eq!(
            m.rootfs.file,
            "GeneralBotsLite-agent-rootfs-1.0.0-x86_64.tar.gz"
        );
        assert_eq!(m.workspace_image.sha256, "b".repeat(64));
        assert_eq!(
            m.workspace_image.incus_fingerprint.as_deref(),
            Some(&*"b".repeat(64))
        );
    }

    #[test]
    fn wrong_schema_version_is_rejected() {
        let json = serde_json::to_vec(&serde_json::json!({
            "schema_version": 2, "runtime_version": "1.0.0",
            "release_tag": "", "assets": {}
        }))
        .expect("json");
        assert!(parse(&json).is_err());
    }

    #[test]
    fn missing_assets_are_rejected() {
        let json = serde_json::to_vec(&serde_json::json!({
            "schema_version": 1, "runtime_version": "1.0.0",
            "release_tag": "", "assets": {}
        }))
        .expect("json");
        let err = parse(&json).unwrap_err();
        assert!(err.contains("wsl_rootfs"), "{err}");
    }

    #[test]
    fn wrong_distro_name_is_rejected() {
        let mut json: serde_json::Value =
            serde_json::from_slice(&valid_manifest_json()).expect("parse");
        json["assets"]["wsl_rootfs"]["distro_name"] = "Other".into();
        let err = parse(&serde_json::to_vec(&json).expect("json")).unwrap_err();
        assert!(err.contains("distro_name"), "{err}");
    }

    #[test]
    fn fingerprint_mismatch_is_rejected() {
        let mut json: serde_json::Value =
            serde_json::from_slice(&valid_manifest_json()).expect("parse");
        json["assets"]["workspace_image"]["incus_fingerprint"] =
            "c".repeat(64).into();
        let err = parse(&serde_json::to_vec(&json).expect("json")).unwrap_err();
        assert!(err.contains("fingerprint"), "{err}");
    }

    #[test]
    fn invalid_sha256_is_rejected() {
        let mut json: serde_json::Value =
            serde_json::from_slice(&valid_manifest_json()).expect("parse");
        json["assets"]["wsl_rootfs"]["sha256"] = "ZZZZ".into();
        assert!(parse(&serde_json::to_vec(&json).expect("json")).is_err());
    }

    #[test]
    fn path_traversal_in_file_name_is_rejected() {
        let mut json: serde_json::Value =
            serde_json::from_slice(&valid_manifest_json()).expect("parse");
        json["assets"]["wsl_rootfs"]["file"] = "../etc/passwd".into();
        assert!(parse(&serde_json::to_vec(&json).expect("json")).is_err());
    }
}
