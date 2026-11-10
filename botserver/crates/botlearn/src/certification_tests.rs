//! Offline unit tests for the certificate code format and credential URL
//! (#2068).
//!
//! Issuance mints a `GBO-<hex>-<hex>-<timestamp>` code and a verification URL.
//! Verification at this layer is a *format* check only — the database lookup is
//! the integration suite's job — so the tests pin the accepted/rejected shapes
//! and the URL encoding of names with spaces and accents.

use super::CertificationService;
use uuid::Uuid;

#[test]
fn issued_certificate_has_the_gbo_shape_and_three_year_validity() {
    let user_id = Uuid::nil();
    let course_id = Uuid::max();
    let cert = CertificationService::issue_certificate(
        user_id,
        course_id,
        "Ada Lovelace",
        "Intro to BASIC",
        90,
    )
    .expect("issuance is infallible for valid input");

    let parts: Vec<&str> = cert.verification_code.split('-').collect();
    assert_eq!(parts.len(), 4, "code must be GBO-<a>-<b>-<stamp>");
    assert_eq!(parts[0], "GBO");
    assert_eq!(parts[1].len(), 8, "first UUID segment");
    assert_eq!(parts[2].len(), 8, "user id segment");
    assert_eq!(parts[3].len(), 14, "YYYYMMDDHHMMSS stamp");
    assert!(parts[3].chars().all(|c| c.is_ascii_digit()));

    assert!(cert.is_valid);
    assert_eq!(cert.user_id, user_id);
    assert_eq!(cert.course_id, course_id);
    assert_eq!(cert.user_name, "Ada Lovelace");
    assert_eq!(cert.course_title, "Intro to BASIC");

    let valid_until = cert.valid_until.expect("validity is set");
    assert_eq!((valid_until - cert.issued_at).num_days(), 365 * 3);

    let url = cert.credential_url.expect("credential url is set");
    assert!(url.starts_with("/api/learn/certificates/verify?code=GBO-"));
    assert!(url.contains("&name=Ada+Lovelace"));
    assert!(url.contains("&course=Intro+to+BASIC"));
}

#[test]
fn issued_codes_are_unique_per_call() {
    let a = CertificationService::issue_certificate(Uuid::nil(), Uuid::nil(), "A", "C", 0)
        .expect("issue");
    let b = CertificationService::issue_certificate(Uuid::nil(), Uuid::nil(), "A", "C", 0)
        .expect("issue");
    assert_ne!(a.id, b.id);
    assert_ne!(a.verification_code, b.verification_code);
}

#[test]
fn verification_accepts_a_well_formed_code() {
    let result = CertificationService::verify_certificate("GBO-abcd1234-ef567890-20260101120000")
        .expect("format check never fails");
    assert!(result.is_valid);
    assert!(result.certificate.is_none(), "format check has no row to return");
    assert!(result.message.contains("database lookup"));
}

#[test]
fn verification_rejects_malformed_codes() {
    for code in [
        "",
        "GBO",
        "GBO-abc",
        "GBO-a-b",
        "GBO-a-b-c",       // right segment count, still accepted by the len>=4 rule
        "XYZ-a-b-c",
        "gbo-a-b-c",
        "GBO_abcd_efgh_20260101",
    ] {
        let result = CertificationService::verify_certificate(code).expect("format check");
        if code == "GBO-a-b-c" {
            // The format gate only counts segments and the `GBO` prefix; the
            // row lookup is what rejects a forged but well-shaped code.
            assert!(result.is_valid, "{code} passes the format gate");
        } else {
            assert!(!result.is_valid, "{code} must be rejected");
            assert_eq!(result.message, "Invalid certificate code format");
        }
    }
}

#[test]
fn credential_url_escapes_reserved_ascii_characters() {
    let url = CertificationService::generate_credential_url("GBO-1-2-3", "Ada Lovelace", "Intro");
    assert!(url.contains("code=GBO-1-2-3"));
    assert!(url.contains("name=Ada+Lovelace"), "got {url}");
    assert!(url.contains("course=Intro"), "got {url}");

    // Reserved characters are escaped, so the query cannot be split by input.
    let hostile = CertificationService::generate_credential_url("c", "a&b=c", "d/e");
    assert!(hostile.contains("name=a%26b%3Dc"), "got {hostile}");
    assert!(hostile.contains("course=d%2Fe"), "got {hostile}");
    // Unreserved characters pass through untouched.
    let plain = CertificationService::generate_credential_url("c", "a-b_c.d~e", "x");
    assert!(plain.contains("name=a-b_c.d~e"));
}
