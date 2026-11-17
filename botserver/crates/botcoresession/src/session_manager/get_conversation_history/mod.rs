use super::*;

/// Strips the `\0GB_META\0{json}` trailer that `message_meta::encode` appends,
/// returning only the visible content. Legacy messages without the trailer pass
/// through unchanged.
pub(super) fn strip_meta_block(content: &str) -> String {
    const SEP: &str = "\0GB_META\0";
    if let Some(pos) = content.find(SEP) {
        content[..pos].trim_end_matches('\n').to_string()
    } else {
        content.to_string()
    }
}

impl SessionManager {
    pub fn get_conversation_history(
        &mut self,
        sess_id: Uuid,
        _uid: Uuid,
        history_limit: Option<i64>,
    ) -> Result<Vec<(String, String)>, Box<dyn Error + Send + Sync>> {
        let mut conn = self.pool.get()?;
        let limit_val = history_limit.unwrap_or(50);
        let messages = message_history::table
            .filter(message_history::session_id.eq(sess_id))
            .order(message_history::message_index.asc())
            .select((
                message_history::role,
                message_history::content_encrypted,
                message_history::message_index,
            ))
            .load::<(i32, String, i32)>(&mut conn)?;
        let total_messages_needed = (limit_val * 2) as usize;
        let start_idx = messages.len().saturating_sub(total_messages_needed);
        let recent_messages: Vec<_> = messages.into_iter().skip(start_idx).collect();

        let bot_id: Uuid = user_sessions::table
            .filter(user_sessions::id.eq(sess_id))
            .select(user_sessions::bot_id)
            .first(&mut conn)?;
        let key_bytes = derive_scope_key(&self.master_key, "message", &bot_id);

        let mut history: Vec<(String, String)> = Vec::new();
        let mut unreadable = 0usize;
        for (other_role, content, _idx) in recent_messages {
            let role_str = match other_role {
                1 => "user".to_string(),
                2 => "assistant".to_string(),
                3 => "system".to_string(),
                9 => "episodic".to_string(),
                _ => "unknown".to_string(),
            };

            // A message encrypted under a previous master key (Vault
            // re-init/re-key) can never be read back. Injecting the ciphertext
            // as history made the model narrate the garbage ("I've received 12
            // encrypted messages"), so the row is dropped instead — the
            // conversation continues without it and the count is reported once.
            match decrypt_field(&content, &key_bytes) {
                Ok(decrypted) => {
                    // The tool-call frame is a control message: the tool's
                    // result is stored as its own assistant turn. Replaying
                    // the raw JSON showed the protocol to the user.
                    if other_role == 2 && decrypted.trim_start().starts_with("{\"__tool_call__\"") {
                        continue;
                    }
                    // #2562 — strip the metadata block (reasoning etc.) so
                    // the model never sees its own reasoning replayed.
                    let visible = strip_meta_block(&decrypted);
                    if visible.trim().is_empty() {
                        continue;
                    }
                    history.push((role_str, visible));
                }
                Err(_) => unreadable += 1,
            }
        }
        if unreadable > 0 {
            botlog::safe_warn!(
                "get_conversation_history: {unreadable} message(s) skipped for session {sess_id} \
                 (encrypted under another master key)"
            );
        }
        Ok(history)
    }
}

mod paged_raw;
mod session_ops;
