# Sources - Prompts & Templates

> **Your bot configuration hub**

<img src="../../assets/suite/sources-screen.svg" alt="Sources Interface Screen" style="max-width: 100%; height: auto;">

---

## Overview

Sources is the configuration center in General Bots Suite. Manage your bots, prompts, templates, and knowledge bases all in one place. Sources is where you create new bots, customize their behavior, and organize the content that powers your AI assistant.

---

## Features

### Managing Bots

#### Creating a New Bot

1. Click **+ New Bot** in the top right
2. Fill in the bot details:

| Field | Description |
|-------|-------------|
| **Bot ID** | Unique identifier (lowercase, numbers, hyphens only) |
| **Display Name** | User-friendly name shown in chat |
| **Description** | Brief explanation of what the bot does |
| **Start from** | Blank, Template, or Clone existing |

#### Bot Settings

Click the **⚙️** icon on any bot to configure:

**General Settings:**

| Setting | Description |
|---------|-------------|
| **Display Name** | Name shown to users |
| **Welcome Message** | First message when conversation starts |
| **Language** | Primary language for the bot |
| **Timezone** | Bot's timezone for date/time operations |
| **Status** | Live, Draft, or Maintenance |

**Status Options:**
- **Live** - Bot is active and responding
- **Draft** - Bot is hidden from users
- **Maintenance** - Shows maintenance message

#### AI Settings

| Setting | Description |
|---------|-------------|
| **Provider** | AI provider (OpenAI, Azure, etc.) |
| **Model** | Model to use (GPT-4, etc.) |
| **Temperature** | Creativity level (0 = focused, 1 = creative) |
| **Max Tokens** | Maximum response length |
| **System Prompt** | Bot personality and instructions |
| **Knowledge Base** | Connected .gbkb for answers |

---

### Managing Prompts

Prompts are reusable text templates for AI interactions.

**Prompt Types:**

| Type | Purpose |
|------|---------|
| **System Prompt** | Bot personality/behavior |
| **Task Prompt** | Specific task instructions |
| **Template** | Reusable text with variables |

**Creating a Prompt:**

1. Click **+ New Prompt**
2. Enter a name (e.g., "support-agent")
3. Select type
4. Write prompt content with optional `{{variables}}`
5. Save and link to bots

**Example Prompt:**

```botserver/docs/src/chapter-04-gbui/apps/sources-prompt-example.txt
You are a friendly and professional customer support agent
for {{company_name}}.

## Your Personality
- Be warm and empathetic
- Use simple, clear language
- Be patient and thorough

## Guidelines
- Always verify customer identity before sharing account info
- If unsure, search the knowledge base
- Escalate complex issues to human agents
- Never make promises about refunds or compensation
```

---

### Managing Templates

Templates are pre-built bot packages you can reuse.

**Installed Templates:**

| Template | Description |
|----------|-------------|
| **📋 CRM** | Full CRM with leads, contacts |
| **📋 Support** | Ticket management and customer service |
| **📋 FAQ** | Answer common questions from KB |

**Available Templates:**

| Template | Description |
|----------|-------------|
| **📋 HR** | Employee self-service |
| **📋 Analytics** | Dashboard and metrics |
| **📋 Compliance** | LGPD, GDPR compliance |

**Template Contents:**

Templates include:
- Dialog scripts (.bas files)
- Bot configuration
- Knowledge base documentation
- Sample conversations

---

### Managing Knowledge Bases

Knowledge bases store documents that your bot can search for answers.

| Field | Description |
|-------|-------------|
| **Documents** | Count of uploaded files |
| **Size** | Total storage used |
| **Last Indexed** | When content was last processed |
| **Used By** | Bots connected to this KB |

**Uploading Documents:**

1. Open the knowledge base
2. Click **Upload** or drag files
3. Organize into folders
4. Click **Reindex** to process new content

**Supported Formats:**
- PDF, DOCX, TXT, MD
- CSV, XLSX
- HTML, JSON

---

## Import and Export

### Exporting a Bot

1. Click **⚙️** on the bot
2. Select **Export**
3. Choose what to include:
   - Bot configuration
   - Dialog scripts (.bas files)
   - Prompts
   - Knowledge base (optional, large)
   - Conversation history (optional)
