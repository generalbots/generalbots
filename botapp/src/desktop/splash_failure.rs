//! The splash's failure state: what went wrong, where the server log is, and
//! the person's choice between trying again and closing the app.
//!
//! The page words the message in its own language (this process has no
//! locale on Windows), so the shell only sends the facts. The page answers
//! the same way it reports its first paint: it stamps a fragment into its
//! URL, which the shell polls.

use super::boot_guard::BootFailure;
use serde_json::{json, Value};
use std::path::Path;
use std::time::{Duration, Instant};
use tauri::Manager;

/// What the person chose on the failure state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Retry,
    Close,
}

const ACTION_FRAGMENT: &str = "splash-action=";
const POLL: Duration = Duration::from_millis(300);

/// The facts the page needs to word one failure.
pub fn failure_payload(failure: &BootFailure, log: &Path) -> Value {
    let log = log.display().to_string();
    match failure {
        BootFailure::Exited { code } => json!({ "kind": "exited", "code": code, "log": log }),
        BootFailure::Deadline { seconds } => {
            json!({ "kind": "deadline", "seconds": seconds, "log": log })
        }
        BootFailure::PortTaken { port, program } => {
            json!({ "kind": "port", "port": port, "program": program, "log": log })
        }
        BootFailure::NotStarted => json!({ "kind": "start", "log": log }),
    }
}

/// The choice stamped into the splash URL, if any.
pub fn choice_in_url(url: &str) -> Option<Choice> {
    let (_, action) = url.split_once(ACTION_FRAGMENT)?;
    if action.starts_with("retry") {
        Some(Choice::Retry)
    } else if action.starts_with("close") {
        Some(Choice::Close)
    } else {
        None
    }
}

/// Whether there is a splash to show a failure on.
pub fn splash_is_open(app: &tauri::AppHandle) -> bool {
    app.get_webview_window("splash").is_some()
}

/// Puts the splash in its failure state and brings it forward.
pub async fn show(app: &tauri::AppHandle, failure: &BootFailure, log: &Path) {
    // A script evaluated before the page has loaded is lost.
    let painted_by = Instant::now() + Duration::from_secs(5);
    while !super::splash::painted() && Instant::now() < painted_by {
        tokio::time::sleep(POLL).await;
    }
    let Some(splash) = app.get_webview_window("splash") else {
        return;
    };
    let script = format!(
        "if(typeof window.gbSplashFail==='function')window.gbSplashFail({});",
        failure_payload(failure, log)
    );
    let _ = splash.eval(&script);
    if !super::splash::is_test_rig() {
        let _ = splash.set_skip_taskbar(false);
        let _ = splash.unminimize();
        let _ = splash.set_focus();
    }
}

/// Waits for the person's choice. A splash closed by other means counts as
/// closing the app: nothing else is on screen.
pub async fn wait_choice(app: &tauri::AppHandle) -> Choice {
    loop {
        let Some(splash) = app.get_webview_window("splash") else {
            return Choice::Close;
        };
        if let Some(choice) = splash.url().ok().and_then(|url| choice_in_url(url.as_str())) {
            return choice;
        }
        tokio::time::sleep(POLL).await;
    }
}

/// Takes the page back to its progress state for another boot.
pub fn restart(app: &tauri::AppHandle) {
    if let Some(splash) = app.get_webview_window("splash") {
        let _ = splash.eval("if(typeof window.gbSplashRestart==='function')window.gbSplashRestart();");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn the_page_gets_the_facts_of_each_failure() {
        let log = PathBuf::from("stack").join("logs").join("botserver-sidecar.log");
        let shown = log.display().to_string();
        assert_eq!(
            failure_payload(&BootFailure::Exited { code: Some(3) }, &log),
            json!({ "kind": "exited", "code": 3, "log": shown })
        );
        assert_eq!(
            failure_payload(&BootFailure::Exited { code: None }, &log)["code"],
            Value::Null
        );
        assert_eq!(
            failure_payload(&BootFailure::Deadline { seconds: 180 }, &log),
            json!({ "kind": "deadline", "seconds": 180, "log": shown })
        );
        assert_eq!(
            failure_payload(
                &BootFailure::PortTaken { port: 8080, program: "nginx.exe".into() },
                &log
            ),
            json!({ "kind": "port", "port": 8080, "program": "nginx.exe", "log": shown })
        );
        assert_eq!(failure_payload(&BootFailure::NotStarted, &log)["kind"], "start");
    }

    #[test]
    fn only_a_known_action_in_the_fragment_is_a_choice() {
        assert_eq!(choice_in_url("tauri://localhost/splash.html#splash-action=retry"), Some(Choice::Retry));
        assert_eq!(choice_in_url("http://tauri.localhost/splash.html#splash-action=close"), Some(Choice::Close));
        assert_eq!(choice_in_url("http://tauri.localhost/splash.html#splash-painted"), None);
        assert_eq!(choice_in_url("http://tauri.localhost/splash.html#splash-action=other"), None);
    }
}
