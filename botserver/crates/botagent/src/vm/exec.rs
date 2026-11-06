//! Program execution inside a conversation folder: dependency installs,
//! source upload, interpreter selection and self-healing retries.

use axum::http::StatusCode;
use uuid::Uuid;

use crate::AgentService;

use super::identity::sanitize_id;
use super::incus::exec_in_vm;
use super::{bad_request, CONVERSATIONS_ROOT};

/// Shell command that runs a generated program as the unprivileged `runner`
/// user (#1658). Paths are already sanitized by the caller.
fn runner_command(folder: &str, interpreter: &str, script_path: &str) -> String {
    format!("su runner -s /bin/sh -c 'cd {folder} && {interpreter} {script_path}'")
}

/// Execute a program in the conversation folder. The code is written to a
/// temp file inside the folder and run with the appropriate interpreter.
/// Returns (success, stdout, stderr).
pub async fn exec_in_conversation(
    state: &AgentService,
    vm_name: &str,
    conversation_id: &str,
    language: &str,
    code: &str,
    timeout_ms: u64,
    packages: &[String],
) -> Result<(bool, String, String), (StatusCode, String)> {
    let clean_cid = sanitize_id(conversation_id)?;
    let folder = format!("{CONVERSATIONS_ROOT}/{clean_cid}");
    // Create the conversation folders FIRST: dependency installs `cd` into
    // this folder, and a fresh conversation has no folder yet.
    let bootstrap = format!(
        "mkdir -p {folder}/program {folder}/output {folder}/data"
    );
    tracing::info!("vm exec: bootstrap {conversation_id}");
    run_with_timeout(state, vm_name, bootstrap, 30_000).await?;
    tracing::info!("vm exec: folders ready {conversation_id}");

    let (interpreter, ext) = match language {
        "python" => ("python3", "py"),
        "node" => ("node", "js"),
        "shell" => ("sh", "sh"),
        _ => return Err(bad_request("Unsupported language")),
    };

    // Declared dependencies are installed BEFORE the program runs, so the
    // model does not have to shell out to pip/npm inside generated code (which
    // also broke the "program/ is source only" contract).
    let declared: Vec<String> = packages
        .iter()
        .filter_map(|p| sanitize_package(p))
        .collect();
    let mut install_log = String::new();
    if !declared.is_empty() && language != "shell" {
        let install = install_packages(state, vm_name, &folder, language, &declared).await?;
        if !install.0 {
            return Ok((false, String::new(), install.1));
        }
        install_log.push_str(&format!("installed: {}\n", declared.join(", ")));
    }

    let run_id: String = Uuid::new_v4().simple().to_string().chars().take(8).collect();
    // #1581: generated programs live in program/, never in the conversation
    // root. output/ and data/ are created up front so the script can write
    // without a race.
    let program_dir = format!("{folder}/program");
    let script_path = format!("{program_dir}/run_{run_id}.{ext}");

    // Large generated programs (a PPTX deck easily reaches tens of KB) must not
    // travel as a single `sh -c` argument: the command guard rejects them with
    // "Script too long". Create the folders, write the source in small base64
    // chunks, then run it.
    write_program(state, vm_name, &script_path, code).await?;
    tracing::info!("vm exec: program written {script_path}");

    // The program file is written by root; hand the tree to `runner` before
    // executing it so generated code cannot touch root-owned state.
    let _ = run_with_timeout(
        state,
        vm_name,
        format!("chown -R runner:runner {folder}"),
        15_000,
    )
    .await;
    let run_cmd = runner_command(&folder, interpreter, &script_path);
    let (mut first_ok, mut stdout, mut stderr) =
        run_with_timeout(state, vm_name, run_cmd.clone(), timeout_ms).await?;

    // A freshly provisioned workspace installs its toolkit in the background.
    // Running a program before that finishes fails with "python3: not found",
    // which reads like a fatal bug. Wait for the provisioning marker (bounded)
    // and retry once so the first request of a new install succeeds.
    if !first_ok && is_interpreter_missing(&stderr) {
        tracing::info!("vm exec: interpreter missing, waiting for workspace provisioning");
        // First boot of a workspace runs apt (python, node, pip) and pip
        // installs the document toolkit; on a cold image that takes minutes,
        // so wait up to three minutes for the interpreter itself instead of
        // failing the user's first request.
        for _ in 0..36 {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let probe = run_with_timeout(
                state,
                vm_name,
                format!("command -v {interpreter}"),
                15_000,
            )
            .await?;
            if probe.0 {
                let retried = run_with_timeout(state, vm_name, run_cmd.clone(), timeout_ms).await?;
                first_ok = retried.0;
                stdout = format!("workspace prepared, retried the program\n{}", retried.1);
                stderr = retried.2;
                break;
            }
        }
    }
    if first_ok {
        stdout = format!("{install_log}{stdout}");
        return Ok((true, stdout, stderr));
    }

    // Self-heal the common failure: the program imported a package that is not
    // installed. Install what the interpreter asked for and retry once.
    if let Some(missing) = missing_package(&stderr, language) {
        let retry_pkgs = vec![missing.clone()];
        let installed = install_packages(state, vm_name, &folder, language, &retry_pkgs).await?;
        if installed.0 {
            let retried = run_with_timeout(state, vm_name, run_cmd, timeout_ms).await?;
            let note = format!("auto-installed missing package: {missing}\n");
            return Ok((retried.0, format!("{install_log}{note}{}", retried.1), retried.2));
        }
    }

    stdout = format!("{install_log}{stdout}");
    Ok((false, stdout, stderr))
}

