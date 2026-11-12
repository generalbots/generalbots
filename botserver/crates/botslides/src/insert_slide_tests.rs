//! Insert-a-slide path (UAT SLIDE-02 / #2247).
//!
//! The toolbar's "New Slide" button must append a slide to the deck that is
//! on screen and the result must survive a reload. These tests drive the real
//! router (`ui_fragments` modals/forms + `/api/slides/slide/add`) with an
//! in-memory Drive, so a broken route, payload or insert position fails here
//! instead of in the browser.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use crate::auth::SlidesUser;
use crate::routes::configure_slides_routes;
use crate::storage::{
    create_new_presentation, load_presentation_by_id, save_presentation_to_drive, DriveOps,
};
use crate::SlidesState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

#[derive(Clone, Default)]
struct MemoryDrive {
    objects: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl MemoryDrive {
    fn object_key(bucket: &str, key: &str) -> String {
        format!("{bucket}/{key}")
    }
}

impl DriveOps for MemoryDrive {
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        _content_type: &str,
    ) -> Result<(), String> {
        let mut objects = self.objects.lock().map_err(|error| error.to_string())?;
        objects.insert(Self::object_key(bucket, key), body);
        Ok(())
    }

    async fn get_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>, String> {
        let objects = self.objects.lock().map_err(|error| error.to_string())?;
        objects
            .get(&Self::object_key(bucket, key))
            .cloned()
            .ok_or_else(|| "404 Not Found".to_string())
    }

    async fn list_objects(&self, bucket: &str, prefix: &str) -> Result<Vec<String>, String> {
        let needle = Self::object_key(bucket, prefix);
        let objects = self.objects.lock().map_err(|error| error.to_string())?;
        let mut keys: Vec<String> = objects
            .keys()
            .filter(|key| key.starts_with(&needle))
            .map(|key| key[needle.len()..].to_string())
            .collect();
        keys.sort();
        Ok(keys)
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), String> {
        let mut objects = self.objects.lock().map_err(|error| error.to_string())?;
        objects.remove(&Self::object_key(bucket, key));
        Ok(())
    }
}

fn slides_state(drive: MemoryDrive) -> Arc<SlidesState<MemoryDrive>> {
    let manager =
        diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new("postgres://localhost/unused");
    let conn = diesel::r2d2::Pool::builder()
        .max_size(1)
        .build_unchecked(manager);
    Arc::new(SlidesState {
        drive: Some(drive),
        conn: Arc::new(conn),
        get_default_bot: Arc::new(|_conn: &mut diesel::PgConnection| {
            (uuid::Uuid::nil(), "test-bot".to_string())
        }),
    })
}

async fn as_alice(
    mut request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    request
        .extensions_mut()
        .insert(SlidesUser::new("user-a", "Alice"));
    next.run(request).await
}

fn app(state: Arc<SlidesState<MemoryDrive>>) -> axum::Router {
    configure_slides_routes::<MemoryDrive>()
        .layer(axum::middleware::from_fn(as_alice))
        .with_state(state)
}

fn json_post(uri: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("request")
}

async fn body_text(response: axum::response::Response) -> String {
    let bytes = axum::body::to_bytes(response.into_body(), 256 * 1024)
        .await
        .expect("body");
    String::from_utf8_lossy(&bytes).to_string()
}

/// Builds a six-slide deck so an insert "after slide 3" has neighbours on both
/// sides of the insertion point.
async fn seeded_deck(drive: &MemoryDrive) -> crate::types::Presentation {
    let mut deck = create_new_presentation("user-a");
    let theme = deck.theme.clone();
    while deck.slides.len() < 6 {
        deck.slides.push(crate::utils::create_content_slide(&theme));
    }
    save_presentation_to_drive(drive, "user-a", &deck)
        .await
        .expect("seed deck");
    deck
}

