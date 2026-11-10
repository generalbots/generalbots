//! Unit tests for `super` (ensure_tenant_and_org.rs).
//!
//! Split out of `ensure_tenant_and_org.rs` to keep the file inside the project
//! size limit; the production code stays beside it. The upsert itself needs
//! Postgres (out of scope), so the tests cover the pure reuse-order decision
//! behind issue #779.

use super::*;

#[test]
fn the_signup_slug_is_tried_before_the_legacy_org_suffix() {
    assert_eq!(
        org_slug_candidates("acme"),
        ["acme".to_string(), "acme-org".to_string()]
    );
}

#[test]
fn the_first_candidate_is_the_bare_tenant_slug() {
    // First hit wins in `ensure_tenant_and_org`, so the order is the contract:
    // a signup-created org (`slug = {name}`) must be reused instead of a second
    // legacy `{name}-org` row.
    let candidates = org_slug_candidates("pragmatismo");
    assert_eq!(candidates[0], "pragmatismo");
    assert_eq!(candidates[1], "pragmatismo-org");
}

#[test]
fn an_empty_tenant_slug_still_yields_two_candidates() {
    assert_eq!(
        org_slug_candidates(""),
        [String::new(), "-org".to_string()]
    );
}

#[test]
fn a_slug_already_ending_in_org_is_not_special_cased() {
    assert_eq!(
        org_slug_candidates("acme-org"),
        ["acme-org".to_string(), "acme-org-org".to_string()]
    );
}

#[test]
fn the_candidate_list_has_exactly_two_entries() {
    // The caller iterates all candidates and stops on the first hit, so the
    // length is part of the contract: one extra name would add a reuse path.
    assert_eq!(org_slug_candidates("acme").len(), 2);
}
