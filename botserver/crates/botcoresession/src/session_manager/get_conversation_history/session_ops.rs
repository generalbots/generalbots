use super::*;

impl SessionManager {
    /// #2562 — Truncates a conversation from `from_index` onward: all messages
    /// with `message_index >= from_index` are deleted. Ownership is checked
    /// against `owner_id`. Returns the number of rows deleted, or 0 when the
    /// session does not exist or belongs to another user.
    pub fn truncate_from(
        &mut self,
        sess_id: Uuid,
        owner_id: Uuid,
        from_index: i32,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut conn = self.pool.get()?;
        let owner: Option<String> = user_sessions::table
            .filter(user_sessions::id.eq(sess_id))
            .select(user_sessions::user_id)
            .first(&mut conn)
            .optional()?;
        let Some(owner_uuid) = owner else {
            return Ok(0);
        };
        if owner_uuid != owner_id.to_string() {
            return Ok(0);
        }
        let deleted = diesel::delete(
            message_history::table
                .filter(message_history::session_id.eq(sess_id))
                .filter(message_history::message_index.ge(from_index)),
        )
        .execute(&mut conn)?;
        botlog::safe_trace!(
            "truncate_from: session {} from index {}: deleted {} rows",
            sess_id,
            from_index,
            deleted
        );
        Ok(deleted)
    }

    /// Deletes one conversation (messages first, then the session row) when it
    /// belongs to `owner_id`. Returns `false` when the session does not exist
    /// or belongs to another user, so callers can answer 404 without leaking
    /// ownership information. The caller is responsible for removing the
    /// conversation folder inside the agent container.
    pub fn delete_session(
        &mut self,
        sess_id: Uuid,
        owner_id: Uuid,
    ) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let mut conn = self.pool.get()?;
        let owner: Option<String> = user_sessions::table
            .filter(user_sessions::id.eq(sess_id))
            .select(user_sessions::user_id)
            .first(&mut conn)
            .optional()?;
        let Some(owner_uuid) = owner else {
            return Ok(false);
        };
        if owner_uuid != owner_id.to_string() {
            return Ok(false);
        }
        diesel::delete(message_history::table.filter(message_history::session_id.eq(sess_id)))
            .execute(&mut conn)?;
        diesel::delete(user_sessions::table.filter(user_sessions::id.eq(sess_id)))
            .execute(&mut conn)?;
        self.sessions.remove(&sess_id);
        self.waiting_for_input.remove(&sess_id);
        Ok(true)
    }

    /// Renames a conversation owned by `owner_id`. Returns `false` when the
    /// session does not exist or belongs to another user. The new title is
    /// truncated to TITLE_MAX_CHARS.
    pub fn rename_session(
        &mut self,
        sess_id: Uuid,
        owner_id: Uuid,
        new_title: &str,
    ) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let mut conn = self.pool.get()?;
        let owner: Option<String> = user_sessions::table
            .filter(user_sessions::id.eq(sess_id))
            .select(user_sessions::user_id)
            .first(&mut conn)
            .optional()?;
        let Some(owner_uuid) = owner else {
            return Ok(false);
        };
        if owner_uuid != owner_id.to_string() {
            return Ok(false);
        }
        let trimmed: String = new_title.trim().chars().take(80).collect();
        if trimmed.is_empty() {
            return Ok(false);
        }
        diesel::update(user_sessions::table.filter(user_sessions::id.eq(sess_id)))
            .set(user_sessions::title.eq(&trimmed))
            .execute(&mut conn)?;
        Ok(true)
    }

    /// #1948 — stores an auto-generated title only while the conversation is
    /// still untitled. A user-set title (or one written by an earlier turn) is
    /// never overwritten; returns `false` when nothing was written.
    pub fn rename_session_if_untitled(
        &mut self,
        sess_id: Uuid,
        owner_id: Uuid,
        new_title: &str,
    ) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let trimmed: String = new_title.trim().chars().take(80).collect();
        if trimmed.is_empty() {
            return Ok(false);
        }
        let mut conn = self.pool.get()?;
        let current: Option<String> = user_sessions::table
            .filter(user_sessions::id.eq(sess_id))
            .filter(user_sessions::user_id.eq(owner_id.to_string()))
            .select(user_sessions::title)
            .first(&mut conn)
            .optional()?;
        let Some(current) = current else {
            return Ok(false);
        };
        let current = current.trim();
        let placeholder = current.is_empty()
            || current.eq_ignore_ascii_case("new conversation")
            || current.eq_ignore_ascii_case("nova conversa")
            || current.eq_ignore_ascii_case("untitled");
        if !placeholder {
            return Ok(false);
        }
        diesel::update(user_sessions::table.filter(user_sessions::id.eq(sess_id)))
            .set(user_sessions::title.eq(&trimmed))
            .execute(&mut conn)?;
        Ok(true)
    }
}
