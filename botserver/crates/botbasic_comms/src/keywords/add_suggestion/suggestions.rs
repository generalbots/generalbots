use super::*;

/// #2547 — (scoped key, legacy key) for a session-scoped Redis set.
///
/// The legacy key is dual-written because sessions created before `org_id`
/// existed, and read paths that cannot resolve the org, still use it. It is
/// safe: the last component is a UUID v4 session id, unique across orgs.
pub(super) fn session_keys(user_session: &UserSession, prefix: &str) -> (String, Option<String>) {
    let parts = [
        prefix,
        &user_session.bot_id.to_string(),
        &user_session.id.to_string(),
    ];
    let legacy = botlib::key_utils::legacy_key(&parts);
    match user_session.org_id() {
        Some(org) => (
            botlib::key_utils::scoped_key(Some(org), &parts),
            Some(legacy),
        ),
        None => (legacy, None),
    }
}

/// Dual-write a set member to the org-scoped and the historical key.
fn sadd_scoped(
    conn: &mut redis::Connection,
    scoped: &str,
    legacy: Option<&String>,
    member: &str,
) {
    let _: Result<i64, redis::RedisError> =
        redis::cmd("SADD").arg(scoped).arg(member).query(conn);
    if let Some(legacy) = legacy.filter(|key| key.as_str() != scoped) {
        let _: Result<i64, redis::RedisError> =
            redis::cmd("SADD").arg(legacy).arg(member).query(conn);
    }
}

pub(super) fn add_context_suggestion(
    cache: Option<&Arc<redis::Client>>,
    user_session: &UserSession,
    context_name: &str,
    button_text: &str,
) -> Result<(), Box<rhai::EvalAltResult>> {
    if let Some(cache_client) = cache {
        let (redis_key, legacy_key) = session_keys(user_session, "suggestions");

        let suggestion = json!({
            "type": "context",
            "context": context_name,
            "text": button_text,
            "action": {
                "type": "select_context",
                "context": context_name
            }
        });

        let mut conn = match get_redis_connection(cache_client) {
            Some(conn) => conn,
            None => {
                trace!("Cache not ready, skipping add context suggestion");
                return Ok(());
            }
        };

        sadd_scoped(&mut conn, &redis_key, legacy_key.as_ref(), &suggestion.to_string());

        trace!(
            "Added context suggestion '{}' to session {}",
            context_name,
            user_session.id
        );

        let (active_key, legacy_active_key) = session_keys(user_session, "active_context");

        let _: Result<i64, redis::RedisError> = redis::cmd("HSET")
            .arg(&active_key)
            .arg(context_name)
            .arg("inactive")
            .query(&mut conn);
        if let Some(legacy) = legacy_active_key.as_ref().filter(|key| *key != &active_key) {
            let _: Result<i64, redis::RedisError> = redis::cmd("HSET")
                .arg(legacy)
                .arg(context_name)
                .arg("inactive")
                .query(&mut conn);
        }
    } else {
        trace!("No cache configured, suggestion not added");
    }

    Ok(())
}

pub(super) fn add_text_suggestion(
    cache: Option<&Arc<redis::Client>>,
    user_session: &UserSession,
    text_value: &str,
    button_text: &str,
) -> Result<(), Box<rhai::EvalAltResult>> {
    if let Some(cache_client) = cache {
        let (redis_key, legacy_key) = session_keys(user_session, "suggestions");

        let suggestion = json!({
            "type": "text_value",
            "text": button_text,
            "value": text_value,
            "action": {
                "type": "send_message",
                "message": text_value
            }
        });

        let mut conn = match get_redis_connection(cache_client) {
            Some(conn) => conn,
            None => {
                trace!("Cache not ready, skipping add text suggestion");
                return Ok(());
            }
        };

        sadd_scoped(&mut conn, &redis_key, legacy_key.as_ref(), &suggestion.to_string());

        trace!(
            "Added text suggestion '{}' to session {}",
            text_value,
            user_session.id
        );
    } else {
        trace!("No cache configured, text suggestion not added");
    }

    Ok(())
}

pub(super) fn add_tool_suggestion(
    cache: Option<&Arc<redis::Client>>,
    user_session: &UserSession,
    tool_name: &str,
    params: Option<Vec<String>>,
    button_text: &str,
) -> Result<(), Box<rhai::EvalAltResult>> {
    info!(
        "ADD_SUGGESTION_TOOL called: tool={}, button={}",
        tool_name, button_text
    );
    if let Some(cache_client) = cache {
        let (redis_key, legacy_key) = session_keys(user_session, "suggestions");
        info!("Adding suggestion to Redis key: {}", redis_key);

        let prompt_for_params = params.as_ref().is_some_and(|p| !p.is_empty());
        let action_obj = json!({
            "type": "invoke_tool",
            "tool": tool_name,
            "params": params,
            "prompt_for_params": prompt_for_params
        });

        let suggestion = json!({
            "type": "invoke_tool",
            "text": button_text,
            "tool": tool_name,
            "action": action_obj
        });

        let mut conn = match get_redis_connection(cache_client) {
            Some(conn) => conn,
            None => {
                trace!("Cache not ready, skipping add tool suggestion");
                return Ok(());
            }
        };

        sadd_scoped(&mut conn, &redis_key, legacy_key.as_ref(), &suggestion.to_string());

        info!(
            "Added tool suggestion '{}' to session {}",
            tool_name, user_session.id
        );
    } else {
        trace!("No cache configured, tool suggestion not added");
    }

    Ok(())
}
