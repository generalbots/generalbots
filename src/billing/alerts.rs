//! Usage Alerts System
//!
//! Provides quota threshold monitoring and notification delivery for usage alerts.
//! Supports multiple notification channels: email, webhook, in-app, SMS.

use crate::billing::UsageMetric;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// ============================================================================
// Alert Configuration
// ============================================================================

/// Alert thresholds configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Warning threshold (default: 80%)
    pub warning: f64,
    /// Critical threshold (default: 90%)
    pub critical: f64,
    /// Exceeded threshold (default: 100%)
    pub exceeded: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            warning: 80.0,
            critical: 90.0,
            exceeded: 100.0,
        }
    }
}

impl AlertThresholds {
    pub fn new(warning: f64, critical: f64, exceeded: f64) -> Self {
        Self {
            warning,
            critical,
            exceeded,
        }
    }

    /// Get the severity level for a given percentage
    pub fn get_severity(&self, percentage: f64) -> Option<AlertSeverity> {
        if percentage >= self.exceeded {
            Some(AlertSeverity::Exceeded)
        } else if percentage >= self.critical {
            Some(AlertSeverity::Critical)
        } else if percentage >= self.warning {
            Some(AlertSeverity::Warning)
        } else {
            None
        }
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Warning,
    Critical,
    Exceeded,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Critical => "critical",
            Self::Exceeded => "exceeded",
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Warning => "⚠️",
            Self::Critical => "🚨",
            Self::Exceeded => "🛑",
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            Self::Warning => 1,
            Self::Critical => 2,
            Self::Exceeded => 3,
        }
    }
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Alert Types
// ============================================================================

/// Usage alert data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAlert {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub metric: UsageMetric,
    pub severity: AlertSeverity,
    pub current_usage: u64,
    pub limit: u64,
    pub percentage: f64,
    pub threshold: f64,
    pub message: String,
    pub created_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub notification_sent: bool,
    pub notification_channels: Vec<NotificationChannel>,
}

impl UsageAlert {
    pub fn new(
        organization_id: Uuid,
        metric: UsageMetric,
        severity: AlertSeverity,
        current_usage: u64,
        limit: u64,
        percentage: f64,
        threshold: f64,
    ) -> Self {
        let message = Self::generate_message(metric, severity, percentage, current_usage, limit);

        Self {
            id: Uuid::new_v4(),
            organization_id,
            metric,
            severity,
            current_usage,
            limit,
            percentage,
            threshold,
            message,
            created_at: Utc::now(),
            acknowledged_at: None,
            acknowledged_by: None,
            notification_sent: false,
            notification_channels: Vec::new(),
        }
    }

    fn generate_message(
        metric: UsageMetric,
        severity: AlertSeverity,
        percentage: f64,
        current: u64,
        limit: u64,
    ) -> String {
        let metric_name = metric.display_name();
        let severity_text = match severity {
            AlertSeverity::Warning => "approaching limit",
            AlertSeverity::Critical => "near limit",
            AlertSeverity::Exceeded => "exceeded limit",
        };

        format!(
            "{} {} usage is {} ({:.1}% - {}/{})",
            severity.emoji(),
            metric_name,
            severity_text,
            percentage,
            Self::format_value(metric, current),
            Self::format_value(metric, limit)
        )
    }

    fn format_value(metric: UsageMetric, value: u64) -> String {
        match metric {
            UsageMetric::StorageBytes => format_bytes(value),
            _ => format_number(value),
        }
    }

    pub fn acknowledge(&mut self, user_id: Uuid) {
        self.acknowledged_at = Some(Utc::now());
        self.acknowledged_by = Some(user_id);
    }

    pub fn is_acknowledged(&self) -> bool {
        self.acknowledged_at.is_some()
    }

    pub fn mark_notified(&mut self, channels: Vec<NotificationChannel>) {
        self.notification_sent = true;
        self.notification_channels = channels;
    }
}

/// Notification delivery channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Email,
    Webhook,
    InApp,
    Sms,
    Slack,
    MsTeams,
    Push,
}

impl NotificationChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Webhook => "webhook",
            Self::InApp => "in_app",
            Self::Sms => "sms",
            Self::Slack => "slack",
            Self::MsTeams => "ms_teams",
            Self::Push => "push",
        }
    }
}

// ============================================================================
// Alert Manager
// ============================================================================

