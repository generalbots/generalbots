//! Read-tracking pixel injection. The pixel must land inside the document for
//! every spelling of the closing tag — HTML mail ships `</body>`, `</BODY>` and
//! bodies with no wrapper at all.
use super::*;

use diesel::r2d2::{ConnectionManager, Pool};

fn state(server_url: &'static str) -> Arc<AppState> {
    let manager = ConnectionManager::new("postgres://localhost/gb_test");
    let pool = Pool::builder().build_unchecked(manager);
    Arc::new(AppState {
        pool: Arc::new(pool),
        get_default_bot: Arc::new(|_conn: &mut diesel::PgConnection| {
            (Uuid::nil(), "default".to_string())
        }),
        secrets_provider: Arc::new(move |key: &str| {
            if key == "server-url" {
                Ok(server_url.to_string())
            } else {
                Err(format!("secret {key} is not set"))
            }
        }),
    })
}

const TRACKING_ID: &str = "3f2504e0-4f89-11d3-9a0c-0305e82c3301";

#[test]
fn the_pixel_is_appended_when_there_is_no_body_tag() {
    let html = inject_tracking_pixel("<p>Hello</p>", TRACKING_ID, &state("https://gb.example"));

    assert!(html.starts_with("<p>Hello</p>"));
    assert!(html.ends_with("/>"));
    assert!(html.contains("https://gb.example/api/email/tracking/pixel/"));
    assert!(html.contains(TRACKING_ID));
}

#[test]
fn the_pixel_is_inserted_before_a_lowercase_closing_tag() {
    let html = inject_tracking_pixel(
        "<html><body><p>Hi</p></body></html>",
        TRACKING_ID,
        &state("https://gb.example"),
    );

    let pixel = html.find("<img").expect("pixel present");
    let closing = html.find("</body>").expect("closing tag kept");
    assert!(pixel < closing, "the pixel must be inside the body");
    assert!(html.ends_with("</body></html>"));
}

#[test]
fn an_uppercase_closing_tag_is_still_detected() {
    let html = inject_tracking_pixel(
        "<HTML><BODY><p>Hi</p></BODY></HTML>",
        TRACKING_ID,
        &state("https://gb.example"),
    );

    let pixel = html.find("<img").expect("pixel must be injected");
    let closing = html.find("</BODY>").expect("closing tag kept");
    assert!(pixel < closing);
    assert!(html.contains("</BODY></HTML>"));
}

#[test]
fn the_pixel_is_inserted_only_once() {
    let html = inject_tracking_pixel(
        "<body><p>Hi</p></body><body><p>Bye</p></body>",
        TRACKING_ID,
        &state("https://gb.example"),
    );

    assert_eq!(html.matches("<img").count(), 1);
}

#[test]
fn a_missing_server_url_yields_a_relative_pixel_url() {
    let html = inject_tracking_pixel("<p>Hi</p>", TRACKING_ID, &state(""));

    assert!(html.contains("src=\"/api/email/tracking/pixel/"));
}

#[test]
fn non_ascii_content_keeps_its_bytes_intact() {
    let body = "<body><p>Ação & coração</p></body>";
    let html = inject_tracking_pixel(body, TRACKING_ID, &state("https://gb.example"));

    assert!(html.contains("<p>Ação & coração</p>"));
    assert!(html.contains("</body>"));
}
