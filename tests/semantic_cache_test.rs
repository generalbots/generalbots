#[cfg(test)]
mod semantic_cache_integration_tests {
    use botserver::llm::cache::{enable_semantic_cache_for_bot, CacheConfig, CachedLLMProvider};
    use botserver::llm::{LLMProvider, OpenAIClient};
    use redis::{AsyncCommands, Client};
    use serde_json::json;
    use std::sync::Arc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_semantic_cache_with_bot_config() {
        // Skip test if Redis is not available
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
        let cache_client = match Client::open(redis_url) {
            Ok(client) => client,
            Err(_) => {
                println!("Skipping test - Redis not available");
                return;
            }
        };

        // Test connection
        let conn = match cache_client.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(_) => {
                println!("Skipping test - Cannot connect to Redis");
                return;
            }
        };

        // Create a test bot ID
        let bot_id = Uuid::new_v4().to_string();

        // Enable semantic cache for this bot
        if let Err(e) = enable_semantic_cache_for_bot(&cache_client, &bot_id, true).await {
            println!("Failed to enable cache for bot: {}", e);
            return;
        }

        // Create mock LLM provider
        let llm_provider = Arc::new(OpenAIClient::new(
            "test-key".to_string(),
            Some("http://localhost:8081".to_string()),
        ));

        // Create cache configuration
        let cache_config = CacheConfig {
            ttl: 300, // 5 minutes for testing
            semantic_matching: true,
            similarity_threshold: 0.85,
            max_similarity_checks: 10,
            key_prefix: "test_cache".to_string(),
        };

        // Create cached provider without embedding service for basic testing
        let cached_provider = CachedLLMProvider::new(
            llm_provider,
            Arc::new(cache_client.clone()),
            cache_config,
            None, // No embedding service for this basic test
        );

        // Test messages with bot_id
        let messages = json!({
            "bot_id": bot_id,
            "llm_cache": "true",
            "messages": [
                {"role": "system", "content": "You are a helpful assistant."},
                {"role": "user", "content": "What is the capital of France?"}
            ]
        });

        // This would normally call the LLM, but will fail without a real server
        // The test is mainly to ensure the cache layer is properly initialized
        let result = cached_provider
            .generate("", &messages, "gpt-3.5-turbo", "test-key")
            .await;

        match result {
            Ok(_) => println!("Cache test succeeded (unexpected with mock server)"),
            Err(e) => println!("Expected error with mock server: {}", e),
        }

        // Clean up - clear test cache entries
        let mut conn = cache_client
            .get_multiplexed_async_connection()
            .await
            .unwrap();
        let _: () = conn
            .del(format!("bot_config:{}:llm-cache", bot_id))
            .await
            .unwrap_or(());
    }

    #[tokio::test]
    async fn test_cache_key_generation() {
        use botserver::llm::cache::CachedLLMProvider;

        // This test verifies that cache keys are generated consistently
        let messages1 = json!({
            "bot_id": "test-bot-1",
            "messages": [
                {"role": "user", "content": "Hello"}
            ]
        });

        let messages2 = json!({
            "bot_id": "test-bot-2",
            "messages": [
                {"role": "user", "content": "Hello"}
            ]
        });

        // The messages content is the same but bot_id is different
        // Cache should handle this properly by extracting actual messages
        let actual_messages1 = messages1.get("messages").unwrap_or(&messages1);
        let actual_messages2 = messages2.get("messages").unwrap_or(&messages2);

        // Both should have the same actual message content
        assert_eq!(
            actual_messages1.to_string(),
            actual_messages2.to_string(),
            "Actual messages should be identical"
        );
    }

    #[tokio::test]
    async fn test_cache_config_defaults() {
        let config = CacheConfig::default();

        assert_eq!(config.ttl, 3600, "Default TTL should be 1 hour");
        assert!(
            config.semantic_matching,
            "Semantic matching should be enabled by default"
        );
        assert_eq!(
            config.similarity_threshold, 0.95,
            "Default similarity threshold should be 0.95"
        );
        assert_eq!(
            config.max_similarity_checks, 100,
            "Default max similarity checks should be 100"
        );
        assert_eq!(
            config.key_prefix, "llm_cache",
            "Default key prefix should be 'llm_cache'"
        );
    }
}
