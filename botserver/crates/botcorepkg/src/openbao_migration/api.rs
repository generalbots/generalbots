//! The few secrets-server HTTP calls the move needs. Vault and OpenBao share
//! this API. Errors carry the request path and status only: response bodies
//! may hold secrets and are never logged or returned.

use super::tree::{Fields, KvStore, KvVersion, Mount};
use anyhow::{anyhow, Context, Result};
use reqwest::{Method, StatusCode, Url};
use serde_json::Value;
use std::time::{Duration, Instant};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const POLL: Duration = Duration::from_millis(300);
const ACTIVE_TIMEOUT: Duration = Duration::from_secs(60);

/// What `sys/init` hands back, wiped on drop.
pub struct InitData {
    pub keys_b64: Vec<String>,
    pub keys_hex: Vec<String>,
    pub root_token: String,
}

impl Drop for InitData {
    fn drop(&mut self) {
        self.keys_b64.iter_mut().chain(self.keys_hex.iter_mut()).for_each(super::tree::wipe_string);
        super::tree::wipe_string(&mut self.root_token);
    }
}

pub struct Api {
    client: reqwest::Client,
    base: Url,
    token: Option<String>,
}

impl Drop for Api {
    fn drop(&mut self) {
        if let Some(token) = self.token.as_mut() {
            super::tree::wipe_string(token);
        }
    }
}

impl Api {
    /// A client for `addr` (`http://127.0.0.1:8200`). Proxies are never used:
    /// secrets must not leave the loopback connection.
    pub fn new(addr: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .context("secrets HTTP client")?;
        let base = Url::parse(addr).context("secrets server address")?;
        Ok(Self { client, base, token: None })
    }

    /// The same client, authenticated with `token`.
    #[must_use]
    pub fn with_token(&self, token: &str) -> Self {
        Self { client: self.client.clone(), base: self.base.clone(), token: Some(token.to_string()) }
    }

    fn url(&self, segments: &[String], list: bool) -> Result<Url> {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .map_err(|()| anyhow!("secrets server address cannot carry a path"))?
            .clear()
            .push("v1")
            .extend(segments);
        if list {
            url.set_query(Some("list=true"));
        }
        Ok(url)
    }

    async fn call(&self, method: Method, segments: &[String], list: bool, body: Option<&Value>) -> Result<(StatusCode, Value)> {
        let label = segments.join("/");
        let mut request = self.client.request(method, self.url(segments, list)?);
        if let Some(token) = &self.token {
            request = request.header("X-Vault-Token", token);
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().await.map_err(|e| anyhow!("{label}: {}", e.without_url()))?;
        let status = response.status();
        let json = if status == StatusCode::NO_CONTENT {
            Value::Null
        } else {
            response.json::<Value>().await.unwrap_or(Value::Null)
        };
        Ok((status, json))
    }

    async fn ok(&self, method: Method, segments: &[String], body: Option<&Value>) -> Result<Value> {
        let (status, json) = self.call(method, segments, false, body).await?;
        if status.is_success() {
            Ok(json)
        } else {
            Err(anyhow!("{} answered {status}", segments.join("/")))
        }
    }

    /// `(initialized, sealed)`, or an error while nothing answers.
    pub async fn seal_status(&self) -> Result<(bool, bool)> {
        let json = self.ok(Method::GET, &seg(&["sys", "seal-status"]), None).await?;
        let flag = |name: &str| json.get(name).and_then(Value::as_bool).ok_or_else(|| anyhow!("sys/seal-status has no {name}"));
        Ok((flag("initialized")?, flag("sealed")?))
    }

    /// Waits until the server answers, up to `timeout`.
    pub async fn wait_answering(&self, timeout: Duration) -> Result<(bool, bool)> {
        let start = Instant::now();
        loop {
            match self.seal_status().await {
                Ok(status) => return Ok(status),
                Err(e) if start.elapsed() >= timeout => return Err(e.context("the secrets server did not answer")),
                Err(_) => tokio::time::sleep(POLL).await,
            }
        }
    }

    /// Waits until nothing answers any more, up to `timeout`.
    pub async fn wait_silent(&self, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        while self.seal_status().await.is_ok() {
            if start.elapsed() >= timeout {
                return Err(anyhow!("the secrets server kept answering after it was stopped"));
            }
            tokio::time::sleep(POLL).await;
        }
        Ok(())
    }

    /// Unseals with `keys`; fails if the server stays sealed.
    pub async fn unseal(&self, keys: &[String]) -> Result<()> {
        for key in keys {
            let json = self.ok(Method::PUT, &seg(&["sys", "unseal"]), Some(&serde_json::json!({ "key": key }))).await?;
            if json.get("sealed").and_then(Value::as_bool) == Some(false) {
                return self.wait_active().await;
            }
        }
        Err(anyhow!("the secrets server is still sealed after {} keys", keys.len()))
    }

    /// Waits until the unsealed server is the active node: raft storage
    /// elects itself a moment after the unseal, and writes fail until then.
    async fn wait_active(&self) -> Result<()> {
        let start = Instant::now();
        loop {
            let status = self.call(Method::GET, &seg(&["sys", "health"]), false, None).await.map(|(s, _)| s);
            if matches!(status, Ok(StatusCode::OK)) {
                return Ok(());
            }
            if start.elapsed() >= ACTIVE_TIMEOUT {
                return Err(anyhow!("the secrets server did not become active after the unseal"));
            }
            tokio::time::sleep(POLL).await;
        }
    }

    pub async fn init(&self, shares: u8, threshold: u8) -> Result<InitData> {
        let body = serde_json::json!({ "secret_shares": shares, "secret_threshold": threshold });
        let json = self.ok(Method::PUT, &seg(&["sys", "init"]), Some(&body)).await?;
        let strings = |name: &str| -> Vec<String> {
            json.get(name)
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
                .unwrap_or_default()
        };
        let data = InitData {
            keys_b64: strings("keys_base64"),
            keys_hex: strings("keys"),
            root_token: json.get("root_token").and_then(Value::as_str).unwrap_or_default().to_string(),
        };
        if data.root_token.is_empty() || data.keys_b64.len() != usize::from(shares) {
            return Err(anyhow!("sys/init answered without the expected keys"));
        }
        Ok(data)
    }

    /// Every KV mount (version 1 or 2).
    pub async fn kv_mounts(&self) -> Result<Vec<Mount>> {
        let json = self.ok(Method::GET, &seg(&["sys", "mounts"]), None).await?;
        let table = json.get("data").filter(|d| d.is_object()).unwrap_or(&json);
        Ok(kv_mounts_in(table))
    }

    pub async fn enable_kv(&self, mount: &Mount) -> Result<()> {
        let version = if mount.version == KvVersion::V2 { "2" } else { "1" };
        let body = serde_json::json!({ "type": "kv", "options": { "version": version } });
        let mut segments = seg(&["sys", "mounts"]);
        segments.extend(mount.path.split('/').filter(|s| !s.is_empty()).map(str::to_string));
        self.ok(Method::POST, &segments, Some(&body)).await.map(|_| ())
    }

    /// Names of the ACL policies other than the built-in `root` and `default`.
    pub async fn policy_names(&self) -> Result<Vec<String>> {
        let (status, json) = self.call(Method::GET, &seg(&["sys", "policies", "acl"]), true, None).await?;
        if !status.is_success() {
            return Err(anyhow!("sys/policies/acl answered {status}"));
        }
        Ok(keys_of(&json).into_iter().filter(|n| n != "root" && n != "default").collect())
    }

    pub async fn policy(&self, name: &str) -> Result<String> {
        let json = self.ok(Method::GET, &seg(&["sys", "policies", "acl", name]), None).await?;
        json.pointer("/data/policy")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow!("policy {name} has no text"))
    }

