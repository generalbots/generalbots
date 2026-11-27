//! Deterministic #2928 tests: org-owned and standalone bots resolve from the
//! DB row, and a wrong org scope is denied. Pure logic — no DB, no OtterIO.

use super::bucket_scope::{reconcile_bucket, DriveBucketContext};
use botbasic_core::utils::resolve_bot_drive_location;

fn allowed(buckets: &[&str]) -> Vec<String> {
    buckets.iter().map(|b| (*b).to_string()).collect()
}

#[test]
fn org_bot_resolves_to_the_gborg_bucket_and_branch_workspace() {
    // Prod shape: bot `pragmatismo` in branch `pragmatismo` of org
    // `pragmatismo`. Its files live in pragmatismo.gborg at
    // pragmatismo.gbai/pragmatismo.gbdrive/, never in pragmatismo.gbai.
    let location =
        resolve_bot_drive_location("pragmatismo", Some("pragmatismo"), Some("pragmatismo"));
    let context = reconcile_bucket(
        &location,
        "pragmatismo",
        None,
        &allowed(&["user-123", "pragmatismo.gborg"]),
        false,
    )
    .expect("org bucket is granted");

    assert_eq!(
        context,
        DriveBucketContext {
            bucket: "pragmatismo.gborg".into(),
            workspace: "pragmatismo".into(),
            bot: "pragmatismo".into(),
            key_prefix: "pragmatismo.gbai/pragmatismo.gbdrive/".into(),
            is_org: true,
        }
    );
}

#[test]
fn legacy_bot_gbai_guess_is_corrected_to_the_org_bucket() {
    // The old UI fallback sent bucket=pragmatismo.gbai with scope=bot. The
    // server must answer with the authoritative bucket instead of granting
    // the guess (which would read a different tenant's standalone layout).
    let location =
        resolve_bot_drive_location("pragmatismo", Some("pragmatismo"), Some("pragmatismo"));
    let context = reconcile_bucket(
        &location,
        "pragmatismo",
        Some("pragmatismo.gbai"),
        &allowed(&["pragmatismo.gborg"]),
        false,
    )
    .expect("legacy guess maps to the granted org bucket");

    assert_eq!(context.bucket, "pragmatismo.gborg");
    assert_eq!(context.workspace, "pragmatismo");
    assert!(context.is_org);
}

#[test]
fn standalone_bot_keeps_its_own_bucket_and_has_no_workspace() {
    let location = resolve_bot_drive_location("beiner", None, None);
    let context = reconcile_bucket(
        &location,
        "beiner",
        Some("beiner.gbai"),
        &allowed(&["beiner.gbai"]),
        false,
    )
    .expect("standalone bucket is granted");

    assert_eq!(context.bucket, "beiner.gbai");
    assert_eq!(context.workspace, "beiner");
    assert_eq!(context.key_prefix, "beiner.gbdrive/");
    assert!(!context.is_org);
}

#[test]
fn branch_slug_not_bot_name_decides_the_workspace() {
    let location =
        resolve_bot_drive_location("oppbot", Some("opportunity-oppbot"), Some("opportunity"));
    let context = reconcile_bucket(
        &location,
        "oppbot",
        None,
        &allowed(&["opportunity.gborg"]),
        false,
    )
    .expect("org bucket is granted");

    assert_eq!(context.bucket, "opportunity.gborg");
    assert_eq!(context.workspace, "opportunity-oppbot");
    assert_eq!(
        context.key_prefix,
        "opportunity-oppbot.gbai/oppbot.gbdrive/"
    );
}

#[test]
fn legacy_org_suffix_never_leaks_into_the_bucket_name() {
    let location = resolve_bot_drive_location("cristo", Some("cristo"), Some("cristo-org"));
    let context = reconcile_bucket(
        &location,
        "cristo",
        Some("cristo.gbai"),
        &allowed(&["cristo.gborg"]),
        false,
    )
    .expect("legacy org alias maps to the plain bucket");

    assert_eq!(context.bucket, "cristo.gborg");
    assert_eq!(context.workspace, "cristo");
}

#[test]
fn a_non_member_cannot_resolve_another_orgs_bucket() {
    let location =
        resolve_bot_drive_location("pragmatismo", Some("pragmatismo"), Some("pragmatismo"));
    // No bucket override at all must not bypass the membership check: the
    // endpoint would otherwise disclose every tenant's bucket by bot name.
    assert_eq!(
        reconcile_bucket(
            &location,
            "pragmatismo",
            None,
            &allowed(&["user-123"]),
            false
        ),
        Err("Access denied for this bucket")
    );
}

#[test]
fn wrong_org_scope_is_denied() {
    let location =
        resolve_bot_drive_location("pragmatismo", Some("pragmatismo"), Some("pragmatismo"));
    let entitled = allowed(&["user-123", "pragmatismo.gborg"]);

    // Another tenant's workspace, and another tenant's standalone guess.
    assert_eq!(
        reconcile_bucket(
            &location,
            "pragmatismo",
            Some("other.gborg"),
            &entitled,
            false
        ),
        Err("Access denied for this bucket")
    );
    assert_eq!(
        reconcile_bucket(
            &location,
            "pragmatismo",
            Some("other.gbai"),
            &entitled,
            false
        ),
        Err("Access denied for this bucket")
    );
}

#[test]
fn legacy_guess_without_org_entitlement_fails_closed() {
    // Membership lookup must not be bypassed by the compatibility mapping:
    // an org bot's bucket is never returned to a non-member.
    let location =
        resolve_bot_drive_location("pragmatismo", Some("pragmatismo"), Some("pragmatismo"));
    assert_eq!(
        reconcile_bucket(
            &location,
            "pragmatismo",
            Some("pragmatismo.gbai"),
            &allowed(&["user-123", "default.gborg"]),
            false,
        ),
        Err("Access denied for this bucket")
    );
}

#[test]
fn a_standalone_bot_cannot_be_renamed_into_a_wrong_tenant_bucket() {
    let location = resolve_bot_drive_location("beiner", None, None);
    assert_eq!(
        reconcile_bucket(
            &location,
            "beiner",
            Some("other.gbai"),
            &allowed(&["other.gbai"]),
            false
        ),
        Err("Access denied for this bucket")
    );
}