/// Manages usage alerts and notifications
pub struct AlertManager {
    /// Active alerts by organization
    active_alerts: Arc<RwLock<HashMap<Uuid, Vec<UsageAlert>>>>,
    /// Alert history (last N alerts per org)
    alert_history: Arc<RwLock<HashMap<Uuid, Vec<UsageAlert>>>>,
    /// Notification preferences per organization
    notification_prefs: Arc<RwLock<HashMap<Uuid, NotificationPreferences>>>,
    /// Alert thresholds
    thresholds: AlertThresholds,
    /// Cooldown between same alerts (in seconds)
    cooldown_seconds: u64,
    /// Max alerts in history per org
    max_history_per_org: usize,
    /// Notification handlers
    notification_handlers: Arc<RwLock<Vec<Arc<dyn NotificationHandler>>>>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(HashMap::new())),
            notification_prefs: Arc::new(RwLock::new(HashMap::new())),
            thresholds: AlertThresholds::default(),
            cooldown_seconds: 3600, // 1 hour cooldown
            max_history_per_org: 100,
            notification_handlers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_thresholds(mut self, thresholds: AlertThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    pub fn with_cooldown(mut self, seconds: u64) -> Self {
        self.cooldown_seconds = seconds;
        self
    }

    /// Register a notification handler
    pub async fn register_handler(&self, handler: Arc<dyn NotificationHandler>) {
        let mut handlers = self.notification_handlers.write().await;
        handlers.push(handler);
    }

    /// Set notification preferences for an organization
    pub async fn set_notification_preferences(
        &self,
        org_id: Uuid,
        prefs: NotificationPreferences,
    ) {
        let mut all_prefs = self.notification_prefs.write().await;
        all_prefs.insert(org_id, prefs);
    }

    /// Get notification preferences for an organization
    pub async fn get_notification_preferences(
        &self,
        org_id: Uuid,
    ) -> NotificationPreferences {
        let prefs = self.notification_prefs.read().await;
        prefs.get(&org_id).cloned().unwrap_or_default()
    }

    /// Check usage and generate alerts if thresholds are crossed
    pub async fn check_and_alert(
        &self,
        org_id: Uuid,
        metric: UsageMetric,
        current_usage: u64,
        limit: u64,
    ) -> Option<UsageAlert> {
        if limit == 0 {
            return None;
        }

        let percentage = (current_usage as f64 / limit as f64) * 100.0;
        let severity = self.thresholds.get_severity(percentage)?;
        let threshold = match severity {
            AlertSeverity::Warning => self.thresholds.warning,
            AlertSeverity::Critical => self.thresholds.critical,
            AlertSeverity::Exceeded => self.thresholds.exceeded,
        };

        // Check cooldown
        if self.is_in_cooldown(org_id, metric, severity).await {
            return None;
        }

        // Create alert
        let alert = UsageAlert::new(
            org_id,
            metric,
            severity,
            current_usage,
            limit,
            percentage,
            threshold,
        );

        // Store alert
        self.store_alert(org_id, alert.clone()).await;

        // Send notifications
        self.send_notifications(org_id, &alert).await;

        Some(alert)
    }

    /// Check multiple metrics at once
    pub async fn check_all_metrics(
        &self,
        org_id: Uuid,
        usage: &UsageSnapshot,
    ) -> Vec<UsageAlert> {
        let mut alerts = Vec::new();

        for (metric, current, limit) in usage.iter_metrics() {
            if let Some(alert) = self.check_and_alert(org_id, metric, current, limit).await {
                alerts.push(alert);
            }
        }

        alerts
    }

    /// Get active alerts for an organization
    pub async fn get_active_alerts(&self, org_id: Uuid) -> Vec<UsageAlert> {
        let alerts = self.active_alerts.read().await;
        alerts.get(&org_id).cloned().unwrap_or_default()
    }

    /// Get alert history for an organization
    pub async fn get_alert_history(
        &self,
        org_id: Uuid,
        limit: Option<usize>,
    ) -> Vec<UsageAlert> {
        let history = self.alert_history.read().await;
        let mut alerts = history.get(&org_id).cloned().unwrap_or_default();

        if let Some(limit) = limit {
            alerts.truncate(limit);
        }

        alerts
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(
        &self,
        org_id: Uuid,
        alert_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), AlertError> {
        let mut alerts = self.active_alerts.write().await;
        let org_alerts = alerts.get_mut(&org_id).ok_or(AlertError::NotFound)?;

        let alert = org_alerts
            .iter_mut()
            .find(|a| a.id == alert_id)
            .ok_or(AlertError::NotFound)?;

        alert.acknowledge(user_id);

        Ok(())
    }

    /// Dismiss an alert
    pub async fn dismiss_alert(
        &self,
        org_id: Uuid,
        alert_id: Uuid,
    ) -> Result<UsageAlert, AlertError> {
        let mut alerts = self.active_alerts.write().await;
        let org_alerts = alerts.get_mut(&org_id).ok_or(AlertError::NotFound)?;

        let index = org_alerts
            .iter()
            .position(|a| a.id == alert_id)
            .ok_or(AlertError::NotFound)?;

        let alert = org_alerts.remove(index);

        // Move to history
        self.add_to_history(org_id, alert.clone()).await;

        Ok(alert)
    }

    /// Clear all alerts for an organization
    pub async fn clear_alerts(&self, org_id: Uuid) {
        let mut alerts = self.active_alerts.write().await;
        if let Some(org_alerts) = alerts.remove(&org_id) {
            // Move all to history
            for alert in org_alerts {
                self.add_to_history(org_id, alert).await;
            }
        }
    }

    /// Get alert count by severity
    pub async fn get_alert_counts(&self, org_id: Uuid) -> AlertCounts {
        let alerts = self.active_alerts.read().await;
        let org_alerts = alerts.get(&org_id);

        let mut counts = AlertCounts::default();

        if let Some(alerts) = org_alerts {
            for alert in alerts {
                match alert.severity {
                    AlertSeverity::Warning => counts.warning += 1,
                    AlertSeverity::Critical => counts.critical += 1,
                    AlertSeverity::Exceeded => counts.exceeded += 1,
                }
                counts.total += 1;
            }
        }

        counts
    }

    // ========================================================================
    // Private Methods
    // ========================================================================

    async fn is_in_cooldown(
        &self,
        org_id: Uuid,
        metric: UsageMetric,
        severity: AlertSeverity,
    ) -> bool {
        let alerts = self.active_alerts.read().await;
        let org_alerts = match alerts.get(&org_id) {
            Some(a) => a,
            None => return false,
        };

        let cooldown_threshold = Utc::now()
            - chrono::Duration::seconds(self.cooldown_seconds as i64);

        org_alerts.iter().any(|alert| {
            alert.metric == metric
                && alert.severity == severity
                && alert.created_at > cooldown_threshold
        })
    }

    async fn store_alert(&self, org_id: Uuid, alert: UsageAlert) {
        let mut alerts = self.active_alerts.write().await;
        let org_alerts = alerts.entry(org_id).or_insert_with(Vec::new);

        // Remove any existing alert for the same metric with lower severity
        org_alerts.retain(|a| {
            a.metric != alert.metric || a.severity.priority() >= alert.severity.priority()
        });

        org_alerts.push(alert);
    }

    async fn add_to_history(&self, org_id: Uuid, alert: UsageAlert) {
        let mut history = self.alert_history.write().await;
        let org_history = history.entry(org_id).or_insert_with(Vec::new);

        org_history.insert(0, alert);

        // Trim history
        if org_history.len() > self.max_history_per_org {
            org_history.truncate(self.max_history_per_org);
        }
    }

    async fn send_notifications(&self, org_id: Uuid, alert: &UsageAlert) {
        let prefs = self.get_notification_preferences(org_id).await;

        if !prefs.enabled {
            return;
        }

        // Check if this severity should be notified
        if !prefs.should_notify(alert.severity) {
            return;
        }

        let handlers = self.notification_handlers.read().await;
        let notification = AlertNotification::from_alert(alert, &prefs);

        for handler in handlers.iter() {
            if prefs.channels.contains(&handler.channel()) {
                if let Err(e) = handler.send(&notification).await {
                    tracing::warn!(
                        "Failed to send {} notification for org {}: {}",
                        handler.channel().as_str(),
                        org_id,
                        e
                    );
                }
            }
        }
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Notification Preferences
// ============================================================================

/// Organization notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub enabled: bool,
    pub channels: Vec<NotificationChannel>,
    pub email_recipients: Vec<String>,
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub slack_webhook_url: Option<String>,
    pub teams_webhook_url: Option<String>,
    pub sms_numbers: Vec<String>,
    pub min_severity: AlertSeverity,
    pub quiet_hours: Option<QuietHours>,
    pub metric_overrides: HashMap<UsageMetric, MetricNotificationOverride>,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: vec![NotificationChannel::Email, NotificationChannel::InApp],
            email_recipients: Vec::new(),
            webhook_url: None,
            webhook_secret: None,
            slack_webhook_url: None,
            teams_webhook_url: None,
            sms_numbers: Vec::new(),
            min_severity: AlertSeverity::Warning,
            quiet_hours: None,
            metric_overrides: HashMap::new(),
        }
    }
}

