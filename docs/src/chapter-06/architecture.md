# Architecture Overview

## Auto Bootstrap Process

The Auto Bootstrap process is responsible for initializing and configuring the entire BotServer environment after installation. It ensures that all system components are installed, configured, and started automatically, and that bots are created from predefined templates.

### 1. Bootstrap Initialization

The process begins with the `BootstrapManager`, which is instantiated with an installation mode (`Local` or `Container`) and an optional tenant name. It initializes the `PackageManager`, which detects the operating system and sets up the base installation path (e.g., `/opt/gbo` or `botserver-stack`).

### 2. Component Registration and Installation

The `PackageManager` registers all system components such as:

- **tables** (PostgreSQL database)
- **cache** (Valkey/Redis)
- **drive** (MinIO object storage)
- **llm** (local LLM server)
- **email**, **proxy**, **directory**, **alm**, **dns**, **meeting**, **table_editor**, **doc_editor**, **desktop**, **devtools**, **bot**, **system**, **vector_db**, **host**

Each component has a `ComponentConfig` defining:
- Ports and dependencies
- Download URLs and binaries
- Pre/post-install commands
- Environment variables
- Execution commands

During bootstrap, required components (`tables`, `drive`, `cache`) are installed and started automatically.  
For example:
- The **tables** component generates secure database credentials, writes them to `.env`, and applies SQL migrations to initialize the schema.
- The **drive** component creates secure credentials and stores them encrypted in the database.

### 3. Bot Configuration

After components are installed, the bootstrap process updates the bot configuration in the database.  
The method `update_bot_config()` ensures each component’s configuration is linked to a bot record in the `bot_configuration` table.  
If no bot exists, a new UUID is generated to associate configuration entries.

### 4. Template-Based Bot Creation

The method `create_bots_from_templates()` scans the `templates/` directory for folders ending in `.gbai` (e.g., `default.gbai`, `announcements.gbai`).  
Each `.gbai` folder represents a bot template.

For each template:
- The folder name is converted into a human-readable bot name (e.g., `default.gbai` → “Default”).
- If the bot doesn’t exist in the `bots` table, a new record is inserted with:
  - Default LLM provider (`openai`)
  - Default configuration (`{"model": "gpt-4", "temperature": 0.7}`)
  - Context provider (`database`)
  - Active status (`true`)

This automatically creates bots from templates during bootstrap.

### 5. Template Upload to MinIO

After bots are created, the method `upload_templates_to_minio()` uploads all template files recursively to a MinIO bucket (S3-compatible storage).  
This makes templates accessible for runtime bot operations and ensures persistence across environments.

### 6. Summary

The Auto Bootstrap process performs the following steps automatically:
1. Detects environment and installation mode.
2. Registers and installs required components.
3. Initializes the database and applies migrations.
4. Updates bot configuration records.
5. Creates bots from `.gbai` templates.
6. Uploads templates to MinIO for storage.

This process ensures that after installation, the system is fully operational with preconfigured bots derived from templates, ready to serve requests immediately.
