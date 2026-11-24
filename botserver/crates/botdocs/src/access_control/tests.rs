//! Offline unit tests for the Docs object access-control core (#2452).

use chrono::{DateTime, Duration, Utc};

use super::{
    decide, redact, Action, Acl, AuditInput, AuditLog, Decision, LinkCredential, LinkPolicy,
    LinkScope, Principal, Reason, Role, REDACTED,
};

fn instant(offset_seconds: i64) -> DateTime<Utc> {
    DateTime::<Utc>::UNIX_EPOCH + Duration::seconds(offset_seconds)
}

fn viewer_link(expires_at: Option<DateTime<Utc>>) -> LinkPolicy {
    let mut link = LinkPolicy::new("token-abc", Role::Viewer, expires_at);
    link.scope = LinkScope::AnyoneWithLink;
    link
}

#[test]
fn owner_has_full_access_and_bypasses_link_restrictions() {
    let mut acl = Acl::new("org-1", "owner-1");
    acl.set_link(viewer_link(Some(instant(100))));

    for action in [
        Action::View,
        Action::Print,
        Action::Copy,
        Action::Comment,
        Action::Edit,
        Action::Export,
        Action::Share,
        Action::Manage,
    ] {
        // A bogus link credential cannot weaken the owner's own capability.
        let principal = Principal::member("owner-1", "org-1", vec![]);
        let decision = decide(&principal, action, &acl, instant(500));
        assert_eq!(decision, Decision::allow(Reason::Owner));
    }
}

#[test]
fn group_grant_authorizes_and_direct_grant_still_gates_actions() {
    let mut acl = Acl::new("org-1", "owner-1");
    acl.grant_group("legal", Role::Editor);
    acl.grant_user("weak", Role::Viewer);

    let legal = Principal::member("alice", "org-1", vec!["legal".to_string()]);
    assert_eq!(
        decide(&legal, Action::Edit, &acl, instant(0)),
        Decision::allow(Reason::GroupGrant("legal".to_string(), Role::Editor))
    );

    let outsider = Principal::member("bob", "org-1", vec!["sales".to_string()]);
    assert_eq!(
        decide(&outsider, Action::View, &acl, instant(0)),
        Decision::deny(Reason::NoGrant)
    );

    let weak = Principal::member("weak", "org-1", vec![]);
    assert_eq!(
        decide(&weak, Action::Edit, &acl, instant(0)),
        Decision::deny(Reason::InsufficientRole {
            required: Role::Editor,
            actual: Role::Viewer,
        })
    );
    assert_eq!(
        decide(&weak, Action::View, &acl, instant(0)),
        Decision::allow(Reason::UserGrant(Role::Viewer))
    );
}

#[test]
fn expired_and_revoked_links_are_denied_server_side() {
    let mut acl = Acl::new("org-1", "owner-1");
    acl.set_link(viewer_link(Some(instant(100))));
    let reader = Principal::anonymous("org-1", LinkCredential::new("token-abc"));

    assert_eq!(
        decide(&reader, Action::View, &acl, instant(99)),
        Decision::allow(Reason::LinkGrant(Role::Viewer))
    );
    assert_eq!(
        decide(&reader, Action::View, &acl, instant(100)),
        Decision::deny(Reason::LinkExpired)
    );

    let wrong = Principal::anonymous("org-1", LinkCredential::new("not-the-token"));
    assert_eq!(
        decide(&wrong, Action::View, &acl, instant(0)),
        Decision::deny(Reason::LinkTokenMismatch)
    );

    acl.revoke_link();
    assert_eq!(
        decide(&reader, Action::View, &acl, instant(0)),
        Decision::deny(Reason::NoLink)
    );
}

