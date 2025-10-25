# Understanding Sessions

Sessions are the core container for conversations in GeneralBots. They maintain state, context, and history for each user interaction.

## Session Components

Each session contains:

- **Session ID**: Unique identifier (UUID)
- **User ID**: Associated user (anonymous or authenticated)
- **Bot ID**: Which bot is handling the conversation
- **Context Data**: JSON object storing session state
- **Answer Mode**: How the bot should respond (direct, with tools, etc.)
- **Current Tool**: Active tool if waiting for input
- **Timestamps**: Creation and last update times

## Session Lifecycle

1. **Creation**: When a user starts a new conversation
2. **Active**: During ongoing interaction
3. **Waiting**: When awaiting user input for tools
4. **Inactive**: After period of no activity
5. **Archived**: Moved to long-term storage

## Session Context

The context data stores:
- Active knowledge base collections
- Available tools for the session
- User preferences and settings
- Temporary variables and state

## Managing Sessions

### Creating Sessions
Sessions are automatically created when:
- A new user visits the web interface
- A new WebSocket connection is established
- API calls specify a new session ID

### Session Persistence
Sessions are stored in PostgreSQL with:
- Full message history
- Context data as JSONB
- Timestamps for auditing

### Session Recovery
Users can resume sessions by:
- Using the same browser (cookies)
- Providing the session ID explicitly
- Authentication that links to previous sessions
