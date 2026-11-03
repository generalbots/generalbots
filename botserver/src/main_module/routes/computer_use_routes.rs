//! Computer-use HTTP surface (GB6-19 / GB6-20).
//!
//! `POST /api/computer/actions` lets the assistant (or a tool) propose a
//! browser or terminal action for a conversation. The suite polls
//! `/api/computer/actions/pending`, renders the consent card, executes the
//! action and posts the outcome to `/api/computer/actions/:id/result`.
//!
//! `POST /api/terminal/exec` is the server-side executor for terminal actions:
//! it refuses to run without an explicit approval flag, refuses destructive
//! patterns, and always executes inside the user's own agent container — never
//! on the bot host.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Json, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use botcore::shared::state::AppState;
use botlib::security::command_guard::SafeCommand;
use serde::Deserialize;
use diesel::sql_types::{Integer, Nullable, Text, Uuid as DieselUuid};
use diesel::RunQueryDsl;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::core::computer_use;

type ApiError = (StatusCode, Json<Value>);

fn api_error(status: StatusCode, message: &str) -> ApiError {
    (status, Json(json!({ "error": message })))
}

fn resolve_user(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "missing token"))?;
    let (claims_user, _roles) =
        crate::main_module::routes::anonymous_auth::resolve_token_user(token)
            .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "invalid token"))?;
    Ok(crate::security::user_role::derive_stable_user_uuid(
        &claims_user,
    ))
}

#[derive(Debug, Deserialize)]
pub struct CreateActionRequest {
    pub session_id: String,
    /// `browser` or `terminal`.
    pub kind: String,
    pub action: Value,
}

/// POST /api/computer/actions
pub async fn create_action(
    headers: HeaderMap,
    Json(body): Json<CreateActionRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    if body.session_id.trim().is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "session_id is required"));
    }
    if body.kind != "browser" && body.kind != "terminal" {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "kind must be 'browser' or 'terminal'",
        ));
    }
    let action =
        computer_use::create(body.session_id.trim(), user_id, &body.kind, body.action).await;
    Ok(Json(action.to_json()))
}

#[derive(Debug, Deserialize)]
pub struct PendingQuery {
    pub session_id: String,
    #[serde(default)]
    pub kind: Option<String>,
}

/// GET /api/computer/actions/pending?session_id=&kind=
pub async fn pending(
    headers: HeaderMap,
    Query(query): Query<PendingQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let kind = query.kind.as_deref().unwrap_or("browser");
    let actions = computer_use::pending_for(query.session_id.trim(), user_id, kind).await;
    Ok(Json(json!({
        "actions": actions.iter().map(computer_use::ComputerAction::to_json).collect::<Vec<_>>(),
    })))
}

/// POST /api/computer/actions/:id/result
pub async fn submit_result(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(result): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    // A denial is an auditable decision even though nothing ran.
    if result.get("status").and_then(Value::as_str) == Some("denied") {
        if let Some(action) = computer_use::get(id, user_id).await {
            if action.kind == "terminal" {
                let command = action
                    .action
                    .get("command")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                audit_terminal(
                    &state.conn,
                    TerminalAuditRow {
                        user_id,
                        session_id: &action.session_id,
                        action_id: Some(id),
                        command,
                        cwd: action
                            .action
                            .get("cwd")
                            .and_then(Value::as_str)
                            .unwrap_or(""),
                        container: "",
                        decision: "denied",
                        exit_code: None,
                        output: "user denied consent",
                    },
                );
            }
        }
    }
    match computer_use::complete(id, user_id, result).await {
        Some(action) => Ok(Json(action.to_json())),
        None => Err(api_error(StatusCode::NOT_FOUND, "action not found")),
    }
}

/// GET /api/computer/actions/:id
pub async fn get_action(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    match computer_use::get(id, user_id).await {
        Some(action) => Ok(Json(action.to_json())),
        None => Err(api_error(StatusCode::NOT_FOUND, "action not found")),
    }
}

