# BotServer Quick Start

## Installation in 3 Steps

### 1. Run BotServer

```bash
./botserver
```

That's it! No configuration needed.

### 2. Wait for Bootstrap (2-5 minutes)

You'll see:

```
🚀 BotServer starting...
📦 Bootstrap: Detecting system...
📦 Installing PostgreSQL...
   ✓ Database created
   ✓ Schema initialized
   ✓ Credentials saved to .env
📦 Installing MinIO...
   ✓ Object storage ready
   ✓ Buckets created
📦 Installing Valkey...
   ✓ Cache server running
🤖 Creating bots from templates...
   ✓ default.gbai → Default bot
   ✓ announcements.gbai → Announcements bot
✅ BotServer ready at http://localhost:8080
```

### 3. Open Browser

```
http://localhost:8080
```

Start chatting with your bot!

---

## What Just Happened?

The **automatic bootstrap** process:

1. ✅ Detected your OS (Linux/macOS/Windows)
2. ✅ Installed PostgreSQL database
3. ✅ Installed MinIO object storage
4. ✅ Installed Valkey cache
5. ✅ Generated secure credentials → `.env`
6. ✅ Created database schema
7. ✅ Created default bots from `templates/`
8. ✅ Started web server on port 8080

**Zero manual configuration required!**

---

## Create Your First Tool

Tools are just `.bas` files. Create `enrollment.bas`:

```bas
' Student enrollment tool
PARAM name AS string    LIKE "John Smith"        DESCRIPTION "Student name"
PARAM email AS string   LIKE "john@example.com"  DESCRIPTION "Email address"
PARAM course AS string  LIKE "Computer Science"  DESCRIPTION "Course to enroll"

DESCRIPTION "Processes student enrollment"

SAVE "enrollments.csv", name, email, course, NOW()
TALK "Welcome to " + course + ", " + name + "!"
```

The LLM automatically discovers this tool and knows when to call it!

---

## Add Knowledge Base

Drop documents in a `.gbkb/` folder:

```
mybot.gbai/
  mybot.gbkb/
    docs/
      manual.pdf
      faq.docx
      guide.txt
```

The bot automatically:
- Indexes documents with vector embeddings
- Answers questions from the content
- Updates when files change

---

## Container Mode (LXC)

BotServer uses **LXC** (Linux Containers) for containerized deployment:

```bash
# Force container mode
./botserver --container

# Components run in isolated LXC containers
# - PostgreSQL in {tenant}-tables container
# - MinIO in {tenant}-drive container
# - Valkey in {tenant}-cache container
```

**Benefits**: 
- ✅ Clean isolation - system-level containers
- ✅ Easy cleanup - `lxc delete {container}`
- ✅ No system pollution - everything in containers
- ✅ Lightweight - more efficient than VMs

**Requires**: LXC/LXD installed (`sudo snap install lxd`)

---

## Build from Source

```bash
git clone https://github.com/GeneralBots/BotServer.git
cd BotServer
cargo run
```

Same automatic bootstrap process!

---

## Optional Components

After installation, add more features:

```bash
./botserver install email      # Stalwart email server
./botserver install directory  # Zitadel identity provider
./botserver install llm        # Local LLM server (offline mode)
./botserver install meeting    # LiveKit video conferencing
```

---

## Example Bot Structure

```
mybot.gbai/
├── mybot.gbdialog/          # Dialog scripts
│   ├── start.bas            # Entry point (required)
│   ├── get-weather.bas      # Tool (auto-discovered)
│   └── send-email.bas       # Tool (auto-discovered)
├── mybot.gbkb/              # Knowledge base
│   ├── docs/                # Document collection
│   └── faq/                 # FAQ collection
├── mybot.gbot/              # Configuration
│   └── config.csv           # Bot parameters
└── mybot.gbtheme/           # UI theme (optional)
    └── custom.css
```

Save to `templates/mybot.gbai/` and restart - bot created automatically!

---

## How It Really Works

You DON'T write complex dialog flows. Instead:

### 1. Add Documents
```
mybot.gbkb/
  policies/enrollment-policy.pdf
  catalog/courses.pdf
```

### 2. Create Tools (Optional)
```bas
' enrollment.bas - just define what it does
PARAM name AS string
PARAM course AS string
SAVE "enrollments.csv", name, course
```

### 3. Start Chatting!
```
User: I want to enroll in computer science
Bot: I'll help you enroll! What's your name?
User: John Smith
Bot: [Automatically calls enrollment.bas with collected params]
     Welcome to Computer Science, John Smith!
```

The LLM handles ALL conversation logic automatically!

---

## Configuration (Optional)

Bootstrap creates `.env` automatically:

```env
DATABASE_URL=postgres://gbuser:RANDOM_PASS@localhost:5432/botserver
DRIVE_SERVER=http://localhost:9000
DRIVE_ACCESSKEY=GENERATED_KEY
DRIVE_SECRET=GENERATED_SECRET
```

You can also configure per-bot settings in `config.csv`:

```csv
name,value
server_port,8080
llm-url,http://localhost:8081
prompt-compact,4
theme-color1,#0d2b55
```

---

## Troubleshooting

### Port 8080 in use?

Edit `templates/default.gbai/default.gbot/config.csv`:

```csv
name,value
server_port,3000
```

### Clean install?

```bash
# Remove everything and start fresh
rm -rf /opt/gbo  # Linux/macOS
./botserver
```

### Check component status

```bash
./botserver status tables    # PostgreSQL
./botserver status drive     # MinIO
./botserver status cache     # Valkey
```

---

## Documentation

- **[Full Installation Guide](docs/src/chapter-01/installation.md)** - Detailed bootstrap explanation
- **[Tool Definition](docs/src/chapter-08/tool-definition.md)** - Creating tools
- **[BASIC Keywords](docs/src/chapter-05/keywords.md)** - Language reference
- **[Package System](docs/src/chapter-02/README.md)** - Creating bots
- **[Architecture](docs/src/chapter-06/architecture.md)** - How it works

---

## The Magic Formula

```
📚 Documents + 🔧 Tools + 🤖 LLM = ✨ Intelligent Bot
```

### What You DON'T Need:
- ❌ IF/THEN logic
- ❌ Intent detection  
- ❌ Dialog flow charts
- ❌ State machines
- ❌ Complex routing

### What You DO:
- ✅ Drop documents in `.gbkb/`
- ✅ Create simple `.bas` tools (optional)
- ✅ Start chatting!

The LLM understands context, calls tools, searches documents, and maintains conversation naturally.

---

## Philosophy

1. **Just Run It** - No manual configuration
2. **Simple Scripts** - BASIC-like language anyone can learn  
3. **Automatic Discovery** - Tools and KBs auto-detected
4. **Secure by Default** - Credentials auto-generated
5. **Production Ready** - Built for real-world use

---

## Real Example: Education Bot

1. **Add course materials:**
   ```
   edu.gbkb/
     courses/computer-science.pdf
     policies/enrollment.pdf
   ```

2. **Create enrollment tool:**
   ```bas
   ' enrollment.bas
   PARAM name AS string
   PARAM course AS string
   SAVE "enrollments.csv", name, course
   ```

3. **Just chat:**
   ```
   User: What courses do you offer?
   Bot: [Searches PDFs] We offer Computer Science, Data Science...
   
   User: I want to enroll
   Bot: [Calls enrollment.bas] Let me help you enroll...
   ```

**No programming logic needed - the LLM handles everything!** 🎉