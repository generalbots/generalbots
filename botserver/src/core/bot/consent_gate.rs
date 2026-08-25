//! Consent gate for agent-initiated app commands (issue #1176).
//! Holds the process-wide `ConsentService` and offers a single decision
//! entry point used by the chat command path and the api/ui loopback executor.
use std::sync::{Arc, OnceLock};

static SERVICE: OnceLock<Arc<botconsent::ConsentService>> = OnceLock::new();

pub fn init(pool: botconsent::DbPool) {
    let service = Arc::new(botconsent::ConsentService::new(pool));
    service.ensure_sweeper();
    let _ = SERVICE.set(service);
}

fn service() -> Option<&'static Arc<botconsent::ConsentService>> {
    SERVICE.get()
}

pub struct GateOutcome {
    pub allowed: bool,
    pub request_id: Option<String>,
    pub pending_request: Option<botconsent::PendingRequest>,
    pub card_html: Option<String>,
}

/// Decide whether `user_id` may run `action_class` on `app_id`.
/// When the consent service is not initialized (feature disabled at boot)
/// the gate is open, preserving legacy behavior.
pub async fn check(
    user_id: uuid::Uuid,
    app_id: &str,
    action_class: &str,
    detail: serde_json::Value,
) -> GateOutcome {
    let Some(service) = service() else {
        return GateOutcome { allowed: true, request_id: None, pending_request: None, card_html: None };
    };
    match botconsent::enforce::authorize(service, user_id, app_id, action_class, detail).await {
        botconsent::enforce::ConsentDecision::Granted(_) => GateOutcome {
            allowed: true,
            request_id: None,
            pending_request: None,
            card_html: None,
        },
        botconsent::enforce::ConsentDecision::Pending(request) => {
            let card = botconsent::cards::prompt_card_html(&request);
            GateOutcome {
                allowed: false,
                request_id: Some(request.request_id.clone()),
                pending_request: Some(request),
                card_html: Some(card),
            }
        }
        botconsent::enforce::ConsentDecision::Denied => GateOutcome {
            allowed: false,
            request_id: None,
            pending_request: None,
            card_html: None,
        },
    }
}

pub fn deny_message(app_id: &str) -> String {
    format!(
        "This action on '{app_id}' requires your permission. Approve it in Settings > App permissions."
    )
}

/// Marker protocol consumed by `chat/modules/40_consent_cards.js`: a plain
/// sentence for legacy clients plus a base64url JSON payload the suite
/// renderer swaps for an interactive card.
pub fn marker_content(request: &botconsent::PendingRequest) -> String {
    use base64::Engine as _;
    let payload = serde_json::json!({
        "request_id": request.request_id,
        "app_id": request.app_id,
        "action_class": request.action_class,
        "detail": request.detail,
    });
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(payload.to_string().as_bytes());
    format!(
        "Permission needed: the assistant wants to run '{class}' on '{app}'. Reply via the buttons or Settings.\nGB-CONSENT-REQUEST:{encoded}",
        class = request.action_class,
        app = request.app_id,
        encoded = encoded,
    )
}
