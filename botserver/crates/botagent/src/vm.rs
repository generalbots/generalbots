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
use botschema::users;
use diesel::prelude::*;
use std::process::Stdio;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::{Duration as StdDuration, Instant};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::models::{AgentConversationFolderRow, AgentSessionRow};
use crate::schema::{agent_conversation_folders, agent_sessions};
use crate::stream::{pump_lines, wait_for_child, ExecEvent};
use crate::AgentService;

pub const STATUS_PROVISIONING: &str = "provisioning";
pub const STATUS_RUNNING: &str = "running";
pub const STATUS_STOPPED: &str = "stopped";
pub const STATUS_ERROR: &str = "error";

/// Guest identities minted by the anonymous chat path. They are created per
/// page load (`guest_<uuid>@anonymous.local`), so they must never own a
/// container: one container per guest exhausted the production host disk.
const ANONYMOUS_EMAIL_DOMAIN: &str = "@anonymous.local";
const ANONYMOUS_USERNAME_PREFIX: &str = "guest_";

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

fn is_anonymous_email(email: &str) -> bool {
    let lower = email.to_ascii_lowercase();
    lower
        .strip_suffix(ANONYMOUS_EMAIL_DOMAIN)
        .is_some_and(|local| local.starts_with(ANONYMOUS_USERNAME_PREFIX))
}

/// Anonymous visitors must never provision a container (#1596). The guest row
/// is minted by `/api/auth` on every anonymous page load, so provisioning one
/// container per guest leaks disk until the host is full. Only signed-in
/// accounts (real email rows) may own an agent workspace.
///
/// A caller whose identity has no `users` row is likewise refused: the websocket
/// pipeline passes `Uuid::nil()` for sessions it could not attribute, and each
/// of those used to spin up an `ag-u00000000-*` container (production host disk
/// exhaustion). Single-user/desktop builds that run without accounts opt out
/// explicitly with `AGENT_ALLOW_UNREGISTERED_IDENTITIES=1`.
fn require_authenticated_identity(
    state: &AgentService,
    user_id: &Uuid,
) -> Result<(), (StatusCode, String)> {
    // Single-user/desktop installs run without accounts: the opt-out covers
    // guest identities and nil callers too, otherwise a fresh local install can
    // never reach the agent workspace.
    if allow_unregistered_identities() {
        return Ok(());
    }
    if user_id.is_nil() {
        tracing::warn!("agent workspace provisioning denied for nil identity");
        return Err((
            StatusCode::FORBIDDEN,
            "Sign in to use the agent workspace".to_string(),
        ));
    }

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let email = users::table
        .filter(users::id.eq(*user_id))
        .select(users::email)
        .first::<String>(&mut conn)
        .optional()
        .map_err(|e| internal(format!("users lookup: {e}")))?;

    match email {
        Some(email) if is_anonymous_email(&email) => {
            tracing::warn!("agent workspace provisioning denied for anonymous identity {user_id}");
            Err((
                StatusCode::FORBIDDEN,
                "Sign in to use the agent workspace".to_string(),
            ))
        }
        Some(_) => Ok(()),
        None => {
            if allow_unregistered_identities() {
                return Ok(());
            }
            tracing::warn!(
                "agent workspace provisioning denied for identity {user_id} without a users row"
            );
            Err((
                StatusCode::FORBIDDEN,
                "Sign in to use the agent workspace".to_string(),
            ))
        }
    }
}

