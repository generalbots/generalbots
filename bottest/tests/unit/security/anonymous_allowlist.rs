//! Anonymous access is an exact `(method, path)` declaration (#2133).
//!
//! The previous model treated an allowlist entry as a namespace: `/api/auth`
//! also admitted `/api/auth/directory-config`, and `/api/products/items` also
//! admitted `POST /api/products/items`. These tests drive the production
//! declaration list and the webhook signature gate, so a route that widens
//! back into a prefix fails here instead of in production.

use botsecurity_auth::auth_api::platform_public_routes;
use botwhatsapp::utils::verify_webhook_signature;
use hmac::{Hmac, Mac};
use http::Method;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Whether the production allowlist admits this exact request.
fn is_public(method: Method, path: &str) -> bool {
    platform_public_routes()
        .iter()
        .any(|route| route.matches(&method, path))
}

#[test]
fn a_new_route_under_an_allowlisted_prefix_is_not_anonymous() {
    for path in [
        // `/api/auth` used to admit every sibling, including the directory
        // configuration that exposes the identity-provider URL.
        "/api/auth/directory-config",
        "/api/auth/login/extra",
        "/api/auth/2fa/verify/extra",
        // `/api/products/items` used to admit the whole subtree.
        "/api/products/items/00000000-0000-0000-0000-000000000000/stock",
        // `/api/sheet` and its UI fragments used to be anonymous.
        "/api/sheet/list",
        "/api/sheet/load",
        "/api/ui/sheet/fragments/sheet-list",
        "/suite/sheet/fragments/sheet-list",
        // A future `/api/cloud/auth/*` route is not automatically public.
        "/api/cloud/auth/anything",
    ] {
        assert!(
            !is_public(Method::GET, path),
            "{path} must require a verified identity (#2133)"
        );
    }
}

#[test]
fn a_write_method_is_never_anonymous_by_prefix() {
    for (method, path) in [
        (Method::POST, "/api/products/items"),
        (Method::PUT, "/api/products/items/00000000-0000-0000-0000-000000000000"),
        (Method::DELETE, "/api/products/items/00000000-0000-0000-0000-000000000000"),
        (Method::POST, "/api/products/items/00000000-0000-0000-0000-000000000000/stock"),
        (Method::POST, "/api/sheet/save"),
        (Method::POST, "/api/sheet/delete"),
        (Method::POST, "/api/cloud/store/purchase"),
        (Method::POST, "/api/auth/directory-config"),
    ] {
        assert!(
            !is_public(method.clone(), path),
            "{method} {path} must not be reachable without a credential (#2133)"
        );
    }
}

#[test]
fn the_real_public_endpoints_still_work() {
    // Health probes.
    assert!(is_public(Method::GET, "/health"));
    assert!(is_public(Method::GET, "/api/health"));
    // Login/signup are explicit credential-entry routes (the request carries
    // the credential being verified).
    assert!(is_public(Method::POST, "/api/auth/login"));
    assert!(is_public(Method::POST, "/api/auth/refresh"));
    assert!(is_public(Method::POST, "/api/auth/bootstrap"));
    assert!(is_public(Method::POST, "/api/cloud/auth/login"));
    assert!(is_public(Method::POST, "/api/cloud/auth/signup"));
    // Public catalog reads remain public; the writes above do not.
    assert!(is_public(Method::GET, "/api/products/items"));
    assert!(is_public(Method::GET, "/api/catalog/products"));
    // Provider webhooks must be admitted so the handler can verify them.
    assert!(is_public(Method::POST, "/api/whatsapp/webhook"));
    assert!(is_public(Method::POST, "/api/msteams/messages"));
    assert!(is_public(Method::POST, "/webhook/telegram/acme"));
    // Static UI files are public; server-rendered fragments under the same
    // prefix are not.
    assert!(is_public(Method::GET, "/suite/desktop.html"));
    assert!(is_public(Method::GET, "/suite/sheet/sheet.js"));
    assert!(!is_public(Method::GET, "/suite/dashboards"));
}

/// `X-Hub-Signature-256: sha256=<lowercase hex>` over the raw bytes.
fn meta_signature(body: &[u8], secret: &str) -> String {
    // HMAC accepts a key of any length; `None` is unreachable.
    let digest = HmacSha256::new_from_slice(secret.as_bytes())
        .ok()
        .map(|mut mac| {
            mac.update(body);
            mac.finalize().into_bytes().to_vec()
        })
        .expect("HMAC accepts any key length");
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    format!("sha256={hex}")
}

/// A delivery that passes the transport gate still has to prove its origin:
/// the handler runs the Meta `X-Hub-Signature-256` HMAC over the raw bytes.
#[test]
fn a_webhook_with_a_valid_signature_still_passes_the_gate() {
    assert!(
        is_public(Method::POST, "/api/whatsapp/webhook"),
        "the signature gate can only run if the route is admitted"
    );

    let body = br#"{"object":"whatsapp_business_account","entry":[]}"#;
    let secret = "offline-allowlist-app-secret";
    let signature = meta_signature(body, secret);

    assert!(verify_webhook_signature(body, &signature, secret));
    assert!(!verify_webhook_signature(
        br#"{"object":"whatsapp_business_account","entry":[{"id":"tampered"}]}"#,
        &signature,
        secret
    ));
    assert!(!verify_webhook_signature(body, "", secret));
}
