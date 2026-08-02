use std::collections::HashMap;

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
