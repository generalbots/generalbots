











use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{error, info, trace, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPISpec {
    pub openapi: Option<String>,
    pub swagger: Option<String>,
    pub info: OpenAPIInfo,
    pub servers: Option<Vec<OpenAPIServer>>,
    pub paths: HashMap<String, HashMap<String, OpenAPIOperation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPIInfo {
    pub title: String,
    pub description: Option<String>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPIServer {
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPIOperation {
    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub parameters: Option<Vec<OpenAPIParameter>>,
    #[serde(rename = "requestBody")]
    pub request_body: Option<OpenAPIRequestBody>,
    pub responses: Option<HashMap<String, OpenAPIResponse>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPIParameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub schema: Option<OpenAPISchema>,
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPISchema {
    #[serde(rename = "type")]
    pub schema_type: Option<String>,
    pub format: Option<String>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    pub default: Option<serde_json::Value>,
    pub example: Option<serde_json::Value>,
    pub properties: Option<HashMap<String, OpenAPISchema>>,
    pub required: Option<Vec<String>>,
    pub items: Option<Box<OpenAPISchema>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPIRequestBody {
    pub description: Option<String>,
    pub required: Option<bool>,
    pub content: Option<HashMap<String, OpenAPIMediaType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPIMediaType {
    pub schema: Option<OpenAPISchema>,
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPIResponse {
    pub description: Option<String>,
    pub content: Option<HashMap<String, OpenAPIMediaType>>,
}


#[derive(Debug, Clone)]
pub struct GeneratedEndpoint {
    pub operation_id: String,
    pub method: String,
    pub path: String,
    pub description: String,
    pub parameters: Vec<EndpointParameter>,
    pub base_url: String,
}

#[derive(Debug, Clone)]
pub struct EndpointParameter {
    pub name: String,
    pub param_type: String,
    pub location: String,
    pub description: String,
    pub required: bool,
    pub example: Option<String>,
}


pub struct ApiToolGenerator {
    state: Arc<AppState>,
    bot_id: Uuid,
    work_path: String,
}

impl std::fmt::Debug for ApiToolGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiToolGenerator")
            .field("bot_id", &self.bot_id)
            .field("work_path", &self.work_path)
            .finish_non_exhaustive()
    }
}

impl ApiToolGenerator {
    pub fn new(state: Arc<AppState>, bot_id: Uuid, work_path: &str) -> Self {
        Self {
            state,
            bot_id,
            work_path: work_path.to_string(),
        }
    }



    pub async fn sync_all_api_tools(&self) -> Result<SyncResult, String> {
        let api_configs = self.get_api_configs().await?;
        let mut result = SyncResult::default();


        let api_configs_for_cleanup = api_configs.clone();

        for (api_name, spec_url) in api_configs {
            info!("Processing API: {} from {}", api_name, spec_url);

            match self.sync_api_tools(&api_name, &spec_url).await {
                Ok(count) => {
                    result.apis_synced += 1;
                    result.tools_generated += count;
                    info!("Generated {} tools for API: {}", count, api_name);
                }
                Err(e) => {
                    result.errors.push(format!("{}: {}", api_name, e));
                    error!("Failed to sync API {}: {}", api_name, e);
                }
            }
        }


        let removed = self.cleanup_removed_apis(&api_configs_for_cleanup).await?;
        result.tools_removed = removed;

        Ok(result)
    }


    pub async fn sync_api_tools(&self, api_name: &str, spec_url: &str) -> Result<usize, String> {

        let spec_content = self.fetch_spec(spec_url).await?;
        let spec_hash = self.calculate_hash(&spec_content);


        if !self.has_spec_changed(api_name, &spec_hash).await? {
            trace!("API spec unchanged for {}, skipping", api_name);
            return Ok(0);
        }


        let spec: OpenAPISpec = serde_json::from_str(&spec_content)
            .map_err(|e| format!("Failed to parse OpenAPI spec: {}", e))?;


        let endpoints = self.extract_endpoints(&spec)?;


        let api_folder = format!(
            "{}/{}.gbai/.gbdialog/{}",
            self.work_path, self.bot_id, api_name
        );
        std::fs::create_dir_all(&api_folder)
            .map_err(|e| format!("Failed to create API folder: {}", e))?;


        let mut generated_count = 0;
        for endpoint in &endpoints {
            let bas_content = self.generate_bas_file(&api_name, endpoint)?;
            let file_path = format!("{}/{}.bas", api_folder, endpoint.operation_id);

            std::fs::write(&file_path, &bas_content)
                .map_err(|e| format!("Failed to write .bas file: {}", e))?;

            generated_count += 1;
        }


        self.update_api_record(api_name, spec_url, &spec_hash, generated_count)
            .await?;

        Ok(generated_count)
    }


    async fn fetch_spec(&self, spec_url: &str) -> Result<String, String> {

        if spec_url.starts_with("./") || spec_url.starts_with("/") {
            return std::fs::read_to_string(spec_url)
                .map_err(|e| format!("Failed to read local spec file: {}", e));
        }


        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
            .get(spec_url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch spec: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Failed to fetch spec: HTTP {}", response.status()));
        }

        response
            .text()
            .await
            .map_err(|e| format!("Failed to read spec body: {}", e))
    }


    fn extract_endpoints(&self, spec: &OpenAPISpec) -> Result<Vec<GeneratedEndpoint>, String> {
        let mut endpoints = Vec::new();


        let base_url = spec
            .servers
            .as_ref()
            .and_then(|s| s.first())
            .map(|s| s.url.clone())
            .unwrap_or_else(|| "http://localhost".to_string());

        for (path, methods) in &spec.paths {
            for (method, operation) in methods {

                let operation_id = match &operation.operation_id {
                    Some(id) => self.sanitize_operation_id(id),
                    None => self.generate_operation_id(&method, &path),
                };


                let description = operation
                    .summary
                    .clone()
                    .or_else(|| operation.description.clone())
                    .unwrap_or_else(|| format!("{} {}", method.to_uppercase(), path));


                let mut parameters = Vec::new();


                if let Some(params) = &operation.parameters {
                    for param in params {
                        parameters.push(self.convert_parameter(param));
                    }
                }


                if let Some(body) = &operation.request_body {
                    if let Some(content) = &body.content {
                        if let Some(json_content) = content.get("application/json") {
                            if let Some(schema) = &json_content.schema {
                                let body_params = self.extract_body_parameters(
                                    schema,
                                    body.required.unwrap_or(false),
                                );
                                parameters.extend(body_params);
                            }
                        }
                    }
                }

                endpoints.push(GeneratedEndpoint {
                    operation_id,
                    method: method.to_uppercase(),
                    path: path.clone(),
                    description,
                    parameters,
                    base_url: base_url.clone(),
                });
            }
        }

        Ok(endpoints)
    }


    fn convert_parameter(&self, param: &OpenAPIParameter) -> EndpointParameter {
        let param_type = param
            .schema
            .as_ref()
            .and_then(|s| s.schema_type.clone())
            .unwrap_or_else(|| "string".to_string());

        let example = param
            .example
            .as_ref()
            .or_else(|| param.schema.as_ref().and_then(|s| s.example.as_ref()))
            .map(|v| self.value_to_string(v));

        EndpointParameter {
            name: param.name.clone(),
            param_type: self.map_openapi_type(&param_type),
            location: param.location.clone(),
            description: param.description.clone().unwrap_or_default(),
            required: param.required.unwrap_or(false),
            example,
        }
    }


    fn extract_body_parameters(
        &self,
        schema: &OpenAPISchema,
        required: bool,
    ) -> Vec<EndpointParameter> {
        let mut params = Vec::new();

        if let Some(properties) = &schema.properties {
            let required_fields = schema.required.clone().unwrap_or_default();

            for (name, prop_schema) in properties {
                let param_type = prop_schema
                    .schema_type
                    .clone()
                    .unwrap_or_else(|| "string".to_string());

                let example = prop_schema
                    .example
                    .as_ref()
                    .map(|v| self.value_to_string(v));

                params.push(EndpointParameter {
                    name: name.clone(),
                    param_type: self.map_openapi_type(&param_type),
                    location: "body".to_string(),
                    description: String::new(),
                    required: required && required_fields.contains(name),
                    example,
                });
            }
        }

        params
    }


    fn generate_bas_file(
        &self,
        api_name: &str,
        endpoint: &GeneratedEndpoint,
    ) -> Result<String, String> {
        let mut bas = String::new();


        bas.push_str(&format!("' Auto-generated tool for {} API\n", api_name));
        bas.push_str(&format!(
            "' Endpoint: {} {}\n",
            endpoint.method, endpoint.path
        ));
        bas.push_str(&format!(
            "' Generated at: {}\n\n",
            chrono::Utc::now().to_rfc3339()
        ));


        for param in &endpoint.parameters {
            let example = param.example.as_deref().unwrap_or("");
            let required_marker = if param.required { "" } else { " ' optional" };

            bas.push_str(&format!(
                "PARAM {} AS {} LIKE \"{}\" DESCRIPTION \"{}\"{}\n",
                self.sanitize_param_name(&param.name),
                param.param_type,
                example,
                self.escape_description(&param.description),
                required_marker
            ));
        }


        bas.push_str(&format!(
            "\nDESCRIPTION \"{}\"\n\n",
            self.escape_description(&endpoint.description)
        ));


        let mut url = format!("{}{}", endpoint.base_url, endpoint.path);
        let path_params: Vec<&EndpointParameter> = endpoint
            .parameters
            .iter()
            .filter(|p| p.location == "path")
            .collect();

        for param in &path_params {
            url = url.replace(
                &format!("{{{}}}", param.name),
                &format!("\" + {} + \"", self.sanitize_param_name(&param.name)),
            );
        }


        let query_params: Vec<&EndpointParameter> = endpoint
            .parameters
            .iter()
            .filter(|p| p.location == "query")
            .collect();

        if !query_params.is_empty() {
            bas.push_str("' Build query string\n");
            bas.push_str("query_params = \"\"\n");
            for (i, param) in query_params.iter().enumerate() {
                let name = self.sanitize_param_name(&param.name);
                let sep = if i == 0 { "?" } else { "&" };
                bas.push_str(&format!(
                    "IF NOT ISEMPTY({}) THEN query_params = query_params + \"{}{}=\" + {}\n",
                    name, sep, param.name, name
                ));
            }
            bas.push('\n');
        }


        let body_params: Vec<&EndpointParameter> = endpoint
            .parameters
            .iter()
            .filter(|p| p.location == "body")
            .collect();

        if !body_params.is_empty() {
            bas.push_str("' Build request body\n");
            bas.push_str("body = {}\n");
            for param in &body_params {
                let name = self.sanitize_param_name(&param.name);
                bas.push_str(&format!(
                    "IF NOT ISEMPTY({}) THEN body.{} = {}\n",
                    name, param.name, name
                ));
            }
            bas.push('\n');
        }


        bas.push_str("' Make API request\n");
        let full_url = if query_params.is_empty() {
            format!("\"{}\"", url)
        } else {
            format!("\"{}\" + query_params", url)
        };

        if body_params.is_empty() {
            bas.push_str(&format!("result = {} HTTP {}\n", endpoint.method, full_url));
        } else {
            bas.push_str(&format!(
                "result = {} HTTP {} WITH body\n",
                endpoint.method, full_url
            ));
        }


        bas.push_str("\n' Return result\n");
        bas.push_str("RETURN result\n");

        Ok(bas)
    }


    async fn get_api_configs(&self) -> Result<Vec<(String, String)>, String> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

        #[derive(QueryableByName)]
        struct ConfigRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            config_key: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            config_value: String,
        }

        let configs: Vec<ConfigRow> = diesel::sql_query(
            "SELECT config_key, config_value FROM bot_configuration \
             WHERE bot_id = $1 AND config_key LIKE '%-api-server'",
        )
        .bind::<diesel::sql_types::Uuid, _>(self.bot_id)
        .load(&mut conn)
        .map_err(|e| format!("Failed to query API configs: {}", e))?;

        let result: Vec<(String, String)> = configs
            .into_iter()
            .map(|c| {
                let api_name = c.config_key.trim_end_matches("-api-server").to_string();
                (api_name, c.config_value)
            })
            .collect();

        Ok(result)
    }


    async fn has_spec_changed(&self, api_name: &str, current_hash: &str) -> Result<bool, String> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

        #[derive(QueryableByName)]
        struct HashRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            spec_hash: String,
        }

        let result: Option<HashRow> = diesel::sql_query(
            "SELECT spec_hash FROM generated_api_tools \
             WHERE bot_id = $1 AND api_name = $2 LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(self.bot_id)
        .bind::<diesel::sql_types::Text, _>(api_name)
        .get_result(&mut conn)
        .optional()
        .map_err(|e| format!("Failed to check spec hash: {}", e))?;

        match result {
            Some(row) => Ok(row.spec_hash != current_hash),
            None => Ok(true),
        }
    }


    async fn update_api_record(
        &self,
        api_name: &str,
        spec_url: &str,
        spec_hash: &str,
        tool_count: usize,
    ) -> Result<(), String> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

        let now = chrono::Utc::now();
        let new_id = Uuid::new_v4();

        diesel::sql_query(
            "INSERT INTO generated_api_tools \
             (id, bot_id, api_name, spec_url, spec_hash, tool_count, last_synced_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $7) \
             ON CONFLICT (bot_id, api_name) DO UPDATE SET \
             spec_url = EXCLUDED.spec_url, \
             spec_hash = EXCLUDED.spec_hash, \
             tool_count = EXCLUDED.tool_count, \
             last_synced_at = EXCLUDED.last_synced_at",
        )
        .bind::<diesel::sql_types::Uuid, _>(new_id)
        .bind::<diesel::sql_types::Uuid, _>(self.bot_id)
        .bind::<diesel::sql_types::Text, _>(api_name)
        .bind::<diesel::sql_types::Text, _>(spec_url)
        .bind::<diesel::sql_types::Text, _>(spec_hash)
        .bind::<diesel::sql_types::Integer, _>(tool_count as i32)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to update API record: {}", e))?;

        Ok(())
    }


    async fn cleanup_removed_apis(
        &self,
        current_apis: &[(String, String)],
    ) -> Result<usize, String> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

        #[derive(QueryableByName)]
        struct ApiRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            api_name: String,
        }

        let existing: Vec<ApiRow> =
            diesel::sql_query("SELECT api_name FROM generated_api_tools WHERE bot_id = $1")
                .bind::<diesel::sql_types::Uuid, _>(self.bot_id)
                .load(&mut conn)
                .map_err(|e| format!("Failed to query existing APIs: {}", e))?;

        let current_names: Vec<&str> = current_apis.iter().map(|(n, _)| n.as_str()).collect();
        let mut removed_count = 0;

        for api in existing {
            if !current_names.contains(&api.api_name.as_str()) {

                diesel::sql_query(
                    "DELETE FROM generated_api_tools WHERE bot_id = $1 AND api_name = $2",
                )
                .bind::<diesel::sql_types::Uuid, _>(self.bot_id)
                .bind::<diesel::sql_types::Text, _>(&api.api_name)
                .execute(&mut conn)
                .ok();


                let api_folder = format!(
                    "{}/{}.gbai/.gbdialog/{}",
                    self.work_path, self.bot_id, api.api_name
                );
                if let Err(e) = std::fs::remove_dir_all(&api_folder) {
                    warn!("Failed to remove API folder {}: {}", api_folder, e);
                } else {
                    info!("Removed API folder: {}", api_folder);
                    removed_count += 1;
                }
            }
        }

        Ok(removed_count)
    }



    fn calculate_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn sanitize_operation_id(&self, id: &str) -> String {
        id.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .to_lowercase()
    }

    fn generate_operation_id(&self, method: &str, path: &str) -> String {
        let path_part = path
            .trim_matches('/')
            .replace('/', "_")
            .replace('{', "")
            .replace('}', "");
        format!("{}_{}", method.to_lowercase(), path_part)
    }

    fn sanitize_param_name(&self, name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .to_lowercase()
    }

    fn map_openapi_type(&self, openapi_type: &str) -> String {
        match openapi_type.to_lowercase().as_str() {
            "integer" | "number" => "number".to_string(),
            "boolean" => "boolean".to_string(),
            "array" => "array".to_string(),
            "object" => "object".to_string(),
            _ => "string".to_string(),
        }
    }

    fn value_to_string(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => serde_json::to_string(value).unwrap_or_default(),
        }
    }

    fn escape_description(&self, desc: &str) -> String {
        desc.replace('"', "'").replace('\n', " ").trim().to_string()
    }
}


#[derive(Debug, Default)]
pub struct SyncResult {
    pub apis_synced: usize,
    pub tools_generated: usize,
    pub tools_removed: usize,
    pub errors: Vec<String>,
}

impl SyncResult {
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}
