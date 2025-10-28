# .gbai Architecture

The `.gbai` extension defines the overall architecture and structure of a GeneralBots application. It serves as the container for all other package types.

## What is .gbai?

`.gbai` (General Bot Application Interface) is the root package that contains:
- Bot identity and metadata
- Organizational structure
- References to other package types
- Application-level configuration

## .gbai Structure

A typical `.gbai` package contains:

```
my-bot.gbai/
├── manifest.json          # Application metadata
├── .gbdialog/            # Dialog scripts
├── .gbkb/               # Knowledge bases  
├── .gbot/               # Bot configuration
├── .gbtheme/            # UI themes
└── dependencies.json    # External dependencies
```

## Manifest File

The `manifest.json` defines application properties:

```json
{
  "name": "Customer Support Bot",
  "version": "1.0.0",
  "description": "AI-powered customer support assistant",
  "author": "Your Name",
  "bot_id": "uuid-here",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

## Application Lifecycle

1. **Initialization**: Bot loads .gbai structure and dependencies
2. **Configuration**: Applies .gbot settings and parameters
3. **Activation**: Loads .gbdialog scripts and .gbkb collections
4. **Execution**: Begins processing user interactions
5. **Termination**: Cleanup and state preservation

## Multi-Bot Environments

A single GeneralBots server can host multiple .gbai applications:
- Each runs in isolation with separate configurations
- Can share common knowledge bases if configured
- Maintain separate user sessions and contexts
