use super::{
    AssertionRecord, AssertionResult, BotResponse, ConversationConfig, ConversationRecord,
    ConversationState, RecordedMessage, ResponseContentType,
};
use crate::fixtures::{Channel, Customer, MessageDirection};
use crate::harness::TestContext;
use crate::mocks::MockLLM;
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub struct ConversationBuilder {
    bot_name: String,
    customer: Option<Customer>,
    channel: Channel,
    config: ConversationConfig,
    initial_context: HashMap<String, serde_json::Value>,
    mock_llm: Option<Arc<MockLLM>>,
}

impl ConversationBuilder {
    pub fn new(bot_name: &str) -> Self {
        Self {
            bot_name: bot_name.to_string(),
            customer: None,
            channel: Channel::WhatsApp,
            config: ConversationConfig::default(),
            initial_context: HashMap::new(),
            mock_llm: None,
        }
    }

    pub fn with_customer(mut self, customer: Customer) -> Self {
        self.customer = Some(customer);
        self
    }

    pub const fn on_channel(mut self, channel: Channel) -> Self {
        self.channel = channel;
        self
    }

    pub fn with_config(mut self, config: ConversationConfig) -> Self {
        self.config = config;
        self
    }

    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.response_timeout = timeout;
        self
    }

    pub fn with_context(mut self, key: &str, value: serde_json::Value) -> Self {
        self.initial_context.insert(key.to_string(), value);
        self
    }

    pub const fn without_recording(mut self) -> Self {
        self.config.record = false;
        self
    }

    pub const fn with_real_llm(mut self) -> Self {
        self.config.use_mock_llm = false;
        self
    }

    pub fn with_mock_llm(mut self, mock: Arc<MockLLM>) -> Self {
        self.mock_llm = Some(mock);
        self.config.use_mock_llm = true;
        self
    }

    pub fn build(self) -> ConversationTest {
        let customer = self.customer.unwrap_or_else(|| Customer {
            channel: self.channel,
            ..Default::default()
        });

        let bot_name_for_record = self.bot_name.clone();

        ConversationTest {
            id: Uuid::new_v4(),
            bot_name: self.bot_name,
            customer,
            channel: self.channel,
            config: self.config,
            state: ConversationState::Initial,
            responses: Vec::new(),
            sent_messages: Vec::new(),
            record: ConversationRecord {
                id: Uuid::new_v4(),
                bot_name: bot_name_for_record,
                started_at: Utc::now(),
                ended_at: None,
                messages: Vec::new(),
                assertions: Vec::new(),
                passed: true,
            },
            context: self.initial_context,
            last_response: None,
            last_latency: None,
            mock_llm: self.mock_llm,
            llm_url: None,
        }
    }
}

pub struct ConversationTest {
    id: Uuid,
    bot_name: String,
    customer: Customer,
    channel: Channel,
    config: ConversationConfig,
    state: ConversationState,
    responses: Vec<BotResponse>,
    sent_messages: Vec<String>,
    record: ConversationRecord,
    context: HashMap<String, serde_json::Value>,
    last_response: Option<BotResponse>,
    last_latency: Option<Duration>,
    mock_llm: Option<Arc<MockLLM>>,
    llm_url: Option<String>,
}

impl ConversationTest {
    pub fn new(bot_name: &str) -> Self {
        ConversationBuilder::new(bot_name).build()
    }

    pub fn with_context(ctx: &TestContext, bot_name: &str) -> Result<Self> {
        let mut conv = ConversationBuilder::new(bot_name).build();
        conv.llm_url = Some(ctx.llm_url());
        Ok(conv)
    }

    pub const fn id(&self) -> Uuid {
        self.id
    }

    pub fn bot_name(&self) -> &str {
        &self.bot_name
    }

    pub const fn customer(&self) -> &Customer {
        &self.customer
    }

    pub const fn channel(&self) -> Channel {
        self.channel
    }

    pub const fn state(&self) -> ConversationState {
        self.state
    }

    pub fn responses(&self) -> &[BotResponse] {
        &self.responses
    }

    pub fn sent_messages(&self) -> &[String] {
        &self.sent_messages
    }

    pub const fn last_response(&self) -> Option<&BotResponse> {
        self.last_response.as_ref()
    }

    pub const fn last_latency(&self) -> Option<Duration> {
        self.last_latency
    }

    pub const fn record(&self) -> &ConversationRecord {
        &self.record
    }

