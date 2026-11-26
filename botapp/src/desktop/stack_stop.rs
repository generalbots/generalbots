//! Stops the local services of the desktop stack (database, vault, storage,
//! cache, search index, identity, local model) when the app exits, so closing
//! the assistant does not leave gigabytes of memory in use. Only programs that
//! live under this install's stack folder are touched.

use std::path::Path;

/// How long the database gets to shut down cleanly before it is ended.
const DATABASE_GRACE_SECS: &str = "15";
/// How long each remaining service gets to exit after it is told to.
const EXIT_WAIT_MS: u32 = 3000;

/// True when `image` (a program path) is inside `stack`, compared without
/// regard to case or slash direction as Windows paths are.
pub(crate) fn is_stack_program(image: &str, stack: &str) -> bool {
    let normalize = |p: &str| p.replace('/', "\\").trim_end_matches('\\').to_lowercase();
    let stack = normalize(stack);
    !stack.is_empty() && normalize(image).starts_with(&format!("{stack}\\"))
}

/// Stops every service of the stack. Called once the sidecar is stopped.
pub fn stop_stack() {
    let stack = super::sidecar::Sidecar::stack_path();
    if !stack.join("bin").is_dir() {
        return;
    }
    stop_database(&stack);
    let ended = os::end_stack_processes(&stack.to_string_lossy());
    if ended > 0 {
        log::info!("stopped {ended} local service process(es) of the stack");
    }
}

fn stop_database(stack: &Path) {
    let pg_ctl = stack.join("bin/tables/bin").join(if cfg!(windows) { "pg_ctl.exe" } else { "pg_ctl" });
    let pgdata = stack.join("data/tables/pgdata");
    if !pg_ctl.is_file() || !pgdata.join("postmaster.pid").is_file() {
        return;
    }
    let stopped = botlib::security::command_guard::SafeCommand::new(&pg_ctl.to_string_lossy())
        .and_then(|c| c.args(&["-D", &pgdata.to_string_lossy(), "stop", "-m", "fast", "-w", "-t", DATABASE_GRACE_SECS, "-s"]))
        .and_then(|c| c.execute())
        .map(|o| o.status.success())
        .unwrap_or(false);
    if stopped {
        log::info!("local database stopped");
    } else {
        log::warn!("local database did not stop in time; ending it");
    }
}

#[cfg(windows)]
mod os {
    use super::{is_stack_program, EXIT_WAIT_MS};
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE,
    };

    fn process_ids() -> Vec<u32> {
        let mut ids = Vec::new();
        // SAFETY: the snapshot handle is checked and closed; the entry struct
        // carries its own size as the API requires.
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return ids;
            }
            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
            let mut more = Process32FirstW(snapshot, &mut entry) != 0;
            while more {
                ids.push(entry.th32ProcessID);
                more = Process32NextW(snapshot, &mut entry) != 0;
            }
            CloseHandle(snapshot);
        }
        ids
    }

    /// Ends every process whose program is under `stack`. Returns how many.
    pub(super) fn end_stack_processes(stack: &str) -> usize {
        let own = std::process::id();
        let mut ended = 0;
        for pid in process_ids().into_iter().filter(|pid| *pid != own && *pid > 4) {
            // SAFETY: handle use is bounded to this block and closed at its end.
            unsafe {
                let access = PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE | PROCESS_SYNCHRONIZE;
                let handle = OpenProcess(access, 0, pid);
                if handle.is_null() {
                    continue;
                }
                let mut name = vec![0u16; 1024];
                let mut len = name.len() as u32;
                let readable = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, name.as_mut_ptr(), &mut len) != 0;
                if readable && is_stack_program(&String::from_utf16_lossy(&name[..len as usize]), stack)
                    && TerminateProcess(handle, 0) != 0
                {
                    WaitForSingleObject(handle, EXIT_WAIT_MS);
                    ended += 1;
                }
                CloseHandle(handle);
            }
        }
        ended
    }
}

#[cfg(not(windows))]
mod os {
    /// Unix stacks keep their existing lifecycle (services are managed by the
    /// server's own scripts there).
    pub(super) fn end_stack_processes(stack: &str) -> usize {
        log::debug!("leaving the services of {stack} to their own lifecycle");
        0
    }
}

#[cfg(test)]
mod tests {
    use super::is_stack_program;

    const STACK: &str = r"C:\Users\a\AppData\Local\generalbots\stack";

    #[test]
    fn stack_services_are_recognised() {
        assert!(is_stack_program(r"C:\Users\a\AppData\Local\generalbots\stack\bin\llm\llama-server.exe", STACK));
        assert!(is_stack_program(r"c:/users/a/appdata/local/generalbots/stack/bin/tables/bin/postgres.exe", STACK));
    }

    #[test]
    fn other_programs_are_left_alone() {
        assert!(!is_stack_program(r"C:\Program Files\PostgreSQL\16\bin\postgres.exe", STACK));
        assert!(!is_stack_program(r"C:\Users\b\AppData\Local\generalbots\stack\bin\vault\vault.exe", STACK));
        assert!(!is_stack_program(r"C:\Users\a\AppData\Local\generalbots\stack-old\bin\vault.exe", STACK));
        assert!(!is_stack_program(r"C:\x\vault.exe", ""));
    }
}
