use super::*;

impl SessionManager {
    pub fn get_conversation_history_paged(
        &mut self,
        sess_id: Uuid,
        _uid: Uuid,
        history_limit: Option<i64>,
        offset: Option<i64>,
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
        let off_msgs = (offset.unwrap_or(0) * 2) as usize;
        // Work backwards from the end: skip `off_msgs` most-recent rows, then
        // take the preceding page.
        let end = messages.len().saturating_sub(off_msgs);
        let start = end.saturating_sub(total_messages_needed);
        let page_messages: Vec<_> = messages.into_iter().skip(start).take(end - start).collect();

        let bot_id: Uuid = user_sessions::table
            .filter(user_sessions::id.eq(sess_id))
            .select(user_sessions::bot_id)
            .first(&mut conn)?;
        let key_bytes = derive_scope_key(&self.master_key, "message", &bot_id);

        let mut history: Vec<(String, String)> = Vec::new();
        let mut unreadable = 0usize;
        for (other_role, content, _idx) in page_messages {
            let role_str = match other_role {
                1 => "user".to_string(),
                2 => "assistant".to_string(),
                3 => "system".to_string(),
                9 => "episodic".to_string(),
                _ => "unknown".to_string(),
            };
            match decrypt_field(&content, &key_bytes) {
                Ok(decrypted) => {
                    if other_role == 2 && decrypted.trim_start().starts_with("{\"__tool_call__\"") {
                        continue;
                    }
                    // #2562 — strip the metadata block (reasoning etc.)
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
                "get_conversation_history_paged: {unreadable} message(s) skipped for session {sess_id}"
            );
        }
        Ok(history)
    }

    /// #2562 — Returns raw decrypted messages (metadata trailers intact) plus
    /// the DB `message_index`.  The history API handler decodes reasoning and
    /// attachments itself; the model-facing methods strip metadata.
    pub fn get_conversation_history_raw(
        &mut self,
        sess_id: Uuid,
        _uid: Uuid,
        history_limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<(String, String, i32)>, Box<dyn Error + Send + Sync>> {
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
        let off_msgs = (offset.unwrap_or(0) * 2) as usize;
        let end = messages.len().saturating_sub(off_msgs);
        let start = end.saturating_sub(total_messages_needed);
        let page_messages: Vec<_> = messages.into_iter().skip(start).take(end - start).collect();

        let bot_id: Uuid = user_sessions::table
            .filter(user_sessions::id.eq(sess_id))
            .select(user_sessions::bot_id)
            .first(&mut conn)?;
        let key_bytes = derive_scope_key(&self.master_key, "message", &bot_id);

        let mut history: Vec<(String, String, i32)> = Vec::new();
        let mut unreadable = 0usize;
        for (other_role, content, idx) in page_messages {
            let role_str = match other_role {
                1 => "user".to_string(),
                2 => "assistant".to_string(),
                3 => "system".to_string(),
                9 => "episodic".to_string(),
                _ => "unknown".to_string(),
            };
            match decrypt_field(&content, &key_bytes) {
                Ok(decrypted) => {
                    if other_role == 2 && decrypted.trim_start().starts_with("{\"__tool_call__\"") {
                        continue;
                    }
                    // Return raw content — the caller decodes meta itself.
                    // Still skip empty visible text.
                    let visible = strip_meta_block(&decrypted);
                    if visible.trim().is_empty() {
                        continue;
                    }
                    history.push((role_str, decrypted, idx));
                }
                Err(_) => unreadable += 1,
            }
        }
        if unreadable > 0 {
            botlog::safe_warn!(
                "get_conversation_history_raw: {unreadable} message(s) skipped for session {sess_id}"
            );
        }
        Ok(history)
    }

    pub fn count_conversation_messages(
        &mut self,
        sess_id: Uuid,
    ) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let mut conn = self.pool.get()?;
        let count: i64 = message_history::table
            .filter(message_history::session_id.eq(sess_id))
            .count()
            .get_result(&mut conn)?;
        Ok(count)
    }

    pub fn get_first_user_message(
        &mut self,
        sess_id: Uuid,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let mut conn = self.pool.get()?;
        // A conversation can start with an empty control message; keep
        // scanning user turns until one has readable text, otherwise the rail
        // labels every conversation "Conversation · <date>".
        let rows: Vec<String> = message_history::table
            .filter(message_history::session_id.eq(sess_id))
            .filter(message_history::role.eq(1))
            .order(message_history::message_index.asc())
            .select(message_history::content_encrypted)
            .load::<String>(&mut conn)?;
        let bot_id: Uuid = user_sessions::table
            .filter(user_sessions::id.eq(sess_id))
            .select(user_sessions::bot_id)
            .first(&mut conn)?;
        let key_bytes = derive_scope_key(&self.master_key, "message", &bot_id);
        for content in rows {
            match decrypt_field(&content, &key_bytes) {
                Ok(decrypted) if !decrypted.trim().is_empty() => {
                    return Ok(Some(strip_meta_block(&decrypted)));
                }
                Ok(_) => continue,
                // Ciphertext would surface as the session title in the UI; an
                // unreadable title is better than a wall of base64.
                Err(e) => {
                    botlog::safe_warn!(
                        "get_first_user_message: title unreadable for session {sess_id}: {e}"
                    );
                }
            }
        }
        Ok(None)
    }

    pub fn get_user_sessions(
        &mut self,
        uid: Uuid,
    ) -> Result<Vec<UserSession>, Box<dyn Error + Send + Sync>> {
        let mut conn = self.pool.get()?;
        let sessions = if uid == Uuid::nil() {
            user_sessions::table
                .order(user_sessions::created_at.desc())
                .load::<UserSession>(&mut conn)
                .unwrap_or_else(|_| Vec::new())
        } else {
            user_sessions::table
                .filter(user_sessions::user_id.eq(uid.to_string()))
                .order(user_sessions::created_at.desc())
                .load::<UserSession>(&mut conn)
                .unwrap_or_else(|_| Vec::new())
        };
        Ok(sessions)
    }

    pub fn update_user_id(
        &mut self,
        session_id: Uuid,
        new_user_id: Uuid,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut conn = self.pool.get()?;
        let updated_count =
            diesel::update(user_sessions::table.filter(user_sessions::id.eq(session_id)))
                .set((
                    user_sessions::user_id.eq(new_user_id.to_string()),
                    user_sessions::updated_at.eq(chrono::Utc::now()),
                ))
                .execute(&mut conn)?;
        if updated_count == 0 {
            botlog::safe_warn!("No session found with ID: {}", session_id);
        } else {
            botlog::safe_trace!("Updated user ID for session: {}", session_id);
        }
        Ok(())
    }

    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn total_count(&mut self) -> usize {
        if let Ok(mut conn) = self.pool.get() {
            user_sessions::table
                .count()
                .first::<i64>(&mut conn)
                .unwrap_or(0) as usize
        } else {
            0
        }
    }

    pub fn recent_sessions(
        &mut self,
        hours: i64,
    ) -> Result<Vec<UserSession>, Box<dyn Error + Send + Sync>> {
        let mut conn = self.pool.get()?;
        let since = chrono::Utc::now() - chrono::Duration::hours(hours);
        let sessions = user_sessions::table
            .filter(user_sessions::created_at.gt(since))
            .order(user_sessions::created_at.desc())
            .load::<UserSession>(&mut conn)?;
        Ok(sessions)
    }

    pub fn get_statistics(&mut self) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
        let mut conn = self.pool.get()?;
        let total = user_sessions::table.count().first::<i64>(&mut conn)?;
        let active = self.sessions.len() as i64;
        let today = chrono::Utc::now().date_naive();
        let today_start = today
            .and_hms_opt(0, 0, 0)
            .unwrap_or_else(|| {
                today
                    .and_hms_opt(0, 0, 1)
                    .unwrap_or_else(|| chrono::NaiveDateTime::default())
            })
            .and_utc();
        let today_count = user_sessions::table
            .filter(user_sessions::created_at.ge(today_start))
            .count()
            .first::<i64>(&mut conn)?;
        Ok(serde_json::json!({
            "total_sessions": total,
            "active_sessions": active,
            "today_sessions": today_count,
            "waiting_for_input": self.waiting_for_input.len()
        }))
    }
}
