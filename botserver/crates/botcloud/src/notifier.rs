//! Transactional email for the cloud plan/billing flow (#2094).
//!
//! Every subject and body comes from the catalogue, rendered for the
//! recipient's locale (`EmailVars::locale`) with an English fallback. The SMTP
//! envelope, the MIME headers and the addresses are untouched: only the human
//! text is translated.

use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport,
    Message, Tokio1Executor,
};
use std::collections::HashMap;
use std::sync::OnceLock;

use botlib::i18n::{self, Locale};

static SMTP_CONFIG: OnceLock<SmtpConfig> = OnceLock::new();

/// Cloud dashboard the transactional mail links to.
const DASHBOARD_URL: &str = "https://cloud.generalbots.com/dashboard";
const INVOICES_URL: &str = "https://cloud.generalbots.com/invoices";
const SERVICES_URL: &str = "https://cloud.generalbots.com/services";
const SETTINGS_URL: &str = "https://cloud.generalbots.com/settings";
const OFFERS_URL: &str = "https://cloud.generalbots.com/offers";

#[derive(Debug, Clone)]
struct SmtpConfig {
    host: String,
    port: u16,
    user: String,
    password: String,
    from: String,
}

fn get_smtp_config() -> Option<&'static SmtpConfig> {
    SMTP_CONFIG.get()
}

fn try_send_email(to: &str, subject: &str, body: &str) -> Result<(), String> {
    let cfg = get_smtp_config().ok_or("SMTP not configured")?;

    let email = Message::builder()
        .from(cfg.from.parse().map_err(|e: lettre::address::AddressError| format!("Invalid from address: {e}"))?)
        .to(to.parse().map_err(|e: lettre::address::AddressError| format!("Invalid to address: {e}"))?)
        .subject(subject)
        .body(body.to_string())
        .map_err(|e| format!("Failed to build email: {e}"))?;

    let creds = Credentials::new(cfg.user.clone(), cfg.password.clone());

    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.host)
            .map_err(|e| format!("SMTP relay error: {e}"))?
            .port(cfg.port)
            .credentials(creds)
            .build();

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            mailer.send(email).await.map_err(|e| format!("SMTP send error: {e}"))
        })
    })?;

    Ok(())
}

/// Recipient language from a request field or a stored preference, falling
/// back to English when the value is missing or unknown.
#[must_use]
pub fn recipient_locale(raw: Option<&str>) -> Locale {
    raw.and_then(|value| Locale::new(value.trim()))
        .unwrap_or_default()
}

pub struct EmailVars {
    pub customer_name: String,
    pub customer_email: String,
    pub plan: String,
    pub amount: f64,
    pub currency: String,
    pub invoice_id: String,
    pub org_id: String,
    pub bot_name: String,
    pub extra: HashMap<String, String>,
    /// Language the recipient reads. Defaults to English; callers that know
    /// the user's preference (signup/checkout body, `users.locale`) set it.
    pub locale: Locale,
}

impl EmailVars {
    pub fn new(
        customer_name: &str,
        customer_email: &str,
        plan: &str,
        amount: f64,
        currency: &str,
    ) -> Self {
        Self {
            customer_name: customer_name.to_string(),
            customer_email: customer_email.to_string(),
            plan: plan.to_string(),
            amount,
            currency: currency.to_string(),
            invoice_id: String::new(),
            org_id: String::new(),
            bot_name: "default".to_string(),
            extra: HashMap::new(),
            locale: Locale::default(),
        }
    }

    /// Pins the language of this message to the recipient's preference.
    #[must_use]
    pub fn with_locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    /// Money is rendered once, so no catalogue entry carries a currency symbol.
    fn amount_text(&self) -> String {
        format!("${:.2}", self.amount)
    }

    /// Greeting + body + signature, in the recipient's language.
    fn compose(&self, body_key: &str, args: &[(&str, &str)]) -> String {
        let greeting = i18n::translate(&self.locale, "email-greeting", &[("name", &self.customer_name)]);
        let body = i18n::translate(&self.locale, body_key, args);
        let signoff = i18n::translate(&self.locale, "email-signoff", &[]);
        let team = i18n::translate(&self.locale, "email-team", &[]);

        format!("{greeting}\n\n{body}\n\n{signoff}\n{team}")
    }

    fn subject(&self, key: &str, args: &[(&str, &str)]) -> String {
        i18n::translate(&self.locale, key, args)
    }
}

pub fn notify_welcome(vars: &EmailVars) {
    tracing::info!(
        target: "saas_notifier",
        "[EMAIL] welcome to={email} name={name} plan={plan}",
        email = vars.customer_email,
        name = vars.customer_name,
        plan = vars.plan,
    );

    if let Some(_cfg) = get_smtp_config() {
        let subject = vars.subject("email-welcome-subject", &[("plan", &vars.plan)]);
        let body = vars.compose(
            "email-welcome-body",
            &[("plan", &vars.plan), ("url", DASHBOARD_URL)],
        );
        if let Err(e) = try_send_email(&vars.customer_email, &subject, &body) {
            tracing::warn!(target: "saas_notifier", "Failed to send welcome email: {e}");
        }
    }
}

pub fn notify_invoice_created(vars: &EmailVars) {
    tracing::info!(
        target: "saas_notifier",
        "[EMAIL] invoice_created to={email} invoice={inv} amount={amt} {cur}",
        email = vars.customer_email,
        inv = vars.invoice_id,
        amt = vars.amount,
        cur = vars.currency,
    );

    if let Some(_cfg) = get_smtp_config() {
        let subject = vars.subject(
            "email-invoice-subject",
            &[("invoice", &vars.invoice_id)],
        );
        let amount = vars.amount_text();
        let body = vars.compose(
            "email-invoice-body",
            &[
                ("invoice", &vars.invoice_id),
                ("amount", &amount),
                ("currency", &vars.currency),
                ("url", INVOICES_URL),
            ],
        );
        if let Err(e) = try_send_email(&vars.customer_email, &subject, &body) {
            tracing::warn!(target: "saas_notifier", "Failed to send invoice email: {e}");
        }
    }
}