    pub async fn put_policy(&self, name: &str, text: &str) -> Result<()> {
        let body = serde_json::json!({ "policy": text });
        self.ok(Method::PUT, &seg(&["sys", "policies", "acl", name]), Some(&body)).await.map(|_| ())
    }
}

impl KvStore for Api {
    async fn list(&self, mount: &Mount, dir: &str) -> Result<Vec<String>> {
        let (status, json) = self.call(Method::GET, &mount.list_segments(dir), true, None).await?;
        match status {
            StatusCode::NOT_FOUND => Ok(Vec::new()),
            s if s.is_success() => Ok(keys_of(&json)),
            s => Err(anyhow!("listing {}{dir} answered {s}", mount.path)),
        }
    }

    async fn read(&self, mount: &Mount, path: &str) -> Result<Option<Fields>> {
        let (status, mut json) = self.call(Method::GET, &mount.data_segments(path), false, None).await?;
        let fields = match status {
            StatusCode::NOT_FOUND => None,
            s if s.is_success() => mount.fields_of(&json),
            s => return Err(anyhow!("reading {}{path} answered {s}", mount.path)),
        };
        super::tree::wipe_value(&mut json);
        Ok(fields)
    }

    async fn write(&self, mount: &Mount, path: &str, fields: &Fields) -> Result<()> {
        let mut body = mount.write_body(fields);
        let result = self.ok(Method::POST, &mount.data_segments(path), Some(&body)).await;
        super::tree::wipe_value(&mut body);
        result.map(|_| ()).map_err(|_| anyhow!("writing {}{path} failed", mount.path))
    }
}

fn seg(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| (*s).to_string()).collect()
}

fn keys_of(json: &Value) -> Vec<String> {
    json.pointer("/data/keys")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .unwrap_or_default()
}

/// KV mounts in a `sys/mounts` table.
pub fn kv_mounts_in(table: &Value) -> Vec<Mount> {
    let Some(entries) = table.as_object() else { return Vec::new() };
    let mut mounts: Vec<Mount> = entries
        .iter()
        .filter(|(path, entry)| path.ends_with('/') && entry.get("type").and_then(Value::as_str) == Some("kv"))
        .map(|(path, entry)| Mount {
            path: path.clone(),
            version: if entry.pointer("/options/version").and_then(Value::as_str) == Some("2") {
                KvVersion::V2
            } else {
                KvVersion::V1
            },
        })
        .collect();
    mounts.sort_by(|a, b| a.path.cmp(&b.path));
    mounts
}
