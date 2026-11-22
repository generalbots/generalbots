//! #2522 — hostile-file gate adapter for the Sheets import handler.
//!
//! Kept separate from `export_import.rs` so the handler stays under the
//! 450-line cap. The gate runs after macro stripping and before any parser.

use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use botsecurity_core::upload_hostile::password as password_guard;
use botsecurity_core::upload_policy::{self, HostileFilePolicy};

pub(crate) type ResponseError = (StatusCode, Json<serde_json::Value>);

/// Runs the hostile-file gate explicitly at this import boundary and audits
/// its report. `validate_upload` already applies the same gate centrally; this
/// second inspection is defence in depth and yields the report so every
/// accepted import records its entry count, compression ratio and malware
/// decision. Rejections use the shared localized, sanitized response.
pub(crate) fn audit_hostile_upload(
    bytes: &[u8],
    filename: &str,
    headers: &HeaderMap,
    context: &str,
    policy: &HostileFilePolicy,
) -> Result<(), ResponseError> {
    // Mirror `validate_upload`: only zip packages have OOXML/ODF parts. CSV,
    // TSV and legacy XLS must not be parsed as a zip and refused.
    let is_package = filename
        .rsplit('.')
        .next()
        .and_then(upload_policy::mime_for_extension)
        .is_some_and(upload_policy::is_zip_container);
    if !is_package {
        return Ok(());
    }

    let report =
        upload_policy::inspect_hostile_upload_with(bytes, filename, policy).map_err(|error| {
            log::warn!(
                "{context}: hostile-file gate refused upload: {}",
                error.log_detail()
            );
            if password_guard::is_encrypted_rejection(&error) {
                password_guard::actionable_rejection(&error, headers)
            } else {
                upload_policy::rejection_json(&error, headers)
            }
        })?;

    if report.malware_scan == upload_policy::MalwareScanDecision::NotConfigured {
        log::warn!(
            "{context}: no malware scanner configured — scan result is NotConfigured, not clean"
        );
    }
    log::info!(
        "{context}: hostile-file gate: entries={}, compressed={}, uncompressed={}, ratio={}, malware={:?}",
        report.entry_count,
        report.compressed_bytes,
        report.uncompressed_bytes,
        report.compression_ratio,
        report.malware_scan,
    );

    // #2522 — second, streaming pass: XML entity limits and relationship/SSRF
    // re-check at the import boundary, before any spreadsheet parser runs.
    #[cfg(feature = "xlsx")]
    {
        let xml = crate::import_security::inspect_office_package(bytes).map_err(|error| {
            log::warn!(
                "{context}: hostile-file gate refused XML payload: {}",
                error.log_detail()
            );
            upload_policy::rejection_json(&error, headers)
        })?;
        log::info!(
            "{context}: XML safety: parts={}, relationships={}, entity_declarations={}, custom_entity_refs={}",
            xml.parts_checked,
            xml.relationship_targets,
            xml.entity_declarations,
            xml.custom_entity_refs,
        );
    }
    Ok(())
}
