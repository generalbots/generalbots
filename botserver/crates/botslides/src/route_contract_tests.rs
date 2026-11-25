//! Route-contract guard for #2922.
//!
//! Four Slides paths were reported 404 in production while other Slides
//! endpoints worked: `GET /api/slides/media/list`, `POST /api/slides/slide/add`,
//! `POST /api/slides/presenter/start` and `/ws/slides/:presentation_id`. The
//! source already declares them, but nothing proved the *assembled* router
//! actually matches them — a reordered/feature-less build could silently drop
//! a declaration and every endpoint would look healthy in source review.
//!
//! These tests drive the real `configure_slides_routes()` router. They assert
//! each required path reaches its handler (not the desktop-shell fallback and
//! not a 404), that the WebSocket path really upgrades to 101 with a verified
//! token over a real TCP socket, and — as a non-vacuity control — that an
//! unregistered path still returns 404. A drive-less state is deliberate: the
//! handlers answer 503 before touching storage, so a matched route has an
//! unambiguous status. `oneshot` cannot provide hyper's `OnUpgrade` extension,
//! so the WS checks use `axum::serve` on an ephemeral port instead.

use crate::auth::SlidesUser;
use crate::collaboration::ws_auth::TEST_JWT_SECRET;
use crate::routes::configure_slides_routes;
use crate::storage::DriveOps;
use crate::SlidesState;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;
use tower::ServiceExt;

/// Drive is intentionally absent in this contract: a matched handler must
/// stop at the "Drive not configured" 503 instead of touching storage.
#[derive(Clone, Default)]
struct NoopDrive;

impl DriveOps for NoopDrive {
    async fn put_object(
        &self,
        _bucket: &str,
        _key: &str,
        _body: Vec<u8>,
        _content_type: &str,
    ) -> Result<(), String> {
        Err("drive intentionally unavailable in route contract".to_string())
    }

    async fn get_object(&self, _bucket: &str, _key: &str) -> Result<Vec<u8>, String> {
        Err("drive intentionally unavailable in route contract".to_string())
    }

    async fn list_objects(&self, _bucket: &str, _prefix: &str) -> Result<Vec<String>, String> {
        Err("drive intentionally unavailable in route contract".to_string())
    }

    async fn delete_object(&self, _bucket: &str, _key: &str) -> Result<(), String> {
        Err("drive intentionally unavailable in route contract".to_string())
    }
}

fn state() -> Arc<SlidesState<NoopDrive>> {
    let manager =
        diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new("postgres://localhost/unused");
    let conn = diesel::r2d2::Pool::builder()
        .max_size(1)
        .build_unchecked(manager);
    Arc::new(SlidesState {
        drive: None,
        conn: Arc::new(conn),
        get_default_bot: Arc::new(|_conn: &mut diesel::PgConnection| {
            (uuid::Uuid::nil(), "route-contract-bot".to_string())
        }),
    })
}

/// The production router installs `slides_user_middleware`; the test seam uses
/// the same `SlidesUser` extension so handler extraction is identical.
async fn as_alice(mut request: Request<Body>, next: Next) -> Response {
    request
        .extensions_mut()
        .insert(SlidesUser::new("user-a", "Alice"));
    next.run(request).await
}

fn app() -> axum::Router {
    configure_slides_routes::<NoopDrive>()
        .layer(axum::middleware::from_fn(as_alice))
        .with_state(state())
}

