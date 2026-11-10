//! CORS layer behaviour tests (#2129): what a browser actually receives.
//!
//! An empty origin configuration means "no cross-origin access": a foreign
//! `Origin` gets no `Access-Control-Allow-Origin`, credentials are announced
//! only for an explicitly allowlisted origin, a `*` config entry never widens
//! the allowlist, and the `CORS_ALLOW_ANY_ORIGIN=1` development override always
//! refuses credentials.

use super::*;
use crate::cors::cors_test_support::*;

#[tokio::test]
async fn empty_config_denies_arbitrary_origin() -> TestResult {
    let _state = CorsState::with_origins(&[]);

    let response = send(create_cors_layer(), Some(EVIL_ORIGIN), Method::GET).await?;

    assert_denied(&response, "unconfigured GET");
    Ok(())
}

#[tokio::test]
async fn empty_config_denies_preflight() -> TestResult {
    let _state = CorsState::with_origins(&[]);

    let response = send(create_cors_layer(), Some(EVIL_ORIGIN), Method::OPTIONS).await?;

    assert_denied(&response, "unconfigured preflight");
    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_METHODS),
        None
    );
    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_HEADERS),
        None
    );
    Ok(())
}

#[tokio::test]
async fn configured_origin_is_echoed_with_credentials() -> TestResult {
    let _state = CorsState::with_origins(&[APP_ORIGIN]);

    let response = send(create_cors_layer(), Some(APP_ORIGIN), Method::GET).await?;

    assert_allowed_with_credentials(&response, APP_ORIGIN);
    Ok(())
}

#[tokio::test]
async fn configured_list_rejects_unknown_and_lookalike_origin() -> TestResult {
    let _state = CorsState::with_origins(&[APP_ORIGIN]);

    let rejected = [
        EVIL_ORIGIN,
        "https://app.example.evil.example",
        "http://app.example",
    ];

    for origin in rejected {
        let response = send(create_cors_layer(), Some(origin), Method::GET).await?;
        assert_denied(&response, origin);
    }
    Ok(())
}

#[tokio::test]
async fn wildcard_entry_in_config_never_allows_any_origin() -> TestResult {
    let _state = CorsState::with_origins(&["*"]);

    for origin in [EVIL_ORIGIN, APP_ORIGIN] {
        let response = send(create_cors_layer(), Some(origin), Method::GET).await?;
        assert_denied(&response, origin);
    }
    Ok(())
}

#[tokio::test]
async fn dev_override_allows_origin_without_credentials() -> TestResult {
    let state = CorsState::with_origins(&[]);
    state.enable_any_origin_override();

    let response = send(create_cors_layer(), Some(EVIL_ORIGIN), Method::GET).await?;

    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(EVIL_ORIGIN),
        "the dev override echoes a format-valid origin"
    );
    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
        None,
        "the dev override must never allow credentials"
    );
    assert!(
        !response
            .headers()
            .values()
            .any(|value| value.as_bytes() == b"*"),
        "no wildcard is ever emitted"
    );
    Ok(())
}

#[tokio::test]
async fn preflight_does_not_grant_x_user_id() -> TestResult {
    let _state = CorsState::with_origins(&[APP_ORIGIN]);

    let response = send(create_cors_layer(), Some(APP_ORIGIN), Method::OPTIONS).await?;

    assert_eq!(
        header_value(&response, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(APP_ORIGIN)
    );

    let allowed_headers = header_value(&response, header::ACCESS_CONTROL_ALLOW_HEADERS)
        .unwrap_or_default()
        .to_ascii_lowercase();

    assert!(
        !allowed_headers.contains("x-user-id"),
        "x-user-id must not be a CORS-allowed header, got: {allowed_headers}"
    );
    Ok(())
}

#[tokio::test]
async fn every_builder_fails_closed() -> TestResult {
    let _state = CorsState::with_origins(&[]);

    let layers = [
        create_cors_layer(),
        create_cors_layer_for_production(Vec::new()),
        create_cors_layer_with_origins(Vec::new()),
        CorsConfig::production().build(),
        CorsConfig::api().build(),
        CorsConfig::from_config_value(None).build(),
        CorsConfig::from_config_value(Some("  ,  ")).build(),
    ];

    for layer in layers {
        let response = send(layer, Some(EVIL_ORIGIN), Method::GET).await?;
        assert_denied(&response, "empty origin list");
    }
    Ok(())
}