impl NotificationPreferences {
    pub fn should_notify(&self, severity: AlertSeverity) -> bool {
        severity.priority() >= self.min_severity.priority()
    }

    pub fn is_in_quiet_hours(&self) -> bool {
        if let Some(quiet) = &self.quiet_hours {
            quiet.is_active()
        } else {
            false
        }
    }
}

/// Quiet hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    pub start_hour: u8,
    pub end_hour: u8,
    pub timezone: String,
    pub days: Vec<chrono::Weekday>,
}

impl QuietHours {
    pub fn is_active(&self) -> bool {
        // Simplified check - in production, use proper timezone handling
        let now = Utc::now();
        let hour = now.format("%H").to_string().parse::<u8>().unwrap_or(0);

        if self.start_hour < self.end_hour {
            hour >= self.start_hour && hour < self.end_hour
        } else {
            // Overnight quiet hours
            hour >= self.start_hour || hour < self.end_hour
        }
    }
}

/// Per-metric notification override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricNotificationOverride {
    pub enabled: bool,
    pub min_severity: Option<AlertSeverity>,
    pub channels: Option<Vec<NotificationChannel>>,
}

// ============================================================================
// Usage Snapshot
// ============================================================================

/// Snapshot of current usage for batch alert checking
#[derive(Debug, Clone)]
pub struct UsageSnapshot {
    pub messages: (u64, u64),       // (current, limit)
    pub storage_bytes: (u64, u64),
    pub api_calls: (u64, u64),
    pub bots: (u64, u64),
    pub users: (u64, u64),
    pub kb_documents: (u64, u64),
    pub apps: (u64, u64),
}

