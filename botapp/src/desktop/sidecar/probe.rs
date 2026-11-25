//! Sidecar liveness probes and process bookkeeping (#2023).
//!
//! Split out of `sidecar.rs` to keep the process-owning type readable: the
//! startup path must decide whether an assistant already exists before it
//! spawns one, and that decision only needs the port, `/api/product` and the
//! pid file written by the last spawn.

use super::Sidecar;
use log::warn;
use std::path::PathBuf;

/// Path part of the product endpoint, used by the wizard and as a legacy
/// fallback for servers that expose neither `/ready` nor `/health`.
const API_PRODUCT_PATH: &str = "/api/product";
/// Dependency-aware readiness endpoint added in #2385.
const READY_PATH: &str = "/ready";
/// Fast liveness endpoint, used when the running server predates `/ready`.
const HEALTH_PATH: &str = "/health";

/// Port the sidecar listens on. Defaults to 8080; overridden by the
/// `GB_SIDECAR_PORT` environment variable (used by the test rig to run
/// the real app against an isolated server instance).
pub(crate) fn sidecar_port() -> u16 {
    super::super::sidecar_port()
}

/// The assistant's product endpoint on the sidecar port (wizard use).
pub(crate) fn api_product_url() -> String {
    format!("http://127.0.0.1:{}{API_PRODUCT_PATH}", sidecar_port())
}

/// Where the pid of the sidecar this app spawned is recorded, so a later app
/// run (or the wizard) can identify an assistant it did not spawn itself
/// (#2023).
fn pid_file_path() -> PathBuf {
    Sidecar::stack_path().join("logs").join("botserver-sidecar.pid")
}

/// Parses a pid file. Anything that is not a positive integer is ignored: a
/// truncated write must never be mistaken for a process to signal.
fn parse_pid(raw: &str) -> Option<u32> {
    raw.trim().parse::<u32>().ok().filter(|pid| *pid > 0)
}

pub(super) fn recorded_pid() -> Option<u32> {
    std::fs::read_to_string(pid_file_path())
        .ok()
        .and_then(|raw| parse_pid(&raw))
}

pub(super) fn record_pid(pid: u32) {
    let path = pid_file_path();
    let Some(dir) = path.parent() else {
        return;
    };
    if let Err(e) = std::fs::create_dir_all(dir) {
        warn!("could not create {}: {e}", dir.display());
        return;
    }
    if let Err(e) = std::fs::write(&path, format!("{pid}\n")) {
        warn!("could not record the sidecar pid in {}: {e}", path.display());
    }
}

pub(super) fn clear_pid() {
    let path = pid_file_path();
    match std::fs::remove_file(&path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => warn!("could not remove {}: {e}", path.display()),
    }
}

/// Whether a process id is still in use.
#[cfg(unix)]
pub(super) fn pid_is_alive(pid: u32) -> bool {
    let Ok(pid) = i32::try_from(pid) else {
        return false;
    };
    // Signal 0 performs the existence and permission check without delivering
    // anything.
    unsafe { libc::kill(pid, 0) == 0 }
}

#[cfg(windows)]
pub(super) fn pid_is_alive(pid: u32) -> bool {
    tasklist_for_pid(pid).contains(&pid.to_string())
}

#[cfg(not(any(unix, windows)))]
pub(super) fn pid_is_alive(_pid: u32) -> bool {
    false
}

/// Whether the recorded pid really is a `botserver` process. Only Linux exposes
/// a cheap, allocation-free way to check; on Windows the image name comes from
/// `tasklist`; elsewhere the port check below is the guard.
#[cfg(target_os = "linux")]
pub(super) fn pid_looks_like_botserver(pid: u32) -> bool {
    match std::fs::read(format!("/proc/{pid}/cmdline")) {
        Ok(raw) => String::from_utf8_lossy(&raw).contains("botserver"),
        Err(_) => false,
    }
}

