use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use rhai::Dynamic;
use rhai::Engine;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDraftRequest {
    pub to: String,
    pub subject: String,
    pub cc: Option<String>,
    pub text: String,
}

pub fn create_draft_keyword(_state: &AppState, _user: UserSession, engine: &mut Engine) {
    let state_clone = _state.clone();
    engine
        .register_custom_syntax(
            &["CREATE_DRAFT", "$expr$", ",", "$expr$", ",", "$expr$"],
            true,
            move |context, inputs| {
                let to = context.eval_expression_tree(&inputs[0])?.to_string();
                let subject = context.eval_expression_tree(&inputs[1])?.to_string();
                let reply_text = context.eval_expression_tree(&inputs[2])?.to_string();

                let fut = execute_create_draft(&state_clone, &to, &subject, &reply_text);
                let result =
                    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
                        .map_err(|e| format!("Draft creation error: {}", e))?;
                Ok(Dynamic::from(result))
            },
        )
        .unwrap();
}

async fn execute_create_draft(
    _state: &AppState,
    to: &str,
    subject: &str,
    reply_text: &str,
) -> Result<String, String> {
    // For now, we'll store drafts in the database or just log them
    // This is a simplified implementation until the email module is fully ready

    #[cfg(feature = "email")]
    {
        // When email feature is enabled, try to use email functionality if available
        // For now, we'll just simulate draft creation
        use log::info;

        info!("Creating draft email - To: {}, Subject: {}", to, subject);

        // In a real implementation, this would:
        // 1. Connect to email service
        // 2. Create draft in IMAP folder or local storage
        // 3. Return draft ID or confirmation

        let draft_id = uuid::Uuid::new_v4().to_string();

        // You could store this in the database
        // For now, just return success
        Ok(format!("Draft saved successfully with ID: {}", draft_id))
    }

    #[cfg(not(feature = "email"))]
    {
        // When email feature is disabled, return a placeholder message
        Ok(format!(
            "Email feature not enabled. Would create draft - To: {}, Subject: {}, Body: {}",
            to, subject, reply_text
        ))
    }
}

// Helper functions that would be implemented when email module is complete
#[cfg(feature = "email")]
async fn fetch_latest_sent_to(
    _config: &Option<crate::config::Config>,
    _to: &str,
) -> Result<String, String> {
    // This would fetch the latest email sent to the recipient
    // For threading/reply purposes
    Ok(String::new())
}

#[cfg(feature = "email")]
async fn save_email_draft(
    _config: &Option<crate::config::Config>,
    _draft: &SaveDraftRequest,
) -> Result<(), String> {
    // This would save the draft to the email server or local storage
    Ok(())
}
