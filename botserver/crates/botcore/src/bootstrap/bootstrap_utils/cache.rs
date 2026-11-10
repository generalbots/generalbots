//! Cache (Valkey/Redis) readiness probe (#2022).
//!
//! Split out of `health.rs`: the cache needs a real protocol handshake, a
//! platform-aware CLI fallback and an auth path, which is more than the simple
//! port/HTTP probes next door.

use crate::shared::utils::get_stack_path;
use botlib::security::command_guard::SafeCommand;
use log::debug;

/// Port the cache component listens on (the installer always starts it with
/// `--port 6379`, Linux and Windows alike).
const CACHE_PORT: u16 = 6379;

/// Deadline for one cache probe attempt: a loopback connect plus a single RESP
/// round trip. The lifecycle retries every second, so this only bounds one try.
const CACHE_PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(500);

/// Password the cache was started with, when the stack configures one.
///
/// The installer's start command carries no `requirepass`, so this is normally
/// absent; a user-managed cache may set one. Resolved once, because the probe
/// runs once per second during bootstrap and Vault is read through `curl`; a
/// password configured later is picked up on the next boot.
fn cache_password() -> Option<String> {
    static PASSWORD: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    PASSWORD
        .get_or_init(|| {
            for key in ["REDISCLI_AUTH", "CACHE_PASSWORD", "GB_CACHE_PASSWORD"] {
                let value = std::env::var(key).unwrap_or_default();
                if !value.is_empty() {
                    return Some(value);
                }
            }
            let secrets = match botcoresecrets::SecretsManager::get() {
                Ok(secrets) => secrets,
                Err(e) => {
                    debug!("secrets manager unavailable for the cache probe: {e}");
                    return None;
                }
            };
            match secrets.get_cache_config() {
                Ok((_host, _port, password)) => password.filter(|value| !value.is_empty()),
                Err(e) => {
                    debug!("cache password is not available from Vault: {e}");
                    None
                }
            }
        })
        .clone()
}

/// Encodes one RESP command (`*N\r\n$len\r\narg\r\n…`).
fn resp_command(parts: &[&str]) -> String {
    let mut command = format!("*{}\r\n", parts.len());
    for part in parts {
        command.push_str(&format!("${}\r\n{}\r\n", part.len(), part));
    }
    command
}

/// Whether a RESP reply proves the cache is serving.
///
/// `PONG` is the answer of a cache without `requirepass`. An error reply still
/// proves the server is listening and speaking the protocol, so it counts as
/// ready: the credentials are validated by the pool that connects next. The
/// exception is `-LOADING`, where the server is up but cannot serve keys yet.
fn cache_reply_is_ready(reply: &str) -> bool {
    let reply = reply.trim();
    if reply.is_empty() {
        return false;
    }
    reply.lines().any(|line| {
        let line = line.trim();
        line.starts_with("+PONG")
            || (line.starts_with('-') && !line.contains("LOADING"))
            || line.starts_with(':') // integer reply (PING/ECHO variants)
    })
}

/// Arguments for the `redis-cli`/`valkey-cli` fallback probe.
///
/// `--no-auth-warning` is deliberately absent: the Windows bundle is the
/// Microsoft Redis 3.0 port, whose `redis-cli` rejects options it does not
/// know and would fail the probe instead of answering.
fn cache_cli_args(password: Option<&str>) -> Vec<String> {
    let mut args = vec![
        "-h".to_string(),
        "127.0.0.1".to_string(),
        "-p".to_string(),
        CACHE_PORT.to_string(),
    ];
    if let Some(password) = password.filter(|value| !value.is_empty()) {
        args.push("-a".to_string());
        args.push(password.to_string());
    }
    args.push("ping".to_string());
    args
}

