use std::collections::HashMap;
use botbasic_types::DbPool;

#[derive(Debug, Clone)]
pub struct ApiUrls {
    pub base_url: String,
    pub endpoints: HashMap<String, String>,
}

impl ApiUrls {
    pub const DB_TABLE: &'static str = "/api/db/table";
    pub const DB_TABLE_RECORD: &'static str = "/api/db/table/record";
    pub const DB_TABLE_COUNT: &'static str = "/api/db/table/count";
    pub const DB_TABLE_SEARCH: &'static str = "/api/db/table/search";

    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            endpoints: HashMap::new(),
        }
    }

    pub fn url(&self, key: &str) -> String {
        self.endpoints.get(key)
            .map(|e| format!("{}/{}", self.base_url, e))
            .unwrap_or_else(|| self.base_url.clone())
    }
}

pub struct ConfigManager {
    values: HashMap<String, String>,
    pool: Option<DbPool>,
}

impl ConfigManager {
    pub fn new(pool: DbPool) -> Self {
        Self { values: HashMap::new(), pool: Some(pool) }
    }

    pub fn empty() -> Self {
        Self { values: HashMap::new(), pool: None }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn get_config(&self, key: &str) -> Result<String, String> {
        self.get_config_opt(key, None)
    }

    pub fn get_config_opt(&self, key: &str, _bot_id: Option<&str>) -> Result<String, String> {
        if let Some(v) = self.values.get(key) {
            return Ok(v.clone());
        }
        if let Some(pool) = &self.pool {
            if let Ok(mut conn) = pool.get() {
                use diesel::prelude::*;
                use diesel::sql_query;
                use diesel::sql_types::Text;
                #[derive(diesel::QueryableByName)]
                struct ConfigRow {
                    #[diesel(sql_type = Text)]
                    config_value: String,
                }
                let result = sql_query(
                    "SELECT config_value FROM bot_configuration WHERE config_key = $1 LIMIT 1"
                )
                .bind::<Text, _>(key)
                .get_result::<ConfigRow>(&mut conn);
                if let Ok(row) = result {
                    return Ok(row.config_value);
                }
            }
        }
        Err(format!("Config key '{}' not found", key))
    }
}

impl std::fmt::Debug for ConfigManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigManager").field("values_count", &self.values.len()).finish()
    }
}