fn req(method: &str, uri: &str, body: &'static str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// HS256 token built exactly like `ws_auth_tests`, using the test-only
/// verification seam so the upgrade path is proven end to end offline.
fn verified_ws_token() -> String {
    let header = b64url(br#"{"alg":"HS256","typ":"JWT"}"#);
    let claims = serde_json::json!({
        "sub": "user-a",
        "email": "alice@example.com",
        "exp": chrono::Utc::now().timestamp() + 3600,
    });
    let payload = b64url(claims.to_string().as_bytes());
    let signing_input = format!("{header}.{payload}");
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(TEST_JWT_SECRET.as_bytes()) else {
        return signing_input;
    };
    mac.update(signing_input.as_bytes());
    format!("{signing_input}.{}", b64url(&mac.finalize().into_bytes()))
}

async fn status(router: axum::Router, request: Request<Body>) -> StatusCode {
    router
        .oneshot(request)
        .await
        .expect("route contract response")
        .status()
}

/// Real socket handshake: `oneshot` has no `OnUpgrade` extension and would
/// always answer 426 before the handler runs, so it cannot prove a 101.
async fn handshake(path: &str) -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, app()).await;
    });

    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("connect to test server");
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .await
        .expect("write handshake");

    let mut response = Vec::new();
    let mut chunk = [0u8; 2048];
    for _ in 0..8 {
        let read = stream.read(&mut chunk).await.expect("read handshake");
        if read == 0 {
            break;
        }
        response.extend_from_slice(&chunk[..read]);
        let text = String::from_utf8_lossy(&response);
        if text.contains("\r\n\r\n") {
            break;
        }
    }
    server.abort();
    String::from_utf8_lossy(&response).into_owned()
}

#[tokio::test]
async fn required_slides_http_routes_and_websocket_path_are_registered() {
    let deck = "11111111-2222-3333-4444-555555555555";

    // Control: a sibling route that has always worked still answers.
    assert_eq!(
        status(app(), req("GET", "/api/slides/list", "")).await,
        StatusCode::OK
    );

    // The three HTTP endpoints from #2922 must reach their handlers, not 404.
    let media_uri = format!("/api/slides/media/list?presentation_id={deck}&slide_index=0");
    assert_eq!(
        status(app(), req("GET", &media_uri, "")).await,
        StatusCode::SERVICE_UNAVAILABLE,
        "GET /api/slides/media/list must be registered (503 = handler reached)"
    );
    assert_eq!(
        status(
            app(),
            req(
                "POST",
                "/api/slides/slide/add",
                r#"{"presentation_id":"11111111-2222-3333-4444-555555555555","layout":"blank"}"#
            )
        )
        .await,
        StatusCode::SERVICE_UNAVAILABLE,
        "POST /api/slides/slide/add must be registered"
    );
    assert_eq!(
        status(
            app(),
            req(
                "POST",
                "/api/slides/presenter/start",
                r#"{"presentation_id":"11111111-2222-3333-4444-555555555555"}"#
            )
        )
        .await,
        StatusCode::SERVICE_UNAVAILABLE,
        "POST /api/slides/presenter/start must be registered"
    );

    // The collaboration upgrade itself must answer 101 over a real socket,
    // never 404. This is the same handshake the browser performs.
    let ws_uri = format!("/ws/slides/{deck}?token={}", verified_ws_token());
    let upgrade = handshake(&ws_uri).await;
    assert!(
        upgrade.starts_with("HTTP/1.1 101"),
        "/ws/slides/:presentation_id must upgrade with a verified token, got: {upgrade}"
    );

    // Non-vacuity: this router really does answer 404 for an unknown path, so
    // the assertions above cannot pass merely because everything returns 200.
    assert_eq!(
        status(
            app(),
            req("GET", "/api/slides/__contract__/no/such/route", "")
        )
        .await,
        StatusCode::NOT_FOUND,
    );
}

#[tokio::test]
async fn slides_websocket_route_reaches_the_auth_handler_without_a_token() {
    // A request with no token still has to hit the route's own verifier (401
    // plus its machine-readable reason), never the fallback or a 404.
    let response = handshake("/ws/slides/11111111-2222-3333-4444-555555555555").await;
    assert!(
        response.starts_with("HTTP/1.1 401"),
        "unauthenticated slides WS must be refused by the handler: {response}"
    );
    assert!(
        response.to_ascii_lowercase().contains("x-auth-reason:"),
        "the refusal must come from handle_slides_websocket"
    );
}
