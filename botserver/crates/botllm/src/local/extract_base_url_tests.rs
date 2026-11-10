//! Offline tests for the llama.cpp endpoint parsing helpers (issue #2075).
//! `extract_base_url`, `extract_port` and `is_loopback_url` are pure string
//! transforms: no socket is opened and no process is spawned.

use super::*;

#[test]
fn base_url_keeps_scheme_host_and_port_but_drops_the_path() {
    let cases = [
        (
            "https://api.example.test/v1/chat/completions",
            "https://api.example.test",
        ),
        ("http://127.0.0.1:8081/v1", "http://127.0.0.1:8081"),
        ("http://localhost:8081", "http://localhost:8081"),
        ("https://api.example.test/", "https://api.example.test"),
        (
            "https://api.example.test/v1?key=secret",
            "https://api.example.test",
        ),
    ];

    for (input, expected) in cases {
        assert_eq!(extract_base_url(input), expected, "input {input}");
    }
}

#[test]
fn unparsable_urls_are_returned_verbatim() {
    for input in ["", "not a url", "://missing-scheme"] {
        assert_eq!(extract_base_url(input), input, "input {input}");
    }
}

#[test]
fn only_loopback_hosts_are_reported_as_loopback() {
    let cases = [
        ("http://localhost:8081/v1", true),
        ("http://127.0.0.1:8081", true),
        ("http://[::1]:8081/v1", true),
        ("https://api.example.test/v1", false),
        ("http://127.0.0.2:8081", false),
        ("not a url", false),
    ];

    for (input, expected) in cases {
        assert_eq!(is_loopback_url(input), expected, "input {input}");
    }
}

#[test]
fn port_is_extracted_from_a_full_url() {
    let cases = [
        ("http://127.0.0.1:8081/v1", "8081"),
        ("http://127.0.0.1:8082", "8082"),
        ("http://127.0.0.1:8083/", "8083"),
        ("http://[::1]:8084/v1", "8084"),
    ];

    for (input, expected) in cases {
        assert_eq!(extract_port(input), expected, "input {input}");
    }
}

#[test]
fn a_url_without_a_port_falls_back_to_the_documented_default() {
    // `--port` is passed straight to llama-server: an empty argument makes the
    // server fail to start, so a missing port must resolve to the default.
    for input in [
        "http://127.0.0.1/v1",
        "http://localhost",
        "http://[::1]/v1",
        "",
        "not a url",
    ] {
        assert_eq!(extract_port(input), "8081", "input {input}");
    }
}

#[test]
fn non_numeric_port_segments_are_not_forwarded_to_the_server() {
    // A hostname with no port must never leak the scheme or path into `--port`.
    assert_eq!(extract_port("http://llm.internal/8081/v1"), "8081");
    assert_eq!(extract_port("http://llm.internal/v1"), "8081");
}
