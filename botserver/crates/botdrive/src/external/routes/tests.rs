use super::*;

use super::*;

#[test]
fn provider_parsing_rejects_unknown_values() {
    assert_eq!(parse_provider("onedrive").unwrap(), Provider::OneDrive);
    assert_eq!(parse_provider("gdrive").unwrap(), Provider::GoogleDrive);
    assert_eq!(parse_provider("o365").unwrap(), Provider::OneDrive);
    assert!(parse_provider("dropbox").is_err());
    assert!(parse_provider("").is_err());
}

#[test]
fn filenames_cannot_escape_the_header() {
    assert_eq!(sanitize_filename("report\".pdf"), "report.pdf");
    assert_eq!(sanitize_filename("a\\b.txt"), "ab.txt");
    assert_eq!(sanitize_filename("line\nbreak"), "linebreak");
    assert_eq!(sanitize_filename("   "), "download");
    assert_eq!(sanitize_filename("ok.txt"), "ok.txt");
}

#[test]
fn reason_codes_are_url_encoded() {
    assert_eq!(urlencode("access_denied"), "access_denied");
    assert_eq!(urlencode("a b&c"), "a%20b%26c");
}

#[test]
fn unbound_principals_cannot_share_the_nil_branch() {
    // #2865 — two distinct authenticated principals with no crm_contacts
    // binding must both be denied instead of collapsing onto one nil-branch
    // connection (and therefore one OAuth grant / mirrored file set).
    let alice =
        AuthenticatedUser::new(Uuid::new_v4(), "alice".to_string()).with_email("alice@example.com");
    let bob =
        AuthenticatedUser::new(Uuid::new_v4(), "bob".to_string()).with_email("bob@example.com");

    let alice_status = branch_from_binding(&alice, None).err().map(|e| e.0);
    let bob_status = branch_from_binding(&bob, None).err().map(|e| e.0);
    assert_eq!(alice_status, bob_status);
    assert_eq!(alice_status, Some(StatusCode::FORBIDDEN));

    // Bound principals resolve to their own branch and stay isolated, even if
    // a bogus nil binding is supplied.
    let alice_branch = Uuid::new_v4();
    let bob_branch = Uuid::new_v4();
    assert_ne!(alice_branch, bob_branch);
    assert_eq!(
        branch_from_binding(&alice, Some(alice_branch)).ok(),
        Some(alice_branch)
    );
    assert_eq!(
        branch_from_binding(&bob, Some(bob_branch)).ok(),
        Some(bob_branch)
    );
    assert_eq!(branch_from_binding(&alice, Some(Uuid::nil())).ok(), None);

    // Only the desktop-local principal keeps the nil branch.
    let desktop = AuthenticatedUser::desktop_local();
    assert_eq!(branch_from_binding(&desktop, None).ok(), Some(Uuid::nil()));
}
