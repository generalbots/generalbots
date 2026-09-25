//! #1444 M2 — persistent skills storage.
//!
//! Custom `skill/create` rows and marketplace installs used to live only in
//! an in-memory `Vec` inside [`crate::skills::SkillStore`] and vanished on
//! restart. This module mirrors the `vibe_canvases`/`vibe_issues` write-
//! through pattern (`catalog_persistence.rs`) for the `vibe_skills` table
//! (declared in `types.rs::VIBE_SCHEMA`).

use crate::skills::VibeSkill;
use crate::types::DbPool;
use diesel::prelude::*;

/// Upserts one skill row (dedupe by name via the UNIQUE constraint).
pub fn save_skill(pool: &DbPool, skill: &VibeSkill) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("skill persist: pool get: {e}"))?;
    let triggers = serde_json::to_value(&skill.triggers)
        .map_err(|e| format!("skill persist: triggers serialize: {e}"))?;
    diesel::sql_query(
        "INSERT INTO vibe_skills \
         (skill_id, name, description, content, triggers, enabled, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) \
         ON CONFLICT (name) DO UPDATE SET \
           description = EXCLUDED.description, content = EXCLUDED.content, \
           triggers = EXCLUDED.triggers, enabled = EXCLUDED.enabled, \
           updated_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(skill.skill_id)
    .bind::<diesel::sql_types::Text, _>(&skill.name)
    .bind::<diesel::sql_types::Text, _>(&skill.description)
    .bind::<diesel::sql_types::Text, _>(&skill.content)
    .bind::<diesel::sql_types::Jsonb, _>(&triggers)
    .bind::<diesel::sql_types::Bool, _>(skill.enabled)
    .execute(&mut conn)
    .map_err(|e| format!("skill persist: {e}"))?;
    Ok(())
}

/// Deletes one skill row by name. Returns false when nothing matched.
pub fn delete_skill(pool: &DbPool, name: &str) -> Result<bool, String> {
    let mut conn = pool.get().map_err(|e| format!("skill delete: pool get: {e}"))?;
    let rows = diesel::sql_query("DELETE FROM vibe_skills WHERE name = $1")
        .bind::<diesel::sql_types::Text, _>(name)
        .execute(&mut conn)
        .map_err(|e| format!("skill delete: {e}"))?;
    Ok(rows > 0)
}

/// Loads every persisted skill (name-ordered so boot hydration is stable).
pub fn load_skills(pool: &DbPool) -> Result<Vec<VibeSkill>, String> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        skill_id: uuid::Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        description: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        content: String,
        #[diesel(sql_type = diesel::sql_types::Jsonb)]
        triggers: serde_json::Value,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        enabled: bool,
    }
    let mut conn = pool.get().map_err(|e| format!("skill load: pool get: {e}"))?;
    let rows = diesel::sql_query(
        "SELECT skill_id, name, description, content, triggers, enabled \
         FROM vibe_skills ORDER BY name",
    )
    .load::<Row>(&mut conn)
    .map_err(|e| format!("skill load: {e}"))?;
    Ok(rows
        .into_iter()
        .map(|r| VibeSkill {
            skill_id: r.skill_id,
            name: r.name,
            description: r.description,
            content: r.content,
            triggers: serde_json::from_value(r.triggers).unwrap_or_default(),
            enabled: r.enabled,
        })
        .collect())
}
