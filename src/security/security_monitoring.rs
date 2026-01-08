use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

const DEFAULT_BRUTE_FORCE_THRESHOLD: u32 = 5;
const DEFAULT_BRUTE_FORCE_WINDOW_SECONDS: i64 = 300;
const DEFAULT_LOCKOUT_DURATION_MINUTES: i64 = 30;
const DEFAULT_ANOMALY_THRESHOLD: f64 = 3.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMonitoringConfig {
    pub enabled: bool,
    pub brute_force_threshold: u32,
    pub brute_force_window_seconds: i64,
    pub lockout_duration_minutes: i64,
    pub anomaly_detection_enabled: bool,
    pub anomaly_threshold_stddev: f64,
    pub geo_anomaly_detection: bool,
    pub impossible_travel_detection: bool,
    pub max_travel_speed_kmh: f64,
    pub alert_on_critical: bool,
    pub alert_on_high: bool,
    pub retention_hours: u32,
}

impl Default for SecurityMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            brute_force_threshold: DEFAULT_BRUTE_FORCE_THRESHOLD,
            brute_force_window_seconds: DEFAULT_BRUTE_FORCE_WINDOW_SECONDS,
            lockout_duration_minutes: DEFAULT_LOCKOUT_DURATION_MINUTES,
            anomaly_detection_enabled: true,
            anomaly_threshold_stddev: DEFAULT_ANOMALY_THRESHOLD,
            geo_anomaly_detection: true,
            impossible_travel_detection: true,
            max_travel_speed_kmh: 1000.0,
            alert_on_critical: true,
            alert_on_high: true,
            retention_hours: 168,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityEventType {
    LoginAttempt,
    LoginSuccess,
    LoginFailure,
    PasswordReset,
    MfaChallenge,
    MfaFailure,
    SessionCreated,
    SessionRevoked,
    PermissionDenied,
    RateLimitExceeded,
    SuspiciousActivity,
    BruteForceDetected,
    AccountLocked,
    IpBlocked,
    GeoAnomalyDetected,
    ImpossibleTravel,
    NewDeviceLogin,
    ApiKeyUsed,
    PrivilegeEscalation,
    DataExfiltration,
}

