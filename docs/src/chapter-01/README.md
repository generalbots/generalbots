## Run and Talk
```bas
TALK "Welcome! How can I help you today?"
HEAR user_input
```
*Start the server:* `cargo run --release`

### Installation
```bash
# Clone the repository
git clone https://github.com/GeneralBots/BotServer.git
cd BotServer

# Build the project
cargo build --release

# Run the server
cargo run --release
```

### First Conversation
```bas
TALK "Hello! I'm your GeneralBots assistant."
HEAR user_input
IF user_input CONTAINS "weather" THEN
    TALK "Sure, let me check the weather for you."
    CALL GET_WEATHER
ELSE
    TALK "I can help with many tasks, just ask!"
ENDIF
```

### Understanding Sessions
Each conversation is represented by a **BotSession** that persists across multiple interactions. The session manages:

- **User identity** (authenticated or anonymous)
- **Conversation history** (full message transcript)
- **Context state** (variables, knowledge base references, active tools)
- **Interaction metrics** (message counts, timing)

#### Storage Architecture
Sessions use a multi-layer persistence model:
1. **PostgreSQL** - Primary storage for all session data
2. **Redis** - Caching layer for active session state
3. **In-memory** - Hot session data for performance

#### Key API Endpoints
- `POST /api/sessions` - Create new session
- `GET /api/sessions` - List user sessions  
- `POST /api/sessions/{id}/start` - Activate session
- `GET /api/sessions/{id}` - Get conversation history

#### Advanced Features
- **Context compaction** - Reduces memory usage for long conversations
- **Interaction counting** - Tracks message frequency
- **Multi-device sync** - Shared state across clients
