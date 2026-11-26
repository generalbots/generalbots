//! Refuses to start a local service on a port another program already holds,
//! so the desktop stack never silently talks to someone else's server (a
//! database, cache or vault of another app, or of another Windows user).

use std::path::{Path, PathBuf};

/// One listening TCP socket: local port and owning process id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Listener {
    pub port: u16,
    pub pid: u32,
}

const IPV4_ROW_WORDS: usize = 6;
const IPV6_ROW_WORDS: usize = 14;

fn word(buffer: &[u8], index: usize) -> Option<u32> {
    let bytes = buffer.get(index * 4..index * 4 + 4)?;
    Some(u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// `dwLocalPort` keeps the port in network byte order in its low 16 bits.
fn port_from_word(value: u32) -> u16 {
    u16::from_be((value & 0xffff) as u16)
}

/// Parses a `MIB_TCPTABLE_OWNER_PID` (IPv4) or `MIB_TCP6TABLE_OWNER_PID`
/// (IPv6) buffer as returned by `GetExtendedTcpTable`.
pub fn listeners_from_table(buffer: &[u8], ipv6: bool) -> Vec<Listener> {
    let (row_words, port_index, pid_index) = if ipv6 { (IPV6_ROW_WORDS, 5, 13) } else { (IPV4_ROW_WORDS, 2, 5) };
    let count = word(buffer, 0).unwrap_or(0) as usize;
    (0..count)
        .filter_map(|row| {
            let base = 1 + row * row_words;
            Some(Listener {
                port: port_from_word(word(buffer, base + port_index)?),
                pid: word(buffer, base + pid_index)?,
            })
        })
        .collect()
}

/// True when `image` is a program of this stack (under `stack`, compared
/// without regard to case as Windows paths are).
pub fn belongs_to_stack(image: Option<&Path>, stack: &Path) -> bool {
    let Some(image) = image else { return false };
    let normalize = |p: &Path| p.to_string_lossy().replace('/', "\\").trim_end_matches('\\').to_lowercase();
    let stack = normalize(stack);
    let image = normalize(image);
    !stack.is_empty() && image.starts_with(&format!("{stack}\\"))
}

/// The message shown when a port is taken (pt-BR, the edition's language).
pub fn conflict_message(service: &str, port: u16, program: Option<&Path>) -> String {
    let program = program
        .and_then(Path::file_name)
        .map_or_else(|| "outro programa".to_string(), |n| format!("outro programa ({})", n.to_string_lossy()));
    format!(
        "A porta {port} deste computador já está em uso por {program}, por isso o serviço local \"{service}\" não pode iniciar. Feche esse programa ou reinicie o computador e abra o assistente de novo."
    )
}

/// Fails with [`conflict_message`] when any of `ports` is held by a process
/// that is not part of the stack at `stack`.
pub fn ensure_ports_free(service: &str, ports: &[u16], stack: &Path) -> Result<(), String> {
    let listeners = os::listeners();
    for &port in ports {
        for listener in listeners.iter().filter(|l| l.port == port) {
            let image = os::process_image(listener.pid);
            if !belongs_to_stack(image.as_deref(), stack) {
                return Err(conflict_message(service, port, image.as_deref()));
            }
        }
    }
    Ok(())
}

#[cfg(windows)]
mod os {
    use super::{listeners_from_table, Listener};
    use std::path::PathBuf;
    use windows_sys::Win32::Foundation::{CloseHandle, ERROR_INSUFFICIENT_BUFFER, NO_ERROR};
    use windows_sys::Win32::NetworkManagement::IpHelper::{GetExtendedTcpTable, TCP_TABLE_OWNER_PID_LISTENER};
    use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    fn table(family: u16) -> Vec<u8> {
        let mut size: u32 = 0;
        let mut buffer: Vec<u8> = Vec::new();
        for _ in 0..4 {
            buffer.resize(size as usize, 0);
            let pointer = if buffer.is_empty() { std::ptr::null_mut() } else { buffer.as_mut_ptr().cast() };
            // SAFETY: `pointer` is null or `size` writable bytes owned by `buffer`.
            let status = unsafe {
                GetExtendedTcpTable(pointer, &mut size, 0, u32::from(family), TCP_TABLE_OWNER_PID_LISTENER, 0)
            };
            if status == NO_ERROR {
                return buffer;
            }
            if status != ERROR_INSUFFICIENT_BUFFER {
                break;
            }
        }
        Vec::new()
    }

    pub(super) fn listeners() -> Vec<Listener> {
        let mut all = listeners_from_table(&table(AF_INET), false);
        all.extend(listeners_from_table(&table(AF_INET6), true));
        all
    }

    pub(super) fn process_image(pid: u32) -> Option<PathBuf> {
        // SAFETY: plain handle query; the handle is closed before returning.
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return None;
            }
            let mut name = vec![0u16; 1024];
            let mut len = name.len() as u32;
            let ok = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, name.as_mut_ptr(), &mut len);
            CloseHandle(handle);
            (ok != 0).then(|| PathBuf::from(String::from_utf16_lossy(&name[..len as usize])))
        }
    }
}

#[cfg(not(windows))]
mod os {
    use super::Listener;
    use std::path::PathBuf;

    pub(super) fn listeners() -> Vec<Listener> {
        Vec::new()
    }

    pub(super) fn process_image(pid: u32) -> Option<PathBuf> {
        std::fs::read_link(format!("/proc/{pid}/exe")).ok()
    }
}

/// Program path of a process, when the platform lets this user read it.
pub fn process_image(pid: u32) -> Option<PathBuf> {
    os::process_image(pid)
}

#[cfg(test)]
#[path = "port_guard_tests.rs"]
mod tests;
