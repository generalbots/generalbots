#![no_main]
//! Channel webhook bodies (#2119).
//!
//! WhatsApp posts JSON, Twilio posts form-encoded pairs and both carry a
//! signature header. Everything below is reachable from an unauthenticated
//! public endpoint, so the invariant is: parse garbage into `Err`/`None`, never
//! panic, and never report a signature as valid for a body it does not cover.

use libfuzzer_sys::fuzz_target;

const APP_SECRET: &str = "fuzz-app-secret";

fuzz_target!(|data: &[u8]| {
    // JSON webhook body (WhatsApp Cloud API shape).
    if let Ok(body) = std::str::from_utf8(data) {
        let _ = serde_json::from_str::<botwhatsapp::models::WhatsAppWebhookPayload>(body);
        // Form-encoded bodies arrive as flat key/value text; splitting on
        // `&`/`=` must survive unbalanced input.
        for pair in body.split('&') {
            let mut parts = pair.splitn(2, '=');
            let _ = (parts.next(), parts.next());
        }
    }

    // Signature verification parses `sha256=<hex>` from an attacker-supplied
    // header; a malformed digest must be a rejection, not a panic.
    let signature = String::from_utf8_lossy(data);
    assert!(
        !botwhatsapp::utils::verify_webhook_signature(data, &signature, ""),
        "an empty app secret must never validate a delivery"
    );
    assert_eq!(
        botwhatsapp::utils::verify_webhook_signature(data, &signature, APP_SECRET),
        botwhatsapp::utils::verify_webhook_signature(data, &signature, APP_SECRET),
        "signature verification must be deterministic"
    );
});