#[tokio::test]
async fn add_slide_appends_at_the_requested_position_and_persists() {
    let drive = MemoryDrive::default();
    let deck = seeded_deck(&drive).await;
    let before: Vec<String> = deck.slides.iter().map(|slide| slide.id.clone()).collect();

    let response = app(slides_state(drive.clone()))
        .oneshot(json_post(
            "/api/slides/slide/add",
            &serde_json::json!({
                "presentation_id": deck.id,
                "layout": "two_column",
                "position": 3,
            })
            .to_string(),
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    // Reload from Drive: the insert has to survive a page refresh.
    let reloaded = load_presentation_by_id(&drive, "user-a", &deck.id)
        .await
        .expect("reload deck");
    assert_eq!(reloaded.slides.len(), before.len() + 1, "one slide appended");
    let after: Vec<String> = reloaded.slides.iter().map(|slide| slide.id.clone()).collect();
    assert_eq!(after[..3], before[..3], "slides 1..3 keep their order");
    assert_eq!(
        after[4..],
        before[3..],
        "the slides after the insertion point shift down untouched"
    );
    assert_ne!(after[3], before[3], "the new slide sits at index 3");
}

#[tokio::test]
async fn new_slide_modal_and_form_carry_the_open_deck() {
    // The toolbar button posts {presentation_id}, the modal body loads the form
    // and the form posts the insert. A missing route or an empty deck id in
    // any of these steps makes the toolbar button a silent no-op.
    let drive = MemoryDrive::default();
    let deck = seeded_deck(&drive).await;
    let router = app(slides_state(drive));

    let modal = router
        .clone()
        .oneshot(json_post(
            "/suite/slides/modals/new-slide",
            &serde_json::json!({ "presentation_id": deck.id }).to_string(),
        ))
        .await
        .expect("modal response");
    assert_eq!(modal.status(), StatusCode::OK);
    let modal_html = body_text(modal).await;
    assert!(
        modal_html.contains("/suite/slides/forms/new-slide"),
        "the modal must load the insert form: {modal_html}"
    );

    let form = router
        .oneshot(json_post(
            "/suite/slides/forms/new-slide",
            &serde_json::json!({ "presentation_id": deck.id }).to_string(),
        ))
        .await
        .expect("form response");
    assert_eq!(form.status(), StatusCode::OK);
    let form_html = body_text(form).await;
    assert!(
        form_html.contains("/api/slides/slide/add"),
        "the form must post to the insert endpoint: {form_html}"
    );
    assert!(
        form_html.contains(&deck.id),
        "the form must address the deck on screen: {form_html}"
    );
    assert!(
        form_html.contains("two_column"),
        "the two-column layout UAT SLIDE-02 asks for must be offered: {form_html}"
    );
}

#[tokio::test]
async fn two_column_insert_lands_between_slide_three_and_four() {
    let drive = MemoryDrive::default();
    let deck = seeded_deck(&drive).await;

    let response = app(slides_state(drive.clone()))
        .oneshot(json_post(
            "/api/slides/slide/add",
            &serde_json::json!({
                "presentation_id": deck.id,
                "layout": "two_column",
                "position": 3,
            })
            .to_string(),
        ))
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let reloaded = load_presentation_by_id(&drive, "user-a", &deck.id)
        .await
        .expect("reload deck");
    assert_eq!(reloaded.slides.len(), 7);
    let inserted = &reloaded.slides[3];
    assert_eq!(inserted.layout, "two_column");
    // Heading plus one placeholder per column: the two-column layout the UAT
    // asks for must be what actually landed in the deck.
    assert_eq!(inserted.elements.len(), 3);
    let columns: Vec<&crate::types::SlideElement> = inserted.elements.iter().skip(1).collect();
    assert!(
        columns[0].x < columns[1].x,
        "the two body placeholders must sit in separate columns"
    );
    assert_eq!(columns[0].width, columns[1].width);
}

#[tokio::test]
async fn the_dialog_defaults_the_position_after_the_slide_on_screen() {
    // The modal forwards the current slide to the form, which defaults the
    // position box to "right after it" — pressing "+" on slide 3 must not
    // insert in front of the deck (that is what made the deck look unchanged).
    let drive = MemoryDrive::default();
    let deck = seeded_deck(&drive).await;
    let router = app(slides_state(drive));

    let modal = router
        .clone()
        .oneshot(json_post(
            "/suite/slides/modals/new-slide",
            &serde_json::json!({ "presentation_id": deck.id, "slide_index": 2 }).to_string(),
        ))
        .await
        .expect("modal response");
    let modal_html = body_text(modal).await;
    assert!(
        modal_html.contains("slide_index"),
        "the modal must forward the slide on screen: {modal_html}"
    );

    let form = router
        .clone()
        .oneshot(json_post(
            "/suite/slides/forms/new-slide",
            &serde_json::json!({ "presentation_id": deck.id, "slide_index": 2 }).to_string(),
        ))
        .await
        .expect("form response");
    let form_html = body_text(form).await;
    assert!(
        form_html.contains(r#"name="position" value="3""#),
        "position must default to after slide 3: {form_html}"
    );

    // Without a known slide (no JavaScript rewrite) the box stays empty and
    // the handler appends instead of inserting at index 0.
    let form = router
        .oneshot(json_post(
            "/suite/slides/forms/new-slide",
            &serde_json::json!({ "presentation_id": deck.id }).to_string(),
        ))
        .await
        .expect("form response");
    let form_html = body_text(form).await;
    assert!(
        form_html.contains(r#"name="position" value="""#),
        "an unknown current slide must append, never insert at 0: {form_html}"
    );
}