#[derive(Debug, Deserialize)]
pub struct TerminalExecRequest {
    pub command: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    /// Explicit user consent. A request without it is refused: nothing runs
    /// silently on the user's machine.
    #[serde(default)]
    pub approved: bool,
    #[serde(default)]
    pub session_id: Option<String>,
    /// #1645 — the action whose consent record authorises this execution.
    #[serde(default)]
    pub action_id: Option<Uuid>,
}

const DENIED_PATTERNS: &[&str] = &[
    "rm -rf /",
    "mkfs",
    "dd if=",
    "shutdown",
    "reboot",
    ":(){",
    "> /dev/sda",
    "chmod -R 000 /",
];

/// Server-side re-check of destructive patterns. The consent card enforces the
/// same list client-side, but the server must not rely on it.
pub(crate) fn blocked_pattern(command: &str) -> Option<&'static str> {
    let lowered = command.to_ascii_lowercase();
    DENIED_PATTERNS
        .iter()
        .find(|pattern| lowered.contains(&pattern.to_ascii_lowercase()))
        .copied()
}

/// POST /api/terminal/exec
pub async fn terminal_exec(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<TerminalExecRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    if !body.approved {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "terminal execution requires explicit user approval",
        ));
    }
    // #1645 — `approved=true` alone is forgeable. Execution requires the
    // action id whose consent record /approve created.
    let action_id = body.action_id.ok_or_else(|| {
        api_error(
            StatusCode::FORBIDDEN,
            "terminal execution requires an approved action_id",
        )
    })?;
    if !computer_use::is_approved(action_id, user_id).await {
        audit_terminal(
            &state.conn,
            TerminalAuditRow {
                user_id,
                session_id: body.session_id.as_deref().unwrap_or(""),
                action_id: Some(action_id),
                command: body.command.trim(),
                cwd: body.cwd.as_deref().unwrap_or(""),
                container: "",
                decision: "denied",
                exit_code: None,
                output: "no consent record",
            },
        );
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "no consent record for this action; approve it first",
        ));
    }
    let command = body.command.trim();
    if command.is_empty() {
        return Err(api_error(StatusCode::BAD_REQUEST, "command is required"));
    }
    if command.len() > 16_000 {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "command is limited to 16000 characters",
        ));
    }
    if blocked_pattern(command).is_some() {
        log::warn!("terminal exec refused a blocked pattern for user {user_id}");
        audit_terminal(
            &state.conn,
            TerminalAuditRow {
                user_id,
                session_id: body.session_id.as_deref().unwrap_or(""),
                action_id: Some(action_id),
                command,
                cwd: "",
                container: "",
                decision: "denied",
                exit_code: None,
                output: "blocked destructive pattern",
            },
        );
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "command matches a blocked destructive pattern",
        ));
    }
    #[cfg(feature = "agent-vm")]
    let container = crate::core::bot::agent_vm_hook::user_vm_name(user_id).await;
    #[cfg(not(feature = "agent-vm"))]
    let container: Option<String> = None;
    let container = container.ok_or_else(|| {
        api_error(
            StatusCode::FORBIDDEN,
            "no workspace container: send a message in chat first",
        )
    })?;

    let cwd = body
        .cwd
        .clone()
        .filter(|value| value.starts_with('/') && !value.contains(".."))
        .unwrap_or_else(|| "/opt/vibe/app".to_string());
    let timeout = Duration::from_millis(body.timeout_ms.unwrap_or(30_000).clamp(1_000, 120_000));
    let command_owned = command.to_string();
    let container_for_exec = container.clone();
    let cwd_for_exec = cwd.clone();

    let execution = tokio::task::spawn_blocking(move || {
        let guarded = SafeCommand::new("incus")
            .and_then(|cmd| cmd.arg("exec"))
            .and_then(|cmd| cmd.arg(&container_for_exec))
            .and_then(|cmd| cmd.arg("--cwd"))
            .and_then(|cmd| cmd.arg(&cwd_for_exec))
            .and_then(|cmd| cmd.arg("--"))
            .and_then(|cmd| cmd.arg("sh"))
            .and_then(|cmd| cmd.arg("-c"))
            .and_then(|cmd| cmd.trusted_shell_script_arg(&command_owned))?;
        guarded.execute()
    });

    let outcome = match tokio::time::timeout(timeout, execution).await {
        Ok(Ok(Ok(output))) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let truncated = stdout.len() > 16_000 || stderr.len() > 16_000;
            json!({
                "status": if output.status.success() { "ok" } else { "failed" },
                "exit_code": output.status.code(),
                "stdout": stdout.chars().take(16_000).collect::<String>(),
                "stderr": stderr.chars().take(16_000).collect::<String>(),
                "truncated": truncated,
            })
        }
        Ok(Ok(Err(error))) => json!({ "status": "failed", "error": error.to_string() }),
        Ok(Err(error)) => json!({ "status": "failed", "error": error.to_string() }),
        Err(_) => json!({ "status": "failed", "error": "command timed out" }),
    };
    log::info!(
        "terminal exec for {user_id} in {container} ({} chars) -> {}",
        command.len(),
        outcome.get("status").and_then(Value::as_str).unwrap_or("unknown")
    );
    // #1645 — durable row, independent of the in-memory registry TTL.
    let exit_code = outcome
        .get("exit_code")
        .and_then(Value::as_i64)
        .map(|value| value as i32);
    let output_excerpt = format!(
        "{}{}",
        outcome.get("stdout").and_then(Value::as_str).unwrap_or(""),
        outcome.get("stderr").and_then(Value::as_str).unwrap_or("")
    );
    audit_terminal(
        &state.conn,
        TerminalAuditRow {
            user_id,
            session_id: body.session_id.as_deref().unwrap_or(""),
            action_id: Some(action_id),
            command,
            cwd: &cwd,
            container: &container,
            decision: "approved",
            exit_code,
            output: &output_excerpt,
        },
    );

    let recorded = computer_use::record_terminal(
        body.session_id.as_deref().unwrap_or(""),
        user_id,
        json!({ "command": command, "cwd": cwd, "timeout_ms": timeout.as_millis() }),
        outcome,
    )
    .await;
    Ok(Json(recorded.to_json()))
}

