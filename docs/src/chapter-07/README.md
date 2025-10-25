## gbot Reference
`config.csv` defines the bot’s behaviour and parameters.

```csv
# config.csv – Bot configuration
bot_name,GeneralBot
language,en
theme,default.gbtheme
knowledge_base,default.gbkb
max_context_tokens,2048
answer_mode,LLM_ONLY
```

### Key Columns
- **bot_name** – Display name of the bot.
- **language** – Locale for formatting (used by `FORMAT`).
- **theme** – UI theme package (`.gbtheme`).
- **knowledge_base** – Default knowledge‑base package (`.gbkb`).
- **max_context_tokens** – Maximum number of tokens retained in the session context.
- **answer_mode** – Determines how responses are generated:
  - `LLM_ONLY` – Direct LLM response.
  - `TOOLS_FIRST` – Try tools before falling back to LLM.
  - `DOCS_ONLY` – Use only documentation sources.

### Editing the Configuration
The file is a simple CSV; each line is `key,value`. Comments start with `#`. After editing, restart the server to apply changes.

### Runtime Effects
- Changing **theme** updates the UI served from `web/static/`.
- Modifying **knowledge_base** switches the vector collection used for semantic search.
- Adjusting **answer_mode** influences the order of tool invocation and LLM calls.

For advanced configuration, see `src/bot/config.rs` which parses this file into the `BotConfig` struct.