pub fn notify_payment_success(vars: &EmailVars) {
    tracing::info!(
        target: "saas_notifier",
        "[EMAIL] payment_success to={email} plan={plan} amount={amt} {cur}",
        email = vars.customer_email,
        plan = vars.plan,
        amt = vars.amount,
        cur = vars.currency,
    );

    if let Some(_cfg) = get_smtp_config() {
        let subject = vars.subject("email-payment-success-subject", &[]);
        let amount = vars.amount_text();
        let body = vars.compose(
            "email-payment-success-body",
            &[
                ("amount", &amount),
                ("currency", &vars.currency),
                ("plan", &vars.plan),
            ],
        );
        if let Err(e) = try_send_email(&vars.customer_email, &subject, &body) {
            tracing::warn!(target: "saas_notifier", "Failed to send payment success email: {e}");
        }
    }
}

pub fn notify_subscription_activated(vars: &EmailVars) {
    tracing::info!(
        target: "saas_notifier",
        "[EMAIL] subscription_activated to={email} plan={plan}",
        email = vars.customer_email,
        plan = vars.plan,
    );

    if let Some(_cfg) = get_smtp_config() {
        let subject = vars.subject("email-subscription-activated-subject", &[]);
        let body = vars.compose(
            "email-subscription-activated-body",
            &[("plan", &vars.plan), ("url", SERVICES_URL)],
        );
        if let Err(e) = try_send_email(&vars.customer_email, &subject, &body) {
            tracing::warn!(target: "saas_notifier", "Failed to send activation email: {e}");
        }
    }
}

pub fn notify_recurring_charge(vars: &EmailVars) {
    tracing::info!(
        target: "saas_notifier",
        "[EMAIL] recurring_charge to={email} invoice={inv} amount={amt} {cur}",
        email = vars.customer_email,
        inv = vars.invoice_id,
        amt = vars.amount,
        cur = vars.currency,
    );

    if let Some(_cfg) = get_smtp_config() {
        let subject = vars.subject(
            "email-recurring-charge-subject",
            &[("invoice", &vars.invoice_id)],
        );
        let amount = vars.amount_text();
        let body = vars.compose(
            "email-recurring-charge-body",
            &[
                ("invoice", &vars.invoice_id),
                ("amount", &amount),
                ("currency", &vars.currency),
                ("url", INVOICES_URL),
            ],
        );
        if let Err(e) = try_send_email(&vars.customer_email, &subject, &body) {
            tracing::warn!(target: "saas_notifier", "Failed to send recurring charge email: {e}");
        }
    }
}

pub fn notify_payment_failed(vars: &EmailVars) {
    tracing::warn!(
        target: "saas_notifier",
        "[EMAIL] payment_failed to={email} invoice={inv} amount={amt} {cur}",
        email = vars.customer_email,
        inv = vars.invoice_id,
        amt = vars.amount,
        cur = vars.currency,
    );

    if let Some(_cfg) = get_smtp_config() {
        let subject = vars.subject("email-payment-failed-subject", &[]);
        let amount = vars.amount_text();
        let body = vars.compose(
            "email-payment-failed-body",
            &[
                ("amount", &amount),
                ("currency", &vars.currency),
                ("invoice", &vars.invoice_id),
                ("url", SETTINGS_URL),
            ],
        );
        if let Err(e) = try_send_email(&vars.customer_email, &subject, &body) {
            tracing::warn!(target: "saas_notifier", "Failed to send payment failed email: {e}");
        }
    }
}

pub fn notify_subscription_cancelled(vars: &EmailVars) {
    tracing::info!(
        target: "saas_notifier",
        "[EMAIL] subscription_cancelled to={email} plan={plan}",
        email = vars.customer_email,
        plan = vars.plan,
    );

    if let Some(_cfg) = get_smtp_config() {
        let subject = vars.subject("email-subscription-cancelled-subject", &[]);
        let body = vars.compose(
            "email-subscription-cancelled-body",
            &[("plan", &vars.plan), ("url", OFFERS_URL)],
        );
        if let Err(e) = try_send_email(&vars.customer_email, &subject, &body) {
            tracing::warn!(target: "saas_notifier", "Failed to send cancellation email: {e}");
        }
    }
}

pub fn notify_grace_period_warning(vars: &EmailVars, days_left: u32) {
    tracing::warn!(
        target: "saas_notifier",
        "[EMAIL] grace_period_warning to={email} days_left={days}",
        email = vars.customer_email,
        days = days_left,
    );

    if let Some(_cfg) = get_smtp_config() {
        let subject = vars.subject("email-grace-warning-subject", &[]);
        let days = days_left.to_string();
        let body = vars.compose(
            "email-grace-warning-body",
            &[("days", &days), ("url", SETTINGS_URL)],
        );
        if let Err(e) = try_send_email(&vars.customer_email, &subject, &body) {
            tracing::warn!(target: "saas_notifier", "Failed to send grace period warning: {e}");
        }
    }
}

pub fn init_smtp(host: &str, port: u16, user: &str, password: &str, from: &str) {
    let _ = SMTP_CONFIG.set(SmtpConfig {
        host: host.to_string(),
        port,
        user: user.to_string(),
        password: password.to_string(),
        from: from.to_string(),
    });
}