#[derive(Debug, Deserialize)]
pub struct ApproveActionRequest {
    /// The client echoes the action it is asking the user about; a mismatch is
    /// refused so an approval cannot be replayed onto another action.
    #[serde(default)]
    pub action_id: Option<Uuid>,
}

/// POST /api/computer/actions/:id/approve — creates the consent record that
/// the executors require. Refusing is simply *not* calling this endpoint.
pub async fn approve_action(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<ApproveActionRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    if let Some(body_id) = body.action_id {
        if body_id != id {
            return Err(api_error(
                StatusCode::BAD_REQUEST,
                "action_id does not match the path",
            ));
        }
    }
    match computer_use::approve(id, user_id).await {
        Some(action) => Ok(Json(action.to_json())),
        None => Err(api_error(
            StatusCode::NOT_FOUND,
            "action not found or already denied",
        )),
    }
}

#[derive(Debug, Deserialize)]
pub struct ExecuteActionRequest {
    /// Kept for compatibility with older clients; the server requires the
    /// stored consent record created by /approve.
    #[serde(default)]
    pub approved: bool,
}

/// POST /api/computer/actions/:id/execute — runs the browser actions that need
/// the server (fetching a page, extracting its title/text). Actions that act
/// on the user's own browser panel (click/fill) stay client-side.
pub async fn execute_action(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<ExecuteActionRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let action = computer_use::get(id, user_id)
        .await
        .ok_or_else(|| api_error(StatusCode::NOT_FOUND, "action not found"))?;
    if !body.approved || !computer_use::is_approved(id, user_id).await {
        return Err(api_error(
            StatusCode::FORBIDDEN,
            "no consent record for this action; approve it first",
        ));
    }
    let name = action
        .action
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("");
    if name != "open_url" && name != "extract_text" {
        return Err(api_error(
            StatusCode::BAD_REQUEST,
            "this action must run in the browser panel",
        ));
    }
    let url = action
        .action
        .get("url")
        .and_then(Value::as_str)
        .ok_or_else(|| api_error(StatusCode::BAD_REQUEST, "action has no url"))?;
    let page = fetch_page(url)
        .await
        .map_err(|e| api_error(StatusCode::BAD_GATEWAY, &e))?;
    let result = json!({
        "status": "ok",
        "url": page.url,
        "title": page.title,
        "text": page.text,
    });
    computer_use::complete(id, user_id, result.clone()).await;
    Ok(Json(result))
}

