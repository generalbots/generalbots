//! Per-user agent container lifecycle (#1580). One running container per
//! logged-in user; each conversation gets its own folder inside it at
//! `/opt/vibe/conversations/{conversation_id}/`. Switching conversations only
//! changes which folder is active — running processes are never signalled, so
//! background work continues while the user moves between conversations.
//! Isolation is per container (per user); folders are namespaces inside that
//! container. Every invocation goes through
//! `botlib::security::command_guard::SafeCommand` and runs blocking work
//! inside `tokio::task::spawn_blocking`.

use axum::http::StatusCode;
use botlib::security::command_guard::SafeCommand;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{AgentConversationFolderRow, AgentSessionRow};
use crate::schema::{agent_conversation_folders, agent_sessions};
use crate::AgentService;

pub const STATUS_PROVISIONING: &str = "provisioning";
pub const STATUS_RUNNING: &str = "running";
pub const STATUS_STOPPED: &str = "stopped";
pub const STATUS_ERROR: &str = "error";

const LAUNCH_WAIT_SECS: u64 = 60;
const POLL_INTERVAL_SECS: u64 = 3;
const CONVERSATIONS_ROOT: &str = "/opt/vibe/conversations";

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Agent VM operation failed".to_string())
}

fn bad_request(msg: &str) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, msg.to_string())
}

pub fn sanitize_id(raw: &str) -> Result<String, (StatusCode, String)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return Err(bad_request("Invalid identifier"));
    }
    let clean: String = trimmed
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    if clean.is_empty() {
        return Err(bad_request("Invalid identifier"));
    }
    Ok(clean)
}

#[deprecated(note = "use sanitize_id instead")]
pub fn sanitize_session_id(session_id: &str) -> Result<String, (StatusCode, String)> {
    sanitize_id(session_id)
}

/// `ag-u{user8}-{rand4}` — short, unique, DNS-safe container name.
fn generate_vm_name(user_id: &Uuid) -> String {
    let user8: String = user_id.simple().to_string().chars().take(8).collect();
    let rand4: String = Uuid::new_v4().simple().to_string().chars().take(4).collect();
    format!("ag-u{user8}-{rand4}")
}

