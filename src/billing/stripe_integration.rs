use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StripeClient {
    api_key: String,
    webhook_secret: Option<String>,
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeCustomer {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeSubscription {
    pub id: String,
    pub customer: String,
    pub status: StripeSubscriptionStatus,
    pub current_period_start: i64,
    pub current_period_end: i64,
    pub cancel_at_period_end: bool,
    pub canceled_at: Option<i64>,
    pub trial_start: Option<i64>,
    pub trial_end: Option<i64>,
    pub items: StripeSubscriptionItems,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeSubscriptionItems {
    pub data: Vec<StripeSubscriptionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeSubscriptionItem {
    pub id: String,
    pub price: StripePrice,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripePrice {
    pub id: String,
    pub product: String,
    pub unit_amount: Option<i64>,
    pub currency: String,
    pub recurring: Option<StripePriceRecurring>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripePriceRecurring {
    pub interval: String,
    pub interval_count: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StripeSubscriptionStatus {
    Active,
    Canceled,
    Incomplete,
    IncompleteExpired,
    PastDue,
    Paused,
    Trialing,
    Unpaid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeInvoice {
    pub id: String,
    pub customer: String,
    pub subscription: Option<String>,
    pub status: StripeInvoiceStatus,
    pub amount_due: i64,
    pub amount_paid: i64,
    pub currency: String,
    pub created: i64,
    pub hosted_invoice_url: Option<String>,
    pub invoice_pdf: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StripeInvoiceStatus {
    Draft,
    Open,
    Paid,
    Uncollectible,
    Void,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripePaymentMethod {
    pub id: String,
    pub customer: Option<String>,
    #[serde(rename = "type")]
    pub payment_type: String,
    pub card: Option<StripeCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeCard {
    pub brand: String,
    pub last4: String,
    pub exp_month: u32,
    pub exp_year: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeCheckoutSession {
    pub id: String,
    pub url: Option<String>,
    pub customer: Option<String>,
    pub subscription: Option<String>,
    pub status: String,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeBillingPortalSession {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct CreateCustomerParams {
    pub email: String,
    pub name: Option<String>,
    pub organization_id: Uuid,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CreateCheckoutSessionParams {
    pub customer_id: String,
    pub price_id: String,
    pub success_url: String,
    pub cancel_url: String,
    pub trial_days: Option<u32>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CreatePortalSessionParams {
    pub customer_id: String,
    pub return_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeWebhookEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: StripeWebhookData,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeWebhookData {
    pub object: serde_json::Value,
}

#[derive(Debug, Clone)]
pub enum StripeError {
    ApiError(String),
    NetworkError(String),
    InvalidWebhook(String),
    ParseError(String),
    NotConfigured,
}

impl std::fmt::Display for StripeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiError(e) => write!(f, "Stripe API error: {e}"),
            Self::NetworkError(e) => write!(f, "Network error: {e}"),
            Self::InvalidWebhook(e) => write!(f, "Invalid webhook: {e}"),
            Self::ParseError(e) => write!(f, "Parse error: {e}"),
            Self::NotConfigured => write!(f, "Stripe is not configured"),
        }
    }
}

impl std::error::Error for StripeError {}

impl StripeClient {
    pub fn new(api_key: String, webhook_secret: Option<String>) -> Self {
        Self {
            api_key,
            webhook_secret,
            client: reqwest::Client::new(),
            base_url: "https://api.stripe.com/v1".to_string(),
        }
    }

    pub async fn create_customer(&self, params: CreateCustomerParams) -> Result<StripeCustomer, StripeError> {
        let mut form: Vec<(String, String)> = vec![("email".to_string(), params.email)];

        if let Some(name) = params.name {
            form.push(("name".to_string(), name));
        }

        form.push(("metadata[organization_id]".to_string(), params.organization_id.to_string()));

        for (key, value) in params.metadata {
            form.push((format!("metadata[{key}]"), value));
        }

        let response = self
            .client
            .post(format!("{}/customers", self.base_url))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .form(&form)
            .send()
            .await
            .map_err(|e| StripeError::NetworkError(e.to_string()))?;

        self.handle_response(response).await
    }

    pub async fn get_customer(&self, customer_id: &str) -> Result<StripeCustomer, StripeError> {
        let response = self
            .client
            .get(format!("{}/customers/{}", self.base_url, customer_id))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .send()
            .await
            .map_err(|e| StripeError::NetworkError(e.to_string()))?;

        self.handle_response(response).await
    }

    pub async fn create_checkout_session(
        &self,
        params: CreateCheckoutSessionParams,
    ) -> Result<StripeCheckoutSession, StripeError> {
        let mut form: Vec<(String, String)> = vec![
            ("customer".to_string(), params.customer_id),
            ("mode".to_string(), "subscription".to_string()),
            ("success_url".to_string(), params.success_url),
            ("cancel_url".to_string(), params.cancel_url),
            ("line_items[0][price]".to_string(), params.price_id),
            ("line_items[0][quantity]".to_string(), "1".to_string()),
        ];

        if let Some(days) = params.trial_days {
            form.push(("subscription_data[trial_period_days]".to_string(), days.to_string()));
        }

        for (key, value) in params.metadata {
            form.push((format!("metadata[{key}]"), value));
        }

        let response = self
            .client
            .post(format!("{}/checkout/sessions", self.base_url))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .form(&form)
            .send()
            .await
            .map_err(|e| StripeError::NetworkError(e.to_string()))?;

        self.handle_response(response).await
    }

    pub async fn create_portal_session(
        &self,
        params: CreatePortalSessionParams,
    ) -> Result<StripeBillingPortalSession, StripeError> {
        let form: Vec<(String, String)> = vec![
            ("customer".to_string(), params.customer_id),
            ("return_url".to_string(), params.return_url),
        ];

        let response = self
            .client
            .post(format!("{}/billing_portal/sessions", self.base_url))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .form(&form)
            .send()
            .await
            .map_err(|e| StripeError::NetworkError(e.to_string()))?;

        self.handle_response(response).await
    }

    pub async fn get_subscription(&self, subscription_id: &str) -> Result<StripeSubscription, StripeError> {
        let response = self
            .client
            .get(format!("{}/subscriptions/{}", self.base_url, subscription_id))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .send()
            .await
            .map_err(|e| StripeError::NetworkError(e.to_string()))?;

        self.handle_response(response).await
    }

    pub async fn cancel_subscription(&self, subscription_id: &str, at_period_end: bool) -> Result<StripeSubscription, StripeError> {
        let form: Vec<(String, String)> = if at_period_end {
            vec![("cancel_at_period_end".to_string(), "true".to_string())]
        } else {
            vec![]
        };

        let url = if at_period_end {
            format!("{}/subscriptions/{}", self.base_url, subscription_id)
        } else {
            format!("{}/subscriptions/{}", self.base_url, subscription_id)
        };

        let request = if at_period_end {
            self.client.post(&url).form(&form)
        } else {
            self.client.delete(&url)
        };

        let response = request
            .basic_auth(&self.api_key, Option::<&str>::None)
            .send()
            .await
            .map_err(|e| StripeError::NetworkError(e.to_string()))?;

        self.handle_response(response).await
    }

    pub async fn update_subscription(
        &self,
        subscription_id: &str,
        new_price_id: &str,
    ) -> Result<StripeSubscription, StripeError> {
        let subscription = self.get_subscription(subscription_id).await?;

        let item_id = subscription
            .items
            .data
            .first()
            .map(|item| item.id.clone())
            .ok_or_else(|| StripeError::ApiError("No subscription items found".to_string()))?;

        let form: Vec<(String, String)> = vec![
            ("items[0][id]".to_string(), item_id),
            ("items[0][price]".to_string(), new_price_id.to_string()),
            ("proration_behavior".to_string(), "create_prorations".to_string()),
        ];

        let response = self
            .client
            .post(format!("{}/subscriptions/{}", self.base_url, subscription_id))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .form(&form)
            .send()
            .await
            .map_err(|e| StripeError::NetworkError(e.to_string()))?;

        self.handle_response(response).await
    }

    pub async fn list_invoices(&self, customer_id: &str, limit: u32) -> Result<Vec<StripeInvoice>, StripeError> {
        let response = self
            .client
            .get(format!("{}/invoices", self.base_url))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .query(&[("customer", customer_id), ("limit", &limit.to_string())])
            .send()
            .await
            .map_err(|e| StripeError::NetworkError(e.to_string()))?;

        #[derive(Deserialize)]
        struct InvoiceList {
            data: Vec<StripeInvoice>,
        }

        let list: InvoiceList = self.handle_response(response).await?;
        Ok(list.data)
    }

    pub async fn get_payment_methods(&self, customer_id: &str) -> Result<Vec<StripePaymentMethod>, StripeError> {
        let response = self
            .client
            .get(format!("{}/payment_methods", self.base_url))
            .basic_auth(&self.api_key, Option::<&str>::None)
            .query(&[("customer", customer_id), ("type", "card")])
            .send()
            .await
            .map_err(|e| StripeError::NetworkError(e.to_string()))?;

        #[derive(Deserialize)]
        struct PaymentMethodList {
            data: Vec<StripePaymentMethod>,
        }

        let list: PaymentMethodList = self.handle_response(response).await?;
        Ok(list.data)
    }

    pub fn verify_webhook_signature(&self, payload: &str, signature: &str) -> Result<StripeWebhookEvent, StripeError> {
        let webhook_secret = self
            .webhook_secret
            .as_ref()
            .ok_or(StripeError::NotConfigured)?;

        let parts: HashMap<&str, &str> = signature
            .split(',')
            .filter_map(|part| {
                let mut split = part.split('=');
                Some((split.next()?, split.next()?))
            })
            .collect();

        let timestamp = parts
            .get("t")
            .ok_or_else(|| StripeError::InvalidWebhook("Missing timestamp".to_string()))?;

        let received_sig = parts
            .get("v1")
            .ok_or_else(|| StripeError::InvalidWebhook("Missing signature".to_string()))?;

        let signed_payload = format!("{timestamp}.{payload}");

        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes())
            .map_err(|_| StripeError::InvalidWebhook("Invalid webhook secret".to_string()))?;

        mac.update(signed_payload.as_bytes());

        let expected_sig = hex::encode(mac.finalize().into_bytes());

        if expected_sig != *received_sig {
            return Err(StripeError::InvalidWebhook("Signature mismatch".to_string()));
        }

        let timestamp_i64: i64 = timestamp
            .parse()
            .map_err(|_| StripeError::InvalidWebhook("Invalid timestamp".to_string()))?;

        let now = chrono::Utc::now().timestamp();
        let tolerance = 300;

        if (now - timestamp_i64).abs() > tolerance {
            return Err(StripeError::InvalidWebhook("Timestamp too old".to_string()));
        }

        serde_json::from_str(payload).map_err(|e| StripeError::ParseError(e.to_string()))
    }

    pub fn parse_webhook_event(&self, event: &StripeWebhookEvent) -> Result<WebhookEventType, StripeError> {
        match event.event_type.as_str() {
            "customer.subscription.created" => {
                let subscription: StripeSubscription = serde_json::from_value(event.data.object.clone())
                    .map_err(|e| StripeError::ParseError(e.to_string()))?;
                Ok(WebhookEventType::SubscriptionCreated(subscription))
            }
            "customer.subscription.updated" => {
                let subscription: StripeSubscription = serde_json::from_value(event.data.object.clone())
                    .map_err(|e| StripeError::ParseError(e.to_string()))?;
                Ok(WebhookEventType::SubscriptionUpdated(subscription))
            }
            "customer.subscription.deleted" => {
                let subscription: StripeSubscription = serde_json::from_value(event.data.object.clone())
                    .map_err(|e| StripeError::ParseError(e.to_string()))?;
                Ok(WebhookEventType::SubscriptionCanceled(subscription))
            }
            "invoice.paid" => {
                let invoice: StripeInvoice = serde_json::from_value(event.data.object.clone())
                    .map_err(|e| StripeError::ParseError(e.to_string()))?;
                Ok(WebhookEventType::InvoicePaid(invoice))
            }
            "invoice.payment_failed" => {
                let invoice: StripeInvoice = serde_json::from_value(event.data.object.clone())
                    .map_err(|e| StripeError::ParseError(e.to_string()))?;
                Ok(WebhookEventType::InvoicePaymentFailed(invoice))
            }
            "checkout.session.completed" => {
                let session: StripeCheckoutSession = serde_json::from_value(event.data.object.clone())
                    .map_err(|e| StripeError::ParseError(e.to_string()))?;
                Ok(WebhookEventType::CheckoutCompleted(session))
            }
            _ => Ok(WebhookEventType::Unknown(event.event_type.clone())),
        }
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(&self, response: reqwest::Response) -> Result<T, StripeError> {
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| StripeError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            #[derive(Deserialize)]
            struct StripeApiError {
                error: StripeApiErrorDetail,
            }

            #[derive(Deserialize)]
            struct StripeApiErrorDetail {
                message: String,
            }

            if let Ok(error) = serde_json::from_str::<StripeApiError>(&body) {
                return Err(StripeError::ApiError(error.error.message));
            }

            return Err(StripeError::ApiError(format!("HTTP {}: {}", status, body)));
        }

        serde_json::from_str(&body).map_err(|e| StripeError::ParseError(e.to_string()))
    }
}

#[derive(Debug, Clone)]
pub enum WebhookEventType {
    SubscriptionCreated(StripeSubscription),
    SubscriptionUpdated(StripeSubscription),
    SubscriptionCanceled(StripeSubscription),
    InvoicePaid(StripeInvoice),
    InvoicePaymentFailed(StripeInvoice),
    CheckoutCompleted(StripeCheckoutSession),
    Unknown(String),
}

pub struct StripeWebhookHandler {
    client: StripeClient,
}

impl StripeWebhookHandler {
    pub fn new(client: StripeClient) -> Self {
        Self { client }
    }

    pub async fn handle_webhook(&self, payload: &str, signature: &str) -> Result<WebhookAction, StripeError> {
        let event = self.client.verify_webhook_signature(payload, signature)?;
        let event_type = self.client.parse_webhook_event(&event)?;

        match event_type {
            WebhookEventType::SubscriptionCreated(sub) => {
                Ok(WebhookAction::ActivateSubscription {
                    stripe_subscription_id: sub.id,
                    stripe_customer_id: sub.customer,
                    status: sub.status,
                    period_end: sub.current_period_end,
                })
            }
            WebhookEventType::SubscriptionUpdated(sub) => {
                Ok(WebhookAction::UpdateSubscription {
                    stripe_subscription_id: sub.id,
                    status: sub.status,
                    cancel_at_period_end: sub.cancel_at_period_end,
                    period_end: sub.current_period_end,
                })
            }
            WebhookEventType::SubscriptionCanceled(sub) => {
                Ok(WebhookAction::CancelSubscription {
                    stripe_subscription_id: sub.id,
                })
            }
            WebhookEventType::InvoicePaid(invoice) => {
                Ok(WebhookAction::RecordPayment {
                    stripe_invoice_id: invoice.id,
                    amount: invoice.amount_paid,
                    currency: invoice.currency,
                })
            }
            WebhookEventType::InvoicePaymentFailed(invoice) => {
                Ok(WebhookAction::PaymentFailed {
                    stripe_invoice_id: invoice.id,
                    stripe_customer_id: invoice.customer,
                })
            }
            WebhookEventType::CheckoutCompleted(session) => {
                Ok(WebhookAction::CheckoutCompleted {
                    stripe_customer_id: session.customer,
                    stripe_subscription_id: session.subscription,
                })
            }
            WebhookEventType::Unknown(event_type) => {
                tracing::debug!("Unhandled Stripe webhook event: {}", event_type);
                Ok(WebhookAction::None)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum WebhookAction {
    ActivateSubscription {
        stripe_subscription_id: String,
        stripe_customer_id: String,
        status: StripeSubscriptionStatus,
        period_end: i64,
    },
    UpdateSubscription {
        stripe_subscription_id: String,
        status: StripeSubscriptionStatus,
        cancel_at_period_end: bool,
        period_end: i64,
    },
    CancelSubscription {
        stripe_subscription_id: String,
    },
    RecordPayment {
        stripe_invoice_id: String,
        amount: i64,
        currency: String,
    },
    PaymentFailed {
        stripe_invoice_id: String,
        stripe_customer_id: String,
    },
    CheckoutCompleted {
        stripe_customer_id: Option<String>,
        stripe_subscription_id: Option<String>,
    },
    None,
}