/// True when the failure is only "the interpreter is not installed yet", which
/// happens while a brand-new workspace is still installing its toolkit.
fn is_interpreter_missing(stderr: &str) -> bool {
    let text = stderr.to_ascii_lowercase();
    (text.contains("not found") || text.contains("command not found"))
        && (text.contains("python3")
            || text.contains("node")
            || text.contains("sh: 1:")
            || text.contains("sh: 0:"))
}

/// Writes `code` into `path` inside the VM without ever passing the whole
/// source as one shell argument. Base64 chunks are appended, then decoded.
async fn write_program(
    state: &AgentService,
    vm_name: &str,
    path: &str,
    code: &str,
) -> Result<(), (StatusCode, String)> {
    const CHUNK: usize = 1_500;
    let b64 = crate::sandbox::b64_encode(code.as_bytes());
    let tmp = format!("{path}.b64");
    let _ = run_with_timeout(state, vm_name, format!(": > {tmp}"), 30_000).await?;
    let bytes = b64.as_bytes();
    let mut offset = 0;
    while offset < bytes.len() {
        let end = (offset + CHUNK).min(bytes.len());
        let chunk = &b64[offset..end];
        let cmd = format!("printf '%s' '{chunk}' >> {tmp}");
        let (ok, _, err) = run_with_timeout(state, vm_name, cmd, 30_000).await?;
        if !ok {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to write program chunk: {err}"),
            ));
        }
        offset = end;
    }
    let finish = format!("base64 -d {tmp} > {path} && rm -f {tmp}");
    let (ok, _, err) = run_with_timeout(state, vm_name, finish, 30_000).await?;
    if !ok {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to decode program: {err}"),
        ));
    }
    Ok(())
}

/// Runs a shell command in the VM with a hard timeout.
/// Runs a shell snippet as **root** in the VM, without the `runner` handoff.
///
/// Only installers use this (conversation autotasks must write systemd units
/// or a crontab). User-generated programs still go through
/// [`exec_in_conversation`], which drops to `runner`.
pub async fn run_root_shell(
    state: &AgentService,
    vm_name: &str,
    command: &str,
    timeout_ms: u64,
) -> Result<(bool, String, String), (StatusCode, String)> {
    run_with_timeout(state, vm_name, command.to_string(), timeout_ms).await
}

async fn run_with_timeout(
    state: &AgentService,
    vm_name: &str,
    command: String,
    timeout_ms: u64,
) -> Result<(bool, String, String), (StatusCode, String)> {
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        exec_in_vm(state, vm_name, &["sh".into(), "-c".into(), command]),
    )
    .await;
    match result {
        Err(_) => Ok((false, String::new(), "Execution timed out".to_string())),
        Ok(r) => r,
    }
}