    pub async fn user_says(&mut self, message: &str) -> &mut Self {
        self.sent_messages.push(message.to_string());

        if self.config.record {
            self.record.messages.push(RecordedMessage {
                timestamp: Utc::now(),
                direction: MessageDirection::Incoming,
                content: message.to_string(),
                latency_ms: None,
            });
        }

        self.state = ConversationState::WaitingForBot;

        let start = Instant::now();
        let response = self.get_bot_response(message).await;
        let latency = start.elapsed();

        self.last_latency = Some(latency);
        self.last_response = Some(response.clone());
        self.responses.push(response.clone());

        if self.config.record {
            self.record.messages.push(RecordedMessage {
                timestamp: Utc::now(),
                direction: MessageDirection::Outgoing,
                content: response.content,
                latency_ms: Some(latency.as_millis() as u64),
            });
        }

        self.state = ConversationState::WaitingForUser;
        self
    }

    async fn get_bot_response(&self, user_message: &str) -> BotResponse {
        let start = Instant::now();

        if self.config.use_mock_llm {
            if let Some(ref mock) = self.mock_llm {
                let mock_url = mock.url();
                if let Ok(content) = self.call_llm_api(&mock_url, user_message).await {
                    return BotResponse {
                        id: Uuid::new_v4(),
                        content,
                        content_type: ResponseContentType::Text,
                        metadata: self.build_response_metadata(),
                        latency_ms: start.elapsed().as_millis() as u64,
                    };
                }
            } else if let Some(ref llm_url) = self.llm_url {
                if let Ok(content) = self.call_llm_api(llm_url, user_message).await {
                    return BotResponse {
                        id: Uuid::new_v4(),
                        content,
                        content_type: ResponseContentType::Text,
                        metadata: self.build_response_metadata(),
                        latency_ms: start.elapsed().as_millis() as u64,
                    };
                }
            }
        }

        BotResponse {
            id: Uuid::new_v4(),
            content: format!("Response to: {user_message}"),
            content_type: ResponseContentType::Text,
            metadata: self.build_response_metadata(),
            latency_ms: start.elapsed().as_millis() as u64,
        }
    }

    async fn call_llm_api(&self, llm_url: &str, message: &str) -> Result<String> {
        let client = reqwest::Client::builder()
            .timeout(self.config.response_timeout)
            .build()?;

        let request_body = serde_json::json!({
            "model": "gpt-4",
            "messages": [
                {
                    "role": "system",
                    "content": format!("You are a helpful assistant for bot '{}'.", self.bot_name)
                },
                {
                    "role": "user",
                    "content": message
                }
            ]
        });

        let response = client
            .post(format!("{llm_url}/v1/chat/completions"))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("No response")
            .to_string();

        Ok(content)
    }