/// Where the CLI that can talk to the cache may live, best candidate first.
///
/// #2022 — the two platform archives unpack differently: the Linux tarball
/// keeps its binaries in `bin/cache/bin`, while the Windows Redis zip is
/// extracted flat into `bin/cache` (so `valkey-cli.exe` never exists there and
/// the old hardcoded Unix path could never answer). Bare names last, for a
/// CLI installed system-wide.
fn cache_cli_candidates(stack_path: &std::path::Path, windows: bool) -> Vec<std::path::PathBuf> {
    let names: &[&str] = if windows {
        &["redis-cli.exe", "valkey-cli.exe"]
    } else {
        &["valkey-cli", "redis-cli"]
    };
    let root = stack_path.join("bin").join("cache");
    let mut candidates = Vec::new();
    for name in names {
        // Probe the layout the platform's archive produces first.
        if windows {
            candidates.push(root.join(name));
            candidates.push(root.join("bin").join(name));
        } else {
            candidates.push(root.join("bin").join(name));
            candidates.push(root.join(name));
        }
    }
    for name in names {
        candidates.push(std::path::PathBuf::from(name));
    }
    candidates
}

/// Sends `AUTH` (when a password is configured) and `PING` over a raw socket.
///
/// This is the probe the lifecycle actually needs: platform-neutral (no
/// `valkey-cli`, `redis-cli`, `ss` or `netstat`), encoding- and
/// localization-proof, and it proves the process answers the cache protocol
/// rather than merely holding a listening socket.
///
/// `None` means nothing is listening at all, so the fallbacks cannot help and
/// the caller returns immediately instead of spawning a failing CLI every
/// second of the readiness window.
fn cache_resp_ping(password: Option<&str>) -> Option<bool> {
    use std::io::{Read, Write};

    let address = std::net::SocketAddr::from(([127, 0, 0, 1], CACHE_PORT));
    let mut stream = match std::net::TcpStream::connect_timeout(&address, CACHE_PROBE_TIMEOUT) {
        Ok(stream) => stream,
        Err(e) => {
            debug!(
                "cache port {} is not accepting connections yet: {e}",
                CACHE_PORT
            );
            return None;
        }
    };
    if let Err(e) = stream.set_read_timeout(Some(CACHE_PROBE_TIMEOUT)) {
        debug!("could not bound the cache probe read: {e}");
    }
    if let Err(e) = stream.set_write_timeout(Some(CACHE_PROBE_TIMEOUT)) {
        debug!("could not bound the cache probe write: {e}");
    }

    let mut payload = String::new();
    if let Some(password) = password.filter(|value| !value.is_empty()) {
        payload.push_str(&resp_command(&["AUTH", password]));
    }
    payload.push_str(&resp_command(&["PING"]));
    if let Err(e) = stream.write_all(payload.as_bytes()) {
        debug!("cache probe could not send PING: {e}");
        return Some(false);
    }

    let mut buffer = [0u8; 256];
    let read = match stream.read(&mut buffer) {
        Ok(read) => read,
        Err(e) => {
            debug!("cache accepted the connection but did not answer PING: {e}");
            return Some(false);
        }
    };
    Some(cache_reply_is_ready(&String::from_utf8_lossy(&buffer[..read])))
}

/// Whether a CLI answers `PING` for the cache. The Windows bundle is the
/// Microsoft Redis 3.0 port, so the reply is plain `PONG`.
fn cache_cli_answers(cli: &str, args: &[&str]) -> bool {
    match SafeCommand::new(cli)
        .and_then(|c| c.args(args))
        .and_then(|c| c.execute())
    {
        Ok(output) => {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains("PONG")
        }
        Err(_) => false,
    }
}

