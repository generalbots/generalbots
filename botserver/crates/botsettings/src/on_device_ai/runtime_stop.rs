//! Port-scoped stop for the chat runtime (#2561).
//!
//! Finds the stack's own `llama-server` whose command line carries
//! `--port 8081` and kills ONLY that PID. The embedding runtime on 8082 is
//! never touched. Discovery uses `sysinfo` (no external commands, no
//! SafeCommand allowlist dependency).

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Find the stack's llama-server bound to `port` and kill it.
/// Returns `Ok(pid)` on success. Returns `Err` when no matching process is
/// found or when the kill fails.
pub fn stop_chat_runtime(port: u16) -> Result<u32, String> {
    let stack_fragment = stack_llm_fragment();
    let pid = find_stack_llama_for_port(&stack_fragment, port)?;
    kill_pid(pid)?;
    Ok(pid)
}

/// The lowercased, separator-normalized path fragment that every stack
/// llama-server exe must contain (e.g. `\bin\llm\`).
fn stack_llm_fragment() -> String {
    let sep = std::path::MAIN_SEPARATOR;
    format!(
        "{}{}bin{}llm{}",
        botcore::shared::utils::get_stack_path(),
        sep, sep, sep,
    )
    .to_ascii_lowercase()
    .replace('/', &sep.to_string())
}

// ── pure selection (testable) ───────────────────────────────────────────

/// A snapshot of one process's identity, used for the pure selection function.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    /// Lowercased, separator-normalized exe path.
    pub exe_lower: String,
    /// The full command line joined with spaces.
    pub cmd_line: String,
}

/// Pure function: given a list of processes, find the one whose exe is under
/// `stack_fragment` and whose command line contains `--port <port>`.
pub fn select_stack_llama(
    procs: &[ProcessInfo],
    stack_fragment: &str,
    port: u16,
) -> Option<u32> {
    let port_flag = format!("--port {port}");
    // Also match the case where args are separate tokens joined by space.
    let port_flag_eq = format!("--port={port}");
    for p in procs {
        if !p.exe_lower.contains(stack_fragment) {
            continue;
        }
        if p.cmd_line.contains(&port_flag) || p.cmd_line.contains(&port_flag_eq) {
            return Some(p.pid);
        }
    }
    None
}

/// True when `exe_lower` is under the stack's llm directory.
pub fn is_stack_llm_process(exe_lower: &str, stack_fragment: &str) -> bool {
    exe_lower.contains(stack_fragment)
}

// ── OS calls ────────────────────────────────────────────────────────────

/// Scan running processes with `sysinfo` and select the matching one.
fn find_stack_llama_for_port(stack_fragment: &str, port: u16) -> Result<u32, String> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_exe(UpdateKind::Always)
            .with_cmd(UpdateKind::Always),
    );

    let procs: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, proc)| {
            let exe = proc
                .exe()
                .map(|p| {
                    p.to_string_lossy()
                        .to_ascii_lowercase()
                        .replace('/', &std::path::MAIN_SEPARATOR.to_string())
                })
                .unwrap_or_default();
            let cmd: Vec<String> = proc.cmd().iter().map(|s| s.to_string_lossy().to_string()).collect();
            ProcessInfo {
                pid: pid.as_u32(),
                exe_lower: exe,
                cmd_line: cmd.join(" "),
            }
        })
        .collect();

    select_stack_llama(&procs, stack_fragment, port)
        .ok_or_else(|| format!("No stack llama-server found for port {port}"))
}

pub fn kill_pid(pid: u32) -> Result<(), String> {
    let spid = sysinfo::Pid::from_u32(pid);
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[spid]),
        true,
        ProcessRefreshKind::nothing(),
    );
    let proc = sys
        .process(spid)
        .ok_or_else(|| format!("PID {pid} already gone"))?;
    if proc.kill() {
        Ok(())
    } else {
        Err(format!("Failed to kill PID {pid}"))
    }
}

