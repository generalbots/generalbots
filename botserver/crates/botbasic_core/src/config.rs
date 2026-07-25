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
        if let Ok(value) = read_vault_value(key) {
            return Ok(value);
        }
        Err(format!("Config key '{}' not found", key))
    }
}

impl std::fmt::Debug for ConfigManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigManager").field("values_count", &self.values.len()).finish()
    }
}

fn read_vault_value(key: &str) -> Result<String, String> {
    use botcoresecrets::manager::SecretsManager;
    let sm = SecretsManager::get_clone().map_err(|e| format!("SecretsManager unavailable: {}", e))?;
    if !sm.is_enabled() {
        return Err("Vault not enabled".to_string());
    }
    let path = format!("gbo/{}/{}/{}", uuid::Uuid::nil(), uuid::Uuid::nil(), uuid::Uuid::nil());
    let sm_clone = sm.clone();
    let key_owned = key.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = if let Ok(rt) = rt {
            rt.block_on(async move { sm_clone.get_value(&path, &key_owned).await.ok() })
        } else {
            None
        };
        let _ = tx.send(result);
    });
    rx.recv()
        .ok()
        .flatten()
        .ok_or_else(|| format!("Config key '{}' not found in Vault", key))
}
