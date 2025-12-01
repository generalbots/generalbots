use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::urls::ApiUrls;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsEntry {
    pub hostname: String,
    pub ip: IpAddr,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub enabled: bool,
    pub zone_file_path: PathBuf,
    pub domain: String,
    pub max_entries_per_ip: usize,
    pub ttl_seconds: u32,
    pub cleanup_interval_hours: u64,
}

impl Default for DnsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            zone_file_path: PathBuf::from("./botserver-stack/conf/dns/botserver.local.zone"),
            domain: "botserver.local",
            max_entries_per_ip: 5,
            ttl_seconds: 60,
            cleanup_interval_hours: 24,
        }
    }
}

pub struct DynamicDnsService {
    config: DnsConfig,
    entries: Arc<RwLock<HashMap<String, DnsEntry>>>,
    entries_by_ip: Arc<RwLock<HashMap<IpAddr, Vec<String>>>>,
}

impl DynamicDnsService {
    pub fn new(config: DnsConfig) -> Self {
        Self {
            config,
            entries: Arc::new(RwLock::new(HashMap::new())),
            entries_by_ip: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_hostname(&self, hostname: &str, ip: IpAddr) -> Result<()> {
        // Validate hostname
        if !self.is_valid_hostname(hostname) {
            return Err(anyhow::anyhow!("Invalid hostname format"));
        }

        // Check rate limiting
        if !self.check_rate_limit(&ip).await {
            return Err(anyhow::anyhow!("Rate limit exceeded for IP"));
        }

        let full_hostname = format!("{}.{}", hostname, self.config.domain);
        let now = Utc::now();

        let entry = DnsEntry {
            hostname: hostname.to_string(),
            ip,
            created_at: now,
            updated_at: now,
            ttl: self.config.ttl_seconds,
        };

        // Update in-memory store
        {
            let mut entries = self.entries.write().await;
            entries.insert(hostname.to_string(), entry.clone());
        }

        // Track by IP for rate limiting
        {
            let mut by_ip = self.entries_by_ip.write().await;
            by_ip
                .entry(ip)
                .or_insert_with(Vec::new)
                .push(hostname.to_string());

            // Limit entries per IP
            if let Some(ip_entries) = by_ip.get_mut(&ip) {
                if ip_entries.len() > self.config.max_entries_per_ip {
                    let removed = ip_entries.remove(0);
                    let mut entries = self.entries.write().await;
                    entries.remove(&removed);
                }
            }
        }

        // Update zone file
        self.update_zone_file().await?;

        log::info!("Registered hostname {} -> {}", full_hostname, ip);
        Ok(())
    }

    pub async fn remove_hostname(&self, hostname: &str) -> Result<()> {
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.remove(hostname) {
            // Remove from IP tracking
            let mut by_ip = self.entries_by_ip.write().await;
            if let Some(ip_entries) = by_ip.get_mut(&entry.ip) {
                ip_entries.retain(|h| h != hostname);
                if ip_entries.is_empty() {
                    by_ip.remove(&entry.ip);
                }
            }

            self.update_zone_file().await?;
            log::info!("Removed hostname {}.{}", hostname, self.config.domain);
        }

        Ok(())
    }

    pub async fn cleanup_old_entries(&self) -> Result<()> {
        let now = Utc::now();
        let max_age = chrono::Duration::hours(self.config.cleanup_interval_hours as i64);

        let mut entries = self.entries.write().await;
        let mut by_ip = self.entries_by_ip.write().await;
        let mut removed = Vec::new();

        entries.retain(|hostname, entry| {
            if now - entry.updated_at > max_age {
                removed.push((hostname.clone(), entry.ip));
                false
            } else {
                true
            }
        });

        for (hostname, ip) in &removed {
            if let Some(ip_entries) = by_ip.get_mut(ip) {
                ip_entries.retain(|h| h != hostname);
                if ip_entries.is_empty() {
                    by_ip.remove(ip);
                }
            }
        }

        if !removed.is_empty() {
            self.update_zone_file().await?;
            log::info!("Cleaned up {} old DNS entries", removed.len());
        }

        Ok(())
    }

    async fn update_zone_file(&self) -> Result<()> {
        let entries = self.entries.read().await;

        let mut zone_content = String::new();
        zone_content.push_str(&format!(
            "$ORIGIN {}.\n$TTL {}\n",
            self.config.domain, self.config.ttl_seconds
        ));

        zone_content.push_str(&format!(
            "@       IN      SOA     ns1.{}. admin.{}. (\n",
            self.config.domain, self.config.domain
        ));
        zone_content.push_str(&format!(
            "                        {}      ; Serial\n",
            Utc::now().timestamp()
        ));
        zone_content.push_str(
            "                        3600            ; Refresh\n\
             \x20                       1800            ; Retry\n\
             \x20                       604800          ; Expire\n\
             \x20                       60              ; Minimum TTL\n\
             )\n",
        );
        zone_content.push_str(&format!(
            "        IN      NS      ns1.{}.\n",
            self.config.domain
        ));
        zone_content.push_str("ns1     IN      A       127.0.0.1\n\n");

        // Static entries
        zone_content.push_str("; Static service entries\n");
        zone_content.push_str("api     IN      A       127.0.0.1\n");
        zone_content.push_str("auth    IN      A       127.0.0.1\n");
        zone_content.push_str("llm     IN      A       127.0.0.1\n");
        zone_content.push_str("mail    IN      A       127.0.0.1\n");
        zone_content.push_str("meet    IN      A       127.0.0.1\n\n");

        // Dynamic entries
        if !entries.is_empty() {
            zone_content.push_str("; Dynamic entries\n");
            for (hostname, entry) in entries.iter() {
                zone_content.push_str(&format!("{:<16} IN      A       {}\n", hostname, entry.ip));
            }
        }

        fs::write(&self.config.zone_file_path, zone_content)?;
        Ok(())
    }

    fn is_valid_hostname(&self, hostname: &str) -> bool {
        if hostname.is_empty() || hostname.len() > 63 {
            return false;
        }

        hostname
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
            && !hostname.starts_with('-')
            && !hostname.ends_with('-')
    }

    async fn check_rate_limit(&self, ip: &IpAddr) -> bool {
        let by_ip = self.entries_by_ip.read().await;
        if let Some(entries) = by_ip.get(ip) {
            entries.len() < self.config.max_entries_per_ip
        } else {
            true
        }
    }

    pub async fn start_cleanup_task(self: Arc<Self>) {
        let service = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                service.config.cleanup_interval_hours * 3600,
            ));

            loop {
                interval.tick().await;
                if let Err(e) = service.cleanup_old_entries().await {
                    log::error!("Failed to cleanup DNS entries: {}", e);
                }
            }
        });
    }
}

