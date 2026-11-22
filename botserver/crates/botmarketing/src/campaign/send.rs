//! Per-recipient delivery with the #2536 compliance gate and bounded retry.
//!
//! Kept separate from the worker loop so the gate, retry and persistence rules
//! are testable and the worker stays within the file-size limit.

use crate::compliance::bounce::{
    classify_bounce, persist_attempt, retry_delay_millis, AttemptState, BounceClass,
    MAX_SEND_ATTEMPTS,
};
use crate::compliance::suppression::{persist_suppression, SuppressionKey, SuppressionStore};
use crate::compliance::{
    evaluate_send_gate, supports_recipient_send, MarketingChannel, SendGateDecision,
};
use crate::email::{send_campaign_email, EmailCampaignPayload};
use crate::schema::marketing_recipients;
use crate::state::AppState;
use diesel::prelude::*;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

/// Everything needed to decide and perform one recipient's send.
pub struct SendContext<'a> {
    pub branch_id: Uuid,
    pub bot_id: Uuid,
    pub campaign_id: Uuid,
    pub channel: MarketingChannel,
    pub subject: &'a str,
    pub body: &'a str,
    pub recipient_id: Uuid,
    pub address: &'a str,
    pub name: &'a str,
    pub recipient_status: Option<&'a str>,
    pub subscribed: Option<bool>,
    pub metadata: Option<&'a Value>,
    pub response: Option<&'a Value>,
}

/// Result of one recipient's processing. `skipped` is set before any provider
/// call; `status` is the value recorded on the recipient/event row.
#[derive(Debug, Clone)]
pub struct SendOutcome {
    pub delivered: bool,
    pub skipped: Option<SendGateDecision>,
    pub status: &'static str,
    pub error: Option<String>,
}

impl SendOutcome {
    fn skipped(decision: SendGateDecision) -> Self {
        Self {
            delivered: false,
            skipped: Some(decision),
            status: decision.status(),
            error: None,
        }
    }
}

/// Honest result for a channel the per-recipient worker cannot send on. No
/// provider call is made and the recipient is never marked as delivered.
fn unsupported_outcome(channel: MarketingChannel) -> SendOutcome {
    SendOutcome {
        delivered: false,
        skipped: None,
        status: "unsupported_channel",
        error: Some(format!(
            "no per-recipient provider for channel {}",
            channel.as_str()
        )),
    }
}

fn persist_terminal_status(state: &Arc<AppState>, recipient_id: Uuid, status: &str) {
    let Ok(mut conn) = state.conn.get() else {
        log::error!("recipient {recipient_id} terminal status unavailable: DB pool error");
        return;
    };
    if let Err(error) = diesel::update(
        marketing_recipients::table.filter(marketing_recipients::id.eq(recipient_id)),
    )
    .set((
        marketing_recipients::status.eq(Some(status.to_string())),
        marketing_recipients::failed_at.eq(Some(chrono::Utc::now())),
    ))
    .execute(&mut conn)
    {
        log::error!("recipient {recipient_id} terminal status not persisted: {error}");
    }
}

