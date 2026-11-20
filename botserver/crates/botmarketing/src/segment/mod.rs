//! Structured audience segments for marketing lists (#2537).
//!
//! Replaces the `query_text.contains("status=")` heuristic in `lists.rs` with a
//! validated JSON model, a side-effect-free preview and idempotent
//! materialization into `marketing_list_contacts`.

mod evaluate;
mod model;
mod store;
mod validate;

pub use evaluate::{
    candidate_matches, consent_flags, is_suppressed, ConsentFlags, SegmentCandidate,
    SUPPRESSED_STATUSES,
};
pub use model::{
    DriftReport, FieldSpec, Segment, SegmentCondition, SegmentError, SegmentErrorCode,
    SegmentMatch, SegmentMeta, ValueKind, FIELD_CATALOG,
};
pub use store::{
    current_member_count, drop_suppressed, materialize_segment, preview_segment,
    resolve_send_contacts, segment_from_list, suppressed_crm_emails, suppressed_marketing_contact,
    MaterializeResult, SegmentPreview, SegmentPreviewMatch, SegmentStoreError,
};
pub use validate::{
    compile_legacy_query, detect_drift, parse_segment, segment_fingerprint, SegmentField, SegmentOp,
    ValidCondition, ValidSegment,
};

#[cfg(test)]
mod tests;