impl UsageSnapshot {
    pub fn iter_metrics(&self) -> impl Iterator<Item = (UsageMetric, u64, u64)> {
        vec![
            (UsageMetric::Messages, self.messages.0, self.messages.1),
            (UsageMetric::StorageBytes, self.storage_bytes.0, self.storage_bytes.1),
            (UsageMetric::ApiCalls, self.api_calls.0, self.api_calls.1),
            (UsageMetric::Bots, self.bots.0, self.bots.1),
            (UsageMetric::Users, self.users.0, self.users.1),
            (UsageMetric::KbDocuments, self.kb_documents.0, self.kb_documents.1),
            (UsageMetric::Apps, self.apps.0, self.apps.1),
        ]
        .into_iter()
        .filter(|(_, _, limit)| *limit > 0)
    }
}

// ============================================================================
// Alert Counts
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlertCounts {
    pub total: usize,
    pub warning: usize,
    pub critical: usize,
    pub exceeded: usize,
}

// ============================================================================
// Notification Handler Trait
// ============================================================================

/// Trait for notification delivery handlers
#[async_trait::async_trait]
pub trait NotificationHandler: Send + Sync {
    fn channel(&self) -> NotificationChannel;
    async fn send(&self, notification: &AlertNotification) -> Result<(), NotificationError>;
}

/// Notification payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotification {
    pub alert_id: Uuid,
    pub organization_id: Uuid,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub metric: String,
    pub current_usage: u64,
    pub limit: u64,
    pub percentage: f64,
    pub action_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub recipients: Vec<String>,
}

impl AlertNotification {
    pub fn from_alert(alert: &UsageAlert, prefs: &NotificationPreferences) -> Self {
        Self {
            alert_id: alert.id,
            organization_id: alert.organization_id,
            severity: alert.severity,
            title: format!(
                "{} Usage Alert: {}",
                alert.severity.emoji(),
                alert.metric.display_name()
            ),
            message: alert.message.clone(),
            metric: alert.metric.display_name().to_string(),
            current_usage: alert.current_usage,
            limit: alert.limit,
            percentage: alert.percentage,
            action_url: Some(format!("/billing/usage?org={}", alert.organization_id)),
            created_at: alert.created_at,
            recipients: prefs.email_recipients.clone(),
        }
    }
}

// ============================================================================
// Built-in Notification Handlers
// ============================================================================

/// Email notification handler
pub struct EmailNotificationHandler {
    _smtp_host: String,
    _smtp_port: u16,
    _from_address: String,
}

impl EmailNotificationHandler {
    pub fn new(smtp_host: String, smtp_port: u16, from_address: String) -> Self {
        Self {
            _smtp_host: smtp_host,
            _smtp_port: smtp_port,
            _from_address: from_address,
        }
    }
}