#[cfg(windows)]
pub(super) fn pid_looks_like_botserver(pid: u32) -> bool {
    tasklist_for_pid(pid).to_lowercase().contains("botserver")
}

#[cfg(not(any(target_os = "linux", windows)))]
pub(super) fn pid_looks_like_botserver(_pid: u32) -> bool {
    true
}

/// `tasklist` line for one pid, empty when the process is gone.
#[cfg(windows)]
fn tasklist_for_pid(pid: u32) -> String {
    let filter = format!("PID eq {pid}");
    match botlib::security::command_guard::SafeCommand::new("tasklist")
        .and_then(|command| command.args(&["/FI", &filter, "/NH"]))
        .and_then(|command| command.execute())
    {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            warn!("could not inspect pid {pid}: {e}");
            String::new()
        }
    }
}

/// Terminates a process the app did not spawn (its `Child` handle is gone).
#[cfg(unix)]
pub(super) fn terminate_pid(pid: u32) -> bool {
    let Ok(pid) = i32::try_from(pid) else {
        return false;
    };
    // SAFETY: SIGKILL to a pid the caller already verified as our botserver.
    unsafe { libc::kill(pid, libc::SIGKILL) == 0 }
}

#[cfg(windows)]
pub(super) fn terminate_pid(pid: u32) -> bool {
    match botlib::security::command_guard::SafeCommand::new("taskkill")
        .and_then(|command| command.args(&["/PID", &pid.to_string(), "/F"]))
        .and_then(|command| command.execute())
    {
        Ok(output) => output.status.success(),
        Err(e) => {
            warn!("could not terminate pid {pid}: {e}");
            false
        }
    }
}

#[cfg(not(any(unix, windows)))]
pub(super) fn terminate_pid(_pid: u32) -> bool {
    false
}

/// What `Sidecar::start` should do once the three liveness probes have run.
///
/// The decision is pure so the idempotent-start rules (#2023) can be tested
/// without spawning a process or binding a port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StartDecision {
    /// This app already owns a live child.
    ReuseChild,
    /// Another assistant answers `/api/product`; attach to it.
    AttachApi,
    /// Something holds the port but is still booting; attach and wait.
    AttachPort,
    /// Nothing is listening: spawn the sidecar.
    Spawn,
}

/// Chooses the start action from the probe results, most specific first.
pub(super) fn decide_start(
    child_running: bool,
    api_answering: bool,
    port_open: bool,
) -> StartDecision {
    if child_running {
        StartDecision::ReuseChild
    } else if api_answering {
        StartDecision::AttachApi
    } else if port_open {
        StartDecision::AttachPort
    } else {
        StartDecision::Spawn
    }
}

/// Parses the status code of an HTTP response.
fn status_code(response: &str) -> Option<u16> {
    let status_line = response.lines().next()?;
    status_line.split_whitespace().nth(1)?.parse().ok()
}

fn is_success_code(code: Option<u16>) -> bool {
    code.is_some_and(|code| (200..300).contains(&code))
}

/// Pure readiness decision: `/ready` is authoritative, `/health` is the
/// fallback for an older server that does not expose it yet, and `/api/product`
/// covers servers that expose neither. A 503 from `/ready` is never masked.
fn readiness_decision(ready: Option<u16>, health: Option<u16>, product: Option<u16>) -> bool {
    if is_success_code(ready) {
        return true;
    }
    // An explicit 503 from any probe is authoritative: a sidecar that reports
    // itself unavailable must never be masked by another endpoint's 200.
    if ready == Some(503) || health == Some(503) || product == Some(503) {
        return false;
    }
    is_success_code(health) || is_success_code(product)
}

