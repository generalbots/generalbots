#[cfg(feature = "mail")]
use log::warn;

#[cfg(feature = "mail")]
pub async fn send_invitation_email(
    _to_email: String,
    _role: String,
    _custom_message: Option<String>,
    _invitation_id: uuid::Uuid,
) -> Result<(), String> {
    warn!("send_invitation_email called but SecretsManager not available in botcore");
    Err("SecretsManager not available in botcore".to_string())
}
