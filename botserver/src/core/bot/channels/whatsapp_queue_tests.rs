#[cfg(test)]
mod tests {
    use crate::core::bot::channels::whatsapp_queue::*;
    use std::sync::Arc;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_queue_enqueue() {
        let queue = WhatsAppMessageQueue::new("redis://127.0.0.1:6379").unwrap();
        
        let msg = QueuedWhatsAppMessage {
            to: "+5511999999999".to_string(),
            message: "Test message".to_string(),
            api_key: "test_key".to_string(),
            phone_number_id: "123456".to_string(),
            api_version: "v17.0".to_string(),
        };

        let result = queue.enqueue(msg).await;
        assert!(result.is_ok());
    }

    // Note: test_per_recipient_rate_limit removed because we now allow per-recipient bursting.
    // Throttling is handled reactively via 131056 error code.

    #[tokio::test]
    async fn test_different_recipients_no_delay() {
        let queue = Arc::new(WhatsAppMessageQueue::new("redis://127.0.0.1:6379").unwrap());
        
        let msg1 = QueuedWhatsAppMessage {
            to: "+5511999999999".to_string(),
            message: "Message 1".to_string(),
            api_key: "test_key".to_string(),
            phone_number_id: "123456".to_string(),
            api_version: "v17.0".to_string(),
        };

        let msg2 = QueuedWhatsAppMessage {
            to: "+5511888888888".to_string(),
            message: "Message 2".to_string(),
            api_key: "test_key".to_string(),
            phone_number_id: "123456".to_string(),
            api_version: "v17.0".to_string(),
        };

        queue.enqueue(msg1).await.unwrap();
        queue.enqueue(msg2).await.unwrap();

        let start = std::time::Instant::now();
        
        // Process both messages (different recipients)
        let _ = queue.process_next().await;
        let _ = queue.process_next().await;

        let elapsed = start.elapsed();
        
        // Should be fast since different recipients
        assert!(elapsed < Duration::from_millis(500));
    }

    #[tokio::test]
    async fn test_burst_mode_within_window() {
        let queue = Arc::new(WhatsAppMessageQueue::new("redis://127.0.0.1:6379").unwrap());
        
        // Send 3 messages to same recipient in quick succession
        for i in 1..=3 {
            let msg = QueuedWhatsAppMessage {
                to: "+5511999999999".to_string(),
                message: format!("Burst message {}", i),
                api_key: "test_key".to_string(),
                phone_number_id: "123456".to_string(),
                api_version: "v17.0".to_string(),
            };
            queue.enqueue(msg).await.unwrap();
        }

        let start = std::time::Instant::now();
        
        // Process all 3 messages
        let _ = queue.process_next().await;
        let _ = queue.process_next().await;
        let _ = queue.process_next().await;

        let elapsed = start.elapsed();
        
        // Should complete in burst mode (< 6 seconds, not 18 seconds)
        assert!(elapsed < Duration::from_secs(6));
    }
}
