# LLM Package - Large Language Model Integration

## Purpose
Manages large language model integration and operations. Provides unified interface for working with various LLM providers.

## Key Files
- **bedrock.rs**: AWS Bedrock integration
- **cache.rs**: LLM response caching
- **claude.rs**: Anthropic Claude integration
- **context/**: Context management for LLM conversations
- **episodic_memory.rs**: Episodic memory for LLM interactions
- **glm.rs**: GLM model integration
- **hallucination_detector.rs**: Hallucination detection
- **llm_models/**: Supported LLM model definitions
- **local.rs**: Local LLM integration
- **mod.rs**: Module entry point and exports
- **observability.rs**: LLM observability and logging
- **prompt_manager/**: Prompt management system
- **rate_limiter.rs**: LLM API rate limiting
- **smart_router.rs**: Smart routing for LLM requests
- **vertex.rs**: Google Vertex AI integration

## Features

### Multi-Provider Support
```rust
use crate::llm::LLMService;
use crate::llm::models::ModelType;

let llm_service = LLMService::new();

// Generate text with specific model
let result = llm_service.generate_text(
    ModelType::Claude3,
    "Write a poem about technology".to_string(),
    None
).await?;
```

### Context Management
```rust
use crate::llm::context::ConversationContext;

let mut context = ConversationContext::new();
context.add_user_message("What's the capital of France?");
context.add_assistant_message("The capital of France is Paris.");

// Get context for next message
let context_text = context.get_context();
```

### Episodic Memory
```rust
use crate::llm::episodic_memory::EpisodicMemory;

let memory = EpisodicMemory::new();

// Store memory
memory.store_memory(
    user_id,
    "user asked about France".to_string(),
    "Paris is the capital".to_string()
).await?;

// Retrieve relevant memories
let memories = memory.retrieve_relevant_memories(
    user_id,
    "capital of France"
).await?;
```

## Supported Models
- **Claude (Anthropic)**: Claude 3 family
- **Bedrock**: AWS Bedrock models (Claude 3, Titan, etc.)
- **Vertex AI**: Google Cloud LLM models
- **Local Models**: Local inference support
- **GLM**: Chinese language models

## Prompt Management
```rust
use crate::llm::prompt_manager::PromptManager;

let prompt_manager = PromptManager::new();

// Get prompt template
let template = prompt_manager.get_prompt_template("code_review").await?;

// Render prompt with variables
let prompt = template.render(&[("code", code_snippet)]);
```

## Hallucination Detection
```rust
use crate::llm::hallucination_detector::HallucinationDetector;

let detector = HallucinationDetector::new();

// Check response for hallucinations
let result = detector.detect_hallucinations(response_text).await?;

if result.is_hallucination {
    log::warn!("Hallucination detected: {}", result.reason);
}
```

## Rate Limiting
```rust
use crate::llm::rate_limiter::RateLimiter;

let rate_limiter = RateLimiter::new();

// Check rate limit before request
if rate_limiter.is_rate_limited(user_id).await? {
    return Err(Error::RateLimited);
}

// Make LLM request
let response = make_llm_request().await?;

// Update rate limit
rate_limiter.update_rate_limit(user_id).await?;
```

## Observability
```rust
use crate::llm::observability::LLMObservability;

let observability = LLMObservability::new();

// Log LLM request
observability.log_request(
    user_id,
    model_type,
    prompt_text,
    response_text,
    duration_ms
).await?;
```

## Configuration
LLM settings are configured in:
- `botserver/.env` - API keys and endpoints
- `config/llm/` - Model configuration
- Database for dynamic settings

## Error Handling
Use `LLMError` type which includes:
- Provider-specific errors
- Rate limiting errors
- API errors
- Validation errors

## Testing
LLM package is tested with:
- Unit tests for core functionality
- Integration tests with real APIs
- Mocked tests for fast execution
- Error handling tests