//! Conversation folder management inside a user's container.

use axum::http::StatusCode;
use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{AgentConversationFolderRow, AgentSessionRow};
use crate::schema::{agent_conversation_folders, agent_sessions};
use crate::AgentService;

use super::db::{load_session_by_user, set_active_conversation};
use super::identity::sanitize_id;
use super::incus::exec_in_vm;
use super::{bad_request, internal, CONVERSATIONS_ROOT};

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
        let mut conn = state
            .pool()
            .get()
            .map_err(|e| internal(format!("DB pool: {e}")))?;
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

    exec_in_vm(
        state,
        &session.vm_name,
        &["mkdir".into(), "-p".into(), folder_path.clone()],
    )
    .await?;
    for sub in ["output", "program", "data"] {
        let sub_path = format!("{folder_path}/{sub}");
        let _ = exec_in_vm(
            state,
            &session.vm_name,
            &["mkdir".into(), "-p".into(), sub_path],
        )
        .await;
    }

    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
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
        // Newest first: the file the user just asked for must lead the list;
        // older outputs follow as history (#1693).
        "find /opt/vibe/conversations/{clean_cid}/output -maxdepth 1 -type f -printf '%T@ %f\\n' 2>/dev/null | sort -rn | cut -d' ' -f2- | head -100"
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

    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
    let session = load_session_by_user(&mut conn, user_id)?
        .ok_or_else(|| bad_request("No active VM for this user"))?;
    drop(conn);

    ensure_conversation_folder(state, &session, &clean_cid).await?;

    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
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

    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
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
    let kill = format!("pkill -f '[c]onversations/{clean_cid}' 2>/dev/null || true");
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

    let mut conn = state
        .pool()
        .get()
        .map_err(|e| internal(format!("DB pool: {e}")))?;
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