struct PageContent {
    url: String,
    title: String,
    text: String,
}

/// Fetches a page server-side and returns a bounded, readable extract. This is
/// what lets the assistant cite a real page title even when the client iframe
/// cannot read a cross-origin document.
async fn fetch_page(url: &str) -> Result<PageContent, String> {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("only http(s) URLs can be opened".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("GeneralBots-ComputerUse/1.0")
        .build()
        .map_err(|e| format!("http client: {e}"))?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("fetch failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("the page answered HTTP {}", response.status().as_u16()));
    }
    let final_url = response.url().to_string();
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("read failed: {e}"))?;
    let html = String::from_utf8_lossy(&bytes[..bytes.len().min(2_000_000)]).to_string();
    Ok(PageContent {
        url: final_url,
        title: extract_tag(&html, "title").unwrap_or_default(),
        text: html_to_text(&html, 12_000),
    })
}

fn extract_tag(html: &str, tag: &str) -> Option<String> {
    let pattern = format!(r"(?is)<{tag}[^>]*>(.*?)</{tag}>");
    let re = regex::Regex::new(&pattern).ok()?;
    let captures = re.captures(html)?;
    let value = captures.get(1)?.as_str().trim();
    if value.is_empty() {
        None
    } else {
        Some(decode_entities(value))
    }
}

fn html_to_text(html: &str, limit: usize) -> String {
    let mut out = String::with_capacity(limit.min(html.len()));
    let mut tag = String::new();
    let mut in_tag = false;
    let mut skip = false;
    for ch in html.chars() {
        if in_tag {
            if ch == '>' {
                let closing = tag.starts_with('/');
                let name = tag
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_lowercase();
                match name.as_str() {
                    "script" | "style" | "noscript" => skip = !closing,
                    "p" | "br" | "div" | "li" | "tr" | "h1" | "h2" | "h3" | "h4" | "td" => {
                        out.push(' ')
                    }
                    _ => {}
                }
                in_tag = false;
                tag.clear();
            } else {
                tag.push(ch);
            }
        } else if ch == '<' {
            in_tag = true;
            tag.clear();
        } else if !skip {
            out.push(ch);
        }
        if out.len() >= limit {
            break;
        }
    }
    let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
    decode_entities(&collapsed)
}

fn decode_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

struct TerminalAuditRow<'a> {
    user_id: Uuid,
    session_id: &'a str,
    action_id: Option<Uuid>,
    command: &'a str,
    cwd: &'a str,
    container: &'a str,
    decision: &'a str,
    exit_code: Option<i32>,
    output: &'a str,
}