/// Opt-out for single-user/desktop installs that run without accounts.
pub(crate) fn allow_unregistered_identities() -> bool {
    matches!(
        std::env::var("AGENT_ALLOW_UNREGISTERED_IDENTITIES").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
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

/// Last `ERROR` entries of the container's own start log. The host-side
/// reason a container refuses to run lives here, not in the start exit code
/// (the daemon reports exit 0 while the instance stays STOPPED).
async fn instance_log_tail(state: &AgentService, vm_name: &str) -> Option<String> {
    let output = run_incus(
        state,
        vec![
            "info".to_string(),
            vm_name.to_string(),
            "--show-log".to_string(),
        ],
    )
    .await
    .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let errors: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && line.contains("ERROR"))
        .rev()
        .take(2)
        .map(|line| line.chars().take(160).collect::<String>())
        .collect();
    if errors.is_empty() {
        return None;
    }
    Some(errors.join(" | "))
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
    require_authenticated_identity(state, user_id)?;

    let existing = {
        let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
        load_session_by_user(&mut conn, user_id)?
    };

    let row = if let Some(row) = existing {
        match row.status.as_str() {
            // Trust the row only when the container behind it still exists:
            // after a cleanup or a failed provisioning the database could say
            // "running" while `incus list` was empty, so every vm_exec returned
            // nothing and the workspace looked permanently unavailable.
            STATUS_RUNNING if is_running(state, &row.vm_name).await? => {
                let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
                touch_last_active(&mut conn, row.id)?;
                row
            }
            STATUS_RUNNING | STATUS_PROVISIONING if container_exists(state, &row.vm_name).await? => {
                let _ = run_incus(state, vec!["start".into(), row.vm_name.clone()]).await;
                wait_for_running(state, &row.vm_name).await?;
                let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
                set_status(&mut conn, row.id, STATUS_RUNNING)?;
                let mut fresh = row;
                fresh.status = STATUS_RUNNING.to_string();
                fresh
            }
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

    if row.status == STATUS_RUNNING {
        schedule_workspace_preparation(state.clone(), row.vm_name.clone());
    }

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
# Debian containers can boot without /etc/resolv.conf; apt then fails to
# resolve and the runtime is never installed ("python3: not found" on the
# first vm_exec). Point the container at the bridge resolver before apt.
GW=$(ip route 2>/dev/null | awk '/^default/ {print $3; exit}')
{
  [ -n "${GW:-}" ] && echo "nameserver $GW"
  echo "nameserver 1.1.1.1"
  echo "nameserver 8.8.8.8"
} > /etc/resolv.conf
{
  apt-get -o DPkg::Lock::Timeout=300 update -q
  apt-get -o DPkg::Lock::Timeout=300 install -y -q python3 python3-pip nodejs npm iptables iproute2
  mkdir -p /opt/vibe/conversations
  # Base data/document toolkit: the model should not pay an install round-trip
  # for the common agent outputs (decks, sheets, charts, PDFs).
  python3 -m pip install --break-system-packages --quiet --disable-pip-version-check \
    pandas numpy openpyxl python-pptx pillow matplotlib requests python-docx \
    reportlab beautifulsoup4 lxml tabulate || true
  echo v4 > /opt/vibe/.provisioned
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

/// Bumped whenever the base toolkit changes; existing VMs run the prepare
/// step once when their marker does not match.
const WORKSPACE_PREPARE_VERSION: &str = "v5";

/// Idempotent preparation of an existing workspace VM: package index, base
/// Python toolkit and the conversations root. Runs in the background so the
/// first chat message is never blocked by apt/pip.
const WORKSPACE_PREPARE_SCRIPT: &str = r#"
set -eu
export DEBIAN_FRONTEND=noninteractive
# Debian containers can boot without /etc/resolv.conf; apt then fails to
# resolve and the runtime is never installed ("python3: not found" on the
# first vm_exec). Point the container at the bridge resolver before apt.
GW=$(ip route 2>/dev/null | awk '/^default/ {print $3; exit}')
{
  [ -n "${GW:-}" ] && echo "nameserver $GW"
  echo "nameserver 1.1.1.1"
  echo "nameserver 8.8.8.8"
} > /etc/resolv.conf
{
  mkdir -p /opt/vibe/conversations
  # #1658 — two identities: `claude` owns the subscription credentials,
  # `runner` executes model-generated code. Generated code must never be able
  # to read /home/claude.
  id -u runner >/dev/null 2>&1 || useradd -m -s /bin/bash runner
  id -u claude >/dev/null 2>&1 || useradd -m -s /bin/bash claude
  mkdir -p /home/claude/.claude
  chown -R claude:claude /home/claude
  chmod 700 /home/claude
  chmod 700 /home/claude/.claude
  chown -R runner:runner /opt/vibe/conversations
  command -v python3 >/dev/null 2>&1 || (apt-get -o DPkg::Lock::Timeout=300 update -q && apt-get -o DPkg::Lock::Timeout=300 install -y -q python3)
  python3 -m pip --version >/dev/null 2>&1 || (apt-get -o DPkg::Lock::Timeout=300 update -q && apt-get -o DPkg::Lock::Timeout=300 install -y -q python3-pip)
  command -v npm >/dev/null 2>&1 || (apt-get -o DPkg::Lock::Timeout=300 update -q && apt-get -o DPkg::Lock::Timeout=300 install -y -q nodejs npm)
  # #1652 — Claude Code subscription provider. The CLI runs as `claude`; its
  # credentials stay under /home/claude/.claude (0700). Installing it here
  # keeps the first "Connect Claude" click fast; a failed install is reported
  # by the provider, never silently treated as connected.
  command -v claude >/dev/null 2>&1 || npm install -g @anthropic-ai/claude-code >/tmp/claude-install.log 2>&1 || true
  python3 -m pip install --break-system-packages --quiet --disable-pip-version-check \
    pandas numpy openpyxl python-pptx pillow matplotlib requests python-docx \
    reportlab beautifulsoup4 lxml tabulate || true
  echo v5 > /opt/vibe/.provisioned
} >/tmp/agent-prepare.log 2>&1 || true
"#;

/// Schedules workspace preparation once per VM per process. Never blocks the
/// caller: the on-demand installer covers a request that races the setup.
fn schedule_workspace_preparation(state: Arc<AgentService>, vm_name: String) {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static IN_FLIGHT: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let set = IN_FLIGHT.get_or_init(|| Mutex::new(HashSet::new()));
    {
        let mut guard = match set.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        if !guard.insert(vm_name.clone()) {
            return;
        }
    }
    tokio::spawn(async move {
        let ready = exec_in_vm(
            &state,
            &vm_name,
            &[
                "sh".into(),
                "-c".into(),
                format!(
                    "test -f /opt/vibe/.provisioned && grep -q '^{WORKSPACE_PREPARE_VERSION}$' /opt/vibe/.provisioned"
                ),
            ],
        )
        .await;
        let already = matches!(ready, Ok((true, _, _)));
        if !already {
            let _ = exec_in_vm(
                &state,
                &vm_name,
                &["sh".into(), "-c".into(), WORKSPACE_PREPARE_SCRIPT.into()],
            )
            .await;
        }
        if let Ok(mut guard) = set.lock() {
            guard.remove(&vm_name);
        }
    });
}

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

    let launch = run_incus(
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
    .await;
    if let Err(e) = launch {
        // A previous attempt can leave the container created but stopped; the
        // launch then fails with "already exists" and the workspace stayed
        // permanently unavailable. Start the existing container instead.
        tracing::warn!("incus launch for {vm_name} failed ({}); starting the existing container", e.1);
        let _ = run_incus(state, vec!["start".into(), vm_name.into()]).await;
    }

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

    if wait_for_running(state, vm_name).await.is_err() {
        // The first launch of a fresh image can exceed the window while the
        // layers unpack; the container stays created but stopped. Start it
        // explicitly and wait again instead of failing the whole workspace.
        tracing::warn!("container {vm_name} did not start in time; retrying incus start");
        let _ = run_incus(state, vec!["start".into(), vm_name.into()]).await;
        if let Err(wait_error) = wait_for_running(state, vm_name).await {
            // A created-but-never-running container keeps its reason inside
            // the instance log (bpf/mount/cgroup host-side failures); the
            // daemon can even return exit 0 from `start` while the instance
            // stays STOPPED. Attach that evidence so callers surface the
            // true cause instead of a bare timeout.
            let reason = match instance_log_tail(state, vm_name).await {
                Some(tail) => format!("{}; instance log: {tail}", wait_error.1),
                None => wait_error.1,
            };
            return Err((wait_error.0, reason));
        }
    }

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
    // Explicit deletion is the only place a job is terminated. The pkill
    // pattern uses a bracket class (`[c]onversations`) so the shell running
    // the command does not match itself and die before `rm` executes — that
    // bug left every deleted conversation folder behind. `clean_cid` is
    // restricted to ASCII alphanumerics, '-' and '_' by `sanitize_id`.
    let kill = format!(
        "pkill -f '[c]onversations/{clean_cid}' 2>/dev/null || true"
    );
    if let Err((code, err)) = exec_in_vm(
        state,
        &session.vm_name,
        &["sh".to_string(), "-c".to_string(), kill],
    )
    .await
    {
        tracing::warn!("stop processes for {clean_cid} ({code}): {err}");
    }
    let remove = format!("rm -rf '{folder}'");
    if let Err((code, err)) = exec_in_vm(
        state,
        &session.vm_name,
        &["sh".to_string(), "-c".to_string(), remove],
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
/// Shell command that runs a generated program as the unprivileged `runner`
/// user (#1658). Paths are already sanitized by the caller.
fn runner_command(folder: &str, interpreter: &str, script_path: &str) -> String {
    format!("su runner -s /bin/sh -c 'cd {folder} && {interpreter} {script_path}'")
}

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
    let outcome = exec_in_vm_streaming(state, vm_name, cmd_args, None, None, None).await?;
    Ok((outcome.success, outcome.stdout, outcome.stderr))
}

/// The result of one VM command. `success` is kept separate from `exit_code`
/// because a cancelled or timed-out process has no exit code at all.
#[derive(Debug, Clone, Default)]
pub struct ExecOutcome {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub cancelled: bool,
    pub timed_out: bool,
}

/// Runs a command and streams stdout/stderr as line events while it runs.
///
/// The guarded `SafeCommand` still builds the invocation; this path only adds
/// piped readers, a real exit code, a timeout and cooperative cancellation.
pub async fn exec_in_vm_streaming(
    state: &AgentService,
    vm_name: &str,
    cmd_args: &[String],
    timeout: Option<StdDuration>,
    cancel: Option<Arc<AtomicBool>>,
    events: Option<mpsc::UnboundedSender<ExecEvent>>,
) -> Result<ExecOutcome, (StatusCode, String)> {
    let mut cmd = incus_command(state)?;
    cmd = cmd
        .arg("exec")
        .map_err(|error| internal(format!("incus arg: {error}")))?;
    cmd = cmd
        .arg(vm_name)
        .map_err(|error| internal(format!("incus arg: {error}")))?;
    cmd = cmd
        .arg("--")
        .map_err(|error| internal(format!("incus arg: {error}")))?;

    let is_shell_script = cmd_args.len() == 3 && cmd_args[0] == "sh" && cmd_args[1] == "-c";
    if is_shell_script {
        cmd = cmd
            .arg("sh")
            .map_err(|error| internal(format!("incus arg: {error}")))?;
        cmd = cmd
            .arg("-c")
            .map_err(|error| internal(format!("incus arg: {error}")))?;
        cmd = cmd
            .trusted_shell_script_arg(&cmd_args[2])
            .map_err(|error| internal(format!("incus script: {error}")))?;
    } else {
        for arg in cmd_args {
            cmd = cmd
                .arg(arg)
                .map_err(|error| internal(format!("incus arg: {error}")))?;
        }
    }
    cmd = cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    if let Some(sender) = &events {
        let label: String = cmd_args.join(" ").chars().take(500).collect();
        let _ = sender.send(ExecEvent::Started { command: label });
    }

    let started = Instant::now();
    let mut child = cmd
        .spawn()
        .map_err(|error| internal(format!("incus spawn: {error}")))?;

    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();
    let stdout_capture = Arc::new(Mutex::new(String::new()));
    let stderr_capture = Arc::new(Mutex::new(String::new()));
    let mut readers = Vec::new();

    if let Some(pipe) = stdout_pipe {
        let capture = stdout_capture.clone();
        let sender = events.clone();
        readers.push(std::thread::spawn(move || {
            pump_lines(pipe, false, sender, capture);
        }));
    }
    if let Some(pipe) = stderr_pipe {
        let capture = stderr_capture.clone();
        let sender = events.clone();
        readers.push(std::thread::spawn(move || {
            pump_lines(pipe, true, sender, capture);
        }));
    }

    let wait = tokio::task::spawn_blocking(move || {
        wait_for_child(child, started, timeout, cancel)
    })
    .await
    .map_err(|error| internal(format!("incus wait join: {error}")))?;
    let (exit_code, cancelled, timed_out) = wait;

    for reader in readers {
        let _ = reader.join();
    }

    let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let success = exit_code == Some(0) && !cancelled && !timed_out;
    let stdout = stdout_capture
        .lock()
        .map(|buffer| buffer.clone())
        .unwrap_or_default();
    let stderr = stderr_capture
        .lock()
        .map(|buffer| buffer.clone())
        .unwrap_or_default();

    if let Some(sender) = &events {
        let _ = sender.send(ExecEvent::Finished {
            exit_code,
            duration_ms,
            cancelled,
            timed_out,
        });
    }

    Ok(ExecOutcome {
        success,
        exit_code,
        stdout,
        stderr,
        duration_ms,
        cancelled,
        timed_out,
    })
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

#[cfg(test)]
mod tests {
    use super::is_anonymous_email;

    #[test]
    fn guest_identities_are_anonymous() {
        assert!(is_anonymous_email("guest_1b0d9ece@anonymous.local"));
        assert!(is_anonymous_email("GUEST_ABC@ANONYMOUS.LOCAL"));
    }

    #[test]
    fn real_accounts_are_not_anonymous() {
        assert!(!is_anonymous_email("rodriguez@pragmatismo.com.br"));
        assert!(!is_anonymous_email("guest_reader@example.com"));
        assert!(!is_anonymous_email("someone@anonymous.local.example.com"));
        assert!(!is_anonymous_email(""));
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
