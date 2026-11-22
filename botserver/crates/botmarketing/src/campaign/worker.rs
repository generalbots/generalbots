//! Durable multi-channel campaign sender (#731).
//!
//! A background loop processes campaigns whose `status` is `scheduled` or
//! `running`. Per-campaign run control (pause/resume/stop) is persisted in
//! `marketing_campaigns` columns so a restart resumes from the last sent
//! offset (`run_offset`). Every send attempt is recorded in
//! `marketing_campaign_events` for the realtime monitor. Global pause/stop
//! is kept in memory only (it never needs to survive a restart).

use crate::campaign::send::{send_one, SendContext};
use crate::compliance::suppression::hydrate_suppressions;
use crate::compliance::MarketingChannel;
use crate::schema::{
    crm_contacts, marketing_campaign_events, marketing_campaigns, marketing_contacts,
    marketing_recipients,
};
use crate::segment::SUPPRESSED_STATUSES;
use crate::state::AppState;
use chrono::Utc;
use diesel::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Default inter-send delay (seconds) used to throttle campaign sends.
const DEFAULT_EMAIL_INTERVAL_SECS: u64 = 43;

pub struct CampaignWorker {
    state: Arc<AppState>,
    global_pause: Arc<AtomicBool>,
    global_stop: Arc<AtomicBool>,
}

impl CampaignWorker {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state,
            global_pause: Arc::new(AtomicBool::new(false)),
            global_stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_global_pause(&self, paused: bool) {
        self.global_pause.store(paused, Ordering::SeqCst);
        log::info!("Campaign worker global pause set to {paused}");
    }

    pub fn is_global_paused(&self) -> bool {
        self.global_pause.load(Ordering::SeqCst)
    }

    pub fn set_global_stop(&self, stopped: bool) {
        self.global_stop.store(stopped, Ordering::SeqCst);
        log::info!("Campaign worker global stop set to {stopped}");
    }

    pub fn is_global_stopped(&self) -> bool {
        self.global_stop.load(Ordering::SeqCst)
    }

