//! `proxy_sites::tests` — split per #1443.

use super::*;

    use super::*;
    use crate::vm_lifecycle::VmLifecycle;

    #[test]
    fn site_slug_matches_alm_repo_slug() {
        assert_eq!(site_slug("My Web App"), VmLifecycle::alm_repo("My Web App"));
        assert_eq!(site_slug("site1276"), "site1276");
    }

    #[test]
    fn marker_file_is_hidden() {
        assert!(MARKER_FILE.starts_with('.'));
    }

    #[test]
    fn looks_like_python_detects_entry_and_requirements() {
        let py = serde_json::json!([{ "path": "app.py", "content": "" }]);
        let req = serde_json::json!([{ "path": "requirements.txt", "content": "" }]);
        let other = serde_json::json!([{ "path": "index.html", "content": "" }]);
        assert!(looks_like_python(py.as_array().unwrap()));
        assert!(looks_like_python(req.as_array().unwrap()));
        assert!(!looks_like_python(other.as_array().unwrap()));
    }

    #[test]
    fn python_port_is_stable_and_in_range() {
        let a = python_port("site1276");
        assert_eq!(a, python_port("site1276"));
        assert!((20000..30000).contains(&a));
    }

    #[test]
    fn site_block_static_has_file_server() {
        let d = crate::publish::published_domain();
        let b = site_block_with_mode("mysite", false, true);
        assert!(b.starts_with(&format!("mysite.{d}")));
        assert!(b.contains("file_server"));
        assert!(b.contains(&format!("root * {PROXY_SITES_ROOT}/mysite")));
        assert!(b.contains("tls internal"));
        // Production mode: automatic ACME — no explicit tls directive.
        let prod = site_block_with_mode("mysite", false, false);
        assert!(!prod.contains("tls "));
        assert!(prod.contains("file_server"));
    }

    #[test]
    fn site_block_python_has_reverse_proxy() {
        let b = site_block_with_mode("pysite", true, true);
        assert!(b.contains(&format!("reverse_proxy 127.0.0.1:{}", python_port("pysite"))));
        let prod = site_block_with_mode("pysite", true, false);
        assert!(prod.contains(&format!("reverse_proxy 127.0.0.1:{}", python_port("pysite"))));
        assert!(!prod.contains("tls "));
    }

    #[test]
    fn validate_slug_enforces_enterprise_rules() {
        assert!(validate_slug("mysite").is_ok());
        assert!(validate_slug("my-site-2").is_ok());
        // too short / too long
        assert!(validate_slug("ab").is_err());
        assert!(validate_slug(&"a".repeat(64)).is_err());
        // charset
        assert!(validate_slug("MySite").is_err());
        assert!(validate_slug("my_site").is_err());
        assert!(validate_slug(".hidden").is_err());
        // hyphen edges
        assert!(validate_slug("-lead").is_err());
        assert!(validate_slug("trail-").is_err());
        // reserved infrastructure names
        for r in RESERVED_SLUGS {
            assert!(validate_slug(r).is_err(), "{r} must be reserved");
        }
    }

    #[test]
    fn check_payload_limits_enforce_caps() {
        let small = serde_json::json!([{ "path": "index.html", "content": [104, 105] }]);
        assert!(check_payload_limits(small.as_array().unwrap()).is_ok());
        // string content is accepted as utf8 bytes
        let textual = serde_json::json!([{ "path": "index.html", "content": "hi" }]);
        assert!(check_payload_limits(textual.as_array().unwrap()).is_ok());
        // per-file cap: 11 MiB single file
        let big = serde_json::json!([{
            "path": "big.bin",
            "content": "x".repeat(MAX_SINGLE_FILE_BYTES + 1)
        }]);
        assert!(check_payload_limits(big.as_array().unwrap()).is_err());
        // total cap: many files just under the per-file cap
        let file = "x".repeat(MAX_SINGLE_FILE_BYTES - 1);
        let many: Vec<serde_json::Value> = (0..6)
            .map(|i| serde_json::json!({ "path": format!("f{i}.bin"), "content": file }))
            .collect();
        assert!(check_payload_limits(&many).is_err());
        // file count cap
        let too_many: Vec<serde_json::Value> = (0..(MAX_FILES + 1))
            .map(|i| serde_json::json!({ "path": format!("f{i}.txt"), "content": "x" }))
            .collect();
        assert!(check_payload_limits(&too_many).is_err());
    }

    #[test]
    fn check_serveability_requires_entry_files() {
        let no_index = serde_json::json!([{ "path": "page.html", "content": "x" }]);
        assert!(check_serveability(no_index.as_array().unwrap(), false).is_err());
        let with_index =
            serde_json::json!([{ "path": "index.html", "content": "x" }]);
        assert!(check_serveability(with_index.as_array().unwrap(), false).is_ok());
        let no_app = serde_json::json!([{ "path": "main.py", "content": "x" }]);
        assert!(check_serveability(no_app.as_array().unwrap(), true).is_err());
        let with_app = serde_json::json!([{ "path": "app.py", "content": "x" }]);
        assert!(check_serveability(with_app.as_array().unwrap(), true).is_ok());
    }

    #[test]
    fn drop_site_block_keeps_peers_and_removes_target() {
        let d = crate::publish::published_domain();
        let section = format!(
            "a.{d} {{\n\ttls internal\n}}\nb.{d} {{\n\ttls internal\n}}\n"
        );
        let kept = drop_site_block(&section, &format!("a.{d}"));
        assert!(!kept.contains(&format!("a.{d}")));
        assert!(kept.contains(&format!("b.{d}")));
        // Unknown host → section unchanged.
        let kept2 = drop_site_block(&section, &format!("zz.{d}"));
        assert!(kept2.contains(&format!("a.{d}")));
        assert!(kept2.contains(&format!("b.{d}")));
    }

    #[test]
    fn drop_foreign_domain_blocks_removes_stale_domains() {
        let d = crate::publish::published_domain();
        let stale_domain = if d == "generalbots.org" { "gb.solutions" } else { "generalbots.org" };
        let section = format!(
            "site.{d} {{\n\tfile_server\n}}\nstale.{stale_domain} {{\n\tfile_server\n}}\ncustom.example.com {{\n\tfile_server\n}}\n"
        );
        let kept = drop_foreign_domain_blocks(&section);
        assert!(kept.contains(&format!("site.{d}")));
        assert!(!kept.contains("stale."));
        assert!(!kept.contains("custom.example.com"));
    }

    #[test]
    fn extract_section_handles_missing_markers() {
        assert_eq!(extract_section("no markers here"), "");
        let doc = format!("head\n{SECTION_BEGIN}\nb.{}.com {{}}\n{SECTION_END}\ntail", "x");
        assert!(extract_section(&doc).contains("b."));
    }

    #[test]
    fn upsert_replaces_marker_section_only() {
        let original = format!(
            "pre existing site one {{\n\troot * /srv/one\n}}\n\n{SECTION_BEGIN}\nold.example.com {{}}\n{SECTION_END}\n\npost site two {{}}\n"
        );
        let blocks = "new.example.com {\n\tfile_server\n}\n";
        let managed = format!("{SECTION_BEGIN}\n{blocks}{SECTION_END}\n");
        let (b, e) = (
            original.find(SECTION_BEGIN).unwrap(),
            original.find(SECTION_END).unwrap(),
        );
        let updated = format!(
            "{}{}{}",
            &original[..b],
            managed,
            original[e + SECTION_END.len()..].trim_start_matches('\n')
        );
        assert!(updated.contains("pre existing site one"));
        assert!(updated.contains("post site two"));
        assert!(updated.contains("new.example.com"));
        assert!(!updated.contains("old.example.com"));
        assert_eq!(updated.matches(SECTION_BEGIN).count(), 1);
    }

    #[test]
    fn upsert_appends_section_when_missing() {
        let original = "admin {\n\tlocal\n}\n".to_string();
        let blocks = "x.example.com {\n}\n";
        let managed = format!("{SECTION_BEGIN}\n{blocks}{SECTION_END}\n");
        let updated = format!("{}\n{managed}", original.trim_end());
        assert!(updated.starts_with("admin {"));
        assert!(updated.contains(SECTION_BEGIN));
        assert!(updated.trim_end().ends_with(SECTION_END));
    }