/// Evaluates the gate, sends with exponential backoff on retryable failures,
/// auto-suppresses hard bounces and returns a typed outcome.
pub async fn send_one(
    state: &Arc<AppState>,
    ctx: &SendContext<'_>,
) -> Result<SendOutcome, String> {
    let decision = evaluate_send_gate(
        ctx.branch_id,
        ctx.channel,
        ctx.address,
        ctx.recipient_status,
        ctx.subscribed,
        ctx.metadata,
    );
    if !decision.is_allowed() {
        log::info!(
            "campaign {} recipient {} skipped: {}",
            ctx.campaign_id,
            ctx.recipient_id,
            decision.status()
        );
        return Ok(SendOutcome::skipped(decision));
    }

    // #2536 — SMS and Instagram have no per-recipient sender. Falling through
    // to the email provider used to address a phone number and report success.
    if !supports_recipient_send(ctx.channel) {
        log::warn!(
            "campaign {} recipient {} channel {} has no per-recipient provider",
            ctx.campaign_id,
            ctx.recipient_id,
            ctx.channel.as_str()
        );
        return Ok(unsupported_outcome(ctx.channel));
    }

    let personalized = ctx
        .body
        .replace("{{name}}", ctx.name)
        .replace("{{email}}", ctx.address);
    let mut attempt = AttemptState::from_json(ctx.response);
    let mut last_error: Option<String> = None;
    let mut terminal_hard = false;

    // A previous pass may already have consumed part (or all) of the budget.
    // Never grant a fresh three attempts on every worker run.
    let remaining = MAX_SEND_ATTEMPTS.saturating_sub(attempt.attempts);
    if remaining == 0 {
        log::info!(
            "campaign {} recipient {} already exhausted after {} attempts",
            ctx.campaign_id,
            ctx.recipient_id,
            attempt.attempts
        );
        return Ok(SendOutcome {
            delivered: false,
            skipped: None,
            status: "dead_letter",
            error: attempt.last_error.clone(),
        });
    }

    for _ in 0..remaining {
        let result = attempt_send(state, ctx, &personalized).await;
        match result {
            Ok(()) => {
                return Ok(SendOutcome {
                    delivered: true,
                    skipped: None,
                    status: "sent",
                    error: None,
                });
            }
            Err(error) => {
                let class = classify_bounce(&error);
                if matches!(class, BounceClass::Hard | BounceClass::Complaint) {
                    terminal_hard = true;
                }
                attempt.record(&error, class);
                last_error = Some(error);
                if !class.is_retryable() || attempt.is_exhausted() {
                    break;
                }
                let delay = retry_delay_millis(attempt.attempts.saturating_sub(1));
                persist_attempt_for(state, ctx.recipient_id, &attempt);
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
        }
    }

    if terminal_hard && !ctx.address.is_empty() {
        let key = SuppressionKey::new(ctx.branch_id, ctx.channel, ctx.address);
        if let Err(error) = SuppressionStore::global().add(key.clone()) {
            log::error!("bounce suppression failed: {}", error.reason());
        }
        if let Ok(mut conn) = state.conn.get() {
            if let Err(error) = persist_suppression(&mut conn, &key, "hard_bounce") {
                log::error!("bounce suppression not persisted: {error}");
            }
        }
    }
    persist_attempt_for(state, ctx.recipient_id, &attempt);
    let status = if attempt.is_exhausted() {
        "dead_letter"
    } else {
        "failed"
    };
    if status == "dead_letter" {
        persist_terminal_status(state, ctx.recipient_id, status);
    }
    Ok(SendOutcome {
        delivered: false,
        skipped: None,
        status,
        error: last_error,
    })
}

fn persist_attempt_for(state: &Arc<AppState>, recipient_id: Uuid, attempt: &AttemptState) {
    let Ok(mut conn) = state.conn.get() else {
        log::error!("recipient {recipient_id} attempt state unavailable: DB pool error");
        return;
    };
    if let Err(error) = persist_attempt(&mut conn, recipient_id, attempt) {
        log::error!("recipient {recipient_id} attempt state not persisted: {error}");
    }
}

async fn attempt_send(
    state: &Arc<AppState>,
    ctx: &SendContext<'_>,
    personalized: &str,
) -> Result<(), String> {
    if ctx.channel == MarketingChannel::Whatsapp {
        return (state.send_whatsapp)(
            ctx.bot_id,
            ctx.address,
            personalized,
            Some(ctx.name),
            None,
        )
        .map(|_| ());
    }
    let payload = EmailCampaignPayload {
        to: ctx.address.to_string(),
        subject: ctx.subject.to_string(),
        body_html: Some(personalized.to_string()),
        body_text: None,
        campaign_id: Some(ctx.campaign_id),
        recipient_id: Some(ctx.recipient_id),
    };
    match send_campaign_email(state, ctx.bot_id, payload).await {
        Ok(result) if result.success => Ok(()),
        Ok(result) => Err(result.error.unwrap_or_else(|| "unknown error".to_string())),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_channels_never_report_delivery() {
        for channel in [MarketingChannel::Sms, MarketingChannel::Instagram] {
            let outcome = unsupported_outcome(channel);
            assert!(!outcome.delivered);
            assert!(outcome.skipped.is_none());
            assert_eq!(outcome.status, "unsupported_channel");
            let error = outcome.error.unwrap_or_default();
            assert!(error.contains(channel.as_str()));
        }
    }

    #[test]
    fn provider_matrix_covers_only_sendable_channels() {
        assert!(supports_recipient_send(MarketingChannel::Email));
        assert!(supports_recipient_send(MarketingChannel::Whatsapp));
        assert!(!supports_recipient_send(MarketingChannel::Sms));
        assert!(!supports_recipient_send(MarketingChannel::Instagram));
    }
}
