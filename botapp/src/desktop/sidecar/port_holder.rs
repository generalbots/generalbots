//! Who holds the sidecar port when the app is about to start the assistant.
//!
//! A port that is already taken used to be treated as "an assistant that is
//! still booting", so a different program on it left the splash waiting for
//! an API that would never answer. The holder is now identified first: the
//! sidecar this install recorded, another assistant, or a foreign program,
//! which is a conflict the person has to resolve.

use super::probe::{recorded_pid, sidecar_port};
use log::warn;
use std::fmt;

/// What holds the sidecar port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PortHolder {
    /// The sidecar this install spawned and recorded in its pid file.
    OurSidecar,
    /// A `botserver` process this install did not record, or a holder that
    /// could not be identified; the existing attach-and-wait path handles it.
    Assistant,
    /// Another program: starting the sidecar would collide with it.
    Foreign { pid: u32, image: String },
}

/// Classifies the port holder from what the system reports about it.
///
/// `holder` is the owning process id and its image name, when the platform
/// could tell. An unknown holder is never reported as a conflict: refusing to
/// start on a guess would be worse than the boot deadline catching a hang.
pub(crate) fn classify(holder: Option<(u32, Option<&str>)>, recorded: Option<u32>) -> PortHolder {
    let Some((pid, image)) = holder else {
        return PortHolder::Assistant;
    };
    if recorded == Some(pid) {
        return PortHolder::OurSidecar;
    }
    match image.map(str::trim).filter(|name| !name.is_empty()) {
        Some(name) if name.to_ascii_lowercase().contains("botserver") => PortHolder::Assistant,
        Some(name) => PortHolder::Foreign {
            pid,
            image: name.to_string(),
        },
        None => PortHolder::Assistant,
    }
}

/// The port is taken by another program; carried as the start error so the
/// splash can name the port and the program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PortConflict {
    pub port: u16,
    pub image: String,
}

impl fmt::Display for PortConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "port {} is already in use by {}", self.port, self.image)
    }
}

impl std::error::Error for PortConflict {}

/// The conflict to report when a foreign program holds the sidecar port.
pub(super) fn foreign_conflict() -> Option<PortConflict> {
    let port = sidecar_port();
    let holder = listener_pid(port).map(|pid| (pid, image_name(pid)));
    let holder_ref = holder
        .as_ref()
        .map(|(pid, image)| (*pid, image.as_deref()));
    match classify(holder_ref, recorded_pid()) {
        PortHolder::Foreign { pid, image } => {
            warn!("port {port} is held by {image} (pid {pid}); the assistant will not start into it");
            Some(PortConflict { port, image })
        }
        PortHolder::OurSidecar | PortHolder::Assistant => None,
    }
}

/// Pid of the process listening on a loopback-reachable TCP port, read from
/// `netstat -ano -p TCP`. The state column is localised (LISTENING, ABHÖREN,
/// ESCUTANDO...), so a listener is recognised by its zero foreign port.
pub(crate) fn listener_pid_in(netstat: &str, port: u16) -> Option<u32> {
    let suffix = format!(":{port}");
    netstat.lines().find_map(|line| {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 5 || !cols[0].eq_ignore_ascii_case("TCP") {
            return None;
        }
        let local = cols[1];
        let listening = cols[2].ends_with(":0");
        let reachable = local.starts_with("127.0.0.1:") || local.starts_with("0.0.0.0:");
        if listening && reachable && local.ends_with(&suffix) {
            cols[cols.len() - 1].parse::<u32>().ok().filter(|pid| *pid > 0)
        } else {
            None
        }
    })
}

/// Image name from one `tasklist /FO CSV /NH` row (`"name.exe","1234",...`).
/// The "no tasks" notice is localised and not CSV, so it yields nothing.
pub(crate) fn image_from_tasklist_csv(csv: &str) -> Option<String> {
    let line = csv.lines().find(|line| line.starts_with('"'))?;
    let name = line.trim_start_matches('"').split('"').next()?.trim();
    (!name.is_empty()).then(|| name.to_string())
}

