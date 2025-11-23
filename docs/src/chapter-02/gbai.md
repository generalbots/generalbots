# .gbai Architecture

The `.gbai` extension defines a bot application package in the GeneralBots template system. It serves as the container directory for all bot-related resources including dialogs, knowledge bases, and configuration.

## What is .gbai?

`.gbai` (General Bot Application Interface) is a directory-based package structure that contains all components needed to define a bot. During bootstrap, the system scans the `templates/` directory for folders ending in `.gbai` and automatically creates bot instances from them.

## Real .gbai Structure

A `.gbai` package is a directory containing subdirectories for different bot components:

```
my-bot.gbai/
├── my-bot.gbdialog/      # Dialog scripts (.bas files)
│   ├── start.bas
│   ├── auth.bas
│   └── *.bas
├── my-bot.gbkb/          # Knowledge base collections
│   ├── collection1/
│   └── collection2/
├── my-bot.gbot/          # Bot configuration
│   └── config.csv
└── my-bot.gbtheme/       # UI themes (optional)
    ├── css/
    └── html/
```

## Included Templates

BotServer includes two template `.gbai` packages:

### default.gbai

The default bot template with minimal configuration:
- `default.gbot/config.csv` - Basic server and LLM configuration

### announcements.gbai

A feature-rich example bot:
- `announcements.gbdialog/` - Multiple dialog scripts:
  - `start.bas` - Initialization and context setup
  - `auth.bas` - Authentication flow
  - `change-subject.bas` - Topic switching
  - `update-summary.bas` - Summary generation
- `announcements.gbkb/` - Knowledge base collections:
  - `auxiliom/` - Auxiliom product information
  - `news/` - News and announcements
  - `toolbix/` - Toolbix product information
- `annoucements.gbot/` - Bot configuration

## Package Components

### .gbdialog - Dialog Scripts

Contains BASIC-like scripts (`.bas` files) that define conversation logic:
- Simple English-like syntax
- Custom keywords: `TALK`, `HEAR`, `LLM`, `GET_BOT_MEMORY`, `SET_CONTEXT`
- Control flow and variables
- Tool integration

Example from `announcements.gbai/announcements.gbdialog/start.bas`:

```basic
resume1 = GET BOT MEMORY "resume"
resume2 = GET BOT MEMORY "auxiliom"
resume3 = GET BOT MEMORY "toolbix"

SET CONTEXT "general" AS resume1
SET CONTEXT "auxiliom" AS resume2
SET CONTEXT "toolbix" AS resume3

TALK resume1
TALK "You can ask me about any of the announcements or circulars."
```

### .gbkb - Knowledge Base

Directory structure containing collections of documents for semantic search:
- Each subdirectory represents a collection
- Documents are indexed into vector database
- Used for context retrieval during conversations

### .gbot - Configuration

Contains `config.csv` with bot parameters:
- Server settings (host, port)
- LLM configuration (API keys, model paths, URLs)
- Embedding model settings
- Email integration settings
- Database connection strings
- Component toggles (MCP server, LLM server)

### .gbtheme - UI Theme (Optional)

Custom UI themes with CSS and HTML assets for the web interface.

## Bootstrap Process

During the Auto Bootstrap process:

1. **Template Scanning**: System scans `templates/` directory for `.gbai` folders
2. **Bot Creation**: Each `.gbai` folder generates a bot record in the database
   - Folder name `default.gbai` → Bot name "Default"
   - Folder name `announcements.gbai` → Bot name "Announcements"
3. **Configuration Loading**: Bot configuration from `.gbot/config.csv` is loaded
4. **Template Upload**: All template files are uploaded to MinIO storage
5. **Dialog Loading**: BASIC scripts from `.gbdialog` are loaded and ready to execute
6. **KB Indexing**: Documents from `.gbkb` are indexed into Qdrant vector database

## Creating Custom .gbai Packages

To create a custom bot:

1. Create a new directory in `templates/` with `.gbai` extension:
   ```bash
   mkdir templates/mybot.gbai
   ```

2. Create the required subdirectories:
   ```bash
   mkdir -p templates/mybot.gbai/mybot.gbdialog
   mkdir -p templates/mybot.gbai/mybot.gbkb
   mkdir -p templates/mybot.gbai/mybot.gbot
   ```

3. Add dialog scripts to `.gbdialog/`:
   ```bash
   # Create start.bas with your conversation logic
   touch templates/mybot.gbai/mybot.gbdialog/start.bas
   ```

4. Add bot configuration to `.gbot/config.csv`:
   ```csv
   name,value
   server_host,0.0.0.0
   server_port,8080
   llm-key,your-api-key
   llm-url,https://api.openai.com/v1
   llm-model,gpt-4
   ```

5. Add knowledge base documents to `.gbkb/`:
   ```bash
   mkdir templates/mybot.gbai/mybot.gbkb/docs
   # Copy your documents into this directory
   ```

6. Restart BotServer - the bootstrap process will detect and create your bot

## Package Lifecycle

1. **Development**: Edit files in `templates/your-bot.gbai/`
2. **Bootstrap**: System creates bot from template
3. **Storage**: Files uploaded to MinIO for persistence
4. **Runtime**: Bot loads dialogs and configuration from storage
5. **Updates**: Modify template files and restart to apply changes

## Multi-Bot Support

A single BotServer instance can host multiple bots:
- Each `.gbai` package creates a separate bot
- Bots run in isolation with separate configurations
- Each bot has its own knowledge base collections
- Session state is maintained per bot

## Package Storage

After bootstrap, packages are stored in:
- **MinIO/S3**: Template files and assets
- **PostgreSQL**: Bot metadata and configuration
- **Qdrant**: Vector embeddings from knowledge bases
- **Redis**: Session and cache data

## Naming Conventions

- Package directory: `name.gbai`
- Dialog directory: `name.gbdialog`
- Knowledge base: `name.gbkb`
- Configuration: `name.gbot`
- Theme directory: `name.gbtheme`

The `name` should be consistent across all subdirectories within a package.