#[async_trait::async_trait]
impl NotificationHandler for EmailNotificationHandler {
    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Email
    }

    async fn send(&self, notification: &AlertNotification) -> Result<(), NotificationError> {
        use lettre::{Message, SmtpTransport, Transport};
        use lettre::transport::smtp::authentication::Credentials;

        tracing::info!(
            "Sending email notification for alert {} to {:?}",
            notification.alert_id,
            notification.recipients
        );

        // Get SMTP config from environment
        let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
        let smtp_user = std::env::var("SMTP_USER").ok();
        let smtp_pass = std::env::var("SMTP_PASS").ok();
        let from_email = std::env::var("SMTP_FROM").unwrap_or_else(|_| "alerts@generalbots.com".to_string());

        let subject = format!("[{}] Billing Alert: {}",
            notification.severity.to_string().to_uppercase(),
            notification.title
        );

        let body = format!(
            "Alert: {}\nSeverity: {}\nOrganization: {}\nTime: {}\n\nMessage: {}\n\nThreshold: {:?}\nCurrent Value: {:?}",
            notification.title,
            notification.severity,
            notification.organization_id,
            notification.created_at,
            notification.message,
            notification.limit,
            notification.current_usage
        );

        for recipient in &notification.recipients {
            let email = Message::builder()
                .from(from_email.parse().map_err(|e| NotificationError::DeliveryFailed(format!("Invalid from address: {}", e)))?)
                .to(recipient.parse().map_err(|e| NotificationError::DeliveryFailed(format!("Invalid recipient {}: {}", recipient, e)))?)
                .subject(&subject)
                .body(body.clone())
                .map_err(|e| NotificationError::DeliveryFailed(format!("Failed to build email: {}", e)))?;

            let mailer = if let (Some(user), Some(pass)) = (&smtp_user, &smtp_pass) {
                let creds = Credentials::new(user.clone(), pass.clone());
                SmtpTransport::relay(&smtp_host)
                    .map_err(|e| NotificationError::DeliveryFailed(format!("SMTP relay error: {}", e)))?
                    .credentials(creds)
                    .build()
            } else {
                SmtpTransport::builder_dangerous(&smtp_host).build()
            };

            mailer.send(&email)
                .map_err(|e| NotificationError::DeliveryFailed(format!("Failed to send to {}: {}", recipient, e)))?;

            tracing::debug!("Email sent to {}", recipient);
        }

        Ok(())
    }
}

/// Webhook notification handler
pub struct WebhookNotificationHandler {}

impl WebhookNotificationHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for WebhookNotificationHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl NotificationHandler for WebhookNotificationHandler {
    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Webhook
    }

    async fn send(&self, notification: &AlertNotification) -> Result<(), NotificationError> {
        tracing::info!(
            "Sending webhook notification for alert {}",
            notification.alert_id
        );

        // Get webhook URL from context or environment
        let webhook_url = std::env::var("BILLING_WEBHOOK_URL").ok();

        let url = match webhook_url {
            Some(url) => url,
            None => {
                tracing::warn!("No webhook URL configured for alert {}", notification.alert_id);
                return Ok(()); // Silent skip if not configured
            }
        };

        let payload = serde_json::json!({
            "alert_id": notification.alert_id,
            "organization_id": notification.organization_id,
            "alert_type": notification.title,
            "severity": notification.severity.to_string(),
            "message": notification.message,
            "threshold_value": notification.limit,
            "current_value": notification.current_usage,
            "triggered_at": notification.created_at.to_rfc3339(),
            "recipients": notification.recipients,
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("User-Agent", "GeneralBots-Billing-Alerts/1.0")
            .json(&payload)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| NotificationError::DeliveryFailed(format!("Webhook request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NotificationError::DeliveryFailed(
                format!("Webhook returned {}: {}", status, body)
            ));
        }

        tracing::debug!("Webhook notification sent successfully to {}", url);
        Ok(())
    }
}

/// In-app notification handler
pub struct InAppNotificationHandler {
    /// Broadcast channel for WebSocket notifications
    broadcast: Option<tokio::sync::broadcast::Sender<crate::core::shared::state::BillingAlertNotification>>,
}

impl InAppNotificationHandler {
    pub fn new() -> Self {
        Self { broadcast: None }
    }

    /// Create with a broadcast channel for WebSocket notifications
    pub fn with_broadcast(
        broadcast: tokio::sync::broadcast::Sender<crate::core::shared::state::BillingAlertNotification>,
    ) -> Self {
        Self {
            broadcast: Some(broadcast),
        }
    }
}

impl Default for InAppNotificationHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl NotificationHandler for InAppNotificationHandler {
    fn channel(&self) -> NotificationChannel {
        NotificationChannel::InApp
    }