fn incus_command(state: &AgentService) -> Result<SafeCommand, (StatusCode, String)> {
    SafeCommand::new(state.incus_bin()).map_err(|e| {
        tracing::error!("incus command rejected by guard: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, "VM subsystem unavailable".to_string())
    })
}

async fn run_incus(state: &AgentService, args: Vec<String>) -> Result<std::process::Output, (StatusCode, String)> {
    let mut cmd = incus_command(state)?;
    for arg in &args {
        cmd = cmd.arg(arg).map_err(|e| internal(format!("incus arg guard: {e}")))?;
    }
    tokio::task::spawn_blocking(move || cmd.execute())
        .await
        .map_err(|e| internal(format!("incus join: {e}")))?
        .map_err(|e| internal(format!("incus execution: {e}")))
}

fn running_from_list(stdout: &str, vm_name: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(stdout) {
        Ok(serde_json::Value::Array(items)) => items.iter().any(|item| {
            item.get("name").and_then(|n| n.as_str()) == Some(vm_name)
                && item
                    .get("status")
                    .and_then(|s| s.as_str())
                    .map(|s| s.eq_ignore_ascii_case("running"))
                    .unwrap_or(false)
        }),
        _ => false,
    }
}

pub(crate) async fn is_running(state: &AgentService, vm_name: &str) -> Result<bool, (StatusCode, String)> {
    let output = run_incus(state, vec!["list".into(), "--format".into(), "json".into()]).await?;
    Ok(running_from_list(&String::from_utf8_lossy(&output.stdout), vm_name))
}

pub(crate) async fn wait_for_running(state: &AgentService, vm_name: &str) -> Result<(), (StatusCode, String)> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(LAUNCH_WAIT_SECS);
    while std::time::Instant::now() < deadline {
        if is_running(state, vm_name).await? {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
    Err(internal(format!("container {vm_name} did not reach RUNNING within {LAUNCH_WAIT_SECS}s")))
}

// ─── DB helpers ────────────────────────────────────────────────────────────

fn load_session_by_user(
    conn: &mut diesel::PgConnection,
    user_id: &Uuid,
) -> Result<Option<AgentSessionRow>, (StatusCode, String)> {
    use agent_sessions::dsl;
    dsl::agent_sessions
        .filter(dsl::user_id.eq(user_id))
        .filter(dsl::status.ne(STATUS_STOPPED))
        .first::<AgentSessionRow>(conn)
        .optional()
        .map_err(|e| internal(format!("agent_sessions select by user: {e}")))
}

fn load_session(
    conn: &mut diesel::PgConnection,
    session_id: &str,
) -> Result<Option<AgentSessionRow>, (StatusCode, String)> {
    use agent_sessions::dsl;
    dsl::agent_sessions
        .filter(dsl::session_id.eq(session_id))
        .first::<AgentSessionRow>(conn)
        .optional()
        .map_err(|e| internal(format!("agent_sessions select: {e}")))
}

fn touch_last_active(conn: &mut diesel::PgConnection, row_id: Uuid) -> Result<(), (StatusCode, String)> {
    use agent_sessions::dsl;
    diesel::update(dsl::agent_sessions.filter(dsl::id.eq(row_id)))
        .set(dsl::last_active_at.eq(Utc::now()))
        .execute(conn)
        .map_err(|e| internal(format!("agent_sessions touch: {e}")))?;
    Ok(())
}

fn set_status(conn: &mut diesel::PgConnection, row_id: Uuid, status_value: &str) -> Result<(), (StatusCode, String)> {
    use agent_sessions::dsl;
    diesel::update(dsl::agent_sessions.filter(dsl::id.eq(row_id)))
        .set(dsl::status.eq(status_value))
        .execute(conn)
        .map_err(|e| internal(format!("agent_sessions set status: {e}")))?;
    Ok(())
}

fn set_active_conversation(
    conn: &mut diesel::PgConnection,
    row_id: Uuid,
    conversation_id: &str,
) -> Result<(), (StatusCode, String)> {
    use agent_sessions::dsl;
    diesel::update(dsl::agent_sessions.filter(dsl::id.eq(row_id)))
        .set((
            dsl::active_conversation_id.eq(Some(conversation_id)),
            dsl::last_active_at.eq(Utc::now()),
        ))
        .execute(conn)
        .map_err(|e| internal(format!("agent_sessions set conversation: {e}")))?;
    Ok(())
}

async fn container_exists(state: &AgentService, vm_name: &str) -> Result<bool, (StatusCode, String)> {
    let output = run_incus(state, vec!["list".into(), "--format".into(), "json".into()]).await?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(match serde_json::from_str::<serde_json::Value>(&stdout) {
        Ok(serde_json::Value::Array(items)) => items
            .iter()
            .any(|item| item.get("name").and_then(|n| n.as_str()) == Some(vm_name)),
        _ => false,
    })
}

// ─── Core: per-user VM guarantee ──────────────────────────────────────────

/// Idempotently guarantee a RUNNING VM for the user (#1580). One VM per
/// logged-in user; the `conversation_id` is used to set up the working
/// folder inside the VM.
pub async fn ensure_vm(
    state: &Arc<AgentService>,
    user_id: &Uuid,
    bot_id: &Uuid,
    conversation_id: Option<&str>,
) -> Result<AgentSessionRow, (StatusCode, String)> {
    let existing = {
        let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
        load_session_by_user(&mut conn, user_id)?
    };

    let row = if let Some(row) = existing {
        match row.status.as_str() {
            STATUS_RUNNING => {
                let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
                touch_last_active(&mut conn, row.id)?;
                row
            }
            STATUS_PROVISIONING => row,
            _ => {
                if is_running(state, &row.vm_name).await? {
                    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
                    set_status(&mut conn, row.id, STATUS_RUNNING)?;
                    let mut fresh = row;
                    fresh.status = STATUS_RUNNING.to_string();
                    fresh
                } else if container_exists(state, &row.vm_name).await? {
                    run_incus(state, vec!["start".into(), row.vm_name.clone()]).await?;
                    wait_for_running(state, &row.vm_name).await?;
                    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
                    set_status(&mut conn, row.id, STATUS_RUNNING)?;
                    let mut fresh = row;
                    fresh.status = STATUS_RUNNING.to_string();
                    fresh
                } else {
                    provision(state, user_id, bot_id, Some(row)).await?
                }
            }
        }
    } else {
        provision(state, user_id, bot_id, None).await?
    };

    if let Some(cid) = conversation_id {
        let _ = ensure_conversation_folder(state, &row, cid).await;
    }

    Ok(row)
}

/// Legacy wrapper for code still passing session_id. Routes through user_id.
pub async fn ensure_vm_by_session(
    state: &Arc<AgentService>,
    _session_id: &str,
    user_id: &Uuid,
    bot_id: &Uuid,
) -> Result<AgentSessionRow, (StatusCode, String)> {
    ensure_vm(state, user_id, bot_id, None).await
}

async fn provision(
    state: &Arc<AgentService>,
    user_id: &Uuid,
    bot_id: &Uuid,
    previous: Option<AgentSessionRow>,
) -> Result<AgentSessionRow, (StatusCode, String)> {
    let vm_name = generate_vm_name(user_id);
    let session_id_str = user_id.to_string();

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let now: DateTime<Utc> = Utc::now();
    let row_id = match previous {
        Some(prev) => {
            use agent_sessions::dsl;
            diesel::update(dsl::agent_sessions.filter(dsl::id.eq(prev.id)))
                .set((
                    dsl::vm_name.eq(&vm_name),
                    dsl::status.eq(STATUS_PROVISIONING),
                    dsl::last_active_at.eq(now),
                    dsl::expires_at.eq(None::<DateTime<Utc>>),
                    dsl::metadata.eq(None::<serde_json::Value>),
                    dsl::active_conversation_id.eq(None::<String>),
                ))
                .execute(&mut conn)
                .map_err(|e| internal(format!("agent_sessions reprovision: {e}")))?;
            prev.id
        }
        None => {
            let new_id = Uuid::new_v4();
            let row = AgentSessionRow {
                id: new_id,
                session_id: session_id_str.clone(),
                user_id: *user_id,
                org_id: None,
                branch_id: None,
                bot_id: *bot_id,
                vm_name: vm_name.clone(),
                status: STATUS_PROVISIONING.to_string(),
                last_active_at: now,
                expires_at: None,
                metadata: None,
                created_at: now,
                active_conversation_id: None,
            };
            diesel::insert_into(agent_sessions::table)
                .values(&row)
                .execute(&mut conn)
                .map_err(|e| internal(format!("agent_sessions insert: {e}")))?;
            new_id
        }
    };
    drop(conn);

    if let Err(e) = launch_and_wait(state, &vm_name).await {
        let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
        let _ = set_status(&mut conn, row_id, STATUS_ERROR);
        return Err(e);
    }

    // Bootstrap the conversations root inside the VM.
    let _ = exec_in_vm(state, &vm_name, &["mkdir".into(), "-p".into(), CONVERSATIONS_ROOT.into()]).await;

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    set_status(&mut conn, row_id, STATUS_RUNNING)?;
    load_session_by_user(&mut conn, user_id)?
        .ok_or_else(|| internal(format!("agent session for user {user_id} vanished after provision")))
}

/// Installs the runtime and enforces internet-only egress inside the
/// container. It runs once at provision and appends to
/// `/tmp/agent-bootstrap.log`. Private ranges are rejected while the bridge
/// resolver keeps working, so package installs and public APIs are reachable
/// but the host, the LAN and sibling tenants are not.
const BOOTSTRAP_SCRIPT: &str = r#"
set -eu
export DEBIAN_FRONTEND=noninteractive
{
  apt-get update -q
  apt-get install -y -q python3 python3-pip nodejs npm iptables iproute2
  cat > /usr/local/sbin/agent-egress.sh <<'EOS'
#!/bin/sh
set -eu
GW=$(ip route | awk '/^default/ {print $3; exit}')
[ -n "$GW" ] || exit 0
iptables -F OUTPUT
iptables -A OUTPUT -o lo -j ACCEPT
iptables -A OUTPUT -d "$GW" -p udp --dport 53 -j ACCEPT
iptables -A OUTPUT -d "$GW" -p tcp --dport 53 -j ACCEPT
iptables -A OUTPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
for net in 10.0.0.0/8 172.16.0.0/12 192.168.0.0/16 169.254.0.0/16; do
  iptables -A OUTPUT -d "$net" -j REJECT
done
iptables -A OUTPUT -j ACCEPT
EOS
  chmod +x /usr/local/sbin/agent-egress.sh
  cat > /etc/systemd/system/agent-egress.service <<'EOS'
[Unit]
Description=Agent internet-only egress
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
ExecStart=/usr/local/sbin/agent-egress.sh
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOS
  systemctl daemon-reload
  systemctl enable --now agent-egress.service
} >/tmp/agent-bootstrap.log 2>&1 || true
"#;

/// Provision the container the same way the Vibe runtime does: a Debian
/// system container with explicit CPU/memory limits attached to the managed
/// bridge (internet egress). Runtime packages (python3/node) are installed
/// in the background so the first user message is not blocked by apt.
async fn launch_and_wait(state: &Arc<AgentService>, vm_name: &str) -> Result<(), (StatusCode, String)> {
    let image = std::env::var("AGENT_VM_IMAGE")
        .unwrap_or_else(|_| "images:debian/13".to_string());
    let cpu = std::env::var("AGENT_VM_CPU").unwrap_or_else(|_| "1".to_string());
    let mem = std::env::var("AGENT_VM_MEMORY").unwrap_or_else(|_| "1GiB".to_string());
    let bridge = std::env::var("AGENT_VM_BRIDGE").unwrap_or_else(|_| "incusbr0".to_string());

    run_incus(
        state,
        vec![
            "launch".into(),
            image,
            vm_name.into(),
            "--config".into(),
            format!("limits.cpu={cpu}"),
            "--config".into(),
            format!("limits.memory={mem}"),
        ],
    )
    .await?;

    // Attach the managed bridge. The default profile often ships no NIC;
    // ignore the error when a NIC already exists (idempotent provisioning).
    let _ = run_incus(
        state,
        vec![
            "config".into(),
            "device".into(),
            "add".into(),
            vm_name.into(),
            "eth0".into(),
            "nic".into(),
            format!("network={bridge}"),
            "name=eth0".into(),
        ],
    )
    .await;

    wait_for_running(state, vm_name).await?;

    // Best-effort runtime bootstrap. Logs land inside the container; a
    // failure is reported by the first exec, never by provisioning.
    let state_for_bootstrap = Arc::clone(state);
    let name_for_bootstrap = vm_name.to_string();
    tokio::spawn(async move {
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(600),
            exec_in_vm(
                &state_for_bootstrap,
                &name_for_bootstrap,
                &["sh".into(), "-c".into(), BOOTSTRAP_SCRIPT.into()],
            ),
        )
        .await;
        match result {
            Ok(Ok((_, _, stderr))) if !stderr.trim().is_empty() => {
                tracing::warn!("agent runtime bootstrap {name_for_bootstrap}: {stderr}");
            }
            Ok(Err((code, err))) => {
                tracing::warn!("agent runtime bootstrap {name_for_bootstrap} ({code}): {err}");
            }
            Err(_) => tracing::warn!("agent runtime bootstrap {name_for_bootstrap} timed out"),
            _ => {}
        }
    });

    Ok(())
}

// ─── Conversation folder management ──────────────────────────────────────

/// Create `/opt/vibe/conversations/{conversation_id}/` in the VM and record
/// it in `agent_conversation_folders`. Idempotent.
pub async fn ensure_conversation_folder(
    state: &AgentService,
    session: &AgentSessionRow,
    conversation_id: &str,
) -> Result<String, (StatusCode, String)> {
    let clean_cid = sanitize_id(conversation_id)?;
    let folder_path = format!("{CONVERSATIONS_ROOT}/{clean_cid}");

    // Check DB first.
    let exists = {
        let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
        use agent_conversation_folders::dsl as cf;
        cf::agent_conversation_folders
            .filter(cf::agent_session_id.eq(session.id))
            .filter(cf::conversation_id.eq(&clean_cid))
            .first::<AgentConversationFolderRow>(&mut conn)
            .optional()
            .map_err(|e| internal(format!("conv_folders select: {e}")))?
    };

    if let Some(row) = exists {
        return Ok(row.folder_path);
    }

    exec_in_vm(state, &session.vm_name, &["mkdir".into(), "-p".into(), folder_path.clone()]).await?;
    for sub in ["output", "program", "data"] {
        let sub_path = format!("{folder_path}/{sub}");
        let _ = exec_in_vm(state, &session.vm_name, &["mkdir".into(), "-p".into(), sub_path]).await;
    }

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = AgentConversationFolderRow {
        id: Uuid::new_v4(),
        agent_session_id: session.id,
        conversation_id: clean_cid,
        folder_path: folder_path.clone(),
        created_at: Utc::now(),
    };
    diesel::insert_into(agent_conversation_folders::table)
        .values(&row)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .map_err(|e| internal(format!("conv_folders insert: {e}")))?;

    Ok(folder_path)
}

/// List regular files directly under the conversation's `output/` folder.
/// Used for the chat attachment manifest (#1582).
pub async fn list_conversation_output(
    state: &Arc<AgentService>,
    vm_name: &str,
    conversation_id: &str,
) -> Result<Vec<String>, (StatusCode, String)> {
    let clean_cid = sanitize_id(conversation_id)?;
    let script = format!(
        "find /opt/vibe/conversations/{clean_cid}/output -maxdepth 1 -type f -printf '%f\\n' 2>/dev/null | sort | head -100"
    );
    let (ok, stdout, _) = exec_in_vm(
        state,
        vm_name,
        &["sh".to_string(), "-c".to_string(), script],
    )
    .await?;
    if !ok {
        return Ok(Vec::new());
    }
    Ok(stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

/// Activate a conversation: ensure its folder exists and record it as the
/// active one. Running processes in other conversations keep running — the
/// container is the isolation boundary, not the conversation (#BBSIA).
pub async fn switch_conversation(
    state: &Arc<AgentService>,
    user_id: &Uuid,
    new_conversation_id: &str,
) -> Result<AgentSessionRow, (StatusCode, String)> {
    let clean_cid = sanitize_id(new_conversation_id)?;

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let session = load_session_by_user(&mut conn, user_id)?
        .ok_or_else(|| bad_request("No active VM for this user"))?;
    drop(conn);

    ensure_conversation_folder(state, &session, &clean_cid).await?;

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    set_active_conversation(&mut conn, session.id, &clean_cid)?;

    let mut updated = session;
    updated.active_conversation_id = Some(clean_cid);
    Ok(updated)
}

/// Delete one conversation workspace: stop processes rooted in the folder,
/// remove the folder tree, and drop the folder row. This runs only on an
/// explicit history deletion — never on a conversation switch.
pub async fn delete_conversation_folder(
    state: &Arc<AgentService>,
    user_id: &Uuid,
    conversation_id: &str,
) -> Result<(), (StatusCode, String)> {
    let clean_cid = sanitize_id(conversation_id)?;

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let Some(session) = load_session_by_user(&mut conn, user_id)? else {
        return Ok(());
    };
    drop(conn);

    let folder = format!("{CONVERSATIONS_ROOT}/{clean_cid}");
    // Explicit deletion is the only place a job is terminated. `rm -rf` and
    // `pkill` run through the trusted shell-script path; `clean_cid` is
    // restricted to ASCII alphanumerics, '-' and '_' by `sanitize_id`.
    let script = format!("pkill -f '{folder}' 2>/dev/null || true; rm -rf '{folder}'");
    if let Err((code, err)) = exec_in_vm(
        state,
        &session.vm_name,
        &["sh".to_string(), "-c".to_string(), script],
    )
    .await
    {
        tracing::warn!("delete conversation folder {clean_cid} ({code}): {err}");
    }

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    use agent_conversation_folders::dsl as cf;
    diesel::delete(
        cf::agent_conversation_folders
            .filter(cf::agent_session_id.eq(session.id))
            .filter(cf::conversation_id.eq(&clean_cid)),
    )
    .execute(&mut conn)
    .map_err(|e| internal(format!("conv_folders delete: {e}")))?;

    if session.active_conversation_id.as_deref() == Some(clean_cid.as_str()) {
        use agent_sessions::dsl as s;
        diesel::update(s::agent_sessions.filter(s::id.eq(session.id)))
            .set(s::active_conversation_id.eq(None::<String>))
            .execute(&mut conn)
            .map_err(|e| internal(format!("agent_sessions clear active: {e}")))?;
    }
    Ok(())
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

    let b64 = crate::sandbox::b64_encode(code.as_bytes());
    let setup = format!(
        "mkdir -p {folder} {program_dir} {folder}/output {folder}/data \
         && echo {b64} | base64 -d > {script_path} \
         && cd {folder} && {interpreter} {script_path}"
    );

    let (first_ok, mut stdout, mut stderr) = run_with_timeout(state, vm_name, setup.clone(), timeout_ms).await?;
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
            let retried = run_with_timeout(state, vm_name, setup, timeout_ms).await?;
            let note = format!("auto-installed missing package: {missing}\n");
            return Ok((retried.0, format!("{install_log}{note}{}", retried.1), retried.2));
        }
    }

    stdout = format!("{install_log}{stdout}");
    Ok((false, stdout, stderr))
}

/// Runs a shell command in the VM with a hard timeout.
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
    let command = match language {
        "python" => format!(
            "cd {folder} && python3 -m pip install --user --quiet --disable-pip-version-check {names}"
        ),
        "node" => format!(
            "cd {folder} && npm install --no-save --no-audit --no-fund --silent {names}"
        ),
        _ => return Ok((true, String::new())),
    };
    let (ok, stdout, stderr) = run_with_timeout(state, vm_name, command, 120_000).await?;
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

// ─── Low-level VM operations ─────────────────────────────────────────────

pub async fn stop_vm(state: &Arc<AgentService>, session_id: &str) -> Result<(), (StatusCode, String)> {
    let clean = sanitize_id(session_id)?;
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = load_session(&mut conn, &clean)?
        .ok_or_else(|| bad_request("Unknown session"))?;
    drop(conn);

    if container_exists(state, &row.vm_name).await? {
        run_incus(state, vec!["stop".into(), "--force".into(), row.vm_name.clone()]).await?;
    }
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    set_status(&mut conn, row.id, STATUS_STOPPED)
}

pub async fn stop_vm_for_user(state: &Arc<AgentService>, user_id: &Uuid) -> Result<(), (StatusCode, String)> {
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = match load_session_by_user(&mut conn, user_id)? {
        Some(r) => r,
        None => return Ok(()),
    };
    drop(conn);

    if container_exists(state, &row.vm_name).await? {
        run_incus(state, vec!["stop".into(), "--force".into(), row.vm_name.clone()]).await?;
    }
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    set_status(&mut conn, row.id, STATUS_STOPPED)
}

pub async fn delete_vm(state: &Arc<AgentService>, session_id: &str) -> Result<(), (StatusCode, String)> {
    let clean = sanitize_id(session_id)?;
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let row = load_session(&mut conn, &clean)?
        .ok_or_else(|| bad_request("Unknown session"))?;
    drop(conn);

    if container_exists(state, &row.vm_name).await? {
        run_incus(state, vec!["delete".into(), "--force".into(), row.vm_name.clone()]).await?;
    }
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    use agent_sessions::dsl;
    diesel::delete(dsl::agent_sessions.filter(dsl::id.eq(row.id)))
        .execute(&mut conn)
        .map_err(|e| internal(format!("agent_sessions delete: {e}")))?;
    Ok(())
}

pub async fn exec_in_vm(
    state: &AgentService,
    vm_name: &str,
    cmd_args: &[String],
) -> Result<(bool, String, String), (StatusCode, String)> {
    // Special-case `sh -c <script>`: the script may contain shell metacharacters
    // (pipes, redirects, base64) that the generic arg guard rejects. Route it
    // through the trusted shell-script escape hatch instead.
    if cmd_args.len() == 3 && cmd_args[0] == "sh" && cmd_args[1] == "-c" {
        let script = &cmd_args[2];
        let mut cmd = incus_command(state)?;
        cmd = cmd.arg("exec").map_err(|e| internal(format!("incus arg: {e}")))?;
        cmd = cmd.arg(vm_name).map_err(|e| internal(format!("incus arg: {e}")))?;
        cmd = cmd.arg("--").map_err(|e| internal(format!("incus arg: {e}")))?;
        cmd = cmd.arg("sh").map_err(|e| internal(format!("incus arg: {e}")))?;
        cmd = cmd.arg("-c").map_err(|e| internal(format!("incus arg: {e}")))?;
        cmd = cmd.trusted_shell_script_arg(script).map_err(|e| internal(format!("incus script: {e}")))?;
        let output = tokio::task::spawn_blocking(move || cmd.execute())
            .await
            .map_err(|e| internal(format!("incus join: {e}")))?
            .map_err(|e| internal(format!("incus execution: {e}")))?;
        return Ok((
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let mut args = vec!["exec".into(), vm_name.into(), "--".into()];
    args.extend(cmd_args.iter().cloned());
    let output = run_incus(state, args).await?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    ))
}

pub fn expiry_from_now(idle_timeout_secs: i64) -> Option<DateTime<Utc>> {
    Utc::now().checked_add_signed(Duration::seconds(idle_timeout_secs))
}

/// Return the user's active VM if one exists, without provisioning.
pub fn get_user_vm(
    state: &AgentService,
    user_id: &Uuid,
) -> Result<Option<AgentSessionRow>, (StatusCode, String)> {
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    load_session_by_user(&mut conn, user_id)
}
