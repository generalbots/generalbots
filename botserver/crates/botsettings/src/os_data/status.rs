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
