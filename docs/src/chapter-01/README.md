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
Each conversation is represented by a **BotSession**. The session stores:
- User identifier
- Conversation history
- Current context (variables, knowledge base references, etc.)

Sessions are persisted in the SQLite database defined in `src/shared/models.rs`.
