use super::*;

impl TwilioSmsChannel {
    pub fn new(config: TwilioConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
            conversations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Builds the `application/x-www-form-urlencoded` body Twilio expects.
    ///
    /// `From` and `MessagingServiceSid` are mutually exclusive: the messaging
    /// service wins when configured, because that is the routing pool the
    /// account pays for. Media URLs are numbered from zero.
    pub(super) fn build_send_sms_params(&self, request: &SendSmsRequest) -> HashMap<String, String> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("To".to_string(), request.to.clone());
        params.insert("Body".to_string(), request.body.clone());

        if let Some(ref msid) = self.config.messaging_service_sid {
            params.insert("MessagingServiceSid".to_string(), msid.clone());
        } else {
            params.insert("From".to_string(), self.config.from_number.clone());
        }

        if let Some(ref callback) = request
            .status_callback
            .clone()
            .or(self.config.status_callback_url.clone())
        {
            params.insert("StatusCallback".to_string(), callback.clone());
        }

        if let Some(ref media_urls) = request.media_url {
            for (index, url) in media_urls.iter().enumerate() {
                // Owned key: the previous `Box::leak` version leaked one
                // allocation per media URL on every send.
                params.insert(format!("MediaUrl{}", index), url.clone());
            }
        }

        if let Some(ref max_price) = request.max_price {
            params.insert("MaxPrice".to_string(), max_price.clone());
        }

        if let Some(validity) = request.validity_period {
            params.insert("ValidityPeriod".to_string(), validity.to_string());
        }

        if let Some(ref schedule_type) = request.schedule_type {
            let schedule_str = match schedule_type {
                ScheduleType::Fixed => "fixed",
            };
            params.insert("ScheduleType".to_string(), schedule_str.to_string());
            if let Some(send_at) = request.send_at {
                params.insert("SendAt".to_string(), send_at.to_rfc3339());
            }
        }

        params
    }

    pub async fn send_sms(&self, request: SendSmsRequest) -> Result<SmsMessage, TwilioError> {
        let url = format!(
            "{}/Accounts/{}/Messages.json",
            Self::API_BASE, self.config.account_sid
        );

        let params = self.build_send_sms_params(&request);

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .form(&params)
            .send()
            .await
            .map_err(|e| TwilioError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error: TwilioApiError = response
                .json()
                .await
                .unwrap_or_else(|_| TwilioApiError {
                    code: 0,
                    message: "Unknown error".to_string(),
                    more_info: None,
                    status: 500,
                });
            return Err(TwilioError::ApiError(error));
        }

        let message: TwilioMessageResponse = response
            .json()
            .await
            .map_err(|e| TwilioError::ParseError(e.to_string()))?;

        let sms_message = self.convert_response_to_message(message);

        self.update_conversation(&request.to, &sms_message, MessageDirection::OutboundApi)
            .await;

        Ok(sms_message)
    }

    pub async fn send_bulk_sms(
        &self,
        recipients: Vec<String>,
        body: &str,
    ) -> Vec<Result<SmsMessage, TwilioError>> {
        let mut results = Vec::with_capacity(recipients.len());

        for recipient in recipients {
            let request = SendSmsRequest {
                to: recipient,
                body: body.to_string(),
                media_url: None,
                status_callback: None,
                max_price: None,
                validity_period: None,
                schedule_type: None,
                send_at: None,
            };

            let result = self.send_sms(request).await;
            results.push(result);

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        results
    }

    pub async fn get_message(&self, message_sid: &str) -> Result<SmsMessage, TwilioError> {
        let url = format!(
            "{}/Accounts/{}/Messages/{}.json",
            Self::API_BASE, self.config.account_sid, message_sid
        );

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .send()
            .await
            .map_err(|e| TwilioError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error: TwilioApiError = response
                .json()
                .await
                .unwrap_or_else(|_| TwilioApiError {
                    code: 0,
                    message: "Message not found".to_string(),
                    more_info: None,
                    status: 404,
                });
            return Err(TwilioError::ApiError(error));
        }

        let message: TwilioMessageResponse = response
            .json()
            .await
            .map_err(|e| TwilioError::ParseError(e.to_string()))?;

        Ok(self.convert_response_to_message(message))
    }

    pub async fn list_messages(
        &self,
        to: Option<&str>,
        from: Option<&str>,
        date_sent: Option<&str>,
        page_size: Option<u32>,
    ) -> Result<Vec<SmsMessage>, TwilioError> {
        let mut url = format!(
            "{}/Accounts/{}/Messages.json",
            Self::API_BASE, self.config.account_sid
        );

        let mut query_params = Vec::new();
        if let Some(to_number) = to {
            query_params.push(format!("To={}", urlencoding::encode(to_number)));
        }
        if let Some(from_number) = from {
            query_params.push(format!("From={}", urlencoding::encode(from_number)));
        }
        if let Some(date) = date_sent {
            query_params.push(format!("DateSent={}", urlencoding::encode(date)));
        }
        if let Some(size) = page_size {
            query_params.push(format!("PageSize={}", size));
        }

        if !query_params.is_empty() {
            url = format!("{}?{}", url, query_params.join("&"));
        }

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .send()
            .await
            .map_err(|e| TwilioError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error: TwilioApiError = response
                .json()
                .await
                .unwrap_or_else(|_| TwilioApiError {
                    code: 0,
                    message: "Failed to list messages".to_string(),
                    more_info: None,
                    status: 500,
                });
            return Err(TwilioError::ApiError(error));
        }

        let list_response: MessageListResponse = response
            .json()
            .await
            .map_err(|e| TwilioError::ParseError(e.to_string()))?;

        let messages = list_response
            .messages
            .into_iter()
            .map(|m| self.convert_response_to_message(m))
            .collect();

        Ok(messages)
    }

    pub async fn delete_message(&self, message_sid: &str) -> Result<(), TwilioError> {
        let url = format!(
            "{}/Accounts/{}/Messages/{}.json",
            Self::API_BASE, self.config.account_sid, message_sid
        );

        let response = self
            .http_client
            .delete(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .send()
            .await
            .map_err(|e| TwilioError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error: TwilioApiError = response
                .json()
                .await
                .unwrap_or_else(|_| TwilioApiError {
                    code: 0,
                    message: "Failed to delete message".to_string(),
                    more_info: None,
                    status: 500,
                });
            return Err(TwilioError::ApiError(error));
        }

        Ok(())
    }

    pub async fn cancel_scheduled_message(&self, message_sid: &str) -> Result<SmsMessage, TwilioError> {
        let url = format!(
            "{}/Accounts/{}/Messages/{}.json",
            Self::API_BASE, self.config.account_sid, message_sid
        );

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.account_sid, Some(&self.config.auth_token))
            .form(&[("Status", "canceled")])
            .send()
            .await
            .map_err(|e| TwilioError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error: TwilioApiError = response
                .json()
                .await
                .unwrap_or_else(|_| TwilioApiError {
                    code: 0,
                    message: "Failed to cancel message".to_string(),
                    more_info: None,
                    status: 500,
                });
            return Err(TwilioError::ApiError(error));
        }

        let message: TwilioMessageResponse = response
            .json()
            .await
            .map_err(|e| TwilioError::ParseError(e.to_string()))?;

        Ok(self.convert_response_to_message(message))
    }
}

#[cfg(test)]
#[path = "send_payload_tests.rs"]
mod send_payload_tests;
