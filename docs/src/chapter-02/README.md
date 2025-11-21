# Chapter 02: About Packages

BotServer uses a template-based package system to organize bot resources. Each bot is defined by a `.gbai` package directory containing various component subdirectories.

## Package Types

| Component | Extension | Role |
|-----------|-----------|------|
| Application Interface | `.gbai` | Root directory container for all bot resources |
| Dialog scripts | `.gbdialog` | BASIC-style conversational logic (`.bas` files) |
| Knowledge bases | `.gbkb` | Document collections for semantic search |
| Bot configuration | `.gbot` | CSV configuration file (`config.csv`) |
| UI themes | `.gbtheme` | CSS/HTML assets for web interface customization |
| File storage | `.gbdrive` | Object storage integration (MinIO/S3) |

## How Packages Work

### Template-Based System

BotServer uses a template-based approach:

1. **Templates Directory**: Bot packages are stored in `templates/` as `.gbai` folders
2. **Auto-Discovery**: During bootstrap, the system scans for `.gbai` directories
3. **Bot Creation**: Each `.gbai` package automatically creates a bot instance
4. **Storage Upload**: Template files are uploaded to MinIO for persistence
5. **Runtime Loading**: Bots load their resources from storage when serving requests

### Package Structure

Each `.gbai` package is a directory containing subdirectories:

```
botname.gbai/
├── botname.gbdialog/     # Dialog scripts
│   ├── start.bas         # Entry point script
│   ├── auth.bas          # Authentication flow
│   └── *.bas             # Other dialog scripts
├── botname.gbkb/         # Knowledge base
│   ├── collection1/      # Document collection
│   └── collection2/      # Another collection
├── botname.gbot/         # Configuration
│   └── config.csv        # Bot parameters
└── botname.gbtheme/      # UI theme (optional)
    ├── css/
    ├── html/
    └── assets/
```

## Included Templates

BotServer ships with two example templates:

### default.gbai

A minimal bot with basic configuration:
- Includes only `default.gbot/config.csv`
- Suitable starting point for new bots
- Demonstrates core configuration parameters

### announcements.gbai

A complete example bot showcasing all features:
- **Dialogs**: Multiple `.bas` scripts demonstrating conversation flows
- **Knowledge Base**: Three collections (auxiliom, news, toolbix)
- **Configuration**: Full configuration with LLM, email, and database settings
- **Features**: Context management, suggestions, memory retrieval

## Creating Your Own Package

To create a new bot package:

1. **Create Package Directory**:
   ```bash
   mkdir templates/mybot.gbai
   ```

2. **Add Subdirectories**:
   ```bash
   mkdir -p templates/mybot.gbai/mybot.gbdialog
   mkdir -p templates/mybot.gbai/mybot.gbkb
   mkdir -p templates/mybot.gbai/mybot.gbot
   ```

3. **Create Dialog Scripts**: Add `.bas` files to `.gbdialog/`

4. **Add Configuration**: Create `config.csv` in `.gbot/`

5. **Add Knowledge Base**: Place documents in `.gbkb/` subdirectories

6. **Restart BotServer**: Bootstrap process will detect and create your bot

## Package Lifecycle

```
Development → Bootstrap → Storage → Runtime → Updates
     ↓            ↓          ↓         ↓         ↓
  Edit files   Scan .gbai  Upload   Load from  Modify &
  in templates  folders    to MinIO  storage   restart
```

### Development Phase

- Create or modify files in `templates/your-bot.gbai/`
- Edit dialog scripts, configuration, and knowledge base documents
- Use version control (Git) to track changes

### Bootstrap Phase

- System scans `templates/` directory on startup
- Creates database records for new bots
- Generates bot names from folder names
- Applies default LLM and context settings

### Storage Phase

- Uploads all template files to MinIO (S3-compatible storage)
- Indexes documents into Qdrant vector database
- Stores configuration in PostgreSQL
- Ensures persistence across restarts

### Runtime Phase

- Bots load dialogs on-demand from storage
- Configuration is read from database
- Knowledge base queries hit vector database
- Session state maintained in Redis cache

### Update Phase

- Modify template files as needed
- Restart BotServer to re-run bootstrap
- Changes are detected and applied
- Existing bot data is updated

## Multi-Bot Hosting

A single BotServer instance can host multiple bots:

- **Isolation**: Each bot has separate configuration and state
- **Resource Sharing**: Bots share infrastructure (database, cache, storage)
- **Independent Updates**: Update one bot without affecting others
- **Tenant Support**: Optional multi-tenancy for enterprise deployments

## Package Storage Locations

After bootstrap, package data is distributed across services:

- **PostgreSQL**: Bot metadata, users, sessions, configuration
- **MinIO/S3**: Template files, uploaded documents, assets
- **Qdrant**: Vector embeddings for semantic search
- **Redis/Valkey**: Session cache, temporary data
- **File System**: Optional local caching

## Best Practices

### Naming Conventions

- Use consistent naming: `mybot.gbai`, `mybot.gbdialog`, `mybot.gbot`
- Use lowercase with hyphens: `customer-support.gbai`
- Avoid spaces and special characters

### Directory Organization

- Keep related dialogs in `.gbdialog/`
- Organize knowledge base by topic in `.gbkb/subdirectories/`
- Use descriptive collection names
- Include a `start.bas` as the entry point

### Configuration Management

- Store sensitive data in environment variables, not `config.csv`
- Document custom configuration parameters
- Use reasonable defaults
- Test configuration changes in development first

### Knowledge Base Structure

- Organize documents into logical collections
- Use subdirectories to separate topics
- Include metadata files if needed
- Keep documents in supported formats (PDF, TXT, MD)

### Version Control

- Commit entire `.gbai` packages to Git
- Use `.gitignore` for generated files
- Tag releases for production deployments
- Document changes in commit messages

## Package Component Details

For detailed information about each package type:

- **[.gbai Architecture](./gbai.md)** - Package structure and lifecycle
- **[.gbdialog Dialogs](./gbdialog.md)** - BASIC scripting and conversation flows
- **[.gbkb Knowledge Base](./gbkb.md)** - Document indexing and semantic search
- **[.gbot Bot Configuration](./gbot.md)** - Configuration parameters and settings
- **[.gbtheme UI Theming](./gbtheme.md)** - Web interface customization
- **[.gbdrive File Storage](./gbdrive.md)** - MinIO/S3 object storage integration

## Migration from Other Platforms

If you're migrating from other bot platforms:

- **Dialog Flows**: Convert to BASIC scripts in `.gbdialog/`
- **Intents/Entities**: Use LLM-based understanding instead
- **Knowledge Base**: Import documents into `.gbkb/` collections
- **Configuration**: Map settings to `config.csv` parameters
- **Custom Code**: Implement as Rust keywords or external tools

## Troubleshooting

### Bot Not Created

- Check `templates/` directory exists
- Verify `.gbai` folder name ends with extension
- Review bootstrap logs for errors
- Ensure subdirectories follow naming convention

### Configuration Not Applied

- Verify `config.csv` format is correct
- Check for typos in parameter names
- Restart BotServer after changes
- Review database for updated values

### Knowledge Base Not Indexed

- Ensure `.gbkb/` contains subdirectories with documents
- Check Qdrant is running and accessible
- Verify embedding model is configured
- Review indexing logs for errors

### Dialogs Not Executing

- Check `.bas` file syntax
- Verify `start.bas` exists
- Review runtime logs for errors
- Test with simple dialog first