# Chapter 02: About Packages

How bots are organized in General Bots.

## What You'll Learn

- Package structure (`.gbai` folders)
- Dialog scripts in BASIC
- Knowledge bases
- Configuration basics
- How packages load

## Package Structure

A bot is just a folder ending in `.gbai`:

```
my-bot.gbai/
├── my-bot.gbdialog/    # BASIC scripts
├── my-bot.gbkb/        # Documents
├── my-bot.gbot/        # Configuration
├── my-bot.gbtheme/     # Optional styling
└── my-bot.gbdrive/     # Optional storage
```

Drop the folder in `templates/`, it loads automatically.

## Key Concepts

### Dialogs (.gbdialog)
- BASIC scripts that control conversation
- `start.bas` is optional (but needed to activate tools/KB with USE TOOL/USE KB)
- Simple commands like TALK and HEAR

### Knowledge Base (.gbkb)
- Put PDFs and documents in folders
- Automatically becomes searchable
- Bot can answer questions from documents

### Configuration (.gbot)
- Single `config.csv` file
- Simple name,value pairs
- Missing values use defaults

### Themes (.gbtheme)
- Optional CSS styling
- Most bots don't need this

### Storage (.gbdrive)
- Links to S3-compatible storage
- For large files and uploads

## How It Works

1. **Discovery**: Finds `.gbai` folders
2. **Loading**: Reads all components
3. **Indexing**: Processes documents
4. **Activation**: Bot is ready

No build process. No compilation. Just folders and files.

The web UI uses **HTMX with server-side rendering** - minimal JavaScript, no build process, just HTML templates powered by Rust.

## Topics Covered

- [.gbai Architecture](./gbai.md) - Package details
- [.gbdialog Dialogs](./gbdialog.md) - BASIC scripting
- [.gbkb Knowledge Base](./gbkb.md) - Document management
- [.gbot Configuration](./gbot.md) - Settings
- [.gbtheme UI Theming](./gbtheme.md) - Styling
- [.gbdrive File Storage](./gbdrive.md) - Storage integration
- [Bot Templates](./templates.md) - Example bots

---

<div align="center">
  <img src="https://pragmatismo.com.br/icons/general-bots-text.svg" alt="General Bots" width="200">
</div>