# First Conversation

## Starting a Session

When you first access the GeneralBots web interface, the system automatically:

1. Creates an anonymous user session
2. Loads the default bot configuration
3. Executes the `start.bas` script (if present)
4. Presents the chat interface

## Basic Interaction

The conversation flow follows this pattern:

```
User: [Message] → Bot: [Processes with LLM/Tools] → Bot: [Response]
```

## Session Management

- Each conversation is tied to a **session ID**
- Sessions maintain conversation history and context
- Users can have multiple simultaneous sessions
- Sessions can be persisted or temporary

## Example Flow

1. **User**: "Hello"
2. **System**: Creates session, runs start script
3. **Bot**: "Hello! How can I help you today?"
4. **User**: "What can you do?"
5. **Bot**: Explains capabilities based on available tools and knowledge

## Session Persistence

Sessions are automatically saved and can be:
- Retrieved later using the session ID
- Accessed from different devices (with proper authentication)
- Archived for historical reference

The system maintains conversation context across multiple interactions within the same session.
