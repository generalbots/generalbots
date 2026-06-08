use base64::Engine;
use crate::connector_types::*;
use chrono::Utc;
use diesel::sql_query;
use diesel::prelude::*;
use uuid::Uuid;

pub struct ConnectorEngine;

impl ConnectorEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn create_connector(
        conn: &mut PgConnection,
        req: CreateConnectorRequest,
    ) -> Result<ConnectorConfig, String> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        sql_query(
            "INSERT INTO connectors (id, bot_id, name, connector_type, description, auth_config, schedule, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Uuid, _>(req.bot_id)
        .bind::<diesel::sql_types::Text, _>(&req.name)
        .bind::<diesel::sql_types::Text, _>(&req.connector_type)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&req.description)
        .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(&req.auth_config).unwrap_or_default())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&req.schedule)
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(conn)
        .map_err(|e| format!("Failed to create connector: {e}"))?;

        let endpoints = req.endpoints.unwrap_or_default();
        for ep in &endpoints {
            Self::insert_endpoint(conn, id, ep, now)?;
        }

        Self::get_connector(conn, id)
    }

    pub fn get_connector(
        conn: &mut PgConnection,
        id: Uuid,
    ) -> Result<ConnectorConfig, String> {
        let row: ConnectorRow  = sql_query("SELECT * FROM connectors WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .get_result(conn)
            .map_err(|e| format!("Connector not found: {e}"))?;

        Self::row_to_config(conn, row)
    }

    pub fn list_connectors(
        conn: &mut PgConnection,
        bot_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ConnectorConfig>, String> {
        let rows: Vec<ConnectorRow> = sql_query(
            "SELECT * FROM connectors WHERE bot_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::BigInt, _>(limit)
        .bind::<diesel::sql_types::BigInt, _>(offset)
        .get_results(conn)
        .map_err(|e| format!("Failed to list connectors: {e}"))?;

        let mut configs = Vec::new();
        for row in rows {
            configs.push(Self::row_to_config(conn, row)?);
        }
        Ok(configs)
    }

    pub fn update_connector(
        conn: &mut PgConnection,
        id: Uuid,
        req: UpdateConnectorRequest,
    ) -> Result<ConnectorConfig, String> {
        if let Some(ref name) = req.name {
            sql_query("UPDATE connectors SET name = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Text, _>(name)
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(conn)
                .map_err(|e| format!("Failed to update name: {e}"))?;
        }
        if let Some(ref desc) = req.description {
            sql_query("UPDATE connectors SET description = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Text, _>(desc)
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(conn)
                .map_err(|e| format!("Failed to update description: {e}"))?;
        }
        if let Some(ref auth) = req.auth_config {
            sql_query("UPDATE connectors SET auth_config = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(auth).unwrap_or_default())
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(conn)
                .map_err(|e| format!("Failed to update auth: {e}"))?;
        }
        if let Some(ref schedule) = req.schedule {
            sql_query("UPDATE connectors SET schedule = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Text, _>(schedule)
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(conn)
                .map_err(|e| format!("Failed to update schedule: {e}"))?;
        }
        if let Some(active) = req.is_active {
            sql_query("UPDATE connectors SET is_active = $1 WHERE id = $2")
                .bind::<diesel::sql_types::Bool, _>(active)
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(conn)
                .map_err(|e| format!("Failed to update active status: {e}"))?;
        }

        sql_query("UPDATE connectors SET updated_at = NOW() WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(conn)
            .map_err(|e| format!("Failed to update timestamp: {e}"))?;

        if let Some(ref endpoints) = req.endpoints {
            sql_query("DELETE FROM connector_endpoints WHERE connector_id = $1")
                .bind::<diesel::sql_types::Uuid, _>(id)
                .execute(conn)
                .map_err(|e| format!("Failed to clear endpoints: {e}"))?;

            let now = Utc::now();
            for ep in endpoints {
                Self::insert_endpoint(conn, id, ep, now)?;
            }
        }

        Self::get_connector(conn, id)
    }

    pub fn delete_connector(
        conn: &mut PgConnection,
        id: Uuid,
    ) -> Result<(), String> {
        sql_query("DELETE FROM connector_sync_logs WHERE connector_id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(conn)
            .ok();
        sql_query("DELETE FROM connector_endpoints WHERE connector_id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(conn)
            .map_err(|e| format!("Failed to delete endpoints: {e}"))?;
        sql_query("DELETE FROM connectors WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(conn)
            .map_err(|e| format!("Failed to delete connector: {e}"))?;
        Ok(())
    }

    pub fn test_connection(
        conn: &mut PgConnection,
        id: Uuid,
    ) -> Result<String, String> {
        let config = Self::get_connector(conn, id)?;
        let client = build_blocking_client(&config.auth_config)
            .map_err(|e| format!("Failed to build client: {e}"))?;

        let test_url = match config.connector_type {
            ConnectorType::Salesforce => Some("/services/data/v52.0/".to_string()),
            ConnectorType::SharePoint => Some("/v1.0/sites?search=".to_string()),
            _ => config.endpoints.first().map(|ep| ep.url.clone()),
        };

        let base = config.auth_config.base_url
            .as_ref()
            .ok_or_else(|| "Base URL not configured".to_string())?;
        let url = format!("{}{}", base.trim_end_matches('/'), test_url.as_deref().unwrap_or("/"));

        let response = client.get(&url).send()
            .map_err(|e| format!("Connection failed: {e}"))?;
        let status = response.status();
        if status.is_success() || status.as_u16() == 401 {
            Ok("Connection successful".to_string())
        } else {
            Err(format!("Connection returned status {status}"))
        }
    }

    pub fn discover_schema(
        conn: &mut PgConnection,
        id: Uuid,
        endpoint_name: Option<String>,
    ) -> Result<Vec<DiscoveredSchema>, String> {
        let config = Self::get_connector(conn, id)?;
        let client = build_blocking_client(&config.auth_config)
            .map_err(|e| format!("Failed to build client: {e}"))?;

        let mut schemas = Vec::new();
        let base_url = config.auth_config.base_url.as_ref()
            .ok_or_else(|| "Base URL not configured".to_string())?;

        for ep in &config.endpoints {
            if let Some(ref filter) = endpoint_name {
                if &ep.name != filter { continue; }
            }

            let url = format!("{}{}", base_url.trim_end_matches('/'), &ep.url);
            let schema = match config.connector_type {
                ConnectorType::Salesforce => {
                    let describe_url = url.replace("/query?", "/sobjects/")
                        .split('?').next().unwrap_or(&url).to_string() + "/describe";
                    let resp = client.get(&describe_url).send()
                        .map_err(|e| format!("Schema discovery failed: {e}"))?;
                    let body: serde_json::Value = resp.json().unwrap_or(serde_json::Value::Null);
                    if let Some(fields) = body["fields"].as_array() {
                        DiscoveredSchema {
                            object_name: ep.name.clone(),
                            fields: fields.iter().map(|f| SchemaField {
                                name: f["name"].as_str().unwrap_or("").to_string(),
                                field_type: f["type"].as_str().unwrap_or("string").to_string(),
                                required: f["nillable"].as_bool().map(|n| !n).unwrap_or(false),
                                max_length: f["length"].as_i64().map(|l| l as i32),
                                values: f["picklistValues"].as_array().map(|v| {
                                    v.iter().filter_map(|e| e["value"].as_str().map(String::from)).collect()
                                }),
                            }).collect(),
                            total_count: None,
                        }
                    } else { continue; }
                }
                _ => {
                    let resp = client.get(&url).send()
                        .map_err(|e| format!("Schema discovery failed: {e}"))?;
                    let body: serde_json::Value = resp.json().unwrap_or(serde_json::Value::Null);
                    let first_item = body.as_array().and_then(|a| a.first())
                        .or_else(|| body["data"].as_array().and_then(|a| a.first()))
                        .or_else(|| body["records"].as_array().and_then(|a| a.first()));

                    if let Some(item) = first_item {
                        DiscoveredSchema {
                            object_name: ep.name.clone(),
                            fields: item.as_object().map(|obj| {
                                obj.iter().map(|(k, v)| SchemaField {
                                    name: k.clone(),
                                    field_type: match v {
                                        serde_json::Value::String(_) => "string",
                                        serde_json::Value::Number(_) => "number",
                                        serde_json::Value::Bool(_) => "boolean",
                                        serde_json::Value::Array(_) => "array",
                                        serde_json::Value::Object(_) => "object",
                                        _ => "string",
                                    }.to_string(),
                                    required: false,
                                    max_length: None,
                                    values: None,
                                }).collect()
                            }).unwrap_or_default(),
                            total_count: body["total_size"].as_i64()
                                .or_else(|| body["total"].as_i64())
                                .or_else(|| body["count"].as_i64()),
                        }
                    } else { continue; }
                }
            };
            schemas.push(schema);
        }

        Ok(schemas)
    }

    pub fn sync_connector(
        conn: &mut PgConnection,
        id: Uuid,
    ) -> Result<SyncResult, String> {
        let config = Self::get_connector(conn, id)?;
        let client = build_blocking_client(&config.auth_config)
            .map_err(|e| format!("Failed to build client: {e}"))?;
        let started_at = Utc::now();
        let mut total_synced = 0i64;
        let mut total_failed = 0i64;
        let mut endpoint_results = Vec::new();

        sql_query("UPDATE connectors SET last_sync_status = 'running' WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(conn).ok();

        let base_url = config.auth_config.base_url.as_ref()
            .ok_or_else(|| "Base URL not configured".to_string())?.clone();

        for ep in &config.endpoints {
            let ep_start = std::time::Instant::now();
            let url = format!("{}{}", base_url.trim_end_matches('/'), &ep.url);

            let ep_result = match client.get(&url).send() {
                Ok(response) => {
                    match response.json::<serde_json::Value>() {
                        Ok(body) => {
                            let records = body.as_array().cloned()
                                .or_else(|| body["data"].as_array().cloned())
                                .or_else(|| body["records"].as_array().cloned())
                                .unwrap_or_default();
                            let count = records.len() as i64;
                            total_synced += count;
                            EndpointSyncResult {
                                endpoint_name: ep.name.clone(),
                                status: SyncStatus::Success,
                                records_synced: count,
                                records_failed: 0,
                                error_message: None,
                            }
                        }
                        Err(e) => {
                            total_failed += 1;
                            EndpointSyncResult {
                                endpoint_name: ep.name.clone(),
                                status: SyncStatus::Failed,
                                records_synced: 0,
                                records_failed: 1,
                                error_message: Some(format!("Failed to parse response: {e}")),
                            }
                        }
                    }
                }
                Err(e) => {
                    total_failed += 1;
                    EndpointSyncResult {
                        endpoint_name: ep.name.clone(),
                        status: SyncStatus::Failed,
                        records_synced: 0,
                        records_failed: 1,
                        error_message: Some(format!("HTTP request failed: {e}")),
                    }
                }
            };

            let now = Utc::now();
            let sync_log_id = Uuid::new_v4();
            sql_query(
                "INSERT INTO connector_sync_logs (id, connector_id, endpoint_id, status, records_synced, records_failed, duration_ms, error_message, started_at, completed_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
            )
            .bind::<diesel::sql_types::Uuid, _>(sync_log_id)
            .bind::<diesel::sql_types::Uuid, _>(id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(None::<Uuid>)
            .bind::<diesel::sql_types::Text, _>(serde_json::to_string(&ep_result.status).unwrap_or_default())
            .bind::<diesel::sql_types::BigInt, _>(ep_result.records_synced)
            .bind::<diesel::sql_types::BigInt, _>(ep_result.records_failed)
            .bind::<diesel::sql_types::BigInt, _>(ep_start.elapsed().as_millis() as i64)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(ep_result.error_message.as_ref())
            .bind::<diesel::sql_types::Timestamptz, _>(started_at)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(Some(now))
            .execute(conn)
            .map_err(|e| format!("Failed to insert sync log: {e}"))?;

            endpoint_results.push(ep_result);
        }

        let duration_ms = (Utc::now() - started_at).num_milliseconds();
        let overall_status = if total_failed == 0 {
            SyncStatus::Success
        } else if total_synced == 0 {
            SyncStatus::Failed
        } else {
            SyncStatus::Partial
        };

        let now = Utc::now();
        sql_query("UPDATE connectors SET last_sync_at = $1, last_sync_status = $2, error_log = $3, updated_at = $4 WHERE id = $5")
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Text, _>(overall_status.to_string())
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(
                if total_failed > 0 { Some(format!("{total_failed} endpoint(s) failed")) } else { None }
            )
            .bind::<diesel::sql_types::Timestamptz, _>(now)
            .bind::<diesel::sql_types::Uuid, _>(id)
            .execute(conn).ok();

        Ok(SyncResult {
            status: overall_status,
            records_synced: total_synced,
            records_failed: total_failed,
            duration_ms,
            error_message: if total_failed > 0 { Some(format!("{total_failed} endpoint(s) failed")) } else { None },
            endpoint_results,
        })
    }

    pub fn get_sync_logs(
        conn: &mut PgConnection,
        connector_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<SyncLog>, String> {
        #[derive(diesel::QueryableByName)]
        struct SyncLogRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)] connector_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)] endpoint_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)] status: String,
            #[diesel(sql_type = diesel::sql_types::BigInt)] records_synced: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)] records_failed: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)] duration_ms: i64,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] error_message: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)] started_at: chrono::DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] completed_at: Option<chrono::DateTime<Utc>>,
        }

        let rows: Vec<SyncLogRow> = sql_query(
            "SELECT * FROM connector_sync_logs WHERE connector_id = $1 ORDER BY started_at DESC LIMIT $2 OFFSET $3"
        )
        .bind::<diesel::sql_types::Uuid, _>(connector_id)
        .bind::<diesel::sql_types::BigInt, _>(limit)
        .bind::<diesel::sql_types::BigInt, _>(offset)
        .get_results(conn)
        .map_err(|e| format!("Failed to get sync logs: {e}"))?;

        Ok(rows.into_iter().map(|r| SyncLog {
            id: r.id, connector_id: r.connector_id,
            endpoint_id: r.endpoint_id,
            status: SyncStatus::from(r.status.as_str()),
            records_synced: r.records_synced,
            records_failed: r.records_failed,
            duration_ms: r.duration_ms,
            error_message: r.error_message,
            started_at: r.started_at,
            completed_at: r.completed_at,
        }).collect())
    }

    pub fn get_endpoints(
        conn: &mut PgConnection,
        connector_id: Uuid,
    ) -> Result<Vec<EndpointConfig>, String> {
        #[derive(diesel::QueryableByName)]
        struct EndpointRow {
            #[diesel(sql_type = diesel::sql_types::Text)] name: String,
            #[diesel(sql_type = diesel::sql_types::Text)] method: String,
            #[diesel(sql_type = diesel::sql_types::Text)] url: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)] headers: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Text)] sync_direction: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)] field_mapping: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] schedule: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)] pagination: Option<serde_json::Value>,
        }

        let rows: Vec<EndpointRow> = sql_query(
            "SELECT name, method, url, headers, sync_direction, field_mapping, schedule, pagination FROM connector_endpoints WHERE connector_id = $1 ORDER BY created_at"
        )
        .bind::<diesel::sql_types::Uuid, _>(connector_id)
        .get_results(conn)
        .map_err(|e| format!("Failed to get endpoints: {e}"))?;

        Ok(rows.into_iter().map(|r| EndpointConfig {
            name: r.name, method: r.method, url: r.url,
            headers: r.headers,
            auth_type: AuthType::ApiKey,
            sync_direction: serde_json::from_str(&r.sync_direction).unwrap_or(SyncDirection::Pull),
            field_mapping: serde_json::from_value(r.field_mapping).unwrap_or_default(),
            schedule: r.schedule,
            pagination: r.pagination.and_then(|p| serde_json::from_value(p).ok()),
        }).collect())
    }

    fn insert_endpoint(
        conn: &mut PgConnection,
        connector_id: Uuid,
        ep: &EndpointConfig,
        now: chrono::DateTime<Utc>,
    ) -> Result<(), String> {
        sql_query(
            "INSERT INTO connector_endpoints (id, connector_id, name, method, url, headers, sync_direction, field_mapping, schedule, pagination, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(connector_id)
        .bind::<diesel::sql_types::Text, _>(&ep.name)
        .bind::<diesel::sql_types::Text, _>(&ep.method)
        .bind::<diesel::sql_types::Text, _>(&ep.url)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>, _>(ep.headers.as_ref())
        .bind::<diesel::sql_types::Text, _>(serde_json::to_string(&ep.sync_direction).unwrap_or_default())
        .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(&ep.field_mapping).unwrap_or_default())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&ep.schedule)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Jsonb>, _>(ep.pagination.as_ref().map(|p| serde_json::to_value(p).unwrap_or_default()))
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(conn)
        .map_err(|e| format!("Failed to create endpoint: {e}"))?;
        Ok(())
    }

    fn row_to_config(
        conn: &mut PgConnection,
        row: ConnectorRow,
    ) -> Result<ConnectorConfig, String> {
        let endpoints = Self::get_endpoints(conn, row.id)?;

        let auth_config: AuthConfig = serde_json::from_value(row.auth_config).unwrap_or(AuthConfig {
            auth_type: AuthType::None, api_key: None, api_key_header: None,
            username: None, password: None, oauth2_client_id: None,
            oauth2_client_secret: None, oauth2_token_url: None,
            oauth2_scopes: None, base_url: None, extra_headers: None,
        });

        Ok(ConnectorConfig {
            id: row.id, bot_id: row.bot_id, name: row.name,
            connector_type: ConnectorType::from(row.connector_type.as_str()),
            description: row.description, auth_config, endpoints,
            schedule: row.schedule, is_active: row.is_active,
            last_sync_at: row.last_sync_at,
            last_sync_status: row.last_sync_status.clone(),
            error_log: row.error_log,
            created_at: row.created_at, updated_at: row.updated_at,
        })
    }
}

fn build_blocking_client(auth: &AuthConfig) -> Result<reqwest::blocking::Client, String> {
    let mut headers = reqwest::header::HeaderMap::new();

    match &auth.auth_type {
        AuthType::ApiKey => {
            let header_name = auth.api_key_header.as_deref().unwrap_or("X-API-Key");
            if let Some(ref key) = auth.api_key {
                headers.insert(
                    reqwest::header::HeaderName::from_bytes(header_name.as_bytes())
                        .map_err(|e| format!("Invalid header name: {e}"))?,
                    reqwest::header::HeaderValue::from_str(key)
                        .map_err(|e| format!("Invalid header value: {e}"))?,
                );
            }
        }
        AuthType::Bearer => {
            if let Some(ref key) = auth.api_key {
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {key}"))
                        .map_err(|e| format!("Invalid bearer token: {e}"))?,
                );
            }
        }
        AuthType::Basic => {
            if let (Some(ref user), Some(ref pass)) = (&auth.username, &auth.password) {
                let encoded = base64::engine::general_purpose::STANDARD
                    .encode(format!("{user}:{pass}"));
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Basic {encoded}"))
                        .map_err(|e| format!("Invalid basic auth: {e}"))?,
                );
            }
        }
        AuthType::OAuth2 | AuthType::None | AuthType::Custom(_) => {}
    }

    if let Some(ref extra) = auth.extra_headers {
        for (name, value) in extra {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(name.as_bytes())
                    .map_err(|e| format!("Invalid header name: {e}"))?,
                reqwest::header::HeaderValue::from_str(value)
                    .map_err(|e| format!("Invalid header value: {e}"))?,
            );
        }
    }

    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))
}

#[derive(diesel::QueryableByName)]
struct ConnectorRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)] id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)] bot_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)] name: String,
    #[diesel(sql_type = diesel::sql_types::Text)] connector_type: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Jsonb)] auth_config: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] schedule: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Bool)] is_active: bool,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)] last_sync_at: Option<chrono::DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] last_sync_status: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)] error_log: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)] created_at: chrono::DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)] updated_at: chrono::DateTime<Utc>,
}