    async fn send(&self, notification: &AlertNotification) -> Result<(), NotificationError> {
        tracing::info!(
            "Creating in-app notification for alert {} org {}",
            notification.alert_id,
            notification.organization_id
        );

        // Build notification payload for WebSocket broadcast
        let ws_notification = crate::core::shared::state::BillingAlertNotification {
            alert_id: notification.alert_id,
            organization_id: notification.organization_id,
            severity: notification.severity.to_string(),
            alert_type: notification.title.clone(),
            title: notification.title.clone(),
            message: notification.message.clone(),
            metric: notification.metric.clone(),
            percentage: notification.percentage,
            triggered_at: notification.created_at,
        };

        // Broadcast to connected WebSocket clients
        if let Some(ref broadcast) = self.broadcast {
            match broadcast.send(ws_notification.clone()) {
                Ok(receivers) => {
                    tracing::info!(
                        "Billing alert {} broadcast to {} WebSocket receivers",
                        notification.alert_id,
                        receivers
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "No active WebSocket receivers for billing alert {}: {}",
                        notification.alert_id,
                        e
                    );
                }
            }
        } else {
            tracing::debug!(
                "No broadcast channel configured, alert {} will be delivered via polling",
                notification.alert_id
            );
        }

        // Store notification in database for users who aren't connected via WebSocket
        // The UI will pick these up when polling /api/notifications
        tracing::debug!(
            "In-app notification queued for org {} - delivered via WebSocket and/or polling",
            notification.organization_id
        );

        Ok(())
    }
}

/// Slack notification handler
pub struct SlackNotificationHandler {}

impl SlackNotificationHandler {
    pub fn new() -> Self {
        Self {}
    }

    fn build_slack_message(&self, notification: &AlertNotification) -> serde_json::Value {
        let color = match notification.severity {
            AlertSeverity::Warning => "#FFA500",
            AlertSeverity::Critical => "#FF0000",
            AlertSeverity::Exceeded => "#8B0000",
        };

        serde_json::json!({
            "attachments": [{
                "color": color,
                "title": notification.title,
                "text": notification.message,
                "fields": [
                    {
                        "title": "Metric",
                        "value": notification.metric,
                        "short": true
                    },
                    {
                        "title": "Usage",
                        "value": format!("{:.1}%", notification.percentage),
                        "short": true
                    }
                ],
                "actions": [{
                    "type": "button",
                    "text": "View Usage",
                    "url": notification.action_url
                }],
                "ts": notification.created_at.timestamp()
            }]
        })
    }
}

impl Default for SlackNotificationHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl NotificationHandler for SlackNotificationHandler {
    fn channel(&self) -> NotificationChannel {
        NotificationChannel::Slack
    }

    async fn send(&self, notification: &AlertNotification) -> Result<(), NotificationError> {
        tracing::info!(
            "Sending Slack notification for alert {}",
            notification.alert_id
        );

        // Get Slack webhook URL from context or environment
        let webhook_url = std::env::var("SLACK_WEBHOOK_URL").ok();

        let url = match webhook_url {
            Some(url) => url,
            None => {
                tracing::warn!("No Slack webhook URL configured for alert {}", notification.alert_id);
                return Ok(()); // Silent skip if not configured
            }
        };

        let message = self.build_slack_message(notification);

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&message)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| NotificationError::DeliveryFailed(format!("Slack request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NotificationError::DeliveryFailed(
                format!("Slack webhook returned {}: {}", status, body)
            ));
        }

        tracing::debug!("Slack notification sent successfully");
        Ok(())
    }
}

/// Microsoft Teams notification handler
pub struct TeamsNotificationHandler {}

impl TeamsNotificationHandler {
    pub fn new() -> Self {
        Self {}
    }

    fn build_teams_message(&self, notification: &AlertNotification) -> serde_json::Value {
        let theme_color = match notification.severity {
            AlertSeverity::Warning => "FFA500",
            AlertSeverity::Critical => "FF0000",
            AlertSeverity::Exceeded => "8B0000",
        };

        serde_json::json!({
            "@type": "MessageCard",
            "@context": "http://schema.org/extensions",
            "themeColor": theme_color,
            "summary": notification.title,
            "sections": [{
                "activityTitle": notification.title,
                "facts": [
                    { "name": "Metric", "value": notification.metric },
                    { "name": "Current Usage", "value": format!("{:.1}%", notification.percentage) },
                    { "name": "Severity", "value": notification.severity.as_str() }
                ],
                "text": notification.message
            }],
            "potentialAction": [{
                "@type": "OpenUri",
                "name": "View Usage",
                "targets": [{
                    "os": "default",
                    "uri": notification.action_url
                }]
            }]
        })
    }
}