    /// Spawns the durable background loop. The loop runs forever; a global
    /// stop transitions every active campaign to `stopped` and idles.
    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(5));
            ticker.tick().await;
            loop {
                ticker.tick().await;
                if let Err(e) = self.run_pass().await {
                    log::error!("Campaign worker pass failed: {e}");
                }
            }
        });
        log::info!("Campaign worker started");
    }

    async fn run_pass(&self) -> Result<(), String> {
        /// Selected `marketing_campaigns` columns of a campaign due for a pass.
        type ActiveCampaignRow = (
            Uuid,
            String,
            Option<chrono::DateTime<Utc>>,
            Option<i32>,
            Option<bool>,
            Option<bool>,
        );

        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("DB pool error: {e}"))?;

        // #2536 — reload the durable do-not-contact list before any send in
        // this pass. A restart must not resurrect a suppressed recipient.
        if let Err(error) = hydrate_suppressions(&mut conn) {
            log::error!("campaign worker could not hydrate suppressions: {error}");
        }

        let active: Vec<ActiveCampaignRow> = marketing_campaigns::table
                .filter(
                    marketing_campaigns::status
                        .eq(Some("scheduled".to_string()))
                        .or(marketing_campaigns::status.eq(Some("running".to_string()))),
                )
                .select((
                    marketing_campaigns::id,
                    marketing_campaigns::campaign_type,
                    marketing_campaigns::starts_at,
                    marketing_campaigns::run_offset,
                    marketing_campaigns::pause_requested,
                    marketing_campaigns::stop_requested,
                ))
                .load(&mut conn)
                .map_err(|e| format!("Query error: {e}"))?;

        drop(conn);

        for (id, campaign_type, starts_at, offset, paused, stop) in active {
            if self.is_global_stopped() || stop.unwrap_or(false) {
                self.set_campaign_status(id, "stopped").await?;
                self.log_event(id, &campaign_type, "stop", None, Some("stopped"), None)
                    .await?;
                continue;
            }

            if self.is_global_paused() || paused.unwrap_or(false) {
                continue;
            }

            if let Some(start) = starts_at {
                if start > Utc::now() {
                    continue;
                }
            }

            self.set_campaign_status(id, "running").await?;
            self.log_event(id, &campaign_type, "send", None, Some("running"), None)
                .await?;

            let result = self.send_remaining(id, &campaign_type, offset.unwrap_or(0)).await;
            match result {
                Ok(()) => {
                    self.set_campaign_status(id, "completed").await?;
                    self.log_event(id, &campaign_type, "send", None, Some("completed"), None)
                        .await?;
                }
                Err(e) => {
                    log::error!("Campaign {id} sending failed: {e}");
                }
            }
        }
        Ok(())
    }

    async fn send_remaining(
        &self,
        campaign_id: Uuid,
        campaign_type: &str,
        start_offset: i32,
    ) -> Result<(), String> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("DB pool error: {e}"))?;

        let campaign: Option<(Option<serde_json::Value>, Option<bigdecimal::BigDecimal>)> =
            marketing_campaigns::table
                .filter(marketing_campaigns::id.eq(campaign_id))
                .select((marketing_campaigns::metrics, marketing_campaigns::budget))
                .first(&mut conn)
                .optional()
                .map_err(|e| format!("Query error: {e}"))?;

        // #2536 — load the persisted status, attempt counter and the consent
        // record behind `contact_id` so every send can be gated without a
        // separate query per recipient.
        type RecipientRow = (
            Uuid,
            Option<String>,
            Option<String>,
            Option<Uuid>,
            Option<String>,
            Option<serde_json::Value>,
        );
        let recipients: Vec<RecipientRow> = marketing_recipients::table
            .filter(marketing_recipients::campaign_id.eq(campaign_id))
            .select((
                marketing_recipients::id,
                marketing_recipients::email,
                marketing_recipients::name,
                marketing_recipients::contact_id,
                marketing_recipients::status,
                marketing_recipients::response,
            ))
            .load::<RecipientRow>(&mut conn)
            .map_err(|e| format!("Query error: {e}"))?;

        let contact_ids: Vec<Uuid> = recipients.iter().filter_map(|row| row.3).collect();
        let consent_rows: Vec<(Uuid, Option<bool>, Option<serde_json::Value>)> =
            marketing_contacts::table
                .filter(marketing_contacts::id.eq_any(&contact_ids))
                .select((
                    marketing_contacts::id,
                    marketing_contacts::subscribed,
                    marketing_contacts::metadata,
                ))
                .load(&mut conn)
                .map_err(|e| format!("Consent query error: {e}"))?;
        // `marketing_recipients.contact_id` may point at either a
        // `marketing_contacts` row (new publish flow) or a legacy
        // `crm_contacts` row, so both consent sources are merged.
        let crm_rows: Vec<(Uuid, String, Option<serde_json::Value>)> = crm_contacts::table
            .filter(crm_contacts::id.eq_any(&contact_ids))
            .select((
                crm_contacts::id,
                crm_contacts::status,
                crm_contacts::custom_fields,
            ))
            .load(&mut conn)
            .map_err(|e| format!("CRM consent query error: {e}"))?;
        let mut consent_by_contact: HashMap<Uuid, (Option<bool>, Option<serde_json::Value>)> =
            crm_rows
                .into_iter()
                .map(|(id, status, custom_fields)| {
                    let opted_out = SUPPRESSED_STATUSES.contains(&status.as_str());
                    (
                        id,
                        (if opted_out { Some(false) } else { None }, custom_fields),
                    )
                })
                .collect();
        for (id, subscribed, metadata) in consent_rows {
            let entry = consent_by_contact.entry(id).or_insert((None, None));
            if subscribed.is_some() {
                entry.0 = subscribed;
            }
            if metadata.is_some() || entry.1.is_none() {
                entry.1 = metadata;
            }
        }

        drop(conn);

        let default_metrics = serde_json::json!({});
        let metrics = campaign
            .as_ref()
            .map(|(m, _)| m.as_ref().unwrap_or(&default_metrics))
            .unwrap_or(&default_metrics);
        let subject = metrics
            .get("subject")
            .and_then(|s| s.as_str())
            .unwrap_or("Newsletter")
            .to_string();
        let raw_body = metrics
            .get("body")
            .and_then(|b| b.as_str())
            .unwrap_or("Newsletter");

        let total = recipients.len() as i32;
        let mut sent = 0;
        let mut failed = 0;
        let mut skipped = 0;

        let (_, branch_id, bot_id) = self.state.get_scope();
        let channel = MarketingChannel::parse(campaign_type).unwrap_or(MarketingChannel::Email);

        for (
            idx,
            (
                recipient_id,
                recipient_email,
                recipient_name,
                contact_id,
                recipient_status,
                response,
            ),
        ) in recipients.iter().enumerate()
        {
            if idx < start_offset as usize {
                continue;
            }
            if self.is_global_stopped() || self.is_global_paused() {
                break;
            }

            let (subscribed, metadata) = contact_id
                .as_ref()
                .and_then(|id| consent_by_contact.get(id))
                .map(|(subscribed, metadata)| (*subscribed, metadata.as_ref()))
                .unwrap_or((None, None));
            let email = recipient_email.as_deref().unwrap_or("");
            let context = SendContext {
                branch_id,
                bot_id,
                campaign_id,
                channel,
                subject: &subject,
                body: raw_body,
                recipient_id: *recipient_id,
                address: email,
                name: recipient_name.as_deref().unwrap_or(""),
                recipient_status: recipient_status.as_deref(),
                subscribed,
                metadata,
                response: response.as_ref(),
            };
            let outcome = send_one(&self.state, &context).await?;

            if let Some(decision) = outcome.skipped {
                skipped += 1;
                self.log_event(
                    campaign_id,
                    campaign_type,
                    "skip",
                    Some(email),
                    Some(decision.status()),
                    None,
                )
                .await?;
            } else if outcome.delivered {
                sent += 1;
                self.log_event(
                    campaign_id,
                    campaign_type,
                    "send",
                    Some(email),
                    Some("sent"),
                    None,
                )
                .await?;
            } else {
                failed += 1;
                self.log_event(
                    campaign_id,
                    campaign_type,
                    "fail",
                    Some(email),
                    Some(outcome.status),
                    outcome.error,
                )
                .await?;
            }

            let new_offset = start_offset + idx as i32 + 1;
            self.update_run_offset(campaign_id, new_offset).await?;

            let interval = self.email_interval();
            tokio::time::sleep(Duration::from_secs(interval)).await;
        }

        log::info!(
            "Campaign {campaign_id} worker sent {sent} ok, {failed} failed, {skipped} skipped, {total} total"
        );
        Ok(())
    }

    fn email_interval(&self) -> u64 {
        DEFAULT_EMAIL_INTERVAL_SECS
    }

    async fn update_run_offset(&self, campaign_id: Uuid, offset: i32) -> Result<(), String> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("DB pool error: {e}"))?;
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(campaign_id)))
            .set(marketing_campaigns::run_offset.eq(Some(offset)))
            .execute(&mut conn)
            .map_err(|e| format!("Update error: {e}"))?;
        Ok(())
    }

    async fn set_campaign_status(&self, campaign_id: Uuid, status: &str) -> Result<(), String> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("DB pool error: {e}"))?;
        let now = Utc::now();
        diesel::update(marketing_campaigns::table.filter(marketing_campaigns::id.eq(campaign_id)))
            .set((
                marketing_campaigns::status.eq(Some(status.to_string())),
                marketing_campaigns::started_at.eq(if status == "running" {
                    Some(now)
                } else {
                    None
                }),
                marketing_campaigns::completed_at.eq(if status == "completed" || status == "stopped" {
                    Some(now)
                } else {
                    None
                }),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Update error: {e}"))?;
        Ok(())
    }

    pub async fn log_event(
        &self,
        campaign_id: Uuid,
        channel: &str,
        event_type: &str,
        recipient_email: Option<&str>,
        status: Option<&str>,
        error_message: Option<String>,
    ) -> Result<(), String> {
        let mut conn = self
            .state
            .conn
            .get()
            .map_err(|e| format!("DB pool error: {e}"))?;
        diesel::insert_into(marketing_campaign_events::table)
            .values((
                marketing_campaign_events::campaign_id.eq(campaign_id),
                marketing_campaign_events::channel.eq(Some(channel.to_string())),
                marketing_campaign_events::event_type.eq(event_type.to_string()),
                marketing_campaign_events::recipient_email.eq(recipient_email.map(str::to_string)),
                marketing_campaign_events::status.eq(status.map(str::to_string)),
                marketing_campaign_events::error_message.eq(error_message),
            ))
            .execute(&mut conn)
            .map_err(|e| format!("Insert error: {e}"))?;
        Ok(())
    }
}