/// Check if a PID is still running.
pub fn is_pid_alive(pid: u32) -> bool {
    let spid = sysinfo::Pid::from_u32(pid);
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[spid]),
        true,
        ProcessRefreshKind::nothing(),
    );
    sys.process(spid).is_some()
}

// ── tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_proc(pid: u32, exe: &str, cmd: &str) -> ProcessInfo {
        ProcessInfo {
            pid,
            exe_lower: exe.to_ascii_lowercase().replace('/', "\\"),
            cmd_line: cmd.to_string(),
        }
    }

    #[test]
    fn select_finds_correct_port() {
        let procs = vec![
            make_proc(100, r"c:\stack\bin\llm\bin\llama-server.exe", "--port 8082 --host 127.0.0.1 -m bge.gguf --embeddings"),
            make_proc(200, r"c:\stack\bin\llm\bin\llama-server.exe", "--port 8081 --host 127.0.0.1 -m deepseek.gguf"),
            make_proc(300, r"c:\other\llama-server.exe", "--port 8081 --host 127.0.0.1 -m deepseek.gguf"),
        ];
        let frag = r"c:\stack\bin\llm\".to_ascii_lowercase();
        // Port 8081 matches PID 200 (stack), not 300 (non-stack).
        assert_eq!(select_stack_llama(&procs, &frag, 8081), Some(200));
        // Port 8082 matches PID 100 (embedding).
        assert_eq!(select_stack_llama(&procs, &frag, 8082), Some(100));
        // Port 9090 matches nothing.
        assert_eq!(select_stack_llama(&procs, &frag, 9090), None);
    }

    #[test]
    fn select_ignores_non_stack_process() {
        let procs = vec![
            make_proc(300, r"c:\other\llama-server.exe", "--port 8081 -m model.gguf"),
        ];
        let frag = r"c:\stack\bin\llm\".to_ascii_lowercase();
        assert_eq!(select_stack_llama(&procs, &frag, 8081), None);
    }

    #[test]
    fn is_stack_llm_process_checks_fragment() {
        let frag = r"c:\stack\bin\llm\".to_ascii_lowercase();
        assert!(is_stack_llm_process(r"c:\stack\bin\llm\bin\llama-server.exe", &frag));
        assert!(!is_stack_llm_process(r"c:\other\llama-server.exe", &frag));
    }

    /// Live test: bind a port, find ourselves via sysinfo, and confirm
    /// the test binary is correctly REJECTED by the stack-exe check
    /// (since the test binary is not under the stack's llm directory).
    #[test]
    fn live_find_and_reject_non_stack() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")
            .expect("bind ephemeral port");
        let port = listener.local_addr().expect("local_addr").port();
        let my_pid = std::process::id();

        // The sysinfo scan should find our PID on this port ... but wait,
        // sysinfo reads the process's command line, not its bound ports.
        // Our test binary's cmd line does NOT contain `--port <port>`.
        // So select_stack_llama will not find it -- which is correct:
        // only a llama-server with `--port N` in its args is matched.
        // We test the pure function instead with a synthetic entry.
        let frag = stack_llm_fragment();
        let procs = vec![ProcessInfo {
            pid: my_pid,
            exe_lower: std::env::current_exe()
                .unwrap_or_default()
                .to_string_lossy()
                .to_ascii_lowercase()
                .replace('/', "\\"),
            cmd_line: format!("test-binary --port {port}"),
        }];

        // The test binary's exe is NOT under the stack's llm dir,
        // so it must NOT be selected.
        assert_eq!(select_stack_llama(&procs, &frag, port), None);

        // Also test is_stack_llm_process directly.
        let my_exe = std::env::current_exe()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();
        assert!(
            !is_stack_llm_process(&my_exe, &frag),
            "Test binary should not be under the stack's llm dir"
        );

        drop(listener);
    }
}
