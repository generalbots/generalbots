## Feature Matrix
This table maps major features of GeneralBots to the chapters and keywords that implement them.

| Feature | Chapter(s) | Primary Keywords |
|---------|------------|------------------|
| Start server & basic chat | 01 (Run and Talk) | `TALK`, `HEAR` |
| Package system overview | 02 (About Packages) | – |
| Knowledge‑base management | 03 (gbkb Reference) | `USE KB`, `SET KB`, `USE WEBSITE` |
| UI theming | 04 (gbtheme Reference) | – (CSS/HTML assets) |
| BASIC dialog scripting | 05 (gbdialog Reference) | All BASIC keywords (`TALK`, `HEAR`, `LLM`, `FORMAT`, `USE KB`, `SET KB`, `USE WEBSITE`, …) |
| Custom Rust extensions | 06 (gbapp Reference) | `USE TOOL`, custom Rust code |
| Bot configuration | 07 (gbot Reference) | `config.csv` fields |
| Built‑in tooling | 08 (Tooling) | All keywords listed in the table |
| Semantic search & Qdrant | 03 (gbkb Reference) | `USE WEBSITE`, vector search |
| Email & external APIs | 08 (Tooling) | `CALL`, `CALL_ASYNC` |
| Scheduling & events | 08 (Tooling) | `SET SCHEDULE`, `ON` |
| Testing & CI | 10 (Contributing) | – |
| Database schema | Appendix I | Tables defined in `src/shared/models.rs` |

## See Also

- [AI and LLM](./ai-llm.md) - AI integration and LLM usage
- [Conversation Flow](./conversation.md) - Managing dialog flows
- [Storage](./storage.md) - Data persistence options
- [Knowledge Base](./knowledge-base.md) - Advanced KB patterns
- [Automation](./automation.md) - Scheduled tasks and events
- [Chapter 2: Packages](../chapter-02/README.md) - Understanding bot components
- [Chapter 3: KB Reference](../chapter-03/README.md) - Knowledge base fundamentals
- [Chapter 5: BASIC Reference](../chapter-05/README.md) - Complete command reference
- [Chapter 6: Extensions](../chapter-06/README.md) - Extending BotServer
- [Chapter 8: Integrations](../chapter-08/README.md) - External integrations
- [Chapter 10: Development](../chapter-10/README.md) - Development tools
- [Chapter 12: Web API](../chapter-12/README.md) - REST and WebSocket APIs

---

<div align="center">
  <img src="https://pragmatismo.com.br/icons/general-bots-text.svg" alt="General Bots" width="200">
</div>