4. Select format: .gbai, ZIP, or JSON

### Importing a Bot

1. Click **Import** at the top
2. Drop file or browse (supported: .gbai, .zip)
3. Choose:
   - Create new bot, or
   - Replace existing bot
4. Configure merge options for prompts and KB

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | New bot |
| `Ctrl+S` | Save changes |
| `Ctrl+E` | Export selected |
| `Ctrl+I` | Import |
| `Delete` | Delete selected |
| `Ctrl+D` | Duplicate |
| `F2` | Rename |
| `/` | Search |
| `Enter` | Open selected |
| `Escape` | Close dialog |

---

## Tips & Tricks

### Bot Management

💡 **Use descriptive names** - "customer-support-v2" is better than "bot1"

💡 **Keep prompts separate** - Reuse prompts across multiple bots

💡 **Version your exports** - Export before major changes

💡 **Test in Draft mode** - Don't go Live until fully tested

### Prompt Writing

💡 **Be specific** - Clear instructions give better results

💡 **Use examples** - Show the AI what good responses look like

💡 **Set boundaries** - Define what the bot should NOT do

💡 **Use variables** - Make prompts reusable with {{placeholders}}

### Knowledge Base

💡 **Organize in folders** - Group related documents together

💡 **Keep documents current** - Remove outdated information

💡 **Use clear filenames** - "refund-policy-2025.pdf" not "doc1.pdf"

💡 **Reindex after changes** - New content isn't searchable until indexed

---

## Troubleshooting

### Bot not responding

**Possible causes:**
1. Bot is in Draft mode
2. AI provider not configured
3. No dialogs or prompts set up

**Solution:**
1. Check bot status is "Live"
2. Verify AI settings have valid API key
3. Ensure at least start.bas exists
4. Check error logs in Analytics

---

### Knowledge base not finding answers

**Possible causes:**
1. Documents not indexed
2. Document format not supported
3. Query doesn't match content

**Solution:**
1. Click "Reindex" and wait for completion
2. Convert documents to supported formats
3. Check document actually contains the information
4. Try different phrasing

---

### Import fails

**Possible causes:**
1. File corrupted
2. Incompatible version
3. Duplicate bot ID

**Solution:**
1. Try re-exporting from source
2. Check General Bots version compatibility
3. Choose "Create new bot" instead of replace
4. Rename bot ID if duplicate

---

### Prompts not applying

**Possible causes:**
1. Prompt not linked to bot
2. Variable not defined
3. Syntax error in prompt

**Solution:**
1. Check AI Settings → System Prompt selection
2. Verify all {{variables}} have values
3. Test prompt with "Test" button
4. Check for unclosed brackets or quotes

---

## BASIC Integration

Access Sources data from dialogs:

### Get Bot Configuration

```botserver/docs/src/chapter-04-gbui/apps/sources-config.basic
config = GET BOT CONFIG
TALK "Bot name: " + config.displayName
TALK "Language: " + config.language
```

### Use Prompts

```botserver/docs/src/chapter-04-gbui/apps/sources-prompts.basic
' Load a prompt template
prompt = GET PROMPT "summarize"

' Use with variables
summary = GENERATE WITH PROMPT prompt, content
TALK summary
```

### Search Knowledge Base

```botserver/docs/src/chapter-04-gbui/apps/sources-search.basic
HEAR question AS TEXT "What would you like to know?"

results = SEARCH KB question IN "support.gbkb"

IF COUNT(results) > 0 THEN
    TALK results[0].answer
    TALK "Source: " + results[0].source
ELSE
    TALK "I couldn't find information about that."
END IF
```

### List Available Bots

```botserver/docs/src/chapter-04-gbui/apps/sources-list.basic
bots = GET BOTS

TALK "Available bots:"
FOR EACH bot IN bots
    IF bot.status = "live" THEN
        TALK "● " + bot.displayName
    ELSE
        TALK "○ " + bot.displayName + " (draft)"
    END IF
NEXT
```

---

## See Also

- [Designer App](./designer.md) - Visual flow builder
- [Drive App](./drive.md) - Upload KB documents
- [How To: Create Your First Bot](../how-to/create-first-bot.md)
- [How To: Add Documents to Knowledge Base](../how-to/add-kb-documents.md)