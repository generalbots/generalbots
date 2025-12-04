# Getting Started with General Bots

This guide will help you install, configure, and run your first General Bots instance.

## Prerequisites

- **Rust** (1.75 or later) - [Install from rustup.rs](https://rustup.rs/)
- **Git** - [Download from git-scm.com](https://git-scm.com/downloads)
- **8GB RAM** minimum (16GB recommended)
- **10GB disk space** for dependencies and data

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/GeneralBots/BotServer
cd BotServer
```

### 2. Run the Server

```bash
cargo run
```

On first run, General Bots automatically:

1. Downloads and compiles dependencies
2. Sets up PostgreSQL database
3. Configures S3-compatible storage (MinIO)
4. Initializes Redis cache
5. Downloads default LLM models
6. Creates template bots
7. Starts the HTTP server

The server will be available at `http://localhost:8080`.

## First Steps

### Access the Web Interface

Open your browser to:

- **Minimal UI**: `http://localhost:8080` - Lightweight chat interface
- **Full Suite**: `http://localhost:8080/suite` - Complete application suite

### Create Your First Bot

1. Navigate to the `templates/` directory
2. Copy the `template.gbai` folder:

```bash
cp -r templates/template.gbai templates/mybot.gbai
```

3. Edit the configuration in `mybot.gbai/mybot.gbot/config.csv`:

```csv
name,value
theme-title,My First Bot
theme-color1,#1565C0
```

4. Add knowledge to `mybot.gbai/mybot.gbkb/`:

```markdown
# Company FAQ

## What are your hours?
We are open Monday to Friday, 9 AM to 5 PM.

## How do I contact support?
Email support@example.com or call 1-800-EXAMPLE.
```

5. Create a dialog in `mybot.gbai/mybot.gbdialog/start.bas`:

```basic
' Welcome dialog
USE KB "mybot.gbkb"

TALK "Welcome to My Bot!"
TALK "How can I help you today?"

SET CONTEXT "assistant" AS "You are a helpful assistant for My Company."
```

6. Restart the server to load your new bot.

## Command-Line Options

```bash
# Default: console UI + web server
cargo run

# Disable console UI (background service)
cargo run -- --noconsole

# Desktop application mode (Tauri)
cargo run -- --desktop

# Specify tenant
cargo run -- --tenant mycompany

# LXC container mode
cargo run -- --container

# Disable all UI
cargo run -- --noui
```

## Project Structure

```
mybot.gbai/
├── mybot.gbot/           # Bot configuration
│   └── config.csv        # Theme and settings
├── mybot.gbkb/           # Knowledge base
│   └── faq.md            # FAQ documents
├── mybot.gbdialog/       # Dialog scripts
│   └── start.bas         # Main dialog
└── mybot.gbdrive/        # File storage
    └── templates/        # Document templates
```

## Essential BASIC Keywords

### Knowledge Base

```basic
USE KB "knowledge-name"     ' Load knowledge base
CLEAR KB                    ' Remove from session
```

### Tools

```basic
USE TOOL "tool-name"        ' Make tool available
CLEAR TOOLS                 ' Remove all tools
```

### Conversation

```basic
TALK "message"              ' Send message to user
answer = HEAR               ' Wait for user input
WAIT 5                      ' Wait 5 seconds
```

### Data

```basic
SAVE "table.csv", field1, field2    ' Save to storage
data = GET "https://api.example.com" ' HTTP request
SEND FILE "document.pdf"             ' Send file to user
```

## Environment Variables

General Bots requires minimal configuration. Only directory service variables are needed:

```bash
export DIRECTORY_URL="https://zitadel.example.com"
export DIRECTORY_CLIENT_ID="your-client-id"
export DIRECTORY_CLIENT_SECRET="your-secret"
```

All other services (database, storage, cache) are configured automatically.

## Testing Your Bot

### Via Web Interface

1. Open `http://localhost:8080`
2. Type a message in the chat box
3. The bot responds using your knowledge base and dialogs

### Via API

```bash
# Create a session
curl -X POST http://localhost:8080/api/sessions \
  -H "Content-Type: application/json" \
  -d '{"bot_id": "mybot"}'

# Send a message
curl -X POST http://localhost:8080/api/sessions/{session_id}/messages \
  -H "Content-Type: application/json" \
  -d '{"content": "What are your hours?"}'
```

### Via WebSocket

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
    ws.send(JSON.stringify({
        type: 'message',
        session_id: 'your-session-id',
        content: 'Hello!'
    }));
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Bot:', data.content);
};
```

## Common Issues

### Port Already in Use

```bash
# Find process using port 8080
lsof -i :8080

# Kill the process or use a different port
cargo run -- --port 8081
```

### Database Connection Failed

Ensure PostgreSQL is running:

```bash
botserver status postgres
botserver restart postgres
```

### LLM Not Responding

Check your LLM configuration in the admin panel or verify API keys are set.

## Next Steps

- **[API Reference](../api/README.md)** - Integrate with external systems
- **[BASIC Language](../reference/basic-language.md)** - Complete keyword reference
- **[Templates](templates.md)** - Pre-built bot templates
- **[Deployment](deployment.md)** - Production setup guide

## Getting Help

- **GitHub Issues**: [github.com/GeneralBots/BotServer/issues](https://github.com/GeneralBots/BotServer/issues)
- **Stack Overflow**: Tag questions with `generalbots`
- **Documentation**: [docs.pragmatismo.com.br](https://docs.pragmatismo.com.br)