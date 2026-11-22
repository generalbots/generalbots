//! Static status payload for the OS Settings data-protection panel (#2470).
//! Split out of `os_data_protection` to keep that module focused and small.

use super::{
    retention_payload, ARCHIVE_EXCLUDES, ARCHIVE_INCLUDES, ARCHIVE_SCOPE_FULL, DATA_ROOT,
    EXPORT_CONTRACT_PATH, RESET_CONFIRMATION_TOKEN, RESET_CONTRACT_PATH, SNAPSHOT_CONTRACT_PATH,
};
use serde_json::{json, Value};

/// Static status payload: no host probing, so the answer cannot drift from
/// what this build actually ships. Honest about the lack of LUKS/TPM, backups,
/// export archives and factory reset.
pub(crate) fn status_payload() -> Value {
    json!({
        "status": "not_implemented",
        "shell_mode_only": true,
        "data_root": DATA_ROOT,
        "encryption": {
            "implemented": false,
            "luks_mapping": false,
            "tpm_sealed": false,
            "recovery_key_export": false,
            "note": "No LUKS mapping, no systemd-cryptenroll provisioning and no TPM sealing are implemented in this image. User data at rest is not encrypted by General Bots."
        },
        "backup": {
            "implemented": false,
            "enabled": false,
            "root": DATA_ROOT,
            "schedule": "none",
            "last_snapshot": null,
            "restore_available": false,
            "retention_policy": retention_payload(),
            "descriptor_adapter": "drive-archive",
            "contract": {
                "status": "adapter_contract_only",
                "method": "POST",
                "path": SNAPSHOT_CONTRACT_PATH,
                "request": { "reason": "string" },
                "adapters": ["btrfs", "drive-archive"],
                "result": "503 when no OS backend is registered; 202 with a staged, non-destructive Drive archive descriptor when Drive is configured",
                "medium": "btrfs snapshot or Drive archive of the data root",
                "note": "The drive-archive adapter stages a bounded manifest + retention descriptor only. It copies no bytes, deletes no source data and is not a scheduled backup."
            }
        },
        "export": {
            "implemented": false,
            "enabled": false,
            "descriptor_available": true,
            "contract": {
                "status": "descriptor_only",
                "method": "POST",
                "path": EXPORT_CONTRACT_PATH,
                "request": { "scope": ARCHIVE_SCOPE_FULL },
                "result": "200 with a deterministic archive manifest; no archive is created and no data leaves the device",
                "includes": ARCHIVE_INCLUDES,
                "excludes": ARCHIVE_EXCLUDES
            }
        },
        "reset": {
            "implemented": false,
            "enabled": false,
            "confirmation_token": RESET_CONFIRMATION_TOKEN,
            "contract": {
                "status": "disabled",
                "method": "POST",
                "path": RESET_CONTRACT_PATH,
                "request": { "confirmation": "exact typed confirmation token required" },
                "result": "400 without the typed token; 501 with it. No wipe backend ships in this build."
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Top-level verdict and the backup shape must stay honest: the panel shows
    /// "not implemented", there is no scheduler and no snapshot has ever run.
    #[test]
    fn status_reports_not_implemented_with_no_schedule_or_snapshot() {
        let p = status_payload();
        assert_eq!(p["status"], "not_implemented");
        assert_eq!(p["backup"]["schedule"], "none");
        assert!(p["backup"]["last_snapshot"].is_null());
    }

    /// No capability flag may claim real LUKS/TPM encryption or a working
    /// backup/restore backend, and no historical "we seal the volume" text may
    /// creep back into the notes.
    #[test]
    fn status_makes_no_luks_tpm_or_backup_claims() {
        let p = status_payload();

        assert_eq!(p["encryption"]["implemented"], false);
        assert_eq!(p["encryption"]["luks_mapping"], false);
        assert_eq!(p["encryption"]["tpm_sealed"], false);
        assert_eq!(p["encryption"]["recovery_key_export"], false);

        assert_eq!(p["backup"]["implemented"], false);
        assert_eq!(p["backup"]["enabled"], false);
        assert_eq!(p["backup"]["restore_available"], false);

        let rendered = p.to_string().to_ascii_lowercase();
        assert!(!rendered.contains("\"enabled\":true"));
        for false_claim in [
            "tpm 2.0 sealing of the root volume is performed",
            "luks mapping is active",
            "backups are enabled",
            "automated backup is running",
        ] {
            assert!(
                !rendered.contains(false_claim),
                "status claimed an unshipped capability: {false_claim}"
            );
        }
    }

    /// The UI reads the typed confirmation token and the named adapter straight
    /// from status; both must be present and non-empty.
    #[test]
    fn status_publishes_confirmation_token_and_named_adapter() {
        let p = status_payload();

        assert_eq!(p["reset"]["confirmation_token"], RESET_CONFIRMATION_TOKEN);
        assert!(p["reset"]["confirmation_token"]
            .as_str()
            .is_some_and(|token| !token.is_empty()));

        assert_eq!(p["backup"]["descriptor_adapter"], "drive-archive");
        assert!(p["backup"]["descriptor_adapter"]
            .as_str()
            .is_some_and(|name| !name.is_empty()));
    }

    /// Recursively collect every string value so path/secret guards inspect the
    /// whole payload, not just the top level.
    fn collect_strings(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::String(s) => out.push(s.clone()),
            Value::Array(items) => {
                for item in items {
                    collect_strings(item, out);
                }
            }
            Value::Object(map) => {
                for item in map.values() {
                    collect_strings(item, out);
                }
            }
            _ => {}
        }
    }

    /// Status is static and must never leak credential markers, endpoint hints
    /// or host filesystem paths. The only absolute strings allowed are the
    /// declared data root and this module's own contract paths.
    #[test]
    fn status_has_no_secret_markers_or_unexpected_paths() {
        let p = status_payload();
        let rendered = p.to_string().to_ascii_lowercase();
        for marker in [
            "sk_live",
            "api_key",
            "password=",
            "secret=",
            "bearer ",
            "private_key",
            "-----begin",
            "http://",
            "https://",
            "s3://",
            "minio",
        ] {
            assert!(
                !rendered.contains(marker),
                "status payload leaked secret/endpoint marker: {marker}"
            );
        }

        let mut strings = Vec::new();
        collect_strings(&p, &mut strings);
        for value in strings {
            if value.starts_with('/') {
                assert!(
                    value == DATA_ROOT || value.starts_with("/api/os/data-protection"),
                    "status payload leaked a host path: {value}"
                );
            }
        }
    }
}