#[test]
fn download_flag_and_guest_restrictions_are_enforced() {
    let mut acl = Acl::new("org-1", "owner-1");
    let mut link = viewer_link(None);
    link.allow_download = false;
    acl.set_link(link);

    let reader = Principal::anonymous("org-1", LinkCredential::new("token-abc"));
    assert_eq!(
        decide(&reader, Action::Export, &acl, instant(0)),
        Decision::deny(Reason::DownloadDisabled)
    );

    // Guest link: email-bound, domain-allowlisted, no download/print/copy.
    let mut guest_link = viewer_link(None);
    guest_link.is_guest = true;
    guest_link.guest_email = Some("counsel@example.com".to_string());
    guest_link.allowed_domains = vec!["example.com".to_string()];
    guest_link.role = Role::Editor;
    guest_link.allow_download = true;
    acl.allow_guests = true;
    acl.set_link(guest_link);

    let guest = Principal::guest(
        "counsel@example.com",
        "org-1",
        LinkCredential::new("token-abc"),
    );
    assert_eq!(
        decide(&guest, Action::View, &acl, instant(0)),
        Decision::allow(Reason::LinkGrant(Role::Editor))
    );
    for forbidden in [
        Action::Edit,
        Action::Export,
        Action::Share,
        Action::Manage,
    ] {
        assert_eq!(
            decide(&guest, forbidden, &acl, instant(0)),
            Decision::deny(Reason::GuestActionNotAllowed(forbidden))
        );
    }

    let wrong_email = Principal::guest(
        "other@elsewhere.test",
        "org-1",
        LinkCredential::new("token-abc"),
    );
    assert_eq!(
        decide(&wrong_email, Action::View, &acl, instant(0)),
        Decision::deny(Reason::GuestEmailMismatch)
    );

    // A guest whose email matches the binding but whose domain is not on the
    // allowlist is a distinct refusal (the email check runs first).
    let mut narrow_link = viewer_link(None);
    narrow_link.is_guest = true;
    narrow_link.guest_email = Some("counsel@example.com".to_string());
    narrow_link.allowed_domains = vec!["partner.test".to_string()];
    narrow_link.role = Role::Editor;
    acl.set_link(narrow_link);
    assert_eq!(
        decide(&guest, Action::View, &acl, instant(0)),
        Decision::deny(Reason::DomainNotAllowed)
    );
    acl.set_link({
        let mut restore = viewer_link(None);
        restore.is_guest = true;
        restore.guest_email = Some("counsel@example.com".to_string());
        restore.allowed_domains = vec!["example.com".to_string()];
        restore.role = Role::Editor;
        restore.allow_download = true;
        restore
    });

    acl.allow_guests = false;
    assert_eq!(
        decide(&guest, Action::View, &acl, instant(0)),
        Decision::deny(Reason::GuestsDisabled)
    );
}

#[test]
fn password_protected_link_requires_verification() {
    let mut acl = Acl::new("org-1", "owner-1");
    let mut link = viewer_link(None);
    link.password_hash = Some("hash".to_string());
    acl.set_link(link);
    let unverified = Principal::anonymous("org-1", LinkCredential::new("token-abc"));
    let verified = Principal::anonymous(
        "org-1",
        LinkCredential::new("token-abc").with_verified_password(),
    );

    assert_eq!(
        decide(&unverified, Action::View, &acl, instant(0)),
        Decision::deny(Reason::PasswordRequired)
    );
    assert!(decide(&verified, Action::View, &acl, instant(0)).is_allowed());
}

#[test]
fn audit_log_appends_and_redacts_secrets() {
    let mut log = AuditLog::new();
    assert!(log.is_empty());

    let detail = "token=abc123 password=hunter2 Authorization: Bearer topsecret";
    let redacted = redact(detail);
    assert!(!redacted.contains("abc123"));
    assert!(!redacted.contains("hunter2"));
    assert!(!redacted.contains("topsecret"));
    assert!(redacted.contains(REDACTED));

    let first = log.record(AuditInput {
        timestamp: instant(10),
        object_id: "doc-1".to_string(),
        actor: "alice".to_string(),
        action: Action::View,
        decision: Decision::allow(Reason::Owner),
        ip: Some("198.51.100.7".to_string()),
        detail: detail.to_string(),
    });
    let second = log.record(AuditInput {
        timestamp: instant(20),
        object_id: "doc-1".to_string(),
        actor: "bob".to_string(),
        action: Action::Export,
        decision: Decision::deny(Reason::DownloadDisabled),
        ip: None,
        detail: "api_key=s3cr3t".to_string(),
    });

    assert_eq!((first, second), (0, 1));
    assert_eq!(log.len(), 2);
    let events = log.events();
    assert_eq!(events[0].detail, redacted);
    assert_eq!(events[0].actor, "alice");
    assert_eq!(events[0].ip.as_deref(), Some("198.51.100.7"));
    assert_eq!(events[1].detail, format!("api_key={REDACTED}"));
    assert_eq!(log.by_object("doc-1").len(), 2);
    assert_eq!(log.by_actor("alice").len(), 1);
    assert_eq!(log.latest().map(|event| event.sequence), Some(1));
}

#[test]
fn ownership_transfer_keeps_grants_intact_and_demotes_old_owner() {
    let mut acl = Acl::new("org-1", "owner-1");
    acl.grant_group("legal", Role::Commenter);
    acl.grant_user("new-owner", Role::Viewer);
    acl.transfer_ownership("new-owner");

    assert_eq!(acl.owner, "new-owner");
    assert_eq!(acl.user_grants.get("owner-1"), Some(&Role::Editor));
    assert!(!acl.user_grants.contains_key("new-owner"));
    assert_eq!(acl.group_grants.get("legal"), Some(&Role::Commenter));

    let new_owner = Principal::member("new-owner", "org-1", vec![]);
    assert_eq!(
        decide(&new_owner, Action::Manage, &acl, instant(0)),
        Decision::allow(Reason::Owner)
    );
}
