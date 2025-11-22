## Feature Matrix
This table maps major features of GeneralBots to the chapters and keywords that implement them.

| Feature | Chapter(s) | Primary Keywords |
|---------|------------|------------------|
| Start server & basic chat | 01 (Run and Talk) | `TALK`, `HEAR` |
| Package system overview | 02 (About Packages) | – |
| Knowledge‑base management | 03 (gbkb Reference) | `USE_KB`, `SET_KB`, `ADD_WEBSITE` |
| UI theming | 04 (gbtheme Reference) | – (CSS/HTML assets) |
| BASIC dialog scripting | 05 (gbdialog Reference) | All BASIC keywords (`TALK`, `HEAR`, `LLM`, `FORMAT`, `USE_KB`, `SET_KB`, `ADD_WEBSITE`, …) |
| Custom Rust extensions | 06 (gbapp Reference) | `USE_TOOL`, custom Rust code |
| Bot configuration | 07 (gbot Reference) | `config.csv` fields |
| Built‑in tooling | 08 (Tooling) | All keywords listed in the table |
| Answer modes & routing | 07 (gbot Reference) | `answer_mode` column |
| Semantic search & Qdrant | 03 (gbkb Reference) | `ADD_WEBSITE`, vector search |
| Email & external APIs | 08 (Tooling) | `CALL`, `CALL_ASYNC` |
| Scheduling & events | 08 (Tooling) | `SET_SCHEDULE`, `ON` |
| Testing & CI | 10 (Contributing) | – |
| Database schema | Appendix I | Tables defined in `src/shared/models.rs` |