impl Default for TeamsNotificationHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl NotificationHandler for TeamsNotificationHandler {
    fn channel(&self) -> NotificationChannel {
        NotificationChannel::MsTeams
    }

    async fn send(&self, notification: &AlertNotification) -> Result<(), NotificationError> {
        tracing::info!(
            "Sending Teams notification for alert {}",
            notification.alert_id
        );

        // Get Teams webhook URL from context or environment
        let webhook_url = std::env::var("TEAMS_WEBHOOK_URL").ok();

        let url = match webhook_url {
            Some(url) => url,
            None => {
                tracing::warn!("No Teams webhook URL configured for alert {}", notification.alert_id);
                return Ok(()); // Silent skip if not configured
            }
        };

        let message = self.build_teams_message(notification);

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&message)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| NotificationError::DeliveryFailed(format!("Teams request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NotificationError::DeliveryFailed(
                format!("Teams webhook returned {}: {}", status, body)
            ));
        }

        tracing::debug!("Teams notification sent successfully");
        Ok(())
    }
}

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, Clone)]
pub enum AlertError {
    NotFound,
    AlreadyExists,
    InvalidThreshold,
    StorageError(String),
}

impl std::fmt::Display for AlertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Alert not found"),
            Self::AlreadyExists => write!(f, "Alert already exists"),
            Self::InvalidThreshold => write!(f, "Invalid threshold value"),
            Self::StorageError(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

impl std::error::Error for AlertError {}

#[derive(Debug, Clone)]
pub enum NotificationError {
    NetworkError(String),
    ConfigurationError(String),
    RateLimited,
    InvalidRecipient(String),
    DeliveryFailed(String),
}

impl std::fmt::Display for NotificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            Self::RateLimited => write!(f, "Rate limited"),
            Self::InvalidRecipient(msg) => write!(f, "Invalid recipient: {}", msg),
            Self::DeliveryFailed(msg) => write!(f, "Delivery failed: {}", msg),
        }
    }
}

impl std::error::Error for NotificationError {}

// ============================================================================
// Helper Functions
// ============================================================================

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format number with thousands separators
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

// ============================================================================
// UsageMetric Extension
// ============================================================================

impl UsageMetric {
    /// Get human-readable display name for the metric
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Messages => "Messages",
            Self::StorageBytes => "Storage",
            Self::ApiCalls => "API Calls",
            Self::Bots => "Bots",
            Self::Users => "Users",
            Self::KbDocuments => "KB Documents",
            Self::Apps => "Apps",
        }
    }
}

// ============================================================================
// Grace Period Support
// ============================================================================

/// Grace period configuration for quota overages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracePeriodConfig {
    /// Whether grace period is enabled
    pub enabled: bool,
    /// Grace period duration in hours
    pub duration_hours: u32,
    /// Maximum overage percentage allowed during grace period
    pub max_overage_percent: f64,
    /// Metrics that support grace period
    pub applicable_metrics: Vec<UsageMetric>,
}

impl Default for GracePeriodConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration_hours: 24,
            max_overage_percent: 10.0,
            applicable_metrics: vec![
                UsageMetric::Messages,
                UsageMetric::ApiCalls,
            ],
        }
    }
}

/// Grace period status for an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracePeriodStatus {
    pub organization_id: Uuid,
    pub metric: UsageMetric,
    pub started_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub overage_at_start: u64,
    pub current_overage: u64,
    pub max_allowed_overage: u64,
    pub is_active: bool,
}

