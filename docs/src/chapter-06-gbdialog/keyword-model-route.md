# MODEL ROUTE Keywords

Route LLM requests to different models based on task type, cost, or capability requirements.

## Keywords

| Keyword | Purpose |
|---------|---------|
| `MODEL ROUTE` | Route request to appropriate model |
| `SET MODEL ROUTE` | Configure routing rules |
| `GET MODEL ROUTES` | List configured routes |

## MODEL ROUTE

```basic
response = MODEL ROUTE "complex-analysis", user_query
```

## SET MODEL ROUTE

```basic
SET MODEL ROUTE "fast", "gpt-3.5-turbo"
SET MODEL ROUTE "smart", "gpt-4o"
SET MODEL ROUTE "code", "claude-sonnet"
SET MODEL ROUTE "vision", "gpt-4o"
```

## Routing Strategies

| Strategy | Description |
|----------|-------------|
| `manual` | Explicitly specify model per request |
| `cost` | Prefer cheaper models when possible |
| `capability` | Match model to task requirements |
| `fallback` | Try models in order until success |

## Example: Cost-Optimized Routing

```basic
SET MODEL ROUTE "default", "gpt-3.5-turbo"
SET MODEL ROUTE "complex", "gpt-4o"

' Simple queries use fast/cheap model
' Complex analysis uses more capable model
response = MODEL ROUTE "complex", "Analyze market trends for Q4"
```

## Configuration

Add to `config.csv`:

```csv
model-routing-strategy,capability
model-default,gpt-3.5-turbo
model-fallback,gpt-4o
```

## See Also

- [USE MODEL](./keyword-use-model.md)