impl SecurityEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LoginAttempt => "login_attempt",
            Self::LoginSuccess => "login_success",
            Self::LoginFailure => "login_failure",
            Self::PasswordReset => "password_reset",
            Self::MfaChallenge => "mfa_challenge",
            Self::MfaFailure => "mfa_failure",
            Self::SessionCreated => "session_created",
            Self::SessionRevoked => "session_revoked",
            Self::PermissionDenied => "permission_denied",
            Self::RateLimitExceeded => "rate_limit_exceeded",
            Self::SuspiciousActivity => "suspicious_activity",
            Self::BruteForceDetected => "brute_force_detected",
            Self::AccountLocked => "account_locked",
            Self::IpBlocked => "ip_blocked",
            Self::GeoAnomalyDetected => "geo_anomaly_detected",
            Self::ImpossibleTravel => "impossible_travel",
            Self::NewDeviceLogin => "new_device_login",
            Self::ApiKeyUsed => "api_key_used",
            Self::PrivilegeEscalation => "privilege_escalation",
            Self::DataExfiltration => "data_exfiltration",
        }
    }

    pub fn severity(&self) -> AlertSeverity {
        match self {
            Self::BruteForceDetected
            | Self::AccountLocked
            | Self::PrivilegeEscalation
            | Self::DataExfiltration => AlertSeverity::Critical,
            Self::LoginFailure
            | Self::MfaFailure
            | Self::PermissionDenied
            | Self::ImpossibleTravel
            | Self::GeoAnomalyDetected => AlertSeverity::High,
            Self::RateLimitExceeded
            | Self::SuspiciousActivity
            | Self::IpBlocked
            | Self::NewDeviceLogin => AlertSeverity::Medium,
            _ => AlertSeverity::Low,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    pub fn score(&self) -> u8 {
        match self {
            Self::Low => 25,
            Self::Medium => 50,
            Self::High => 75,
            Self::Critical => 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub severity: AlertSeverity,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub location: Option<GeoLocation>,
    pub device_fingerprint: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
    pub request_id: Option<String>,
}

impl SecurityEvent {
    pub fn new(event_type: SecurityEventType) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            severity: event_type.severity(),
            user_id: None,
            ip_address: None,
            user_agent: None,
            location: None,
            device_fingerprint: None,
            details: HashMap::new(),
            request_id: None,
        }
    }

    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn with_user_agent(mut self, ua: String) -> Self {
        self.user_agent = Some(ua);
        self
    }

    pub fn with_location(mut self, location: GeoLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_device(mut self, fingerprint: String) -> Self {
        self.device_fingerprint = Some(fingerprint);
        self
    }

    pub fn with_detail(mut self, key: &str, value: serde_json::Value) -> Self {
        self.details.insert(key.to_string(), value);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub fn is_critical(&self) -> bool {
        self.severity == AlertSeverity::Critical
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timezone: Option<String>,
}

impl GeoLocation {
    pub fn new() -> Self {
        Self {
            country: None,
            region: None,
            city: None,
            latitude: None,
            longitude: None,
            timezone: None,
        }
    }

    pub fn with_country(mut self, country: &str) -> Self {
        self.country = Some(country.to_string());
        self
    }

    pub fn with_city(mut self, city: &str) -> Self {
        self.city = Some(city.to_string());
        self
    }

    pub fn with_coordinates(mut self, lat: f64, lon: f64) -> Self {
        self.latitude = Some(lat);
        self.longitude = Some(lon);
        self
    }

    pub fn distance_km(&self, other: &GeoLocation) -> Option<f64> {
        match (self.latitude, self.longitude, other.latitude, other.longitude) {
            (Some(lat1), Some(lon1), Some(lat2), Some(lon2)) => {
                Some(haversine_distance(lat1, lon1, lat2, lon2))
            }
            _ => None,
        }
    }
}

impl Default for GeoLocation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAttemptRecord {
    pub user_id: Option<Uuid>,
    pub ip_address: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub user_agent: Option<String>,
    pub location: Option<GeoLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockoutRecord {
    pub identifier: String,
    pub locked_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub reason: String,
    pub attempt_count: u32,
}

impl LockoutRecord {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn remaining_time(&self) -> Duration {
        if self.is_expired() {
            Duration::zero()
        } else {
            self.expires_at - Utc::now()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub event_ids: Vec<Uuid>,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub acknowledged: bool,
    pub acknowledged_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

impl SecurityAlert {
    pub fn new(severity: AlertSeverity, title: &str, description: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            severity,
            title: title.to_string(),
            description: description.to_string(),
            event_ids: Vec::new(),
            user_id: None,
            ip_address: None,
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
            resolved: false,
            resolved_at: None,
        }
    }

    pub fn with_event(mut self, event_id: Uuid) -> Self {
        self.event_ids.push(event_id);
        self
    }

    pub fn with_events(mut self, event_ids: Vec<Uuid>) -> Self {
        self.event_ids.extend(event_ids);
        self
    }

    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    pub fn acknowledge(&mut self, by: Uuid) {
        self.acknowledged = true;
        self.acknowledged_by = Some(by);
        self.acknowledged_at = Some(Utc::now());
    }

    pub fn resolve(&mut self) {
        self.resolved = true;
        self.resolved_at = Some(Utc::now());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSecurityProfile {
    pub user_id: Uuid,
    pub known_ips: Vec<String>,
    pub known_devices: Vec<String>,
    pub known_locations: Vec<GeoLocation>,
    pub last_login: Option<DateTime<Utc>>,
    pub last_location: Option<GeoLocation>,
    pub login_times: Vec<u32>,
    pub risk_score: f64,
    pub is_locked: bool,
    pub lock_expires_at: Option<DateTime<Utc>>,
}

impl UserSecurityProfile {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            known_ips: Vec::new(),
            known_devices: Vec::new(),
            known_locations: Vec::new(),
            last_login: None,
            last_location: None,
            login_times: Vec::new(),
            risk_score: 0.0,
            is_locked: false,
            lock_expires_at: None,
        }
    }

    pub fn is_known_ip(&self, ip: &str) -> bool {
        self.known_ips.contains(&ip.to_string())
    }

    pub fn is_known_device(&self, device: &str) -> bool {
        self.known_devices.contains(&device.to_string())
    }

    pub fn add_known_ip(&mut self, ip: &str) {
        if !self.is_known_ip(ip) {
            self.known_ips.push(ip.to_string());
            if self.known_ips.len() > 100 {
                self.known_ips.remove(0);
            }
        }
    }

    pub fn add_known_device(&mut self, device: &str) {
        if !self.is_known_device(device) {
            self.known_devices.push(device.to_string());
            if self.known_devices.len() > 50 {
                self.known_devices.remove(0);
            }
        }
    }

    pub fn record_login(&mut self, location: Option<GeoLocation>) {
        let now = Utc::now();
        self.last_login = Some(now);
        self.login_times.push(now.hour());
        if self.login_times.len() > 1000 {
            self.login_times.remove(0);
        }
        if let Some(loc) = location {
            self.last_location = Some(loc.clone());
            self.known_locations.push(loc);
            if self.known_locations.len() > 50 {
                self.known_locations.remove(0);
            }
        }
    }

    pub fn is_unusual_login_time(&self, hour: u32) -> bool {
        if self.login_times.len() < 10 {
            return false;
        }

        let count = self.login_times.iter().filter(|&&h| h == hour).count();
        let percentage = count as f64 / self.login_times.len() as f64;

        percentage < 0.01
    }

    pub fn lock(&mut self, duration: Duration) {
        self.is_locked = true;
        self.lock_expires_at = Some(Utc::now() + duration);
    }

    pub fn unlock(&mut self) {
        self.is_locked = false;
        self.lock_expires_at = None;
    }

    pub fn check_lock_status(&mut self) -> bool {
        if self.is_locked {
            if let Some(expires) = self.lock_expires_at {
                if Utc::now() > expires {
                    self.unlock();
                    return false;
                }
            }
            return true;
        }
        false
    }
}

pub struct SecurityMonitor {
    config: SecurityMonitoringConfig,
    events: Arc<RwLock<Vec<SecurityEvent>>>,
    login_attempts: Arc<RwLock<HashMap<String, Vec<LoginAttemptRecord>>>>,
    lockouts: Arc<RwLock<HashMap<String, LockoutRecord>>>,
    alerts: Arc<RwLock<Vec<SecurityAlert>>>,
    user_profiles: Arc<RwLock<HashMap<Uuid, UserSecurityProfile>>>,
    blocked_ips: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl SecurityMonitor {
    pub fn new(config: SecurityMonitoringConfig) -> Self {
        Self {
            config,
            events: Arc::new(RwLock::new(Vec::new())),
            login_attempts: Arc::new(RwLock::new(HashMap::new())),
            lockouts: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            user_profiles: Arc::new(RwLock::new(HashMap::new())),
            blocked_ips: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(SecurityMonitoringConfig::default())
    }

    pub async fn record_event(&self, event: SecurityEvent) {
        if !self.config.enabled {
            return;
        }

        let should_alert = match event.severity {
            AlertSeverity::Critical => self.config.alert_on_critical,
            AlertSeverity::High => self.config.alert_on_high,
            _ => false,
        };

        if should_alert {
            self.create_alert_from_event(&event).await;
        }

        let mut events = self.events.write().await;
        events.push(event);

        if events.len() > 100_000 {
            events.remove(0);
        }
    }

    pub async fn record_login_attempt(
        &self,
        user_id: Option<Uuid>,
        ip: &str,
        success: bool,
        user_agent: Option<&str>,
        location: Option<GeoLocation>,
    ) -> Option<SecurityAlert> {
        if !self.config.enabled {
            return None;
        }

        let record = LoginAttemptRecord {
            user_id,
            ip_address: ip.to_string(),
            timestamp: Utc::now(),
            success,
            user_agent: user_agent.map(String::from),
            location: location.clone(),
        };

        let key = user_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| ip.to_string());

        let mut attempts = self.login_attempts.write().await;
        let user_attempts = attempts.entry(key.clone()).or_default();
        user_attempts.push(record);

        let window_start = Utc::now() - Duration::seconds(self.config.brute_force_window_seconds);
        user_attempts.retain(|a| a.timestamp > window_start);

        let failed_count = user_attempts.iter().filter(|a| !a.success).count() as u32;

        drop(attempts);

        let event_type = if success {
            SecurityEventType::LoginSuccess
        } else {
            SecurityEventType::LoginFailure
        };

        let mut event = SecurityEvent::new(event_type).with_ip(ip.to_string());

        if let Some(uid) = user_id {
            event = event.with_user(uid);
        }

        if let Some(ua) = user_agent {
            event = event.with_user_agent(ua.to_string());
        }

        if let Some(loc) = location.clone() {
            event = event.with_location(loc);
        }

        self.record_event(event).await;

        if !success && failed_count >= self.config.brute_force_threshold {
            return self.handle_brute_force(&key, ip, user_id).await;
        }

        if success {
            if let Some(uid) = user_id {
                self.check_login_anomalies(uid, ip, location).await;
            }
        }

        None
    }

    async fn handle_brute_force(
        &self,
        key: &str,
        ip: &str,
        user_id: Option<Uuid>,
    ) -> Option<SecurityAlert> {
        let lockout = LockoutRecord {
            identifier: key.to_string(),
            locked_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(self.config.lockout_duration_minutes),
            reason: "Brute force attack detected".to_string(),
            attempt_count: self.config.brute_force_threshold,
        };

        {
            let mut lockouts = self.lockouts.write().await;
            lockouts.insert(key.to_string(), lockout);
        }

        {
            let mut blocked = self.blocked_ips.write().await;
            blocked.insert(
                ip.to_string(),
                Utc::now() + Duration::minutes(self.config.lockout_duration_minutes),
            );
        }

        if let Some(uid) = user_id {
            let mut profiles = self.user_profiles.write().await;
            let profile = profiles
                .entry(uid)
                .or_insert_with(|| UserSecurityProfile::new(uid));
            profile.lock(Duration::minutes(self.config.lockout_duration_minutes));
        }

        let mut event = SecurityEvent::new(SecurityEventType::BruteForceDetected)
            .with_ip(ip.to_string())
            .with_detail(
                "threshold",
                serde_json::json!(self.config.brute_force_threshold),
            );

        if let Some(uid) = user_id {
            event = event.with_user(uid);
        }

        self.record_event(event.clone()).await;

        warn!(
            "Brute force attack detected for {} from IP {}",
            key, ip
        );

        let alert = SecurityAlert::new(
            AlertSeverity::Critical,
            "Brute Force Attack Detected",
            &format!(
                "Multiple failed login attempts detected for {}. Account locked for {} minutes.",
                key, self.config.lockout_duration_minutes
            ),
        )
        .with_event(event.id)
        .with_ip(ip.to_string());

        let alert_with_user = if let Some(uid) = user_id {
            alert.with_user(uid)
        } else {
            alert
        };

        let mut alerts = self.alerts.write().await;
        alerts.push(alert_with_user.clone());

        Some(alert_with_user)
    }

    async fn check_login_anomalies(
        &self,
        user_id: Uuid,
        ip: &str,
        location: Option<GeoLocation>,
    ) {
        if !self.config.anomaly_detection_enabled {
            return;
        }

        let mut profiles = self.user_profiles.write().await;
        let profile = profiles
            .entry(user_id)
            .or_insert_with(|| UserSecurityProfile::new(user_id));

        let is_new_ip = !profile.is_known_ip(ip);
        let is_new_device = false;

        if is_new_ip {
            let event = SecurityEvent::new(SecurityEventType::NewDeviceLogin)
                .with_user(user_id)
                .with_ip(ip.to_string())
                .with_detail("reason", serde_json::json!("new_ip"));

            drop(profiles);
            self.record_event(event).await;
            profiles = self.user_profiles.write().await;
            let profile = profiles
                .entry(user_id)
                .or_insert_with(|| UserSecurityProfile::new(user_id));
            profile.add_known_ip(ip);
        }

        if self.config.impossible_travel_detection {
            if let (Some(last_loc), Some(current_loc)) =
                (profile.last_location.as_ref(), location.as_ref())
            {
                if let Some(last_login) = profile.last_login {
                    if let Some(distance) = last_loc.distance_km(current_loc) {
                        let time_diff = (Utc::now() - last_login).num_hours().max(1) as f64;
                        let speed = distance / time_diff;

                        if speed > self.config.max_travel_speed_kmh {
                            let event = SecurityEvent::new(SecurityEventType::ImpossibleTravel)
                                .with_user(user_id)
                                .with_ip(ip.to_string())
                                .with_location(current_loc.clone())
                                .with_detail("distance_km", serde_json::json!(distance))
                                .with_detail("speed_kmh", serde_json::json!(speed));

                            drop(profiles);
                            self.record_event(event).await;

                            warn!(
                                "Impossible travel detected for user {}: {} km in {} hours",
                                user_id, distance, time_diff
                            );
                            return;
                        }
                    }
                }
            }
        }

        if self.config.geo_anomaly_detection {
            if let Some(current_loc) = location.as_ref() {
                if let Some(ref country) = current_loc.country {
                    let known_countries: Vec<String> = profile
                        .known_locations
                        .iter()
                        .filter_map(|l| l.country.clone())
                        .collect();

                    if !known_countries.is_empty() && !known_countries.contains(country) {
                        let event = SecurityEvent::new(SecurityEventType::GeoAnomalyDetected)
                            .with_user(user_id)
                            .with_ip(ip.to_string())
                            .with_location(current_loc.clone())
                            .with_detail("new_country", serde_json::json!(country));

                        drop(profiles);
                        self.record_event(event).await;
                        return;
                    }
                }
            }
        }

        let profile = profiles
            .entry(user_id)
            .or_insert_with(|| UserSecurityProfile::new(user_id));
        profile.record_login(location);
    }

    pub async fn is_locked(&self, identifier: &str) -> bool {
        let lockouts = self.lockouts.read().await;
        if let Some(lockout) = lockouts.get(identifier) {
            !lockout.is_expired()
        } else {
            false
        }
    }

    pub async fn is_ip_blocked(&self, ip: &str) -> bool {
        let blocked = self.blocked_ips.read().await;
        if let Some(expires) = blocked.get(ip) {
            Utc::now() < *expires
        } else {
            false
        }
    }

    pub async fn get_lockout_info(&self, identifier: &str) -> Option<LockoutRecord> {
        let lockouts = self.lockouts.read().await;
        lockouts.get(identifier).cloned()
    }

    pub async fn unlock(&self, identifier: &str) -> bool {
        let mut lockouts = self.lockouts.write().await;
        lockouts.remove(identifier).is_some()
    }

    pub async fn unblock_ip(&self, ip: &str) -> bool {
        let mut blocked = self.blocked_ips.write().await;
        blocked.remove(ip).is_some()
    }

    pub async fn block_ip(&self, ip: &str, duration: Duration, reason: &str) {
        let mut blocked = self.blocked_ips.write().await;
        blocked.insert(ip.to_string(), Utc::now() + duration);

        let event = SecurityEvent::new(SecurityEventType::IpBlocked)
            .with_ip(ip.to_string())
            .with_detail("reason", serde_json::json!(reason))
            .with_detail("duration_minutes", serde_json::json!(duration.num_minutes()));

        drop(blocked);
        self.record_event(event).await;

        info!("IP {} blocked for {} minutes: {}", ip, duration.num_minutes(), reason);
    }

    async fn create_alert_from_event(&self, event: &SecurityEvent) {
        let alert = SecurityAlert::new(
            event.severity,
            &format!("Security Event: {}", event.event_type.as_str()),
            &format!(
                "{} event detected{}{}",
                event.event_type.as_str(),
                event
                    .user_id
                    .map(|id| format!(" for user {}", id))
                    .unwrap_or_default(),
                event
                    .ip_address
                    .as_ref()
                    .map(|ip| format!(" from IP {}", ip))
                    .unwrap_or_default()
            ),
        )
        .with_event(event.id);

        let alert_with_user = if let Some(uid) = event.user_id {
            alert.with_user(uid)
        } else {
            alert
        };

        let alert_with_ip = if let Some(ref ip) = event.ip_address {
            alert_with_user.with_ip(ip.clone())
        } else {
            alert_with_user
        };

        let mut alerts = self.alerts.write().await;
        alerts.push(alert_with_ip);
    }

    pub async fn get_alerts(&self, unacknowledged_only: bool, limit: usize) -> Vec<SecurityAlert> {
        let alerts = self.alerts.read().await;

        let filtered: Vec<SecurityAlert> = if unacknowledged_only {
            alerts.iter().filter(|a| !a.acknowledged).cloned().collect()
        } else {
            alerts.clone()
        };

        filtered.into_iter().rev().take(limit).collect()
    }

    pub async fn acknowledge_alert(&self, alert_id: Uuid, by: Uuid) -> bool {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledge(by);
            true
        } else {
            false
        }
    }

    pub async fn resolve_alert(&self, alert_id: Uuid) -> bool {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolve();
            true
        } else {
            false
        }
    }

    pub async fn get_user_profile(&self, user_id: Uuid) -> Option<UserSecurityProfile> {
        let profiles = self.user_profiles.read().await;
        profiles.get(&user_id).cloned()
    }

    pub async fn get_recent_events(
        &self,
        event_type: Option<SecurityEventType>,
        user_id: Option<Uuid>,
        limit: usize,
    ) -> Vec<SecurityEvent> {
        let events = self.events.read().await;

        let filtered: Vec<SecurityEvent> = events
            .iter()
            .filter(|e| {
                if let Some(et) = event_type {
                    if e.event_type != et {
                        return false;
                    }
                }
                if let Some(uid) = user_id {
                    if e.user_id != Some(uid) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        filtered.into_iter().rev().take(limit).collect()
    }

    pub async fn cleanup_old_data(&self) -> usize {
        let cutoff = Utc::now() - Duration::hours(self.config.retention_hours as i64);
        let mut total_cleaned = 0;

        {
            let mut events = self.events.write().await;
            let initial = events.len();
            events.retain(|e| e.timestamp > cutoff);
            total_cleaned += initial - events.len();
        }

        {
            let mut attempts = self.login_attempts.write().await;
            for records in attempts.values_mut() {
                let initial = records.len();
                records.retain(|r| r.timestamp > cutoff);
                total_cleaned += initial - records.len();
            }
        }

        {
            let mut lockouts = self.lockouts.write().await;
            let initial = lockouts.len();
            lockouts.retain(|_, l| !l.is_expired());
            total_cleaned += initial - lockouts.len();
        }

        {
            let mut blocked = self.blocked_ips.write().await;
            let initial = blocked.len();
            blocked.retain(|_, expires| Utc::now() < *expires);
            total_cleaned += initial - blocked.len();
        }

        if total_cleaned > 0 {
            info!("Cleaned up {} old security monitoring records", total_cleaned);
        }

        total_cleaned
    }

    pub fn config(&self) -> &SecurityMonitoringConfig {
        &self.config
    }
}

fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let lat1_rad = lat1.to_radians();
    let lat2_rad = lat2.to_radians();
    let delta_lat = (lat2 - lat1).to_radians();
    let delta_lon = (lon2 - lon1).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    EARTH_RADIUS_KM * c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_record_login_success() {
        let monitor = SecurityMonitor::with_defaults();
        let user_id = Uuid::new_v4();

        let alert = monitor
            .record_login_attempt(Some(user_id), "192.168.1.1", true, Some("TestAgent"), None)
            .await;

        assert!(alert.is_none());

        let events = monitor.get_recent_events(Some(SecurityEventType::LoginSuccess), None, 10).await;
        assert!(!events.is_empty());
    }

    #[tokio::test]
    async fn test_brute_force_detection() {
        let mut config = SecurityMonitoringConfig::default();
        config.brute_force_threshold = 3;
        let monitor = SecurityMonitor::new(config);
        let user_id = Uuid::new_v4();

        for _ in 0..2 {
            let alert = monitor
                .record_login_attempt(Some(user_id), "10.0.0.1", false, None, None)
                .await;
            assert!(alert.is_none());
        }

        let alert = monitor
            .record_login_attempt(Some(user_id), "10.0.0.1", false, None, None)
            .await;

        assert!(alert.is_some());
        assert_eq!(alert.unwrap().severity, AlertSeverity::Critical);
    }

    #[tokio::test]
    async fn test_lockout() {
        let mut config = SecurityMonitoringConfig::default();
        config.brute_force_threshold = 2;
        let monitor = SecurityMonitor::new(config);

        let user_id = Uuid::new_v4();
        let identifier = user_id.to_string();

        monitor
            .record_login_attempt(Some(user_id), "10.0.0.1", false, None, None)
            .await;
        monitor
            .record_login_attempt(Some(user_id), "10.0.0.1", false, None, None)
            .await;

        assert!(monitor.is_locked(&identifier).await);

        monitor.unlock(&identifier).await;
        assert!(!monitor.is_locked(&identifier).await);
    }

    #[tokio::test]
    async fn test_ip_blocking() {
        let monitor = SecurityMonitor::with_defaults();

        monitor
            .block_ip("1.2.3.4", Duration::minutes(30), "Test block")
            .await;

        assert!(monitor.is_ip_blocked("1.2.3.4").await);
        assert!(!monitor.is_ip_blocked("5.6.7.8").await);

        monitor.unblock_ip("1.2.3.4").await;
        assert!(!monitor.is_ip_blocked("1.2.3.4").await);
    }

    #[test]
    fn test_security_event_creation() {
        let event = SecurityEvent::new(SecurityEventType::LoginFailure)
            .with_user(Uuid::new_v4())
            .with_ip("192.168.1.1".into())
            .with_detail("reason", serde_json::json!("invalid_password"));

        assert_eq!(event.event_type, SecurityEventType::LoginFailure);
        assert!(event.user_id.is_some());
        assert!(event.ip_address.is_some());
    }

    #[test]
    fn test_alert_creation() {
        let mut alert = SecurityAlert::new(
            AlertSeverity::High,
            "Test Alert",
            "Test description",
        );

        assert!(!alert.acknowledged);
        assert!(!alert.resolved);

        alert.acknowledge(Uuid::new_v4());
        assert!(alert.acknowledged);

        alert.resolve();
        assert!(alert.resolved);
    }

    #[test]
    fn test_geo_location_distance() {
        let loc1 = GeoLocation::new()
            .with_coordinates(40.7128, -74.0060);
        let loc2 = GeoLocation::new()
            .with_coordinates(51.5074, -0.1278);

        let distance = loc1.distance_km(&loc2).unwrap();
        assert!(distance > 5500.0 && distance < 5600.0);
    }

    #[test]
    fn test_haversine_distance() {
        let distance = haversine_distance(40.7128, -74.0060, 51.5074, -0.1278);
        assert!(distance > 5500.0 && distance < 5600.0);
    }

    #[test]
    fn test_user_security_profile() {
        let user_id = Uuid::new_v4();
        let mut profile = UserSecurityProfile::new(user_id);

        assert!(!profile.is_known_ip("192.168.1.1"));

        profile.add_known_ip("192.168.1.1");
        assert!(profile.is_known_ip("192.168.1.1"));

        profile.lock(Duration::minutes(30));
        assert!(profile.check_lock_status());

        profile.unlock();
        assert!(!profile.check_lock_status());
    }

    #[test]
    fn test_lockout_record() {
        let lockout = LockoutRecord {
            identifier: "test".into(),
            locked_at: Utc::now(),
            expires_at: Utc::now() + Duration::minutes(30),
            reason: "Test".into(),
            attempt_count: 5,
        };

        assert!(!lockout.is_expired());
        assert!(lockout.remaining_time() > Duration::zero());
    }

    #[test]
    fn test_event_severity_mapping() {
        assert_eq!(
            SecurityEventType::BruteForceDetected.severity(),
            AlertSeverity::Critical
        );
        assert_eq!(
            SecurityEventType::LoginFailure.severity(),
            AlertSeverity::High
        );
        assert_eq!(
            SecurityEventType::LoginSuccess.severity(),
            AlertSeverity::Low
        );
    }

    #[tokio::test]
    async fn test_alert_acknowledgment() {
        let monitor = SecurityMonitor::with_defaults();

        let event = SecurityEvent::new(SecurityEventType::BruteForceDetected);
        monitor.record_event(event).await;

        let alerts = monitor.get_alerts(true, 10).await;
        assert!(!alerts.is_empty());

        let alert_id = alerts[0].id;
        let admin_id = Uuid::new_v4();

        assert!(monitor.acknowledge_alert(alert_id, admin_id).await);

        let unack_alerts = monitor.get_alerts(true, 10).await;
        assert!(unack_alerts.iter().all(|a| a.id != alert_id));
    }

    #[tokio::test]
    async fn test_cleanup() {
        let mut config = SecurityMonitoringConfig::default();
        config.retention_hours = 0;
        let monitor = SecurityMonitor::new(config);

        let event = SecurityEvent::new(SecurityEventType::LoginSuccess);
        monitor.record_event(event).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let cleaned = monitor.cleanup_old_data().await;
        assert!(cleaned >= 1);
    }

    #[test]
    fn test_alert_severity_score() {
        assert_eq!(AlertSeverity::Low.score(), 25);
        assert_eq!(AlertSeverity::Medium.score(), 50);
        assert_eq!(AlertSeverity::High.score(), 75);
        assert_eq!(AlertSeverity::Critical.score(), 100);
    }
}
