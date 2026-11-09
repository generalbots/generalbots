use super::*;

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
        // #2054 — secrets go to Vault; the row keeps only the sanitized config
        // and the path.
        let (auth_config, vault_path) = vault_auth(req.bot_id, id, &req.auth_config);

        sql_query(
            "INSERT INTO connectors (id, bot_id, name, connector_type, description, auth_config, schedule, is_active, created_at, updated_at, secrets_vault_path)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Uuid, _>(req.bot_id)
        .bind::<diesel::sql_types::Text, _>(&req.name)
        .bind::<diesel::sql_types::Text, _>(&req.connector_type)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&req.description)
        .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(&auth_config).unwrap_or_default())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&req.schedule)
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&vault_path)
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
            // #2054 — an update must not put the secrets back into the row.
            let bot_id: Uuid = sql_query("SELECT bot_id FROM connectors WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(id)
                .get_result::<UuidRow>(conn)
                .map(|row| row.bot_id)
                .map_err(|e| format!("Failed to load connector owner: {e}"))?;
            let (sanitized, vault_path) = vault_auth(bot_id, id, auth);
            sql_query(
                "UPDATE connectors SET auth_config = $1, secrets_vault_path = COALESCE($3, secrets_vault_path) WHERE id = $2",
            )
            .bind::<diesel::sql_types::Jsonb, _>(&serde_json::to_value(&sanitized).unwrap_or_default())
            .bind::<diesel::sql_types::Uuid, _>(id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&vault_path)
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
        // #2052/#2054 — the request path merges the Vaulted grant and refreshes
        // an expired token; the sanitized row config cannot authenticate.
        let config = Self::request_config(conn, id)?;
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
        // #2052 — 401/403 is an authentication failure, never a success: the
        // old code reported a read-only or unauthenticated connector as green.
        classify_probe_status(status.as_u16())?;
        Ok(format!("Connection successful (HTTP {status})"))
    }

    pub fn discover_schema(
        conn: &mut PgConnection,
        id: Uuid,
        endpoint_name: Option<String>,
    ) -> Result<Vec<DiscoveredSchema>, String> {
        let config = Self::request_config(conn, id)?;
        let client = build_blocking_client(&config.auth_config)
            .map_err(|e| format!("Failed to build client: {e}"))?;

        let mut schemas = Vec::new();
        let base_url = config.auth_config.base_url.as_ref()
            .ok_or_else(|| "Base URL not configured".to_string())?;

        for ep in &config.endpoints {
            if let Some(ref filter) = endpoint_name {
                if &ep.name != filter { continue; }
            }

            let url = format!("{}{}", base_url.trim_end_matches('/'), ep.url);
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
}

impl ConnectorEngine {
    /// #2052/#2054 — the config used to make a real request: the row's
    /// non-secret fields plus the credential envelope from Vault, with an
    /// expired OAuth2 token refreshed (and the new grant written back).
    pub(super) fn request_config(
        conn: &mut PgConnection,
        id: Uuid,
    ) -> Result<ConnectorConfig, String> {
        let row: ConnectorRow = sql_query("SELECT * FROM connectors WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .get_result(conn)
            .map_err(|e| format!("Connector not found: {e}"))?;
        let vault_path = row.secrets_vault_path.clone();
        let mut config = Self::row_to_config(conn, row)?;
        if let Some(path) = vault_path.as_deref() {
            merge_vault_secrets(&mut config.auth_config, path);
        }
        let refreshed = oauth2::ensure_fresh(&mut config.auth_config)?;
        if refreshed {
            if let Some(path) = vault_path.as_deref() {
                store_refreshed_grant(&config.auth_config, path);
            }
        }
        Ok(config)
    }
}

/// #2054 — splits the credential fields out of the auth config, stores them in
/// Vault and returns the sanitized config plus the path the row should keep.
fn vault_auth(
    bot_id: Uuid,
    connector_id: Uuid,
    auth: &AuthConfig,
) -> (AuthConfig, Option<String>) {
    let value = serde_json::to_value(auth).unwrap_or_default();
    let (sanitized, secrets) = crate::connector_ops::split_sensitive_auth(&value);
    let sanitized_auth: AuthConfig =
        serde_json::from_value(sanitized).unwrap_or_else(|_| auth.clone());
    if secrets.is_empty() {
        return (sanitized_auth, None);
    }
    let path = crate::connector_ops::secrets_path(
        "default",
        &bot_id.to_string(),
        &connector_id.to_string(),
    );
    match botcoresecrets::SecretsManager::from_env() {
        Ok(manager) => crate::connector_ops::store_secrets(&manager, &path, &secrets),
        Err(_) => log::warn!("connector secrets for {path} not vaulted: Vault is not configured"),
    }
    (sanitized_auth, Some(path))
}

/// #2054 — merges the Vaulted envelope back into the in-memory auth config.
fn merge_vault_secrets(auth: &mut AuthConfig, path: &str) {
    let Ok(manager) = botcoresecrets::SecretsManager::from_env() else {
        log::warn!("connector secrets at {path} unavailable: Vault is not configured");
        return;
    };
    let secrets = crate::connector_ops::load_secrets(&manager, path);
    if secrets.is_empty() {
        return;
    }
    let mut value = serde_json::to_value(&*auth).unwrap_or_default();
    if let Some(object) = value.as_object_mut() {
        for (key, secret) in secrets {
            object.insert(key, serde_json::json!(secret));
        }
    }
    if let Ok(merged) = serde_json::from_value::<AuthConfig>(value) {
        *auth = merged;
    }
}

/// #2052 — a rotated refresh token must not be lost: write the new grant back
/// to the same Vault entry.
fn store_refreshed_grant(auth: &AuthConfig, path: &str) {
    let Ok(manager) = botcoresecrets::SecretsManager::from_env() else {
        log::warn!("refreshed connector grant for {path} not persisted: Vault is not configured");
        return;
    };
    let value = serde_json::to_value(auth).unwrap_or_default();
    let (_, secrets) = crate::connector_ops::split_sensitive_auth(&value);
    crate::connector_ops::store_secrets(&manager, path, &secrets);
}

/// #2052 — only a 2xx answer proves a connector can authenticate. 401/403 is
/// an auth failure with actionable text; anything else is a plain status.
fn classify_probe_status(status: u16) -> Result<(), String> {
    if (200..300).contains(&status) {
        return Ok(());
    }
    if status == 401 || status == 403 {
        return Err(format!(
            "authentication failed (HTTP {status}): the connector's grant is missing, expired or read-only — reconnect the account"
        ));
    }
    Err(format!("Connection returned status {status}"))
}

#[derive(diesel::QueryableByName)]
struct UuidRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    bot_id: Uuid,
}

impl Default for ConnectorEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_2xx_probe_succeeds() {
        assert!(classify_probe_status(200).is_ok());
        assert!(classify_probe_status(204).is_ok());
    }

    #[test]
    fn unauthorized_and_forbidden_are_never_reported_as_success() {
        for status in [401, 403] {
            let error = classify_probe_status(status).expect_err("must fail");
            assert!(error.contains("authentication failed"), "{error}");
            assert!(error.contains("reconnect"), "{error}");
        }
    }

    #[test]
    fn other_statuses_report_the_code() {
        let error = classify_probe_status(500).expect_err("must fail");
        assert!(error.contains("500"), "{error}");
    }
}
