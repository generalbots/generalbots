use super::super::*;

#[cfg(feature = "mail")]
fn notification_email_enabled(conn: &mut PgConnection, user_id: &str, flag: &str) -> bool {
    #[derive(QueryableByName)]
    struct PrefRow {
        #[diesel(sql_type = Nullable<Text>)]
        pub(crate) value: Option<String>,
    }

    let row = diesel::sql_query(
        "SELECT preference_value::text AS value FROM user_preferences \
         WHERE user_id::text = $1 AND preference_key = 'notifications' LIMIT 1",
    )
    .bind::<Text, _>(user_id)
    .get_result::<PrefRow>(conn)
    .ok();
    let Some(row) = row else { return true };
    let Some(value) = row.value else { return true };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&value) else {
        return true;
    };
    parsed
        .get(flag)
        .and_then(|value| value.as_bool())
        .unwrap_or(true)
}

/// Fetch users matching a raw @mention token together with all of their
/// organization memberships. Tenant filtering is intentionally *not* done in
/// SQL here: the pure [`super::notification_email::MentionTenant::in_scope`]
/// filter applies it uniformly to both the author and recipient lookups and is
/// unit-testable with a two-tenant fixture (#2899).
#[cfg(feature = "mail")]
fn mention_candidates(
    conn: &mut PgConnection,
    token: &str,
) -> Vec<super::notification_email::MentionCandidate> {
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        email: String,
        #[diesel(sql_type = diesel::sql_types::Array<SqlUuid>)]
        org_ids: Vec<uuid::Uuid>,
    }
    diesel::sql_query(
        "SELECT u.id::text AS id, u.email, \
                COALESCE(array_agg(uo.org_id) FILTER (WHERE uo.org_id IS NOT NULL), '{}'::uuid[]) AS org_ids \
         FROM users u \
         LEFT JOIN user_organizations uo ON uo.user_id = u.id \
         WHERE LOWER(u.username) = LOWER($1) OR LOWER(u.email) = LOWER($1) \
            OR u.id::text = $1 \
         GROUP BY u.id, u.email",
    )
    .bind::<Text, _>(token)
    .load::<Row>(conn)
    .map_or_else(|_| { log::warn!("defaulting after error at {}:{}", file!(), line!()); Default::default() }, |v| v)
    .into_iter()
    .map(|row| super::notification_email::MentionCandidate {
        id: row.id,
        email: row.email,
        org_ids: row.org_ids,
    })
    .collect()
}

/// `POST /api/collab/comments` — create a comment (or reply via parent_id).
/// Deliver @mention notification emails through the platform notification
/// sender (compiled only with the `mail` feature, which pulls in `lettre`).
/// Runs on a blocking thread, is strictly fire-and-forget, and degrades to a
/// warning log on any failure — never panics and never blocks the comment
/// write.
#[cfg(feature = "mail")]
pub(crate) fn send_mention_emails(
    pool: botcore::shared::utils::DbPool,
    author_id: String,
    author_name: String,
    resource_type: String,
    resource_id: String,
    mentions: Vec<String>,
    body: String,
    caller_org_id: Option<uuid::Uuid>,
) {
    // #2899 — the tenant scope comes from the authenticated caller, never from
    // the mention token. Without a tenant, delivery is disabled (fail closed)
    // so a principal can never trigger a cross-tenant notification email.
    let Some(tenant) = super::notification_email::MentionTenant::from_caller(caller_org_id) else {
        warn!("mention email: no tenant scope for caller; delivery disabled (#2899)");
        return;
    };

    // Platform-owned sender only. Never use the author's or the recipient's
    // `user_email_accounts` mailbox as the sender: notification mail must not
    // leave through a personal Gmail/Outlook account, and delivery is simply
    // disabled when the workspace has no notification sender configured. The
    // recipient address itself comes from `notification_recipient` below.
    let Some(sender) = super::notification_email::NotificationSender::configured() else {
        warn!("mention email: no platform notification sender configured; delivery disabled");
        return;
    };

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            warn!("mention email: db pool unavailable: {e}");
            return;
        }
    };

    // Resolve the author and every recipient through the same tenant-scoped
    // lookup. The SQL returns each matching user with all their organization
    // memberships; `tenant.in_scope` is the pure filter that discards users
    // outside the caller's tenant (#2899), including for self-exclusion.
    let author_candidates = mention_candidates(&mut conn, &author_id);
    let author = tenant.in_scope(&author_candidates).into_iter().next();
    let author_user_id = author.as_ref().map(|candidate| candidate.id.clone());
    let author_email = author
        .as_ref()
        .map(|candidate| candidate.email.to_lowercase());

    let mut recipients: Vec<super::notification_email::MentionCandidate> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for token in &mentions {
        let token = token.trim();
        if token.is_empty() || !seen.insert(token.to_lowercase()) {
            continue;
        }
        let candidates = mention_candidates(&mut conn, token);
        let Some(candidate) = tenant.in_scope(&candidates).into_iter().next() else {
            continue;
        };
        if author_user_id.as_deref() == Some(candidate.id.as_str())
            || author_email.as_deref() == Some(candidate.email.to_lowercase().as_str())
        {
            continue;
        }
        if recipients.iter().any(|r| r.id == candidate.id) {
            continue;
        }
        recipients.push((*candidate).clone());
    }

    if recipients.is_empty() {
        return;
    }

    let preview: String = body.chars().take(400).collect();

    for recipient in recipients {
        if !notification_email_enabled(&mut conn, &recipient.id, "email_mentions") {
            continue;
        }

        // #2094 — one message per recipient, in the language they picked.
        let locale = uuid::Uuid::parse_str(&recipient.id)
            .map(|user_id| botcore::i18n::locale_for_user(&mut conn, user_id))
            .unwrap_or(Default::default());
        let subject = botlib::i18n::translate(
            &locale,
            "email-mention-subject",
            &[("author", &author_name)],
        );
        let text = botlib::i18n::translate(
            &locale,
            "email-mention-body",
            &[
                ("author", &author_name),
                ("resource_type", &resource_type),
                ("resource_id", &resource_id),
                ("preview", &preview),
            ],
        );
        // #1853 — deliver to the mailbox the user configured (primary Mail
        // account or linked account); fall back to the profile address.
        let to = super::notification_email::notification_recipient(&mut conn, &recipient.id)
            .unwrap_or_else(|| recipient.email.clone());
        match sender.send(&to, &subject, &text) {
            Ok(()) => botlog::safe_info!("mention email sent to {to} (platform sender)"),
            Err(e) => warn!("mention email: send failed for {to}: {e}"),
        }
    }
}