/// One raw loopback GET, returning the status code when a response arrives.
///
/// Deliberately raw `std::net`: `Sidecar::start` is called from synchronous
/// setup paths, and a loopback GET of one small document needs no runtime.
fn http_get_status(path: &str) -> Option<u16> {
    use std::io::{Read, Write};

    let port = sidecar_port();
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream =
        std::net::TcpStream::connect_timeout(&address, std::time::Duration::from_millis(500))
            .ok()?;
    let timeout = Some(std::time::Duration::from_millis(1500));
    let _ = stream.set_read_timeout(timeout);
    let _ = stream.set_write_timeout(timeout);
    let request =
        format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).ok()?;
    let mut buffer = [0u8; 128];
    let read = stream.read(&mut buffer).ok()?;
    if read == 0 {
        return None;
    }
    status_code(&String::from_utf8_lossy(&buffer[..read]))
}

/// Synchronous readiness probe used before spawning (#2023, #2385).
pub(super) fn api_is_answering() -> bool {
    readiness_decision(
        http_get_status(READY_PATH),
        http_get_status(HEALTH_PATH),
        http_get_status(API_PRODUCT_PATH),
    )
}

/// Whether something already holds the sidecar port.
pub(super) fn port_is_open() -> bool {
    std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], sidecar_port())),
        std::time::Duration::from_millis(300),
    )
    .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pid_ignores_anything_that_is_not_a_process_id() {
        assert_eq!(parse_pid("28580\n"), Some(28580));
        assert_eq!(parse_pid("  40936  "), Some(40936));
        assert_eq!(parse_pid(""), None);
        assert_eq!(parse_pid("0"), None);
        assert_eq!(parse_pid("-1"), None);
        assert_eq!(parse_pid("botserver"), None);
        // A truncated write must not be read as pid 28.
        assert_eq!(parse_pid("28"), Some(28));
        assert_eq!(parse_pid("28580 partial"), None);
    }

    #[test]
    fn only_a_2xx_status_line_counts_as_ready() {
        assert_eq!(
            status_code("HTTP/1.1 200 OK\r\ncontent-length: 12\r\n"),
            Some(200)
        );
        assert_eq!(status_code("HTTP/1.0 204 No Content\r\n"), Some(204));
        assert_eq!(
            status_code("HTTP/1.1 503 Service Unavailable\r\n"),
            Some(503)
        );
        assert_eq!(status_code(""), None);
        assert_eq!(status_code("not http at all"), None);
        assert!(is_success_code(Some(200)));
        assert!(is_success_code(Some(204)));
        assert!(!is_success_code(Some(503)));
        assert!(!is_success_code(Some(404)));
        assert!(!is_success_code(None));
    }

    #[test]
    fn ready_is_authoritative_and_503_is_never_masked() {
        assert!(readiness_decision(Some(200), None, None));
        assert!(readiness_decision(Some(204), None, None));
        assert!(!readiness_decision(Some(503), Some(200), Some(200)));
    }

    #[test]
    fn health_and_product_are_fallbacks_when_ready_is_unavailable() {
        assert!(readiness_decision(None, Some(200), None));
        assert!(readiness_decision(Some(404), Some(204), None));
        assert!(readiness_decision(Some(500), Some(200), None));
        assert!(readiness_decision(None, None, Some(200)));
        assert!(!readiness_decision(None, Some(503), Some(200)));
        assert!(!readiness_decision(None, None, None));
    }

    #[test]
    fn an_owned_child_is_always_reused() {
        assert_eq!(decide_start(true, false, false), StartDecision::ReuseChild);
        assert_eq!(decide_start(true, true, true), StartDecision::ReuseChild);
        assert_eq!(decide_start(true, true, false), StartDecision::ReuseChild);
    }

    #[test]
    fn a_foreign_assistant_is_attached_never_duplicated() {
        assert_eq!(decide_start(false, true, false), StartDecision::AttachApi);
        assert_eq!(decide_start(false, true, true), StartDecision::AttachApi);
        // The port is held but the API does not answer yet: still attach, a
        // second listener would fail to bind.
        assert_eq!(decide_start(false, false, true), StartDecision::AttachPort);
    }

    #[test]
    fn only_a_free_port_spawns_a_sidecar() {
        assert_eq!(decide_start(false, false, false), StartDecision::Spawn);
    }
}
