use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockStripeCustomer {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created: i64,
    pub default_source: Option<String>,
    pub invoice_settings: MockInvoiceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockInvoiceSettings {
    pub default_payment_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockStripeSubscription {
    pub id: String,
    pub customer: String,
    pub status: SubscriptionStatus,
    pub current_period_start: i64,
    pub current_period_end: i64,
    pub items: MockSubscriptionItems,
    pub metadata: HashMap<String, String>,
    pub cancel_at_period_end: bool,
    pub canceled_at: Option<i64>,
    pub trial_start: Option<i64>,
    pub trial_end: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    PastDue,
    Unpaid,
    Canceled,
    Incomplete,
    IncompleteExpired,
    Trialing,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockSubscriptionItems {
    pub data: Vec<MockSubscriptionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockSubscriptionItem {
    pub id: String,
    pub price: MockPrice,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockPrice {
    pub id: String,
    pub product: String,
    pub unit_amount: i64,
    pub currency: String,
    pub recurring: Option<MockRecurring>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRecurring {
    pub interval: String,
    pub interval_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockStripeInvoice {
    pub id: String,
    pub customer: String,
    pub subscription: Option<String>,
    pub status: InvoiceStatus,
    pub amount_due: i64,
    pub amount_paid: i64,
    pub amount_remaining: i64,
    pub currency: String,
    pub created: i64,
    pub due_date: Option<i64>,
    pub paid: bool,
    pub lines: MockInvoiceLines,
    pub hosted_invoice_url: Option<String>,
    pub invoice_pdf: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Draft,
    Open,
    Paid,
    Uncollectible,
    Void,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockInvoiceLines {
    pub data: Vec<MockInvoiceLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockInvoiceLine {
    pub id: String,
    pub amount: i64,
    pub currency: String,
    pub description: Option<String>,
    pub quantity: u32,
    pub price: MockPrice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockStripePaymentIntent {
    pub id: String,
    pub amount: i64,
    pub currency: String,
    pub status: PaymentIntentStatus,
    pub customer: Option<String>,
    pub payment_method: Option<String>,
    pub client_secret: String,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentIntentStatus {
    RequiresPaymentMethod,
    RequiresConfirmation,
    RequiresAction,
    Processing,
    RequiresCapture,
    Canceled,
    Succeeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockStripePaymentMethod {
    pub id: String,
    pub customer: Option<String>,
    pub payment_type: String,
    pub card: Option<MockCard>,
    pub created: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockCard {
    pub brand: String,
    pub last4: String,
    pub exp_month: u32,
    pub exp_year: u32,
    pub funding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockStripeEvent {
    pub id: String,
    pub event_type: String,
    pub created: i64,
    pub data: MockEventData,
    pub livemode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockEventData {
    pub object: serde_json::Value,
}

pub struct MockStripeClient {
    customers: Arc<RwLock<HashMap<String, MockStripeCustomer>>>,
    subscriptions: Arc<RwLock<HashMap<String, MockStripeSubscription>>>,
    invoices: Arc<RwLock<HashMap<String, MockStripeInvoice>>>,
    payment_intents: Arc<RwLock<HashMap<String, MockStripePaymentIntent>>>,
    payment_methods: Arc<RwLock<HashMap<String, MockStripePaymentMethod>>>,
    events: Arc<RwLock<Vec<MockStripeEvent>>>,
    prices: Arc<RwLock<HashMap<String, MockPrice>>>,
    should_fail: Arc<RwLock<bool>>,
    failure_code: Arc<RwLock<Option<String>>>,
}

impl MockStripeClient {
    pub fn new() -> Self {
        let client = Self {
            customers: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            invoices: Arc::new(RwLock::new(HashMap::new())),
            payment_intents: Arc::new(RwLock::new(HashMap::new())),
            payment_methods: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            prices: Arc::new(RwLock::new(HashMap::new())),
            should_fail: Arc::new(RwLock::new(false)),
            failure_code: Arc::new(RwLock::new(None)),
        };

        tokio::spawn({
            let prices = client.prices.clone();
            async move {
                let mut p = prices.write().await;
                p.insert(
                    "price_free".to_string(),
                    MockPrice {
                        id: "price_free".to_string(),
                        product: "prod_free".to_string(),
                        unit_amount: 0,
                        currency: "usd".to_string(),
                        recurring: Some(MockRecurring {
                            interval: "month".to_string(),
                            interval_count: 1,
                        }),
                    },
                );
                p.insert(
                    "price_starter".to_string(),
                    MockPrice {
                        id: "price_starter".to_string(),
                        product: "prod_starter".to_string(),
                        unit_amount: 2900,
                        currency: "usd".to_string(),
                        recurring: Some(MockRecurring {
                            interval: "month".to_string(),
                            interval_count: 1,
                        }),
                    },
                );
                p.insert(
                    "price_pro".to_string(),
                    MockPrice {
                        id: "price_pro".to_string(),
                        product: "prod_pro".to_string(),
                        unit_amount: 4900,
                        currency: "usd".to_string(),
                        recurring: Some(MockRecurring {
                            interval: "month".to_string(),
                            interval_count: 1,
                        }),
                    },
                );
                p.insert(
                    "price_enterprise".to_string(),
                    MockPrice {
                        id: "price_enterprise".to_string(),
                        product: "prod_enterprise".to_string(),
                        unit_amount: 19900,
                        currency: "usd".to_string(),
                        recurring: Some(MockRecurring {
                            interval: "month".to_string(),
                            interval_count: 1,
                        }),
                    },
                );
            }
        });

        client
    }

    pub async fn set_should_fail(&self, should_fail: bool, code: Option<String>) {
        let mut fail = self.should_fail.write().await;
        *fail = should_fail;
        let mut failure_code = self.failure_code.write().await;
        *failure_code = code;
    }

    async fn check_failure(&self) -> Result<(), MockStripeError> {
        let should_fail = *self.should_fail.read().await;
        if should_fail {
            let code = self.failure_code.read().await.clone();
            return Err(MockStripeError::ApiError(
                code.unwrap_or_else(|| "card_declined".to_string()),
            ));
        }
        Ok(())
    }

    pub async fn create_customer(
        &self,
        email: &str,
        name: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> Result<MockStripeCustomer, MockStripeError> {
        self.check_failure().await?;

        let customer = MockStripeCustomer {
            id: format!("cus_{}", generate_stripe_id()),
            email: email.to_string(),
            name: name.map(|s| s.to_string()),
            metadata,
            created: Utc::now().timestamp(),
            default_source: None,
            invoice_settings: MockInvoiceSettings {
                default_payment_method: None,
            },
        };

        let mut customers = self.customers.write().await;
        customers.insert(customer.id.clone(), customer.clone());

        Ok(customer)
    }

    pub async fn get_customer(&self, customer_id: &str) -> Result<MockStripeCustomer, MockStripeError> {
        self.check_failure().await?;

        let customers = self.customers.read().await;
        customers
            .get(customer_id)
            .cloned()
            .ok_or_else(|| MockStripeError::NotFound(format!("Customer {customer_id} not found")))
    }

    pub async fn update_customer(
        &self,
        customer_id: &str,
        email: Option<&str>,
        name: Option<&str>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<MockStripeCustomer, MockStripeError> {
        self.check_failure().await?;

        let mut customers = self.customers.write().await;
        let customer = customers
            .get_mut(customer_id)
            .ok_or_else(|| MockStripeError::NotFound(format!("Customer {customer_id} not found")))?;

        if let Some(e) = email {
            customer.email = e.to_string();
        }
        if let Some(n) = name {
            customer.name = Some(n.to_string());
        }
        if let Some(m) = metadata {
            customer.metadata = m;
        }

        Ok(customer.clone())
    }

    pub async fn delete_customer(&self, customer_id: &str) -> Result<(), MockStripeError> {
        self.check_failure().await?;

        let mut customers = self.customers.write().await;
        customers
            .remove(customer_id)
            .ok_or_else(|| MockStripeError::NotFound(format!("Customer {customer_id} not found")))?;

        Ok(())
    }

    pub async fn create_subscription(
        &self,
        customer_id: &str,
        price_id: &str,
        metadata: HashMap<String, String>,
        trial_days: Option<u32>,
    ) -> Result<MockStripeSubscription, MockStripeError> {
        self.check_failure().await?;

        let customers = self.customers.read().await;
        if !customers.contains_key(customer_id) {
            return Err(MockStripeError::NotFound(format!(
                "Customer {customer_id} not found"
            )));
        }
        drop(customers);

        let prices = self.prices.read().await;
        let price = prices
            .get(price_id)
            .cloned()
            .ok_or_else(|| MockStripeError::NotFound(format!("Price {price_id} not found")))?;
        drop(prices);

        let now = Utc::now();
        let (trial_start, trial_end, status) = if let Some(days) = trial_days {
            let ts = now.timestamp();
            let te = (now + Duration::days(days as i64)).timestamp();
            (Some(ts), Some(te), SubscriptionStatus::Trialing)
        } else {
            (None, None, SubscriptionStatus::Active)
        };

        let period_end = now + Duration::days(30);

        let subscription = MockStripeSubscription {
            id: format!("sub_{}", generate_stripe_id()),
            customer: customer_id.to_string(),
            status,
            current_period_start: now.timestamp(),
            current_period_end: period_end.timestamp(),
            items: MockSubscriptionItems {
                data: vec![MockSubscriptionItem {
                    id: format!("si_{}", generate_stripe_id()),
                    price,
                    quantity: 1,
                }],
            },
            metadata,
            cancel_at_period_end: false,
            canceled_at: None,
            trial_start,
            trial_end,
        };

        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription.id.clone(), subscription.clone());

        self.emit_event("customer.subscription.created", &subscription).await;

        Ok(subscription)
    }

    pub async fn get_subscription(
        &self,
        subscription_id: &str,
    ) -> Result<MockStripeSubscription, MockStripeError> {
        self.check_failure().await?;

        let subscriptions = self.subscriptions.read().await;
        subscriptions
            .get(subscription_id)
            .cloned()
            .ok_or_else(|| MockStripeError::NotFound(format!("Subscription {subscription_id} not found")))
    }

    pub async fn update_subscription(
        &self,
        subscription_id: &str,
        price_id: Option<&str>,
        cancel_at_period_end: Option<bool>,
    ) -> Result<MockStripeSubscription, MockStripeError> {
        self.check_failure().await?;

        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions.get_mut(subscription_id).ok_or_else(|| {
            MockStripeError::NotFound(format!("Subscription {subscription_id} not found"))
        })?;

        if let Some(pid) = price_id {
            let prices = self.prices.read().await;
            let price = prices
                .get(pid)
                .cloned()
                .ok_or_else(|| MockStripeError::NotFound(format!("Price {pid} not found")))?;

            subscription.items.data[0].price = price;
        }

        if let Some(cancel) = cancel_at_period_end {
            subscription.cancel_at_period_end = cancel;
        }

        let result = subscription.clone();
        drop(subscriptions);

        self.emit_event("customer.subscription.updated", &result).await;

        Ok(result)
    }

    pub async fn cancel_subscription(
        &self,
        subscription_id: &str,
        immediately: bool,
    ) -> Result<MockStripeSubscription, MockStripeError> {
        self.check_failure().await?;

        let mut subscriptions = self.subscriptions.write().await;
        let subscription = subscriptions.get_mut(subscription_id).ok_or_else(|| {
            MockStripeError::NotFound(format!("Subscription {subscription_id} not found"))
        })?;

        if immediately {
            subscription.status = SubscriptionStatus::Canceled;
            subscription.canceled_at = Some(Utc::now().timestamp());
        } else {
            subscription.cancel_at_period_end = true;
        }

        let result = subscription.clone();
        drop(subscriptions);

        self.emit_event("customer.subscription.deleted", &result).await;

        Ok(result)
    }

    pub async fn create_invoice(
        &self,
        customer_id: &str,
        subscription_id: Option<&str>,
        amount: i64,
    ) -> Result<MockStripeInvoice, MockStripeError> {
        self.check_failure().await?;

        let customers = self.customers.read().await;
        if !customers.contains_key(customer_id) {
            return Err(MockStripeError::NotFound(format!(
                "Customer {customer_id} not found"
            )));
        }
        drop(customers);

        let invoice_id = format!("in_{}", generate_stripe_id());

        let invoice = MockStripeInvoice {
            id: invoice_id.clone(),
            customer: customer_id.to_string(),
            subscription: subscription_id.map(|s| s.to_string()),
            status: InvoiceStatus::Draft,
            amount_due: amount,
            amount_paid: 0,
            amount_remaining: amount,
            currency: "usd".to_string(),
            created: Utc::now().timestamp(),
            due_date: Some((Utc::now() + Duration::days(30)).timestamp()),
            paid: false,
            lines: MockInvoiceLines {
                data: vec![MockInvoiceLine {
                    id: format!("il_{}", generate_stripe_id()),
                    amount,
                    currency: "usd".to_string(),
                    description: Some("Subscription".to_string()),
                    quantity: 1,
                    price: MockPrice {
                        id: "price_auto".to_string(),
                        product: "prod_auto".to_string(),
                        unit_amount: amount,
                        currency: "usd".to_string(),
                        recurring: None,
                    },
                }],
            },
            hosted_invoice_url: Some(format!("https://invoice.stripe.com/i/{invoice_id}")),
            invoice_pdf: Some(format!("https://invoice.stripe.com/i/{invoice_id}/pdf")),
        };

        let mut invoices = self.invoices.write().await;
        invoices.insert(invoice.id.clone(), invoice.clone());

        Ok(invoice)
    }

    pub async fn get_invoice(&self, invoice_id: &str) -> Result<MockStripeInvoice, MockStripeError> {
        self.check_failure().await?;

        let invoices = self.invoices.read().await;
        invoices
            .get(invoice_id)
            .cloned()
            .ok_or_else(|| MockStripeError::NotFound(format!("Invoice {invoice_id} not found")))
    }

    pub async fn finalize_invoice(&self, invoice_id: &str) -> Result<MockStripeInvoice, MockStripeError> {
        self.check_failure().await?;

        let mut invoices = self.invoices.write().await;
        let invoice = invoices
            .get_mut(invoice_id)
            .ok_or_else(|| MockStripeError::NotFound(format!("Invoice {invoice_id} not found")))?;

        invoice.status = InvoiceStatus::Open;

        let result = invoice.clone();
        drop(invoices);

        self.emit_event("invoice.finalized", &result).await;

        Ok(result)
    }

    pub async fn pay_invoice(&self, invoice_id: &str) -> Result<MockStripeInvoice, MockStripeError> {
        self.check_failure().await?;

        let mut invoices = self.invoices.write().await;
        let invoice = invoices
            .get_mut(invoice_id)
            .ok_or_else(|| MockStripeError::NotFound(format!("Invoice {invoice_id} not found")))?;

        invoice.status = InvoiceStatus::Paid;
        invoice.paid = true;
        invoice.amount_paid = invoice.amount_due;
        invoice.amount_remaining = 0;

        let result = invoice.clone();
        drop(invoices);

        self.emit_event("invoice.paid", &result).await;

        Ok(result)
    }

    pub async fn void_invoice(&self, invoice_id: &str) -> Result<MockStripeInvoice, MockStripeError> {
        self.check_failure().await?;

        let mut invoices = self.invoices.write().await;
        let invoice = invoices
            .get_mut(invoice_id)
            .ok_or_else(|| MockStripeError::NotFound(format!("Invoice {invoice_id} not found")))?;

        invoice.status = InvoiceStatus::Void;

        let result = invoice.clone();
        drop(invoices);

        self.emit_event("invoice.voided", &result).await;

        Ok(result)
    }

    pub async fn create_payment_intent(
        &self,
        amount: i64,
        currency: &str,
        customer_id: Option<&str>,
    ) -> Result<MockStripePaymentIntent, MockStripeError> {
        self.check_failure().await?;

        let payment_intent = MockStripePaymentIntent {
            id: format!("pi_{}", generate_stripe_id()),
            amount,
            currency: currency.to_string(),
            status: PaymentIntentStatus::RequiresPaymentMethod,
            customer: customer_id.map(|s| s.to_string()),
            payment_method: None,
            client_secret: format!("pi_{}_secret_{}", generate_stripe_id(), generate_stripe_id()),
            created: Utc::now().timestamp(),
        };

        let mut payment_intents = self.payment_intents.write().await;
        payment_intents.insert(payment_intent.id.clone(), payment_intent.clone());

        Ok(payment_intent)
    }

    pub async fn confirm_payment_intent(
        &self,
        payment_intent_id: &str,
        payment_method_id: &str,
    ) -> Result<MockStripePaymentIntent, MockStripeError> {
        self.check_failure().await?;

        let mut payment_intents = self.payment_intents.write().await;
        let pi = payment_intents.get_mut(payment_intent_id).ok_or_else(|| {
            MockStripeError::NotFound(format!("PaymentIntent {payment_intent_id} not found"))
        })?;

        pi.payment_method = Some(payment_method_id.to_string());
        pi.status = PaymentIntentStatus::Succeeded;

        let result = pi.clone();
        drop(payment_intents);

        self.emit_event("payment_intent.succeeded", &result).await;

        Ok(result)
    }

    pub async fn create_payment_method(
        &self,
        card_brand: &str,
        last4: &str,
        exp_month: u32,
        exp_year: u32,
    ) -> Result<MockStripePaymentMethod, MockStripeError> {
        self.check_failure().await?;

        let pm = MockStripePaymentMethod {
            id: format!("pm_{}", generate_stripe_id()),
            customer: None,
            payment_type: "card".to_string(),
            card: Some(MockCard {
                brand: card_brand.to_string(),
                last4: last4.to_string(),
                exp_month,
                exp_year,
                funding: "credit".to_string(),
            }),
            created: Utc::now().timestamp(),
        };

        let mut payment_methods = self.payment_methods.write().await;
        payment_methods.insert(pm.id.clone(), pm.clone());

        Ok(pm)
    }

    pub async fn attach_payment_method(
        &self,
        payment_method_id: &str,
        customer_id: &str,
    ) -> Result<MockStripePaymentMethod, MockStripeError> {
        self.check_failure().await?;

        let customers = self.customers.read().await;
        if !customers.contains_key(customer_id) {
            return Err(MockStripeError::NotFound(format!(
                "Customer {customer_id} not found"
            )));
        }
        drop(customers);

        let mut payment_methods = self.payment_methods.write().await;
        let pm = payment_methods.get_mut(payment_method_id).ok_or_else(|| {
            MockStripeError::NotFound(format!("PaymentMethod {payment_method_id} not found"))
        })?;

        pm.customer = Some(customer_id.to_string());

        Ok(pm.clone())
    }

    pub async fn get_events(&self, limit: usize) -> Vec<MockStripeEvent> {
        let events = self.events.read().await;
        events.iter().rev().take(limit).cloned().collect()
    }

    async fn emit_event<T: Serialize>(&self, event_type: &str, data: &T) {
        let event = MockStripeEvent {
            id: format!("evt_{}", generate_stripe_id()),
            event_type: event_type.to_string(),
            created: Utc::now().timestamp(),
            data: MockEventData {
                object: serde_json::to_value(data).unwrap_or(serde_json::Value::Null),
            },
            livemode: false,
        };

        let mut events = self.events.write().await;
        events.push(event);

        if events.len() > 1000 {
            events.drain(0..100);
        }
    }
}

impl Default for MockStripeClient {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_stripe_id() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string().replace('-', "")[..24].to_string()
}

#[derive(Debug, Clone)]
pub enum MockStripeError {
    NotFound(String),
    ApiError(String),
    InvalidRequest(String),
    AuthenticationError,
    RateLimitError,
    NetworkError(String),
}

impl std::fmt::Display for MockStripeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::ApiError(code) => write!(f, "API error: {code}"),
            Self::InvalidRequest(msg) => write!(f, "Invalid request: {msg}"),
            Self::AuthenticationError => write!(f, "Authentication failed"),
            Self::RateLimitError => write!(f, "Rate limit exceeded"),
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
        }
    }
}

impl std::error::Error for MockStripeError {}

#[derive(Debug, Clone)]
pub struct BillingTestScenario {
    pub name: String,
    pub description: String,
    pub steps: Vec<TestStep>,
}

#[derive(Debug, Clone)]
pub struct TestStep {
    pub action: TestAction,
    pub expected_result: ExpectedResult,
    pub delay_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum TestAction {
    CreateCustomer { email: String, name: Option<String> },
    CreateSubscription { plan: String, trial_days: Option<u32> },
    UpgradeSubscription { new_plan: String },
    DowngradeSubscription { new_plan: String },
    CancelSubscription { immediately: bool },
    SimulatePaymentFailure,
    SimulatePaymentSuccess,
    ProcessInvoice,
    ApplyDiscount { code: String },
    RecordUsage { metric: String, quantity: i64 },
    CheckQuota { metric: String },
}

#[derive(Debug, Clone)]
pub enum ExpectedResult {
    Success,
    SubscriptionStatus(SubscriptionStatus),
    InvoiceStatus(InvoiceStatus),
    QuotaExceeded,
    QuotaAvailable { remaining: i64 },
    Error { code: String },
}

pub fn create_standard_test_scenarios() -> Vec<BillingTestScenario> {
    vec![
        BillingTestScenario {
            name: "New Customer Signup".to_string(),
            description: "Test new customer creating account and subscribing".to_string(),
            steps: vec![
                TestStep {
                    action: TestAction::CreateCustomer {
                        email: "test@example.com".to_string(),
                        name: Some("Test User".to_string()),
                    },
                    expected_result: ExpectedResult::Success,
                    delay_ms: None,
                },
                TestStep {
                    action: TestAction::CreateSubscription {
                        plan: "starter".to_string(),
                        trial_days: Some(14),
                    },
                    expected_result: ExpectedResult::SubscriptionStatus(SubscriptionStatus::Trialing),
                    delay_ms: None,
                },
            ],
        },
        BillingTestScenario {
            name: "Plan Upgrade".to_string(),
            description: "Test customer upgrading from starter to pro".to_string(),
            steps: vec![
                TestStep {
                    action: TestAction::CreateCustomer {
                        email: "upgrade@example.com".to_string(),
                        name: None,
                    },
                    expected_result: ExpectedResult::Success,
                    delay_ms: None,
                },
                TestStep {
                    action: TestAction::CreateSubscription {
                        plan: "starter".to_string(),
                        trial_days: None,
                    },
                    expected_result: ExpectedResult::SubscriptionStatus(SubscriptionStatus::Active),
                    delay_ms: None,
                },
                TestStep {
                    action: TestAction::UpgradeSubscription {
                        new_plan: "pro".to_string(),
                    },
                    expected_result: ExpectedResult::Success,
                    delay_ms: Some(100),
                },
            ],
        },
        BillingTestScenario {
            name: "Payment Failure Recovery".to_string(),
            description: "Test handling payment failure and recovery".to_string(),
            steps: vec![
                TestStep {
                    action: TestAction::CreateCustomer {
                        email: "failure@example.com".to_string(),
                        name: None,
                    },
                    expected_result: ExpectedResult::Success,
                    delay_ms: None,
                },
                TestStep {
                    action: TestAction::CreateSubscription {
                        plan: "pro".to_string(),
                        trial_days: None,
                    },
                    expected_result: ExpectedResult::SubscriptionStatus(SubscriptionStatus::Active),
                    delay_ms: None,
                },
                TestStep {
                    action: TestAction::SimulatePaymentFailure,
                    expected_result: ExpectedResult::SubscriptionStatus(SubscriptionStatus::PastDue),
                    delay_ms: Some(50),
                },
                TestStep {
                    action: TestAction::SimulatePaymentSuccess,
                    expected_result: ExpectedResult::SubscriptionStatus(SubscriptionStatus::Active),
                    delay_ms: Some(50),
                },
            ],
        },
    ]
}