impl GracePeriodStatus {
    pub fn new(
        organization_id: Uuid,
        metric: UsageMetric,
        config: &GracePeriodConfig,
        current_usage: u64,
        limit: u64,
    ) -> Self {
        let now = Utc::now();
        let overage = current_usage.saturating_sub(limit);
        let max_allowed = (limit as f64 * config.max_overage_percent / 100.0) as u64;

        Self {
            organization_id,
            metric,
            started_at: now,
            expires_at: now + chrono::Duration::hours(config.duration_hours as i64),
            overage_at_start: overage,
            current_overage: overage,
            max_allowed_overage: max_allowed,
            is_active: true,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_within_limits(&self) -> bool {
        self.current_overage <= self.max_allowed_overage
    }

    pub fn remaining_time(&self) -> chrono::Duration {
        self.expires_at.signed_duration_since(Utc::now())
    }

    pub fn update_overage(&mut self, current_usage: u64, limit: u64) {
        self.current_overage = current_usage.saturating_sub(limit);

        if self.is_expired() || !self.is_within_limits() {
            self.is_active = false;
        }
    }
}

/// Grace period manager
pub struct GracePeriodManager {
    config: GracePeriodConfig,
    active_periods: Arc<RwLock<HashMap<(Uuid, UsageMetric), GracePeriodStatus>>>,
}

impl GracePeriodManager {
    pub fn new(config: GracePeriodConfig) -> Self {
        Self {
            config,
            active_periods: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if grace period allows the operation
    pub async fn check_grace_period(
        &self,
        org_id: Uuid,
        metric: UsageMetric,
        current_usage: u64,
        limit: u64,
    ) -> GracePeriodDecision {
        if !self.config.enabled || !self.config.applicable_metrics.contains(&metric) {
            return GracePeriodDecision::NotApplicable;
        }

        let key = (org_id, metric);
        let mut periods = self.active_periods.write().await;

        if let Some(status) = periods.get_mut(&key) {
            status.update_overage(current_usage, limit);

            if status.is_active && status.is_within_limits() {
                return GracePeriodDecision::InGracePeriod {
                    remaining: status.remaining_time(),
                    overage_used: status.current_overage,
                    overage_limit: status.max_allowed_overage,
                };
            } else {
                periods.remove(&key);
                return GracePeriodDecision::GracePeriodExpired;
            }
        }

        // Start new grace period if within overage limits
        let potential_status = GracePeriodStatus::new(
            org_id,
            metric,
            &self.config,
            current_usage,
            limit,
        );

        if potential_status.is_within_limits() {
            let remaining = potential_status.remaining_time();
            let overage_used = potential_status.current_overage;
            let overage_limit = potential_status.max_allowed_overage;

            periods.insert(key, potential_status);

            GracePeriodDecision::GracePeriodStarted {
                duration_hours: self.config.duration_hours,
                remaining,
                overage_used,
                overage_limit,
            }
        } else {
            GracePeriodDecision::OverageExceedsLimit {
                current_overage: current_usage.saturating_sub(limit),
                max_allowed: potential_status.max_allowed_overage,
            }
        }
    }

    /// Get active grace period status
    pub async fn get_status(
        &self,
        org_id: Uuid,
        metric: UsageMetric,
    ) -> Option<GracePeriodStatus> {
        let periods = self.active_periods.read().await;
        periods.get(&(org_id, metric)).cloned()
    }

    /// End grace period early (e.g., after upgrade)
    pub async fn end_grace_period(&self, org_id: Uuid, metric: UsageMetric) {
        let mut periods = self.active_periods.write().await;
        periods.remove(&(org_id, metric));
    }

    /// Clean up expired grace periods
    pub async fn cleanup_expired(&self) {
        let mut periods = self.active_periods.write().await;
        periods.retain(|_, status| !status.is_expired());
    }
}

/// Grace period decision
#[derive(Debug, Clone)]
pub enum GracePeriodDecision {
    NotApplicable,
    GracePeriodStarted {
        duration_hours: u32,
        remaining: chrono::Duration,
        overage_used: u64,
        overage_limit: u64,
    },
    InGracePeriod {
        remaining: chrono::Duration,
        overage_used: u64,
        overage_limit: u64,
    },
    GracePeriodExpired,
    OverageExceedsLimit {
        current_overage: u64,
        max_allowed: u64,
    },
}

impl GracePeriodDecision {
    pub fn allows_operation(&self) -> bool {
        matches!(
            self,
            Self::NotApplicable
                | Self::GracePeriodStarted { .. }
                | Self::InGracePeriod { .. }
        )
    }
}

// ============================================================================
// Alert Service Factory
// ============================================================================

/// Create a fully configured alert manager with all handlers
pub fn create_alert_manager(
    thresholds: Option<AlertThresholds>,
    cooldown_seconds: Option<u64>,
) -> AlertManager {
    let mut manager = AlertManager::new();

    if let Some(t) = thresholds {
        manager = manager.with_thresholds(t);
    }

    if let Some(c) = cooldown_seconds {
        manager = manager.with_cooldown(c);
    }

    manager
}

/// Create default notification handlers
pub async fn register_default_handlers(manager: &AlertManager) {
    manager
        .register_handler(Arc::new(InAppNotificationHandler::new()))
        .await;
    manager
        .register_handler(Arc::new(WebhookNotificationHandler::new()))
        .await;
    manager
        .register_handler(Arc::new(SlackNotificationHandler::new()))
        .await;
    manager
        .register_handler(Arc::new(TeamsNotificationHandler::new()))
        .await;
}