fn audit_terminal(pool: &botcore::shared::utils::DbPool, row: TerminalAuditRow<'_>) {
    let Ok(mut conn) = pool.get() else {
        log::warn!("terminal audit: database unavailable, row not written");
        return;
    };
    let excerpt: String = row.output.chars().take(2000).collect();
    if let Err(error) = diesel::sql_query(
        "INSERT INTO terminal_audit
            (user_id, session_id, action_id, command, cwd, container, decision, exit_code, output_excerpt)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind::<DieselUuid, _>(row.user_id)
    .bind::<Nullable<Text>, _>(Some(row.session_id))
    .bind::<Nullable<DieselUuid>, _>(row.action_id)
    .bind::<Text, _>(row.command.chars().take(16_000).collect::<String>())
    .bind::<Nullable<Text>, _>(Some(row.cwd))
    .bind::<Nullable<Text>, _>(Some(row.container))
    .bind::<Text, _>(row.decision)
    .bind::<Nullable<Integer>, _>(row.exit_code)
    .bind::<Nullable<Text>, _>(Some(excerpt))
    .execute(&mut conn)
    {
        log::warn!("terminal audit insert failed: {error}");
    }
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    #[serde(default)]
    pub limit: Option<i64>,
}

/// GET /api/terminal/audit — the visible trail: command, decision, exit code
/// and timestamp, independent of the 30-minute in-memory registry.
pub async fn terminal_audit(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(query): Query<AuditQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = resolve_user(&headers)?;
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let mut conn = state
        .conn
        .get()
        .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        command: String,
        #[diesel(sql_type = Nullable<Text>)]
        cwd: Option<String>,
        #[diesel(sql_type = Text)]
        decision: String,
        #[diesel(sql_type = Nullable<Integer>)]
        exit_code: Option<i32>,
        #[diesel(sql_type = Nullable<Text>)]
        output_excerpt: Option<String>,
        #[diesel(sql_type = Text)]
        created_at: String,
    }
    let rows = diesel::sql_query(
        "SELECT id::text, command, cwd, decision, exit_code, output_excerpt,
                created_at::text AS created_at
         FROM terminal_audit WHERE user_id = $1
         ORDER BY created_at DESC LIMIT $2",
    )
    .bind::<DieselUuid, _>(user_id)
    .bind::<diesel::sql_types::BigInt, _>(limit)
    .load::<Row>(&mut conn)
    .unwrap_or_default();
    Ok(Json(json!({
        "entries": rows.into_iter().map(|row| json!({
            "id": row.id,
            "command": row.command,
            "cwd": row.cwd,
            "decision": row.decision,
            "exit_code": row.exit_code,
            "output_excerpt": row.output_excerpt,
            "created_at": row.created_at,
        })).collect::<Vec<_>>(),
    })))
}

/// #1645 — 90-day retention. Started from the background boot sequence.
pub fn spawn_terminal_audit_purge(state: Arc<AppState>) {
    tokio::spawn(async move {
        let secs: u64 = std::env::var("GB_TERMINAL_AUDIT_PURGE_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|value| *value >= 60)
            .unwrap_or(3600);
        loop {
            tokio::time::sleep(Duration::from_secs(secs)).await;
            match state.conn.get() {
                Ok(mut conn) => {
                    let deleted = diesel::sql_query(
                        "DELETE FROM terminal_audit WHERE created_at < NOW() - INTERVAL '90 days'",
                    )
                    .execute(&mut conn)
                    .unwrap_or(0);
                    if deleted > 0 {
                        log::info!("terminal audit: purged {deleted} row(s) older than 90 days");
                    }
                }
                Err(error) => log::warn!("terminal audit purge: db pool: {error}"),
            }
        }
    });
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/computer/actions", post(create_action))
        .route("/api/computer/actions/pending", get(pending))
        .route("/api/computer/actions/:id", get(get_action))
        .route("/api/computer/actions/:id/approve", post(approve_action))
        .route("/api/computer/actions/:id/execute", post(execute_action))
        .route("/api/computer/actions/:id/result", post(submit_result))
        .route("/api/terminal/exec", post(terminal_exec))
        .route("/api/terminal/audit", get(terminal_audit))
}

#[cfg(test)]
mod tests {
    use super::blocked_pattern;

    #[test]
    fn destructive_commands_are_refused_server_side() {
        assert_eq!(blocked_pattern("rm -rf /"), Some("rm -rf /"));
        assert_eq!(blocked_pattern("sudo SHUTDOWN now"), Some("shutdown"));
        assert_eq!(blocked_pattern("dd if=/dev/zero of=/dev/sda"), Some("dd if="));
        assert_eq!(blocked_pattern(":(){ :|:& };:"), Some(":(){"));
    }

    #[test]
    fn ordinary_commands_are_not_blocked() {
        for command in ["ls -la", "python3 make_pdf.py", "echo hi", "git status"] {
            assert_eq!(blocked_pattern(command), None, "{command} must be allowed");
        }
    }
}
