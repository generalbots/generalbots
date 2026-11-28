//! Hands the desktop session to the assistant page without putting it in a URL.
//!
//! A token in the navigation URL stays in the WebView history, the DevTools
//! target list and any request log even after the page strips it. The shell
//! instead keeps the session it has just minted in memory for a short while,
//! and the page served by the sidecar in the main window takes it once over
//! IPC (`take_desktop_session`). The token is never logged.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::origin::is_allowed_suite_url;

/// How long a minted session waits for the page to take it.
const OFFER_TTL: Duration = Duration::from_secs(60);

/// Whether the main window's entry page takes the session over IPC. The
/// assistant page does; the legacy desktop page still reads `?token=`.
const IPC_HANDOFF: bool = cfg!(feature = "gblite");

/// A one-shot, short-lived slot for the session the page has not taken yet.
#[derive(Default)]
struct Offer {
    pending: Option<(String, Instant)>,
}

impl Offer {
    fn put(&mut self, token: Option<&str>, now: Instant) {
        self.pending = token
            .filter(|t| !t.is_empty())
            .map(|t| (t.to_string(), now));
    }

    fn take(&mut self, now: Instant) -> Option<String> {
        let (token, at) = self.pending.take()?;
        (now.saturating_duration_since(at) <= OFFER_TTL).then_some(token)
    }
}

static OFFER: Mutex<Offer> = Mutex::new(Offer { pending: None });

fn with_offer<T>(f: impl FnOnce(&mut Offer) -> T) -> T {
    let mut guard = OFFER.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut guard)
}

/// The URL the main window navigates to for this session. On the assistant
/// entry the URL never carries the token: it is offered to the page instead,
/// and an earlier offer is replaced (or withdrawn when there is no session).
pub(super) fn suite_url_for_session(url: tauri::Url, token: Option<&str>) -> tauri::Url {
    if IPC_HANDOFF {
        with_offer(|offer| offer.put(token, Instant::now()));
        return url;
    }
    super::session::suite_url_with_token(url, token)
}

/// Whether a webview may take the offered session: only the main window, and
/// only while it shows a page of the sidecar origin.
fn may_take(label: &str, url: Option<&tauri::Url>) -> bool {
    label == "main" && url.is_some_and(is_allowed_suite_url)
}

/// Returns the session the shell minted for this launch, once. A second call,
/// another window, another origin or an offer older than a minute get nothing.
#[tauri::command]
pub fn take_desktop_session(webview: tauri::Webview) -> Option<String> {
    let url = webview.url().ok();
    if !may_take(webview.label(), url.as_ref()) {
        log::warn!("desktop session refused to window {}", webview.label());
        return None;
    }
    with_offer(|offer| offer.take(Instant::now()))
}

#[cfg(test)]
mod tests {
    use super::{may_take, Offer, OFFER_TTL};
    use std::time::{Duration, Instant};

    #[test]
    fn the_session_is_taken_once() {
        let now = Instant::now();
        let mut offer = Offer::default();
        offer.put(Some("gb_session"), now);
        assert_eq!(offer.take(now).as_deref(), Some("gb_session"));
        assert_eq!(offer.take(now), None);
    }

    #[test]
    fn an_old_offer_is_dropped() {
        let now = Instant::now();
        let mut offer = Offer::default();
        offer.put(Some("gb_session"), now);
        assert_eq!(offer.take(now + OFFER_TTL + Duration::from_secs(1)), None);
        assert_eq!(offer.take(now), None);
    }

    #[test]
    fn a_launch_without_a_session_withdraws_the_previous_offer() {
        let now = Instant::now();
        let mut offer = Offer::default();
        offer.put(Some("gb_old"), now);
        offer.put(None, now);
        assert_eq!(offer.take(now), None);
        offer.put(Some(""), now);
        assert_eq!(offer.take(now), None);
    }

    #[test]
    fn a_new_session_replaces_the_previous_offer() {
        let now = Instant::now();
        let mut offer = Offer::default();
        offer.put(Some("gb_old"), now);
        offer.put(Some("gb_new"), now);
        assert_eq!(offer.take(now).as_deref(), Some("gb_new"));
    }

    #[test]
    fn only_the_main_window_on_the_sidecar_may_take_it() {
        let port = crate::desktop::sidecar::probe::sidecar_port();
        let sidecar = tauri::Url::parse(&format!(
            "http://127.0.0.1:{port}/suite/assistant/assistant.html"
        ))
        .ok();
        let foreign = tauri::Url::parse("https://example.com/suite/assistant/assistant.html").ok();
        let bundled = tauri::Url::parse("http://tauri.localhost/index.html").ok();
        assert!(may_take("main", sidecar.as_ref()));
        assert!(!may_take("wizard", sidecar.as_ref()));
        assert!(!may_take("main", foreign.as_ref()));
        assert!(!may_take("main", bundled.as_ref()));
        assert!(!may_take("main", None));
    }
}
