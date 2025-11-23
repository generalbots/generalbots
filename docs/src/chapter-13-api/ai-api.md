# AI API

The AI API is planned for future implementation to provide direct access to AI model operations and advanced text processing capabilities.

## Status

**Not Implemented** - These endpoints are planned but not yet available.

## Current AI Functionality

AI features are currently available through:

1. **LLM Keyword in BASIC Scripts**
   ```basic
   # For background processing only - not for interactive conversations
   let summary = LLM "Generate content for storage"
   SET_BOT_MEMORY "stored_content", summary
   ```

2. **WebSocket Bot Interaction**
   - Send messages via WebSocket
   - Bot uses configured LLM provider
   - Responses streamed back

## Planned Endpoints

### Text Generation

**POST** `/api/ai/generate` (Planned)

Generate text using the configured LLM.

### Text Summarization  

**POST** `/api/ai/summarize` (Planned)

Summarize long text documents.

### Sentiment Analysis

**POST** `/api/ai/sentiment` (Planned)

Analyze sentiment of provided text.

### Entity Extraction

**POST** `/api/ai/entities` (Planned)

Extract named entities from text.

### Translation

**POST** `/api/ai/translate` (Planned)

Translate text between languages.

## Current Implementation

AI functionality is integrated into the bot conversation flow:

1. User sends message via WebSocket
2. Bot processes with configured LLM provider
3. Response generated based on answer mode
4. Streamed back to user

The LLM provider is configured in environment variables:
```bash
OPENAI_API_KEY=your-key
LLM_MODEL=gpt-4
LLM_PROVIDER=openai
```

## Using AI Features Today

To use AI features currently:

1. **Configure LLM Provider**
   - Set OpenAI API key
   - Or configure local model endpoint

2. **Use in BASIC Scripts**
   ```basic
   let prompt = "Summarize: " + document
   let summary = LLM prompt
   TALK summary
   ```

3. **Configure Answer Modes**
   In config.csv:
   ```csv
   Answer Mode,tool
   ```

## Future Implementation

When implemented, the AI API will provide:
- Direct HTTP endpoints for AI operations
- Batch processing capabilities
- Model selection per request
- Advanced parameters control
- Response caching
- Usage tracking

## Alternative Approaches

Until the AI API is implemented:
- Use BASIC scripts with LLM keyword
- Interact through WebSocket chat
- Call OpenAI API directly from client
- Use bot as AI proxy

## Summary

While dedicated AI API endpoints are not yet implemented, BotServer provides comprehensive AI capabilities through its LLM integration in the conversation flow and BASIC scripting language.