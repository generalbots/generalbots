# MODEL ROUTE Keywords

Route LLM requests to different models based on task type, cost, or capability requirements.

## Keywords

| Keyword | Purpose |
|---------|---------|
| `USE MODEL` | Select a specific model for next request |
| `SET MODEL ROUTING` | Configure routing strategy |
| `GET CURRENT MODEL` | Get active model name |
| `LIST MODELS` | List available models |

## USE MODEL

```basic
USE MODEL "fast"
response = ASK "Quick question about the weather"

USE MODEL "quality"
response = ASK "Analyze this complex legal document"
```

## SET MODEL ROUTING

```basic
SET MODEL ROUTING "auto"
SET MODEL ROUTING "cost"
SET MODEL ROUTING "manual"
```

## Routing Strategies

| Strategy | Description |
|----------|-------------|
| `manual` | Explicitly specify model per request |
| `auto` | Auto-select based on task complexity |
| `cost` | Prefer cheaper models when possible |
| `quality` | Always use highest quality model |

## GET CURRENT MODEL

```basic
model = GET CURRENT MODEL
TALK "Currently using: " + model
```

## LIST MODELS

```basic
models = LIST MODELS
FOR EACH m IN models
    TALK m.name + " - " + m.description
NEXT
```

## Configuration

Add to `config.csv`:

```csv
llm-models,default;fast;quality;code
model-routing-strategy,auto
model-default,gpt-3.5-turbo
model-fast,gpt-3.5-turbo
model-quality,gpt-4o
model-code,claude-sonnet
```

## Example: Task-Based Routing

```basic
USE MODEL "code"
code_review = ASK "Review this function for bugs: " + code

USE MODEL "fast"
TALK "Here's what I found:"
TALK code_review
```

## See Also

- [USE MODEL](./keyword-use-model.md)