#[cfg(windows)]
fn listener_pid(port: u16) -> Option<u32> {
    match botlib::security::command_guard::SafeCommand::new("netstat")
        .and_then(|command| command.args(&["-ano", "-p", "TCP"]))
        .and_then(|command| command.execute())
    {
        Ok(output) => listener_pid_in(&String::from_utf8_lossy(&output.stdout), port),
        Err(e) => {
            warn!("could not list the TCP listeners: {e}");
            None
        }
    }
}

#[cfg(not(windows))]
fn listener_pid(_port: u16) -> Option<u32> {
    None
}

#[cfg(windows)]
fn image_name(pid: u32) -> Option<String> {
    let filter = format!("PID eq {pid}");
    match botlib::security::command_guard::SafeCommand::new("tasklist")
        .and_then(|command| command.args(&["/FI", &filter, "/FO", "CSV", "/NH"]))
        .and_then(|command| command.execute())
    {
        Ok(output) => image_from_tasklist_csv(&String::from_utf8_lossy(&output.stdout)),
        Err(e) => {
            warn!("could not inspect pid {pid}: {e}");
            None
        }
    }
}

#[cfg(not(windows))]
fn image_name(_pid: u32) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const NETSTAT_PT: &str = "\r\nConexões ativas\r\n\r\n  Proto  Endereço local         Endereço externo       Estado           PID\r\n  TCP    0.0.0.0:135            0.0.0.0:0              ESCUTANDO        1204\r\n  TCP    127.0.0.1:8080         127.0.0.1:51234        ESTABELECIDA     777\r\n  TCP    127.0.0.1:8080         0.0.0.0:0              ESCUTANDO        4321\r\n";

    #[test]
    fn the_listener_is_found_whatever_the_language() {
        assert_eq!(listener_pid_in(NETSTAT_PT, 8080), Some(4321));
        assert_eq!(listener_pid_in(NETSTAT_PT, 135), Some(1204));
        assert_eq!(listener_pid_in(NETSTAT_PT, 9000), None);
    }

    #[test]
    fn a_longer_port_with_the_same_suffix_is_not_a_match() {
        let rows = "  TCP    127.0.0.1:18080        0.0.0.0:0              LISTENING        55\r\n";
        assert_eq!(listener_pid_in(rows, 8080), None);
        let any = "  TCP    0.0.0.0:8080           0.0.0.0:0              LISTENING        66\r\n";
        assert_eq!(listener_pid_in(any, 8080), Some(66));
    }

    #[test]
    fn the_image_name_comes_from_the_csv_row_only() {
        let row = "\"nginx.exe\",\"4321\",\"Console\",\"1\",\"12.345 K\"\r\n";
        assert_eq!(image_from_tasklist_csv(row), Some("nginx.exe".to_string()));
        let none = "INFORMAÇÕES: não há tarefas em execução que correspondam aos critérios.\r\n";
        assert_eq!(image_from_tasklist_csv(none), None);
        assert_eq!(image_from_tasklist_csv(""), None);
    }

    #[test]
    fn the_recorded_sidecar_is_ours() {
        assert_eq!(classify(Some((4321, Some("botserver.exe"))), Some(4321)), PortHolder::OurSidecar);
        // A recorded pid wins even when the image could not be read.
        assert_eq!(classify(Some((4321, None)), Some(4321)), PortHolder::OurSidecar);
    }

    #[test]
    fn an_unrecorded_botserver_or_an_unknown_holder_is_attached() {
        assert_eq!(classify(Some((10, Some("botserver.exe"))), Some(4321)), PortHolder::Assistant);
        assert_eq!(classify(Some((10, Some("BotServer"))), None), PortHolder::Assistant);
        assert_eq!(classify(Some((10, None)), None), PortHolder::Assistant);
        assert_eq!(classify(None, Some(4321)), PortHolder::Assistant);
    }

    #[test]
    fn any_other_program_is_a_conflict() {
        assert_eq!(
            classify(Some((99, Some("nginx.exe"))), Some(4321)),
            PortHolder::Foreign {
                pid: 99,
                image: "nginx.exe".to_string()
            }
        );
    }
}