/// Installs packages for `language`. Names are validated by `sanitize_package`.
async fn install_packages(
    state: &AgentService,
    vm_name: &str,
    folder: &str,
    language: &str,
    packages: &[String],
) -> Result<(bool, String), (StatusCode, String)> {
    let names = packages.join(" ");
    // Minimal images ship without pip/npm. Bootstrap the toolchain once; apt
    // needs internet, which the workspace is allowed to use.
    let ensure = match language {
        "python" => Some(
            "python3 -m pip --version >/dev/null 2>&1 || \
             python3 -m ensurepip --upgrade >/dev/null 2>&1 || \
             (apt-get -o DPkg::Lock::Timeout=300 update -qq && apt-get -o DPkg::Lock::Timeout=300 install -y -qq python3-pip)",
        ),
        "node" => Some(
            "command -v npm >/dev/null 2>&1 || \
             (apt-get -o DPkg::Lock::Timeout=300 update -qq && apt-get -o DPkg::Lock::Timeout=300 install -y -qq nodejs npm)",
        ),
        _ => None,
    };
    if let Some(ensure) = ensure {
        let (ok, _, err) = run_with_timeout(state, vm_name, ensure.to_string(), 240_000).await?;
        if !ok {
            return Ok((false, format!("toolchain bootstrap failed: {err}")));
        }
    }
    let command = match language {
        "python" => format!(
            "cd {folder} && python3 -m pip install --quiet --disable-pip-version-check --break-system-packages {names}"
        ),
        "node" => format!(
            "cd {folder} && npm install --no-save --no-audit --no-fund --silent {names}"
        ),
        _ => return Ok((true, String::new())),
    };
    let (ok, stdout, stderr) = run_with_timeout(state, vm_name, command, 300_000).await?;
    if ok {
        Ok((true, stdout))
    } else {
        Ok((false, format!("dependency install failed: {stderr}")))
    }
}

/// Package names are interpolated into a shell command: allow only the
/// characters real package names use.
fn sanitize_package(name: &str) -> Option<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 80 {
        return None;
    }
    if trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@' | '/'))
    {
        Some(trimmed.to_string())
    } else {
        None
    }
}

/// Maps the import name the interpreter complained about to an installable
/// package and extracts it from the error text.
fn missing_package(stderr: &str, language: &str) -> Option<String> {
    let (prefix, suffix) = match language {
        "python" => ("ModuleNotFoundError: No module named '", "'"),
        "node" => ("Cannot find module '", "'"),
        _ => return None,
    };
    let start = stderr.find(prefix)? + prefix.len();
    let rest = &stderr[start..];
    let end = rest.find(suffix)?;
    let raw = rest[..end].trim();
    // Node reports paths/relative ids; only bare (or scoped) package names are
    // installable.
    let name = if raw.starts_with('@') {
        raw.splitn(3, '/').take(2).collect::<Vec<_>>().join("/")
    } else {
        raw.split('/').next().unwrap_or(raw).to_string()
    };
    if name.is_empty() {
        return None;
    }
    Some(python_package_alias(&name).to_string())
}

/// The import name is not always the distribution name (`docx` → `python-docx`).
fn python_package_alias(name: &str) -> &str {
    match name {
        "docx" => "python-docx",
        "pptx" => "python-pptx",
        "PIL" => "pillow",
        "yaml" => "pyyaml",
        "cv2" => "opencv-python-headless",
        "sklearn" => "scikit-learn",
        "bs4" => "beautifulsoup4",
        "fitz" => "pymupdf",
        "OpenSSL" => "pyopenssl",
        "dotenv" => "python-dotenv",
        other => other,
    }
}

#[cfg(test)]
mod isolation_tests {
    use super::runner_command;

    #[test]
    fn generated_code_runs_as_runner() {
        let command = runner_command(
            "/opt/vibe/conversations/abc",
            "python3",
            "/opt/vibe/conversations/abc/program/run_1.py",
        );
        assert!(command.starts_with("su runner -s /bin/sh -c"));
        assert!(command.contains("python3"));
        assert!(!command.contains("root"));
    }
}
