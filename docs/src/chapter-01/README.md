# Chapter 01: Run and Talk

**Zero to chatbot in 60 seconds.** This chapter gets BotServer running with a working bot you can talk to immediately.

## The One-Command Install

```bash
./botserver
```

That's literally it. First run triggers auto-bootstrap that:
- Installs PostgreSQL, cache, storage, vector DB
- Downloads AI models
- Creates default bot
- Starts web server

Takes 2-5 minutes. Grab coffee. Come back to a running bot.

## Your First Chat

Once bootstrap finishes:

1. Open browser to `http://localhost:8080`
2. Write a simple tool (see below)
3. Bot responds using GPT-3.5 (or local model)

No configuration. No API keys (for local). It just works.

## What's Actually Happening

Behind that simple `./botserver` command:

```
Installing PostgreSQL 16.2...       ✓
Installing Valkey cache...          ✓  
Installing SeaweedFS storage...     ✓
Installing Qdrant vectors...        ✓
Downloading embeddings...           ✓
Creating database schema...         ✓
Generating secure credentials...    ✓
Loading bot templates...            ✓
Starting web server on :8080        ✓
```

Everything lands in `botserver-stack/` directory. Fully self-contained.

## Make Your Own Bot in 2 Minutes

### Step 1: Create Package
```bash
mkdir templates/my-bot.gbai
mkdir templates/my-bot.gbai/my-bot.gbdialog
```

### Step 2: Write Start Script
```bash
cat > templates/my-bot.gbai/my-bot.gbdialog/start.bas << 'EOF'
TALK "Hi! I'm your personal assistant."
TALK "What can I help you with?"
answer = HEAR
TALK "I can help you with: " + answer
EOF
```

### Step 3: Restart & Test
```bash
./botserver restart
# Visit http://localhost:8080/my-bot
```

Your bot is live.

## Adding Intelligence

### Give It Knowledge
Drop PDFs into knowledge base:
```bash
mkdir templates/my-bot.gbai/my-bot.gbkb
cp ~/Documents/policies.pdf templates/my-bot.gbai/my-bot.gbkb/
```

Bot instantly answers questions from your documents.

### Give It Tools
Create a tool for booking meetings:
```bash
cat > templates/my-bot.gbai/my-bot.gbdialog/book-meeting.bas << 'EOF'
PARAM person, date, time
DESCRIPTION "Books a meeting"

SAVE "meetings.csv", person, date, time
TALK "Meeting booked with " + person + " on " + date
EOF
```

Now just say "Book a meeting with John tomorrow at 2pm" - AI handles the rest.

## Optional Components

Want email? Video calls? Better models?

```bash
./botserver install email      # Full email server
./botserver install meeting    # Video conferencing  
./botserver install llm        # Local AI models
```

Each adds specific functionality. None required to start.

## File Structure After Bootstrap

```
botserver-stack/
  postgres/          # Database files
  valkey/           # Cache data
  seaweedfs/        # Object storage
  qdrant/           # Vector database
  models/           # Embeddings

templates/
  default.gbai/     # Default bot
  my-bot.gbai/      # Your bot

.env                  # Auto-generated config
```

## Troubleshooting Quick Fixes

**Port already in use?**
```bash
HTTP_PORT=3000 ./botserver
```

**Bootstrap fails?**
```bash
./botserver cleanup
./botserver  # Try again
```

**Want fresh start?**
```bash
rm -rf botserver-stack .env
./botserver
```

**Check what's running:**
```bash
./botserver status
```

## Container Deployment

Prefer containers? Use LXC mode:
```bash
./botserver --container
```

Creates isolated LXC containers for each component. Same auto-bootstrap, better isolation.

## What You've Learned

✅ BotServer installs itself completely  
✅ Default bot works immediately  
✅ Create new bots in minutes  
✅ Add documents for instant knowledge  
✅ Write tools for custom actions  

## Next Steps

- **[Quick Start](./quick-start.md)** - Build a real bot
- **[Installation Details](./installation.md)** - How bootstrap works
- **[First Conversation](./first-conversation.md)** - Chat interface tour
- **[Sessions](./sessions.md)** - How conversations persist

## The Philosophy

We believe setup should be invisible. You want a bot, not a DevOps degree. That's why everything auto-configures. Focus on your bot's personality and knowledge, not infrastructure.

Ready for more? Continue to [Quick Start](./quick-start.md) to build something real.