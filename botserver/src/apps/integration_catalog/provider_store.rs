//! Operator-defined integration providers (#2346).
//!
//! Seeded providers remain compile-time Rust data; the rows here are the
//! additions and edits made through the provider editor. A migration creates
//! the table; every query is parameterised through `sql_query` so no user
//! input becomes SQL.

use botcore::shared::utils::pool;
use diesel::sql_types::{Jsonb, Text};
use diesel::prelude::*;
use serde_json::{json, Value};

#[derive(Debug, QueryableByName)]
struct ProviderRow {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Text)]
    category: String,
    #[diesel(sql_type = Text)]
    module: String,
    #[diesel(sql_type = Text)]
    status: String,
    #[diesel(sql_type = Text)]
    source: String,
    #[diesel(sql_type = Jsonb)]
    actions: Value,
    #[diesel(sql_type = Jsonb)]
    setup_steps: Value,
    #[diesel(sql_type = Text)]
    credentials_url: String,
    #[diesel(sql_type = Text)]
    official_docs: String,
    #[diesel(sql_type = Text)]
    guidance: String,
}

impl ProviderRow {
    fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "category": self.category,
            "module": self.module,
            "status": self.status,
            "source": self.source,
            "actions": self.actions,
            "setup_steps": self.setup_steps,
            "credentials_url": self.credentials_url,
            "official_docs": self.official_docs,
            "guidance": self.guidance,
            "provider_id": self.id,
        })
    }
}

const SELECT: &str = "SELECT id, name, category, module, status, source, actions, \
    setup_steps, credentials_url, official_docs, guidance FROM integration_providers";

pub fn list() -> Result<Vec<Value>, String> {
    let pool = pool().map_err(|(status, body)| format!("{status}: {body:?}"))?;
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let rows = diesel::sql_query(format!("{SELECT} ORDER BY id"))
        .load::<ProviderRow>(&mut conn)
        .map_err(|e| format!("DB error: {e}"))?;
    Ok(rows.iter().map(ProviderRow::to_json).collect())
}

pub fn get(id: &str) -> Result<Option<Value>, String> {
    let pool = pool().map_err(|(status, body)| format!("{status}: {body:?}"))?;
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let row = diesel::sql_query(format!("{SELECT} WHERE id = $1"))
        .bind::<Text, _>(id)
        .get_result::<ProviderRow>(&mut conn)
        .optional()
        .map_err(|e| format!("DB error: {e}"))?;
    Ok(row.as_ref().map(ProviderRow::to_json))
}

fn value_or_empty(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

fn json_array(v: Option<&Value>) -> Value {
    match v {
        Some(Value::Array(a)) => Value::Array(a.clone()),
        Some(Value::Object(o)) if o.get("actions").is_some() => o["actions"].clone(),
        _ => json!([]),
    }
}

pub fn upsert(payload: &Value, id: Option<&str>) -> Result<Value, String> {
    let id = id
        .map(str::to_string)
        .unwrap_or_else(|| value_or_empty(payload.get("id")));
    if id.is_empty() {
        return Err("provider id is required".to_string());
    }
    let name = value_or_empty(payload.get("name"));
    if name.is_empty() {
        return Err("provider name is required".to_string());
    }
    let pool = pool().map_err(|(status, body)| format!("{status}: {body:?}"))?;
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    diesel::sql_query(
        "INSERT INTO integration_providers \
         (id, name, category, module, status, source, actions, setup_steps, \
          credentials_url, official_docs, guidance) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) \
         ON CONFLICT (id) DO UPDATE SET \
          name=EXCLUDED.name, category=EXCLUDED.category, module=EXCLUDED.module, \
          status=EXCLUDED.status, source=EXCLUDED.source, actions=EXCLUDED.actions, \
          setup_steps=EXCLUDED.setup_steps, credentials_url=EXCLUDED.credentials_url, \
          official_docs=EXCLUDED.official_docs, guidance=EXCLUDED.guidance, \
          updated_at=NOW()",
    )
    .bind::<Text, _>(&id)
    .bind::<Text, _>(&name)
    .bind::<Text, _>(value_or_empty(payload.get("category")))
    .bind::<Text, _>(value_or_empty(payload.get("module")))
    .bind::<Text, _>(value_or_empty(payload.get("status")))
    .bind::<Text, _>(value_or_empty(payload.get("source")))
    .bind::<Jsonb, _>(json_array(payload.get("actions")))
    .bind::<Jsonb, _>(json_array(payload.get("setup_steps")))
    .bind::<Text, _>(value_or_empty(payload.get("credentials_url")))
    .bind::<Text, _>(value_or_empty(payload.get("official_docs")))
    .bind::<Text, _>(value_or_empty(payload.get("guidance")))
    .execute(&mut conn)
    .map_err(|e| format!("DB error: {e}"))?;
    get(&id)?.ok_or_else(|| "provider was not persisted".to_string())
}

pub fn delete(id: &str) -> Result<bool, String> {
    let pool = pool().map_err(|(status, body)| format!("{status}: {body:?}"))?;
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let count = diesel::sql_query("DELETE FROM integration_providers WHERE id = $1")
        .bind::<Text, _>(id)
        .execute(&mut conn)
        .map_err(|e| format!("DB error: {e}"))?;
    Ok(count > 0)
}
