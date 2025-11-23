# Context Compaction

Context compaction automatically manages conversation history to stay within token limits while preserving important information.

## How It Works

Context compaction is controlled by two parameters in `config.csv`:

```csv
prompt-history,2    # Keep last 2 message exchanges
prompt-compact,4    # Compact after 4 total exchanges
```

## Configuration Parameters

### prompt-history
Determines how many previous exchanges to include in the LLM context:
- Default: `2` (keeps last 2 user messages and 2 bot responses)
- Range: 1-10 depending on your token budget
- Higher values = more context but more tokens used

### prompt-compact
Triggers compaction after N exchanges:
- Default: `4` (compacts conversation after 4 back-and-forth exchanges)
- When reached, older messages are summarized or removed
- Helps manage long conversations efficiently

## Automatic Behavior

The system automatically:
1. Tracks conversation length
2. When exchanges exceed `prompt-compact` value
3. Keeps only the last `prompt-history` exchanges
4. Older messages are dropped from context

## Example Flow

With default settings (`prompt-history=2`, `prompt-compact=4`):

```
Exchange 1: User asks, bot responds
Exchange 2: User asks, bot responds  
Exchange 3: User asks, bot responds
Exchange 4: User asks, bot responds
Exchange 5: Compaction triggers - only exchanges 3-4 kept
Exchange 6: Only exchanges 4-5 in context
```

## Benefits

- **Automatic management** - No manual intervention needed
- **Token efficiency** - Stay within model limits
- **Relevant context** - Keeps recent, important exchanges
- **Cost savings** - Fewer tokens = lower API costs

## Adjusting Settings

### For longer context:
```csv
prompt-history,5     # Keep more history
prompt-compact,10    # Compact less frequently
```

### For minimal context:
```csv
prompt-history,1     # Only last exchange
prompt-compact,2     # Compact aggressively
```

## Use Cases

### Customer Support
- Lower values work well (customers ask independent questions)
- `prompt-history,1` and `prompt-compact,2`

### Complex Discussions
- Higher values needed (maintain conversation flow)
- `prompt-history,4` and `prompt-compact,8`

### FAQ Bots
- Minimal context needed (each question is standalone)
- `prompt-history,1` and `prompt-compact,2`

## Important Notes

- Compaction is automatic based on config.csv
- No BASIC commands control compaction
- Settings apply to all conversations
- Changes require bot restart

## Best Practices

1. **Start with defaults** - Work well for most use cases
2. **Monitor token usage** - Adjust if hitting limits
3. **Consider conversation type** - Support vs discussion
4. **Test different values** - Find optimal balance

The system handles all compaction automatically - just configure the values that work for your use case!