/// Check if Valkey/Redis cache is healthy.
///
/// Order matters: the RESP handshake answers on every platform and with or
/// without `requirepass`; the CLI is a fallback for a cache that only exposes a
/// system-wide client; a bare TCP connect is last, for the few seconds where a
/// server has bound its port but is not reading commands yet.
pub fn cache_health_check() -> bool {
    let password = cache_password();
    match cache_resp_ping(password.as_deref()) {
        Some(true) => return true,
        Some(false) => {}
        None => {
            debug!("cache port {} is closed", CACHE_PORT);
            return false;
        }
    }

    let cli_args = cache_cli_args(password.as_deref());
    let cli_args: Vec<&str> = cli_args.iter().map(String::as_str).collect();
    let cli_candidates = cache_cli_candidates(
        std::path::Path::new(&get_stack_path()),
        cfg!(target_os = "windows"),
    );
    for cli in cli_candidates {
        if cache_cli_answers(&cli.to_string_lossy(), &cli_args) {
            return true;
        }
    }

    // Last resort: is anything accepting connections on the cache port? A
    // listening socket is exactly the readiness signal the lifecycle needs
    // while the server finishes its own start-up.
    if std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], CACHE_PORT)),
        CACHE_PROBE_TIMEOUT,
    )
    .is_ok()
    {
        debug!("cache port {} accepts connections", CACHE_PORT);
        return true;
    }

    false
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resp_command_encodes_auth_and_ping() {
        assert_eq!(resp_command(&["PING"]), "*1\r\n$4\r\nPING\r\n");
        assert_eq!(
            resp_command(&["AUTH", "s3cret"]),
            "*2\r\n$4\r\nAUTH\r\n$6\r\ns3cret\r\n"
        );
    }

    #[test]
    fn a_pong_means_ready() {
        assert!(cache_reply_is_ready("+PONG\r\n"));
        assert!(cache_reply_is_ready("+OK\r\n+PONG\r\n"));
    }

    #[test]
    fn an_auth_error_still_means_the_cache_is_listening() {
        // #2022 — the probe must not report a healthy cache as down just
        // because the server enforces `requirepass`.
        assert!(cache_reply_is_ready("-NOAUTH Authentication required.\r\n"));
        assert!(cache_reply_is_ready("-WRONGPASS invalid username-password pair\r\n"));
        assert!(cache_reply_is_ready("-ERR unknown command 'PING'\r\n"));
    }

    #[test]
    fn a_loading_or_silent_cache_is_not_ready() {
        assert!(!cache_reply_is_ready("-LOADING Redis is loading the dataset in memory\r\n"));
        assert!(!cache_reply_is_ready(""));
        assert!(!cache_reply_is_ready("\r\n"));
    }

    #[test]
    fn windows_candidates_use_the_flat_zip_layout_first() {
        let stack = std::path::Path::new("C:\\gb\\stack");
        let candidates = cache_cli_candidates(stack, true);
        assert_eq!(
            candidates.first(),
            Some(&stack.join("bin").join("cache").join("redis-cli.exe"))
        );
        // The Microsoft Redis zip has no valkey-cli; it must not be probed first.
        let names: Vec<String> = candidates
            .iter()
            .filter_map(|path| path.file_name())
            .map(|name| name.to_string_lossy().to_string())
            .collect();
        assert_eq!(names.first().map(String::as_str), Some("redis-cli.exe"));
        assert!(names.iter().all(|name| name.ends_with(".exe")));
    }

    #[test]
    fn unix_candidates_use_the_tarball_layout_first() {
        let stack = std::path::Path::new("/opt/gbo/stack");
        let candidates = cache_cli_candidates(stack, false);
        assert_eq!(
            candidates.first(),
            Some(&stack.join("bin").join("cache").join("bin").join("valkey-cli"))
        );
        assert!(candidates
            .iter()
            .any(|path| path.ends_with("cache/valkey-cli")));
    }

    #[test]
    fn cli_args_carry_the_password_only_when_one_is_configured() {
        let expected = |args: &[&str]| args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
        assert_eq!(cache_cli_args(None), expected(&["-h", "127.0.0.1", "-p", "6379", "ping"]));
        assert_eq!(cache_cli_args(Some("")), expected(&["-h", "127.0.0.1", "-p", "6379", "ping"]));
        assert_eq!(
            cache_cli_args(Some("pw")),
            expected(&["-h", "127.0.0.1", "-p", "6379", "-a", "pw", "ping"])
        );
    }
}
