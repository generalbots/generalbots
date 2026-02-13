// Email invitation functions
#[cfg(feature = "mail")]
use lettre::{
    message::{header::ContentType, Message},
    transport::smtp::authentication::Credentials,
    SmtpTransport, Transport,
};

/// Send invitation email
#[cfg(feature = "mail")]
pub async fn send_invitation_email(
    to_email: String,
    role: String,
    custom_message: Option<String>,
    invitation_id: Uuid,
) -> Result<(), String> {
    let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
    let smtp_user = std::env::var("SMTP_USER").ok();
    let smtp_pass = std::env::var("SMTP_PASS").ok();
    let smtp_from = std::env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@generalbots.com".to_string());
    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "https://app.generalbots.com".to_string());

    let custom_msg = custom_message.unwrap_or_default();

    let accept_url = format!("{}/accept-invitation?token={}", app_url, invitation_id);

    let body = format!(
        r#"You have been invited to join our organization as a {}.

{}

Click on link below to accept the invitation:
{}

This invitation will expire in 7 days.

If you did not expect this invitation, you can safely ignore this email.

Best regards,
The General Bots Team"#,
        role,
        if custom_msg.is_empty() { "".to_string() } else { format!("\n{}\n", custom_msg) },
        accept_url
    );

    let email = Message::builder()
        .from(smtp_from.parse().map_err(|e| format!("Invalid from address: {}", e))?)
        .to(to_email.parse().map_err(|e| format!("Invalid to address: {}", e))?)
        .subject("You've been invited to join our organization")
        .header(ContentType::TEXT_PLAIN)
        .body(body)
        .map_err(|e| format!("Failed to build email: {}", e))?;

    let mailer = if let (Some(user), Some(pass)) = (smtp_user, smtp_pass) {
        let creds = Credentials::new(user, pass);
        SmtpTransport::relay(&smtp_host)
            .map_err(|e| format!("SMTP relay error: {}", e))?
            .credentials(creds)
            .build()
    } else {
        SmtpTransport::builder_dangerous(&smtp_host).build()
    };

    mailer.send(&email).map_err(|e| format!("Failed to send email: {}", e))?;
    warn!("Invitation email sent successfully to {}", to_email);
    Ok(())
}

/// Send invitation email by fetching details from database
#[cfg(feature = "mail")]
pub async fn send_invitation_email_by_id(invitation_id: Uuid) -> Result<(), String> {
    // TODO: Implement when invitations table is available in schema
    warn!("send_invitation_email_by_id called for {} - not fully implemented", invitation_id);
    Err(format!("Invitation with id {} not found", invitation_id))
}
