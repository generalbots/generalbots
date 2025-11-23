# Chapter 01: Run and Talk

Welcome to General Bots - your journey to AI independence starts here. In a world dominated by expensive, proprietary AI solutions, General Bots offers a refreshing alternative: a complete, open-source AI platform that you control entirely.

## Why General Bots?

Before diving into installation, let's understand what makes General Bots different:

1. **Complete Ownership**: Unlike SaaS solutions that lock your data in the cloud, General Bots runs on your infrastructure. Your conversations, your data, your rules.

2. **Zero-to-AI in Minutes**: Our bootstrap process sets up everything - database, storage, vector search, and AI models - with a single command. No DevOps expertise required.

3. **Cost-Effective**: Running your own AI infrastructure can be 10x cheaper than cloud services at scale.

4. **Privacy First**: Your data never leaves your servers. Perfect for healthcare, finance, or any privacy-conscious application.

## The One-Command Install

```bash
./botserver
```

That's literally it. First run triggers auto-bootstrap that:
- Installs PostgreSQL, cache, storage, vector DB
- Downloads AI models
- Creates default bot
- Starts UI server

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
Starting UI server on :8080         ✓
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

## See Also

### Documentation
- [Overview](./overview.md) - Architecture and concepts
- [Quick Start](./quick-start.md) - Get running in 5 minutes
- [Installation](./installation.md) - Detailed setup instructions
- [First Conversation](./first-conversation.md) - Build your first bot
- [Sessions and Channels](./sessions.md) - Multi-user support
- [Chapter 2: Packages](../chapter-02/README.md) - Understanding bot components

### Further Reading - Blog Posts
- [Why We Chose Open Source](https://pragmatismo.cloud/blog/why-pragmatismo-selected-open-source) - Philosophy behind General Bots
- [Escape from BigTech](https://pragmatismo.cloud/blog/escape-from-bigtech) - Breaking free from proprietary AI platforms
- [Cost-Effective Bot Orchestration](https://pragmatismo.cloud/blog/cost-effective-bot-orchestration) - Economic benefits of self-hosting
- [The Hidden Costs of SaaS](https://pragmatismo.cloud/blog/saas-hidden-costs) - Why owning your stack matters
- [LLM Boom Is Over](https://pragmatismo.cloud/blog/llm-boom-is-over) - Focus on practical AI applications

### Next Chapter
Continue to [Chapter 2: About Packages](../chapter-02/README.md) to learn about the template system that makes General Bots so powerful.
- [Chapter 3: Knowledge Base](../chapter-03/README.md) - Document management
- [Chapter 5: BASIC Reference](../chapter-05/README.md) - Complete command list

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