    fn build_response_metadata(&self) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();
        metadata.insert(
            "bot_name".to_string(),
            serde_json::Value::String(self.bot_name.clone()),
        );
        metadata.insert(
            "customer_id".to_string(),
            serde_json::Value::String(self.customer.id.to_string()),
        );
        metadata.insert(
            "channel".to_string(),
            serde_json::Value::String(format!("{:?}", self.channel)),
        );
        metadata.insert(
            "conversation_id".to_string(),
            serde_json::Value::String(self.id.to_string()),
        );
        metadata
    }

    pub fn assert_response_contains(&mut self, text: &str) -> &mut Self {
        let result = if let Some(ref response) = self.last_response {
            if response.content.contains(text) {
                AssertionResult::pass(&format!("Response contains '{text}'"))
            } else {
                AssertionResult::fail(
                    &format!("Response should contain '{text}'"),
                    text,
                    &response.content,
                )
            }
        } else {
            AssertionResult::fail("No response to check", text, "<no response>")
        };

        self.record_assertion("contains", &result);
        self
    }

    pub fn assert_response_equals(&mut self, text: &str) -> &mut Self {
        let result = if let Some(ref response) = self.last_response {
            if response.content == text {
                AssertionResult::pass(&format!("Response equals '{text}'"))
            } else {
                AssertionResult::fail(
                    "Response should equal expected text",
                    text,
                    &response.content,
                )
            }
        } else {
            AssertionResult::fail("No response to check", text, "<no response>")
        };

        self.record_assertion("equals", &result);
        self
    }

    pub fn assert_response_matches(&mut self, pattern: &str) -> &mut Self {
        let result = if let Some(ref response) = self.last_response {
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    if re.is_match(&response.content) {
                        AssertionResult::pass(&format!("Response matches pattern '{pattern}'"))
                    } else {
                        AssertionResult::fail(
                            &format!("Response should match pattern '{pattern}'"),
                            pattern,
                            &response.content,
                        )
                    }
                }
                Err(e) => AssertionResult::fail(
                    &format!("Invalid regex pattern: {e}"),
                    pattern,
                    "<invalid pattern>",
                ),
            }
        } else {
            AssertionResult::fail("No response to check", pattern, "<no response>")
        };

        self.record_assertion("matches", &result);
        self
    }

    pub fn assert_response_not_contains(&mut self, text: &str) -> &mut Self {
        let result = if let Some(ref response) = self.last_response {
            if response.content.contains(text) {
                AssertionResult::fail(
                    &format!("Response should not contain '{text}'"),
                    &format!("not containing '{text}'"),
                    &response.content,
                )
            } else {
                AssertionResult::pass(&format!("Response does not contain '{text}'"))
            }
        } else {
            AssertionResult::pass("No response (nothing to contain)")
        };

        self.record_assertion("not_contains", &result);
        self
    }

    pub fn assert_transferred_to_human(&mut self) -> &mut Self {
        let is_transferred = self.state == ConversationState::Transferred
            || self.last_response.as_ref().is_some_and(|r| {
                r.content.to_lowercase().contains("transfer")
                    || r.content.to_lowercase().contains("human")
                    || r.content.to_lowercase().contains("agent")
            });

        let result = if is_transferred {
            self.state = ConversationState::Transferred;
            AssertionResult::pass("Conversation transferred to human")
        } else {
            AssertionResult::fail(
                "Should be transferred to human",
                "transferred",
                "not transferred",
            )
        };

        self.record_assertion("transferred", &result);
        self
    }

    pub fn assert_queue_position(&mut self, expected: usize) -> &mut Self {
        let actual = self
            .context
            .get("queue_position")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as usize;

        let result = if actual == expected {
            AssertionResult::pass(&format!("Queue position is {expected}"))
        } else {
            AssertionResult::fail(
                "Queue position mismatch",
                &expected.to_string(),
                &actual.to_string(),
            )
        };

        self.record_assertion("queue_position", &result);
        self
    }

    pub fn assert_response_within(&mut self, max_duration: Duration) -> &mut Self {
        let result = if let Some(latency) = self.last_latency {
            if latency <= max_duration {
                AssertionResult::pass(&format!("Response within {max_duration:?}"))
            } else {
                AssertionResult::fail(
                    "Response too slow",
                    &format!("{max_duration:?}"),
                    &format!("{latency:?}"),
                )
            }
        } else {
            AssertionResult::fail(
                "No latency recorded",
                &format!("{max_duration:?}"),
                "<no latency>",
            )
        };

        self.record_assertion("response_time", &result);
        self
    }

    pub fn assert_response_count(&mut self, expected: usize) -> &mut Self {
        let actual = self.responses.len();

        let result = if actual == expected {
            AssertionResult::pass(&format!("Response count is {expected}"))
        } else {
            AssertionResult::fail(
                "Response count mismatch",
                &expected.to_string(),
                &actual.to_string(),
            )
        };

        self.record_assertion("response_count", &result);
        self
    }

    pub fn assert_response_type(&mut self, expected: ResponseContentType) -> &mut Self {
        let result = if let Some(ref response) = self.last_response {
            if response.content_type == expected {
                AssertionResult::pass(&format!("Response type is {expected:?}"))
            } else {
                AssertionResult::fail(
                    "Response type mismatch",
                    &format!("{expected:?}"),
                    &format!("{:?}", response.content_type),
                )
            }
        } else {
            AssertionResult::fail(
                "No response to check",
                &format!("{expected:?}"),
                "<no response>",
            )
        };

        self.record_assertion("response_type", &result);
        self
    }

    pub fn set_context(&mut self, key: &str, value: serde_json::Value) -> &mut Self {
        self.context.insert(key.to_string(), value);
        self
    }

    pub fn get_context(&self, key: &str) -> Option<&serde_json::Value> {
        self.context.get(key)
    }

    pub fn end(&mut self) -> &mut Self {
        self.state = ConversationState::Ended;
        self.record.ended_at = Some(Utc::now());
        self
    }

    pub const fn all_passed(&self) -> bool {
        self.record.passed
    }

    pub fn failed_assertions(&self) -> Vec<&AssertionRecord> {
        self.record
            .assertions
            .iter()
            .filter(|a| !a.passed)
            .collect()
    }

    fn record_assertion(&mut self, assertion_type: &str, result: &AssertionResult) {
        if !result.passed {
            self.record.passed = false;
        }

        if self.config.record {
            self.record.assertions.push(AssertionRecord {
                timestamp: Utc::now(),
                assertion_type: assertion_type.to_string(),
                passed: result.passed,
                message: result.message.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_builder() {
        let conv = ConversationBuilder::new("test-bot")
            .on_channel(Channel::Web)
            .with_timeout(Duration::from_secs(10))
            .build();

        assert_eq!(conv.bot_name(), "test-bot");
        assert_eq!(conv.channel(), Channel::Web);
        assert_eq!(conv.state(), ConversationState::Initial);
    }

    #[test]
    fn test_conversation_test_new() {
        let conv = ConversationTest::new("my-bot");
        assert_eq!(conv.bot_name(), "my-bot");
        assert!(conv.responses().is_empty());
        assert!(conv.sent_messages().is_empty());
    }

    #[tokio::test]
    async fn test_user_says() {
        let mut conv = ConversationTest::new("test-bot");
        conv.user_says("Hello").await;

        assert_eq!(conv.sent_messages().len(), 1);
        assert_eq!(conv.sent_messages()[0], "Hello");
        assert_eq!(conv.responses().len(), 1);
        assert!(conv.last_response().is_some());
    }

    #[tokio::test]
    async fn test_assert_response_contains() {
        let mut conv = ConversationTest::new("test-bot");
        conv.user_says("test").await;
        conv.assert_response_contains("Response");

        assert!(conv.all_passed());
    }

    #[tokio::test]
    async fn test_assert_response_not_contains() {
        let mut conv = ConversationTest::new("test-bot");
        conv.user_says("test").await;
        conv.assert_response_not_contains("nonexistent");

        assert!(conv.all_passed());
    }

    #[tokio::test]
    async fn test_conversation_recording() {
        let mut conv = ConversationBuilder::new("test-bot").build();
        conv.user_says("Hello").await;
        conv.user_says("How are you?").await;

        let record = conv.record();
        assert_eq!(record.messages.len(), 4);
    }

    #[tokio::test]
    async fn test_conversation_without_recording() {
        let mut conv = ConversationBuilder::new("test-bot")
            .without_recording()
            .build();
        conv.user_says("Hello").await;

        let record = conv.record();
        assert!(record.messages.is_empty());
    }

    #[test]
    fn test_context_variables() {
        let mut conv = ConversationTest::new("test-bot");
        conv.set_context("user_name", serde_json::json!("Alice"));

        let value = conv.get_context("user_name");
        assert!(value.is_some());
        assert_eq!(value.unwrap().as_str().unwrap(), "Alice");
    }

    #[tokio::test]
    async fn test_end_conversation() {
        let mut conv = ConversationTest::new("test-bot");
        conv.user_says("bye").await;
        conv.end();

        assert_eq!(conv.state(), ConversationState::Ended);
        assert!(conv.record().ended_at.is_some());
    }

    #[tokio::test]
    async fn test_failed_assertions() {
        let mut conv = ConversationTest::new("test-bot");
        conv.user_says("test").await;
        conv.assert_response_equals("this will not match");

        assert!(!conv.all_passed());
        assert_eq!(conv.failed_assertions().len(), 1);
    }

    #[tokio::test]
    async fn test_response_metadata() {
        let conv = ConversationBuilder::new("test-bot")
            .on_channel(Channel::WhatsApp)
            .build();

        let metadata = conv.build_response_metadata();
        assert_eq!(
            metadata.get("bot_name").unwrap().as_str().unwrap(),
            "test-bot"
        );
        assert!(metadata
            .get("channel")
            .unwrap()
            .as_str()
            .unwrap()
            .contains("WhatsApp"));
    }

    #[tokio::test]
    async fn test_multiple_messages_flow() {
        let mut conv = ConversationTest::new("support-bot");

        conv.user_says("Hi").await;
        conv.assert_response_contains("Response");

        conv.user_says("I need help").await;
        conv.assert_response_contains("Response");

        conv.user_says("Thanks, bye").await;
        conv.end();

        assert_eq!(conv.sent_messages().len(), 3);
        assert_eq!(conv.responses().len(), 3);
        assert!(conv.all_passed());
    }

    #[tokio::test]
    async fn test_response_time_assertion() {
        let mut conv = ConversationTest::new("test-bot");
        conv.user_says("quick test").await;
        conv.assert_response_within(Duration::from_secs(5));

        assert!(conv.all_passed());
    }

    #[tokio::test]
    async fn test_response_count_assertion() {
        let mut conv = ConversationTest::new("test-bot");
        conv.user_says("one").await;
        conv.user_says("two").await;
        conv.assert_response_count(2);

        assert!(conv.all_passed());
    }

    #[tokio::test]
    async fn test_customer_info_in_metadata() {
        let customer = Customer {
            id: Uuid::new_v4(),
            phone: Some("+15551234567".to_string()),
            ..Default::default()
        };

        let conv = ConversationBuilder::new("test-bot")
            .with_customer(customer.clone())
            .build();

        assert_eq!(conv.customer().id, customer.id);
        assert_eq!(conv.customer().phone, customer.phone);
    }
}
