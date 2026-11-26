//! Signs a person out everywhere after their password changed.

use super::*;

/// Whether a stored session belongs to the person (same id, or same email).
fn session_belongs_to(session: &SessionUserData, user_id: &str, email: &str) -> bool {
    (!user_id.is_empty() && session.user_id == user_id)
        || (!email.trim().is_empty() && session.email.trim().eq_ignore_ascii_case(email.trim()))
}

/// Revokes every suite session of the person except `keep`, in memory and in
/// `login_sessions`. Returns how many sessions were signed out.
pub async fn revoke_user_sessions(user_id: &str, email: &str, keep: Option<&str>) -> usize {
    let mut in_memory = 0usize;
    {
        let mut cache = SESSION_CACHE.write().await;
        cache.retain(|token, session| {
            let stays = !session_belongs_to(session, user_id, email) || keep == Some(token.as_str());
            if !stays {
                in_memory += 1;
            }
            stays
        });
    }
    let mut persisted = 0usize;
    if let Some(pool) = SESSION_POOL.get() {
        if let Ok(mut conn) = pool.get() {
            use diesel::RunQueryDsl;
            persisted = diesel::sql_query(
                "DELETE FROM login_sessions \
                 WHERE token <> $3 \
                   AND (($1 <> '' AND user_data->>'user_id' = $1) \
                     OR ($2 <> '' AND lower(user_data->>'email') = lower($2)))",
            )
            .bind::<diesel::sql_types::Text, _>(user_id)
            .bind::<diesel::sql_types::Text, _>(email.trim())
            .bind::<diesel::sql_types::Text, _>(keep.unwrap_or(""))
            .execute(&mut conn)
            .unwrap_or_else(|e| {
                warn!("Could not revoke persisted sessions: {e}");
                0
            });
        }
    }
    in_memory.max(persisted)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(user_id: &str, email: &str) -> SessionUserData {
        SessionUserData {
            user_id: user_id.to_string(),
            email: email.to_string(),
            username: String::new(),
            first_name: None,
            last_name: None,
            display_name: None,
            organization_id: None,
            roles: Vec::new(),
            bucket: None,
            created_at: 0,
        }
    }

    #[test]
    fn a_session_belongs_to_the_same_id_or_email() {
        let own = session("local-admin@localhost", "admin@localhost");
        assert!(session_belongs_to(&own, "local-admin@localhost", ""));
        assert!(session_belongs_to(&own, "other", "ADMIN@localhost"));
        assert!(!session_belongs_to(&own, "other", "someone@localhost"));
        assert!(!session_belongs_to(&session("", ""), "", ""));
    }
}
