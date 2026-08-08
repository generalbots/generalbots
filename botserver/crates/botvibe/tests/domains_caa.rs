use botvibe::domains::ProjectDomains;

#[test]
fn caa_no_records_allows_acme() {
    assert!(ProjectDomains::caa_allows_acme(""));
    assert!(ProjectDomains::caa_allows_acme("  \n \n"));
}

#[test]
fn caa_letsencrypt_issue_allowed() {
    let records = "0 issue \"letsencrypt.org\"\n0 iodef \"mailto:security@example.com\"";
    assert!(ProjectDomains::caa_allows_acme(records));
}

#[test]
fn caa_other_ca_blocks_acme() {
    let records = "0 issue \"comodoca.com\"";
    assert!(!ProjectDomains::caa_allows_acme(records));
}

#[test]
fn caa_issue_none_blocks_everything() {
    assert!(!ProjectDomains::caa_allows_acme("0 issue \";\""));
    assert!(!ProjectDomains::caa_allows_acme("0 issue \"\""));
}

#[test]
fn caa_iodef_only_allows_acme() {
    let records = "0 iodef \"mailto:security@example.com\"";
    assert!(ProjectDomains::caa_allows_acme(records));
}

#[test]
fn caa_case_insensitive_matching() {
    assert!(ProjectDomains::caa_allows_acme("0 ISSUE \"LetsEncrypt.ORG\""));
    let records = "0 Issue \"ComodoCA.com\"";
    assert!(!ProjectDomains::caa_allows_acme(records));
}

#[test]
fn caa_mixed_records_still_allow_when_acme_present() {
    let records = "0 issue \"comodoca.com\"\n0 issue \"letsencrypt.org\"";
    assert!(ProjectDomains::caa_allows_acme(records));
}

#[test]
fn caa_report_shape_has_records_and_verdict() {
    let report = ProjectDomains::caa_report("unresolvable.invalid.example");
    assert!(report["records"].is_string());
    assert!(report["allows_acme"].is_boolean());
    assert!(report["check"].is_string());
}