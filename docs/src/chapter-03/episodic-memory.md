# Episodic Memory

Episodic memory automatically manages conversation history to stay within token limits while preserving important information through intelligent summarization.

## How It Works

Episodic memory is controlled by two parameters in `config.csv`:

```csv
episodic-memory-history,2    # Keep last 2 message exchanges
episodic-memory-threshold,4    # Compact after 4 total exchanges
```

## Configuration Parameters

### episodic-memory-history
Determines how many previous exchanges to include in the LLM context:
- Default: `2` (keeps last 2 user messages and 2 bot responses)
- Range: 1-10 depending on your token budget
- Higher values = more context but more tokens used

### episodic-memory-threshold
Triggers compaction after N exchanges:
- Default: `4` (compacts conversation after 4 back-and-forth exchanges)
- When reached, older messages are summarized or removed
- Helps manage long conversations efficiently

## Automatic Behavior

The system automatically:
1. Tracks conversation length
2. When exchanges exceed `episodic-memory-threshold` value
3. Summarizes older messages using LLM
4. Keeps only the last `episodic-memory-history` exchanges in full
5. Stores summary as an "episodic memory" for context

## Example Flow

With default settings (`episodic-memory-history=2`, `episodic-memory-threshold=4`):

```
Exchange 1: User asks, bot responds
Exchange 2: User asks, bot responds  
Exchange 3: User asks, bot responds
Exchange 4: User asks, bot responds
Exchange 5: Episodic memory created - exchanges 1-2 summarized, 3-4 kept in full
Exchange 6: Context = [episodic summary] + exchanges 4-5
```

### Visual Flow Diagram

<svg width="600" height="400" viewBox="0 0 600 400" xmlns="http://www.w3.org/2000/svg" style="background: transparent;">
  <!-- Title -->
  <text x="300" y="25" text-anchor="middle" font-family="Arial, sans-serif" font-size="16" font-weight="bold" fill="currentColor">Episodic Memory Flow</text>
  
  <!-- Conversation History -->
  <rect x="200" y="50" width="200" height="40" fill="none" stroke="currentColor" stroke-width="2" rx="5"/>
  <text x="300" y="75" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="currentColor">Conversation History</text>
  
  <!-- Arrow down -->
  <path d="M 300 90 L 300 110" stroke="currentColor" stroke-width="2" fill="none" marker-end="url(#arrowhead)" opacity="0.7"/>
  
  <!-- Check Exchange Count -->
  <rect x="200" y="110" width="200" height="40" fill="none" stroke="currentColor" stroke-width="2" rx="5"/>
  <text x="300" y="135" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="currentColor">Check Exchange Count</text>
  
  <!-- Arrow down -->
  <path d="M 300 150 L 300 170" stroke="currentColor" stroke-width="2" fill="none" marker-end="url(#arrowhead)" opacity="0.7"/>
  
  <!-- Decision Diamond -->
  <path d="M 300 170 L 380 210 L 300 250 L 220 210 Z" fill="none" stroke="currentColor" stroke-width="2"/>
  <text x="300" y="205" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="currentColor">Exceeds</text>
  <text x="300" y="220" text-anchor="middle" font-family="Arial, sans-serif" font-size="11" fill="currentColor">episodic-memory-threshold?</text>
  
  <!-- No branch (left) -->
  <path d="M 220 210 L 150 210 L 150 280" stroke="currentColor" stroke-width="2" fill="none" marker-end="url(#arrowhead)" opacity="0.7"/>
  <text x="135" y="205" text-anchor="end" font-family="Arial, sans-serif" font-size="10" fill="currentColor" opacity="0.7">No</text>
  
  <!-- Keep All -->
  <rect x="75" y="280" width="150" height="40" fill="none" stroke="currentColor" stroke-width="2" rx="5"/>
  <text x="150" y="305" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="currentColor">Keep All Messages</text>
  
  <!-- Yes branch (right) -->
  <path d="M 380 210 L 450 210 L 450 280" stroke="currentColor" stroke-width="2" fill="none" marker-end="url(#arrowhead)" opacity="0.7"/>
  <text x="465" y="205" text-anchor="start" font-family="Arial, sans-serif" font-size="10" fill="currentColor" opacity="0.7">Yes</text>
  
  <!-- Summarize + Keep Last N -->
  <rect x="350" y="280" width="200" height="40" fill="none" stroke="currentColor" stroke-width="2" rx="5"/>
  <text x="450" y="295" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="currentColor">Summarize Old + Keep Last</text>
  <text x="450" y="310" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="currentColor">episodic-memory-history</text>
  
  <!-- Converge paths -->
  <path d="M 150 320 L 150 340 L 300 340" stroke="currentColor" stroke-width="2" fill="none" opacity="0.7"/>
  <path d="M 450 320 L 450 340 L 300 340" stroke="currentColor" stroke-width="2" fill="none" opacity="0.7"/>
  <path d="M 300 340 L 300 360" stroke="currentColor" stroke-width="2" fill="none" marker-end="url(#arrowhead)" opacity="0.7"/>
  
  <!-- Continue Conversation -->
  <rect x="200" y="360" width="200" height="40" fill="none" stroke="currentColor" stroke-width="2" rx="5"/>
  <text x="300" y="385" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="currentColor">Continue Conversation</text>
  
  <!-- Arrow marker definition -->
  <defs>
    <marker id="arrowhead" markerWidth="10" markerHeight="10" refX="5" refY="5" orient="auto">
      <polygon points="0 0, 10 5, 0 10" fill="currentColor"/>
    </marker>
  </defs>
</svg>

## Benefits

- **Automatic management** - No manual intervention needed
- **Token efficiency** - Stay within model limits
- **Context preservation** - Important info kept via summaries
- **Relevant context** - Keeps recent exchanges in full
- **Cost savings** - Fewer tokens = lower API costs

## Adjusting Settings

### For longer context:
```csv
episodic-memory-history,5     # Keep more history in full
episodic-memory-threshold,10  # Summarize less frequently
```

### For minimal context:
```csv
episodic-memory-history,1     # Only last exchange in full
episodic-memory-threshold,2   # Summarize aggressively
```

## Use Cases

### Customer Support
- Lower values work well (customers ask independent questions)
- `episodic-memory-history,1` and `episodic-memory-threshold,2`

### Complex Discussions
- Higher values needed (maintain conversation flow)
- `episodic-memory-history,4` and `episodic-memory-threshold,8`

### FAQ Bots
- Minimal context needed (each question is standalone)
- `episodic-memory-history,1` and `episodic-memory-threshold,2`

## Important Notes

- Episodic memory is automatic based on config.csv
- Summaries are created using the configured LLM
- Settings apply to all conversations
- Changes require bot restart
- Summaries are stored with role "episodic" in message history

## Best Practices

1. **Start with defaults** - Work well for most use cases
2. **Monitor token usage** - Adjust if hitting limits
3. **Consider conversation type** - Support vs discussion
4. **Test different values** - Find optimal balance

The system handles all compaction automatically - just configure the values that work for your use case!