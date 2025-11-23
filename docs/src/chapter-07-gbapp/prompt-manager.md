# Prompt Manager

The Prompt Manager module provides centralized management of LLM prompts, templates, and system instructions used throughout BotServer.

## Overview

Located in `src/prompt_manager/`, this module maintains a library of reusable prompts that can be:
- Versioned and updated without code changes
- Customized per bot instance
- Composed dynamically based on context
- Optimized for different LLM models

## Architecture

```
src/prompt_manager/
├── mod.rs              # Main module interface
├── prompts.csv         # Default prompt library
└── templates/          # Complex prompt templates
    ├── system.md       # System instructions
    ├── tools.md        # Tool-use prompts
    └── context.md      # Context formatting
```

## Prompt Library Format

The `prompts.csv` file stores prompts in a structured format:

```csv
id,category,name,content,model,version
1,system,default,"You are a helpful assistant...",gpt-4,1.0
2,tools,function_call,"To use a tool, follow this format...",any,1.0
3,context,kb_search,"Search the knowledge base for: {query}",any,1.0
```

### Fields

| Field | Description |
|-------|-------------|
| `id` | Unique identifier |
| `category` | Prompt category (system, tools, context, etc.) |
| `name` | Prompt name for retrieval |
| `content` | The actual prompt text with placeholders |
| `model` | Target model or "any" for universal |
| `version` | Version for tracking changes |

## Usage in BASIC

Prompts are automatically loaded and can be referenced in dialogs:

```basic
' For background processing only - not for interactive conversations
' Generate content for storage
summary = LLM "Use prompt: customer_service"
SET BOT MEMORY "service_info", summary

' For interactive conversations, use SET CONTEXT
SET CONTEXT "support_issue", issue
TALK "How can I help you with your technical issue?"
```

## Rust API

### Loading Prompts

```rust
use crate::prompt_manager::PromptManager;

let manager = PromptManager::new();
manager.load_from_csv("prompts.csv")?;
```

### Retrieving Prompts

```rust
// Get a specific prompt
let prompt = manager.get_prompt("system", "default")?;

// Get prompt with variable substitution
let mut vars = HashMap::new();
vars.insert("query", "user question");
let formatted = manager.format_prompt("context", "kb_search", vars)?;
```

### Dynamic Composition

```rust
// Compose multiple prompts
let system = manager.get_prompt("system", "default")?;
let tools = manager.get_prompt("tools", "available")?;
let context = manager.get_prompt("context", "current")?;

let full_prompt = manager.compose(vec![system, tools, context])?;
```

## Prompt Categories

### System Prompts
Define the AI assistant's role and behavior:
- `default`: Standard helpful assistant
- `professional`: Business-focused responses
- `technical`: Developer-oriented assistance
- `creative`: Creative writing and ideation

### Tool Prompts
Instructions for tool usage:
- `function_call`: How to invoke functions
- `parameter_format`: Parameter formatting rules
- `error_handling`: Tool error responses

### Context Prompts
Templates for providing context:
- `kb_search`: Knowledge base query format
- `conversation_history`: Previous message format
- `user_context`: User information format

### Guardrail Prompts
Safety and compliance instructions:
- `content_filter`: Inappropriate content handling
- `pii_protection`: Personal data protection
- `compliance`: Regulatory compliance rules

## Custom Prompts

Bots can override default prompts by providing their own:

```
mybot.gbai/
└── mybot.gbot/
    ├── config.csv
    └── prompts.csv  # Custom prompts override defaults
```

## Model-Specific Optimization

Prompts can be optimized for different models:

```csv
id,category,name,content,model,version
1,system,default,"You are Claude...",claude-3,1.0
2,system,default,"You are GPT-4...",gpt-4,1.0
3,system,default,"You are a helpful assistant",llama-3,1.0
```

The manager automatically selects the best match for the current model.

## Variables and Placeholders

Prompts support variable substitution using `{variable}` syntax:

```
"Search for {query} in {collection} and return {limit} results"
```

Variables are replaced at runtime:

```rust
let vars = hashmap!{
    "query" => "pricing information",
    "collection" => "docs",
    "limit" => "5"
};
let prompt = manager.format_prompt("search", "template", vars)?;
```

## Prompt Versioning

Track prompt evolution:

```csv
id,category,name,content,model,version
1,system,default,"Original prompt...",gpt-4,1.0
2,system,default,"Updated prompt...",gpt-4,1.1
3,system,default,"Latest prompt...",gpt-4,2.0
```

The manager uses the latest version by default but can retrieve specific versions:

```rust
let prompt = manager.get_prompt_version("system", "default", "1.0")?;
```

## Performance Optimization

### Caching
Frequently used prompts are cached in memory:

```rust
manager.cache_prompt("system", "default");
```

### Token Counting
Estimate token usage before sending:

```rust
let tokens = manager.estimate_tokens(prompt, "gpt-4")?;
if tokens > MAX_TOKENS {
    prompt = manager.compress_prompt(prompt, MAX_TOKENS)?;
}
```

### Compression
Automatically compress prompts while maintaining meaning:

```rust
let compressed = manager.compress_prompt(original, target_tokens)?;
```

## Best Practices

1. **Modularity**: Keep prompts focused on single responsibilities
2. **Versioning**: Always version prompts for rollback capability
3. **Testing**: Test prompts across different models
4. **Documentation**: Document the purpose and expected output
5. **Variables**: Use placeholders for dynamic content
6. **Optimization**: Tailor prompts to specific model capabilities

## Integration with BASIC

The Prompt Manager is automatically available in BASIC dialogs:

```basic
' Load custom prompt library
LOAD_PROMPTS "custom_prompts.csv"

' For background processing - generate content once
greeting = LLM PROMPT("customer_greeting")
SET BOT MEMORY "standard_greeting", greeting

' For interactive conversations with variables
SET CONTEXT "customer_name", customer_name
SET CONTEXT "support_ticket", support_ticket
TALK "Let me help you with your support request."
```

## Monitoring and Analytics

Track prompt performance:

```rust
// Log prompt usage
manager.log_usage("system", "default", response_quality);

// Get analytics
let stats = manager.get_prompt_stats("system", "default")?;
println!("Success rate: {}%", stats.success_rate);
println!("Avg response time: {}ms", stats.avg_latency);
```

## Error Handling

Handle missing or invalid prompts gracefully:

```rust
match manager.get_prompt("custom", "missing") {
    Ok(prompt) => use_prompt(prompt),
    Err(PromptError::NotFound) => use_default(),
    Err(PromptError::Invalid) => log_and_fallback(),
    Err(e) => return Err(e),
}
```

