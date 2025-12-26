use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::debug;
use rhai::{Dynamic, Engine, Map};
use std::sync::Arc;
use uuid::Uuid;

pub fn get_posts_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    engine
        .register_custom_syntax(
            ["GET", "$ident$", "POSTS"],
            false,
            move |context, inputs| {
                let platform = context.eval_expression_tree(&inputs[0])?.to_string();
                let platform = platform.to_lowercase();

                let mut conn = state_clone
                    .conn
                    .get()
                    .map_err(|e| format!("DB error: {}", e))?;

                let posts = get_social_posts(&mut conn, user_clone.bot_id, &platform)
                    .map_err(|e| format!("Failed to get posts: {}", e))?;

                let posts_array: Vec<Dynamic> = posts
                    .iter()
                    .map(|p| {
                        let mut map = Map::new();
                        map.insert("id".into(), Dynamic::from(p.id.clone()));
                        map.insert("platform".into(), Dynamic::from(p.platform.clone()));
                        map.insert("content".into(), Dynamic::from(p.content.clone()));
                        map.insert("status".into(), Dynamic::from(p.status.clone()));
                        if let Some(ref media) = p.media_url {
                            map.insert("media_url".into(), Dynamic::from(media.clone()));
                        }
                        Dynamic::from(map)
                    })
                    .collect();

                Ok(Dynamic::from(posts_array))
            },
        )
        .unwrap();

    debug!("Registered GET POSTS keyword");
}

#[derive(Debug, Clone)]
pub struct SocialPost {
    pub id: String,
    pub platform: String,
    pub content: String,
    pub media_url: Option<String>,
    pub status: String,
}

fn get_social_posts(
    conn: &mut diesel::PgConnection,
    bot_id: Uuid,
    platform: &str,
) -> Result<Vec<SocialPost>, String> {
    #[derive(QueryableByName)]
    struct PostRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        id: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        platform: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        content: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        media_url: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
    }

    let query = diesel::sql_query(
        "SELECT id, platform, content, media_url, status FROM social_posts
         WHERE bot_id = $1 AND platform = $2 ORDER BY created_at DESC LIMIT 100",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(platform);

    let rows: Vec<PostRow> = query
        .load(conn)
        .map_err(|e| format!("Failed to get posts: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|row| SocialPost {
            id: row.id,
            platform: row.platform,
            content: row.content,
            media_url: row.media_url,
            status: row.status,
        })
        .collect())
}
