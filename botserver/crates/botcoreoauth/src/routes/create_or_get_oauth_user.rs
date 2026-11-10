use super::*;

use crate::oauth_identity::{
    normalize_email, resolve_oauth_identity, EmailOwner, LocalIdentityFacts, NewOAuthAccount,
    OAuthResolution, OAUTH_NO_PASSWORD_HASH,
};

/// Result of the callback's identity step.
///
/// Only [`OAuthLoginOutcome::LoggedIn`] may mint a session; every other
/// variant means "no local identity was proven for this caller" (#2134).
pub(super) enum OAuthLoginOutcome {
    LoggedIn(Uuid),
    /// The provider email belongs to an existing local account. The caller
    /// must sign in and link the provider from Settings → Accounts.
    LinkRequired { provider: String },
}

/// Non-secret profile data copied into the `oauth_account_links` row.
struct SubjectLink<'a> {
    provider: &'a str,
    provider_id: &'a str,
    email: Option<&'a str>,
    display_name: Option<&'a str>,
    avatar_url: Option<&'a str>,
}

/// Resolves the provider identity to a local user.
///
/// The provider subject `(provider, provider_id)` is the only login identity.
/// An email returned by the provider is used at most to *propose* adopting a
/// password-less account that an earlier OAuth flow created, and only when the
/// provider asserted it as verified (#2134).
pub(super) async fn create_or_get_oauth_user(
    state: &OAuthState_,
    user_info: &OAuthUserInfo,
) -> anyhow::Result<OAuthLoginOutcome> {
    let conn = state.conn.clone();
    let provider = user_info.provider.to_string().to_lowercase();
    let provider_id = user_info.provider_id.trim().to_string();

    if provider_id.is_empty() {
        return Err(anyhow!(
            "OAuth provider {provider} returned an empty subject"
        ));
    }

    let email = normalize_email(user_info.email.as_deref());
    let email_verified = user_info.email_verified.unwrap_or(false);
    let display_name = user_info.name.clone();
    let avatar_url = user_info.avatar_url.clone();

    tokio::task::spawn_blocking(move || {
        // `transaction` comes from the `Connection` trait.
        use diesel::prelude::*;

        let mut db_conn = conn.get().map_err(|e| anyhow!("DB connection error: {e}"))?;
        let db: &mut diesel::PgConnection = &mut db_conn;

        let link = SubjectLink {
            provider: &provider,
            provider_id: &provider_id,
            email: email.as_deref(),
            display_name: display_name.as_deref(),
            avatar_url: avatar_url.as_deref(),
        };

        let facts = load_identity_facts(&mut *db, &provider, &provider_id, email.as_deref())?;

        match resolve_oauth_identity(email.as_deref(), email_verified, &facts) {
            OAuthResolution::Login(user_id) => {
                botlog::safe_debug!(
                    "OAuth login for bound subject {provider}:{provider_id} -> {user_id}"
                );
                Ok(OAuthLoginOutcome::LoggedIn(user_id))
            }
            OAuthResolution::Adopt(user_id) => {
                if bind_subject(&mut *db, user_id, &link)? {
                    botlog::safe_debug!(
                        "OAuth adopted password-less account {user_id} for verified email"
                    );
                    Ok(OAuthLoginOutcome::LoggedIn(user_id))
                } else {
                    warn!(
                        "OAuth link conflict: {provider}:{provider_id} is already bound elsewhere"
                    );
                    Ok(OAuthLoginOutcome::LinkRequired {
                        provider: provider.clone(),
                    })
                }
            }
            OAuthResolution::Create {
                email: account_email,
            } => {
                let user_id = db
                    .transaction::<Uuid, anyhow::Error, _>(|tx| {
                        let user_id = insert_oauth_user(
                            &mut *tx,
                            &provider,
                            &provider_id,
                            account_email.as_deref(),
                            display_name.as_deref(),
                        )?;
                        if !bind_subject(&mut *tx, user_id, &link)? {
                            return Err(anyhow!("oauth link conflict for new user"));
                        }
                        Ok(user_id)
                    })?;
                botlog::safe_debug!(
                    "Created OAuth user {user_id} for subject {provider}:{provider_id}"
                );
                Ok(OAuthLoginOutcome::LoggedIn(user_id))
            }
            OAuthResolution::LinkRequired { .. } => {
                warn!(
                    "OAuth login refused for {provider}:{provider_id}: email already owns a local account"
                );
                Ok(OAuthLoginOutcome::LinkRequired {
                    provider: provider.clone(),
                })
            }
        }
    })
    .await
    .map_err(|e| anyhow!("Task error: {e}"))?
}

/// One row of `oauth_account_links` reduced to the identity we need.
#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct LinkRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    user_id: Uuid,
}

/// The columns of `users` that decide whether an email may be adopted.
#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct UserRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_admin: bool,
    #[diesel(sql_type = diesel::sql_types::Text)]
    password_hash: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    has_bound_identity: bool,
}