// API handlers for dynamic DNS
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub hostname: String,
    pub ip: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub success: bool,
    pub hostname: String,
    pub ip: String,
    pub ttl: u32,
}

pub async fn register_hostname_handler(
    Query(params): Query<RegisterRequest>,
    State(dns_service): State<Arc<DynamicDnsService>>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<std::net::SocketAddr>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let ip = if let Some(ip_str) = params.ip {
        ip_str.parse().map_err(|_| StatusCode::BAD_REQUEST)?
    } else {
        addr.ip()
    };

    dns_service
        .register_hostname(&params.hostname, ip)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RegisterResponse {
        success: true,
        hostname: format!("{}.{}", params.hostname, dns_service.config.domain),
        ip: ip.to_string(),
        ttl: dns_service.config.ttl_seconds,
    }))
}

pub async fn remove_hostname_handler(
    Query(params): Query<RegisterRequest>,
    State(dns_service): State<Arc<DynamicDnsService>>,
) -> Result<StatusCode, StatusCode> {
    dns_service
        .remove_hostname(&params.hostname)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub fn configure_dns_routes(dns_service: Arc<DynamicDnsService>) -> Router {
    Router::new()
        .route(ApiUrls::DNS_REGISTER, post(register_hostname_handler))
        .route(ApiUrls::DNS_REMOVE, post(remove_hostname_handler))
        .with_state(dns_service)
}
