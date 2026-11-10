//! Sidecar liveness probes and process bookkeeping (#2023).
//!
//! Split out of `sidecar.rs` to keep the process-owning type readable: the
//! startup path must decide whether an assistant already exists before it
//! spawns one, and that decision only needs the port, `/api/product` and the
//! pid file written by the last spawn.

use super::Sidecar;
use log::warn;
use std::path::PathBuf;

/// The assistant's readiness endpoint on the sidecar port.
pub(crate) const API_PRODUCT_URL: &str = "http://127.0.0.1:8080/api/product";

/// Path part of [`API_PRODUCT_URL`], used by the synchronous probe.
const API_PRODUCT_PATH: &str = "/api/product";

/// Port the sidecar listens on (`botserver`'s default).
pub(crate) const SIDECAR_PORT: u16 = 8080;

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

/// Parses the status line of an HTTP response.
fn http_status_is_success(response: &str) -> bool {
    let Some(status_line) = response.lines().next() else {
        return false;
    };
    let mut parts = status_line.split_whitespace();
    let _version = parts.next();
    match parts.next().and_then(|code| code.parse::<u16>().ok()) {
        Some(code) => (200..300).contains(&code),
        None => false,
    }
}

/// Synchronous `/api/product` probe used before spawning (#2023).
///
/// Deliberately raw `std::net`: `Sidecar::start` is called from synchronous
/// setup paths, and a loopback GET of one small document needs no runtime.
pub(super) fn api_is_answering() -> bool {
    use std::io::{Read, Write};

    let address = std::net::SocketAddr::from(([127, 0, 0, 1], SIDECAR_PORT));
    let mut stream = match std::net::TcpStream::connect_timeout(
        &address,
        std::time::Duration::from_millis(500),
    ) {
        Ok(stream) => stream,
        Err(_) => return false,
    };
    let timeout = Some(std::time::Duration::from_millis(1500));
    let _ = stream.set_read_timeout(timeout);
    let _ = stream.set_write_timeout(timeout);
    let request = format!(
        "GET {API_PRODUCT_PATH} HTTP/1.1\r\nHost: 127.0.0.1:{SIDECAR_PORT}\r\nConnection: close\r\n\r\n"
    );
    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }
    let mut buffer = [0u8; 128];
    match stream.read(&mut buffer) {
        Ok(read) if read > 0 => http_status_is_success(&String::from_utf8_lossy(&buffer[..read])),
        _ => false,
    }
}

/// Whether something already holds the sidecar port.
pub(super) fn port_is_open() -> bool {
    std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], SIDECAR_PORT)),
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
        assert!(http_status_is_success("HTTP/1.1 200 OK\r\ncontent-length: 12\r\n"));
        assert!(http_status_is_success("HTTP/1.0 204 No Content\r\n"));
        assert!(!http_status_is_success("HTTP/1.1 500 Internal Server Error\r\n"));
        assert!(!http_status_is_success("HTTP/1.1 404 Not Found\r\n"));
        assert!(!http_status_is_success(""));
        assert!(!http_status_is_success("not http at all"));
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
