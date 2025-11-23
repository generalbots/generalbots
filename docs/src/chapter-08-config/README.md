## gbot Reference
`config.csv` defines the bot’s behaviour and parameters.

```csv
# config.csv – Bot configuration
bot_name,GeneralBot
language,en
theme,default.gbtheme
knowledge_base,default.gbkb
max_context_tokens,2048
```

### Key Columns
- **bot_name** – Display name of the bot.
- **language** – Locale for formatting (used by `FORMAT`).
- **theme** – UI theme package (`.gbtheme`).
- **knowledge_base** – Default knowledge‑base package (`.gbkb`).
- **max_context_tokens** – Maximum number of tokens retained in the session context.
- **max_context_tokens** – Limit for the amount of context sent to the LLM.

### Editing the Configuration
The file is a simple CSV; each line is `key,value`. Comments start with `#`. After editing, restart the server to apply changes.

### Runtime Effects
- Changing **theme** updates the UI served from `web/static/`.
- Modifying **knowledge_base** switches the vector collection used for semantic search.
- Adjusting **answer_mode** influences the order of tool invocation and LLM calls.

For advanced configuration, see `src/bot/config.rs` which parses this file into the `BotConfig` struct.

## See Also

- [config.csv Reference](./config-csv.md) - Complete configuration options
- [PostgreSQL Setup](./postgresql.md) - Database configuration
- [MinIO Storage](./minio.md) - Object storage setup
- [Qdrant Vector DB](./qdrant.md) - Vector database configuration
- [Valkey Cache](./valkey.md) - Caching layer setup
- [Chapter 2: .gbot](../chapter-02/gbot.md) - Bot configuration package
- [Chapter 3: Knowledge Base](../chapter-03/README.md) - KB configuration
- [Chapter 5: BASIC Reference](../chapter-05/README.md) - Script configuration
- [Chapter 9: Storage](../chapter-09/storage.md) - Storage architecture
- [Chapter 11: Infrastructure](../chapter-11/README.md) - Complete infrastructure guide
