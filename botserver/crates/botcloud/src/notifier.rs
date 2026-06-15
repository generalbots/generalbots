use std::collections::HashMap;

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
        }
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
}

pub fn notify_subscription_activated(vars: &EmailVars) {
    tracing::info!(
        target: "saas_notifier",
        "[EMAIL] subscription_activated to={email} plan={plan}",
        email = vars.customer_email,
        plan = vars.plan,
    );
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
}

pub fn notify_subscription_cancelled(vars: &EmailVars) {
    tracing::info!(
        target: "saas_notifier",
        "[EMAIL] subscription_cancelled to={email} plan={plan}",
        email = vars.customer_email,
        plan = vars.plan,
    );
}

pub fn notify_grace_period_warning(vars: &EmailVars, days_left: u32) {
    tracing::warn!(
        target: "saas_notifier",
        "[EMAIL] grace_period_warning to={email} days_left={days}",
        email = vars.customer_email,
        days = days_left,
    );
}