/// Reads the bound subject and the owner of the (normalized) provider email.
fn load_identity_facts(
    db: &mut diesel::PgConnection,
    provider: &str,
    provider_id: &str,
    email: Option<&str>,
) -> anyhow::Result<LocalIdentityFacts> {
    use diesel::prelude::*;

    let linked_user_id = diesel::sql_query(
        "SELECT user_id FROM oauth_account_links \
         WHERE provider = $1 AND provider_user_id = $2 LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(provider)
    .bind::<diesel::sql_types::Text, _>(provider_id)
    .load::<LinkRow>(&mut *db)
    .map_err(|e| anyhow!("oauth link lookup failed: {e}"))?
    .into_iter()
    .next()
    .map(|row| row.user_id);

    let email_owner = match email {
        Some(address) => diesel::sql_query(
            "SELECT u.id, u.is_admin, u.password_hash, \
                    EXISTS (SELECT 1 FROM oauth_account_links l WHERE l.user_id = u.id) AS has_bound_identity \
             FROM users u WHERE lower(u.email) = $1 LIMIT 1",
        )
        .bind::<diesel::sql_types::Text, _>(address)
        .load::<UserRow>(&mut *db)
        .map_err(|e| anyhow!("user lookup by email failed: {e}"))?
        .into_iter()
        .next()
        .map(|row| EmailOwner {
            user_id: row.id,
            is_admin: row.is_admin,
            // Fail closed: only the explicit OAuth marker means "no local
            // credential". Any other value — including the empty hash written
            // by the directory provisioning path — counts as a credential.
            has_password: row.password_hash != OAUTH_NO_PASSWORD_HASH,
            has_bound_identity: row.has_bound_identity,
        }),
        None => None,
    };

    Ok(LocalIdentityFacts {
        linked_user_id,
        email_owner,
    })
}

/// Binds `(provider, provider_id)` to `user_id`.
///
/// Returns `false` when the pair or the user's slot for that provider is
/// already taken by a different identity — the caller must then refuse the
/// login instead of rebinding.
fn bind_subject(
    db: &mut diesel::PgConnection,
    user_id: Uuid,
    link: &SubjectLink<'_>,
) -> anyhow::Result<bool> {
    use diesel::prelude::*;

    diesel::sql_query(
        "INSERT INTO oauth_account_links \
         (id, user_id, provider, provider_user_id, email, display_name, avatar_url, linked_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW()) \
         ON CONFLICT DO NOTHING",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(link.provider)
    .bind::<diesel::sql_types::Text, _>(link.provider_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(link.email)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(link.display_name)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(link.avatar_url)
    .execute(&mut *db)
    .map_err(|e| anyhow!("oauth link insert failed: {e}"))?;

    let bound = diesel::sql_query(
        "SELECT user_id FROM oauth_account_links \
         WHERE provider = $1 AND provider_user_id = $2 LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(link.provider)
    .bind::<diesel::sql_types::Text, _>(link.provider_id)
    .load::<LinkRow>(&mut *db)
    .map_err(|e| anyhow!("oauth link re-read failed: {e}"))?
    .into_iter()
    .next()
    .map(|row| row.user_id);

    Ok(bound == Some(user_id))
}

/// Creates the local row for a new OAuth identity.
///
/// The account is never an administrator and never takes an unverified email
/// as its address — see [`NewOAuthAccount`].
fn insert_oauth_user(
    db: &mut diesel::PgConnection,
    provider: &str,
    provider_id: &str,
    verified_email: Option<&str>,
    display_name: Option<&str>,
) -> anyhow::Result<Uuid> {
    use crate::schema::users;
    use diesel::prelude::*;

    let account = NewOAuthAccount::for_subject(provider, provider_id, verified_email, display_name);
    let user_id = Uuid::new_v4();

    diesel::insert_into(users::dsl::users)
        .values((
            users::dsl::id.eq(user_id),
            users::dsl::username.eq(&account.username),
            users::dsl::email.eq(&account.email),
            users::dsl::password_hash.eq(OAUTH_NO_PASSWORD_HASH),
            users::dsl::is_active.eq(true),
            users::dsl::is_admin.eq(account.is_admin),
            users::dsl::created_at.eq(diesel::dsl::now),
            users::dsl::updated_at.eq(diesel::dsl::now),
        ))
        .execute(&mut *db)
        .map_err(|e| anyhow!("Failed to create user: {e}"))?;

    Ok(user_id)
}

pub(super) async fn create_user_session(state: &OAuthState_, user_id: Uuid) -> anyhow::Result<String> {
    let conn = state.conn.clone();
    let (bot_id, branch_id) = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().ok()?;
        use diesel::prelude::*;
        use crate::schema::bots;
        bots::dsl::bots
            .filter(bots::dsl::is_active.eq(true))
            .select((bots::dsl::id, bots::dsl::branch_id))
            .first::<(Uuid, Uuid)>(&mut db_conn)
            .optional()
            .ok()?
    })
    .await
    .ok()
    .flatten()
    .unwrap_or((Uuid::nil(), Uuid::nil()));

    let session_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let conn = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        use diesel::prelude::*;
        use crate::schema::user_sessions;
        diesel::insert_into(user_sessions::dsl::user_sessions)
            .values((
                user_sessions::dsl::id.eq(session_id),
                user_sessions::dsl::branch_id.eq(branch_id),
                user_sessions::dsl::bot_id.eq(bot_id),
                user_sessions::dsl::session_id.eq(session_id.to_string()),
                user_sessions::dsl::user_id.eq(user_id.to_string()),
                user_sessions::dsl::created_at.eq(now),
                user_sessions::dsl::updated_at.eq(now),
            ))
            .execute(&mut conn.get().map_err(|e| anyhow!("DB error: {}", e))?)
            .map_err(|e| anyhow!("Failed to create session: {}", e))?;
        Ok::<(), anyhow::Error>(())
    })
    .await
    .map_err(|e| anyhow!("Task error: {}", e))??;

    Ok(session_id.to_string())
}
pub(super) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
