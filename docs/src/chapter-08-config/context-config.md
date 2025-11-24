# Context Configuration

Configuration for managing conversation context and prompt handling in BotServer.

## Context Parameters

BotServer uses two simple parameters to control how conversation context is managed:

```csv
prompt-compact,4
prompt-history,2
```

## Available Parameters

### prompt-compact

Controls the context compaction level.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `prompt-compact` | Number | `4` | Number of message exchanges before context compaction |

When the conversation reaches this many exchanges, the system compacts older messages to stay within token limits.

Example values:
- `2` - Very aggressive compaction, keeps context minimal
- `4` - Default, balanced approach
- `8` - Less frequent compaction, more context retained

### prompt-history

Controls how many previous messages to keep in the conversation history.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `prompt-history` | Number | Not set | Number of previous messages to maintain |

This parameter is optional. When set, it limits the conversation history to the specified number of messages.

Example values:
- `2` - Keep only last 2 messages
- `5` - Keep last 5 messages  
- Not set - System manages history automatically

## Configuration Examples

### Minimal Context
For quick responses with minimal memory usage:
```csv
prompt-compact,2
prompt-history,1
```

### Balanced Context
Default configuration for most use cases:
```csv
prompt-compact,4
```

### Extended Context
For complex conversations requiring more context:
```csv
prompt-compact,8
prompt-history,10
```

## How It Works

1. **Message Collection**: As the conversation progresses, messages accumulate
2. **Compaction Trigger**: When reaching `prompt-compact` exchanges, older messages are compressed
3. **History Limit**: If `prompt-history` is set, only that many recent messages are kept
4. **Token Management**: Both parameters work together to keep within LLM token limits

## Best Practices

- **Short Tasks**: Use lower values (2-3) for simple Q&A
- **Complex Discussions**: Use higher values (6-8) for detailed conversations
- **Memory Constrained**: Set `prompt-history` to limit memory usage
- **Default Works**: Most bots work well with just `prompt-compact,4`

## Impact on Performance

Lower values:
- ✅ Faster responses
- ✅ Lower token usage
- ✅ Reduced costs (if using paid APIs)
- ❌ Less context awareness

Higher values:
- ✅ Better context understanding
- ✅ More coherent long conversations
- ❌ Slower responses
- ❌ Higher token usage

## Relationship with LLM Settings

These parameters work alongside LLM configuration:
- `llm-server-ctx-size` - Maximum context window
- `llm-server-n-predict` - Maximum response tokens
- `prompt-compact` and `prompt-history` manage what goes into that context window

## Example in Practice

With `prompt-compact,4` and `prompt-history,2`:

```
User: "What is BotServer?"
Bot: "BotServer is..."           [Exchange 1]

User: "How do I install it?"  
Bot: "To install..."             [Exchange 2]

User: "What about configuration?"
Bot: "Configuration..."          [Exchange 3]

User: "Can you explain more?"
Bot: "Sure..."                   [Exchange 4 - Compaction triggered]

User: "What databases work?"
Bot: [Only sees last 2 messages due to prompt-history,2]
```

That's it - just two simple parameters to control context behavior!