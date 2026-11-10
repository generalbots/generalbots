//! Write-scope regression tests for the products API (issue #2128).
//!
//! Offline: the handlers reject an unverified caller *before* touching the
//! pool, so a `build_unchecked` pool (which never connects) is enough to prove
//! the `401` boundary. Branch-scope assertions run against the same
//! `verified_branch` helper the handlers call; the signed `branch_id` claim
//! needs no database round-trip, so they are fully offline too.

use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use botsecurity_core::verified_jwt::resolve_saas_jwt_secret;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn default_branch(_conn: &mut PgConnection) -> Uuid {
    Uuid::new_v4()
}

/// State with a detached pool and a *configured* default branch: the guard must
/// reject an unverified caller even though a fallback exists (that fallback is
/// what used to place the row in another tenant's catalogue).
fn detached_state() -> Arc<ProductsState> {
    let manager = ConnectionManager::<PgConnection>::new("postgres://offline/offline");
    let pool = Pool::builder().build_unchecked(manager);

    Arc::new(ProductsState {
        pool: Arc::new(pool),
        get_default_bot: Some(default_branch),
    })
}

fn mint(claims: &serde_json::Value, secret: &str) -> String {
    let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = URL_SAFE_NO_PAD.encode(claims.to_string().as_bytes());
    let signing_input = format!("{header}.{payload}");
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return String::new();
    };
    mac.update(signing_input.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

    format!("{signing_input}.{signature}")
}

fn claims(branch: Uuid) -> serde_json::Value {
    serde_json::json!({
        "branch_id": branch.to_string(),
        "sub": "user-1",
        "email": "user@example.com",
        "exp": chrono::Utc::now().timestamp() + 3600,
    })
}

fn signed_for(branch: Uuid) -> String {
    mint(&claims(branch), &resolve_saas_jwt_secret())
}

fn with_bearer(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(value) = format!("Bearer {token}").parse() {
        headers.insert("authorization", value);
    }
    headers
}

fn with_branch_header(branch: Uuid) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Ok(value) = branch.to_string().parse() {
        headers.insert("x-branch-id", value);
    }
    headers
}

fn product_request() -> CreateProductRequest {
    CreateProductRequest {
        name: "Injected".to_string(),
        description: None,
        sku: Some("evil".to_string()),
        category: None,
        product_type: None,
        price: 0.0,
        cost: None,
        currency: None,
        tax_rate: None,
        unit: None,
        stock_quantity: Some(999_999),
        low_stock_threshold: None,
        images: None,
    }
}

fn stock_request() -> AdjustStockRequest {
    AdjustStockRequest {
        quantity: -100_000,
        movement_type: "out".to_string(),
        notes: None,
        reference_type: None,
        reference_id: None,
    }
}

#[tokio::test]
async fn anonymous_create_product_is_rejected() {
    let result = create_product(
        State(detached_state()),
        HeaderMap::new(),
        Json(product_request()),
    )
    .await;

    assert!(
        matches!(result, Err((StatusCode::UNAUTHORIZED, _))),
        "an anonymous POST /api/products/items must be 401"
    );
}

#[tokio::test]
async fn create_product_without_a_resolved_branch_is_rejected() {
    let result = create_product(
        State(detached_state()),
        with_bearer("not-a-verified-token"),
        Json(product_request()),
    )
    .await;

    assert!(
        matches!(result, Err((StatusCode::UNAUTHORIZED, _))),
        "an unverifiable token must not fall back to the default branch"
    );
}

#[tokio::test]
async fn anonymous_stock_adjustment_is_rejected() {
    let result = adjust_stock(
        State(detached_state()),
        HeaderMap::new(),
        Path(Uuid::new_v4()),
        Json(stock_request()),
    )
    .await;

    assert!(
        matches!(result, Err((StatusCode::UNAUTHORIZED, _))),
        "an anonymous PUT /api/products/items/:id/stock must be 401"
    );
}

#[tokio::test]
async fn stock_adjustment_without_a_resolved_branch_is_rejected() {
    let result = adjust_stock(
        State(detached_state()),
        with_bearer("not-a-verified-token"),
        Path(Uuid::new_v4()),
        Json(stock_request()),
    )
    .await;

    assert!(
        matches!(result, Err((StatusCode::UNAUTHORIZED, _))),
        "an unverifiable token must not fall back to the default branch"
    );
}

#[test]
fn a_caller_cannot_aim_a_write_at_another_branch() {
    let state = detached_state();
    let branch_a = Uuid::new_v4();
    let branch_b = Uuid::new_v4();

    // Client input is never the scope: a branch header with no verified token
    // resolves to nothing, even though a default branch is configured.
    assert_eq!(verified_branch(&with_branch_header(branch_b), &state.pool), None);

    // A verified token for A stays A, even when the request asks for B.
    let mut headers = with_bearer(&signed_for(branch_a));
    if let Ok(value) = branch_b.to_string().parse() {
        headers.insert("x-branch-id", value);
    }
    assert_eq!(verified_branch(&headers, &state.pool), Some(branch_a));
}

#[test]
fn a_forged_token_cannot_select_another_branch() {
    let state = detached_state();
    let branch_b = Uuid::new_v4();
    let forged = mint(&claims(branch_b), "an-attacker-controlled-secret");

    assert_eq!(verified_branch(&with_bearer(&forged), &state.pool), None);
}
