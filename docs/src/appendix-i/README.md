## Appendix I – Database Model

The core database schema for GeneralBots is defined in `src/shared/models.rs`. It uses **Diesel** with SQLite (or PostgreSQL) and includes the following primary tables:

| Table | Description |
|-------|-------------|
| `users` | Stores user accounts, authentication tokens, and profile data. |
| `sessions` | Tracks active `BotSession` instances, their start/end timestamps, and associated user. |
| `knowledge_bases` | Metadata for each `.gbkb` collection (name, vector store configuration, creation date). |
| `messages` | Individual chat messages (role = user/assistant, content, timestamp, linked to a session). |
| `tools` | Registered custom tools per session (name, definition JSON, activation status). |
| `files` | References to files managed by the `.gbdrive` package (path, size, MIME type, storage location). |

### Relationships
- **User ↔ Sessions** – One‑to‑many: a user can have many sessions.
- **Session ↔ Messages** – One‑to‑many: each session contains a sequence of messages.
- **Session ↔ KnowledgeBase** – Many‑to‑one: a session uses a single knowledge base at a time.
- **Session ↔ Tools** – One‑to‑many: tools are scoped to the session that registers them.
- **File ↔ KnowledgeBase** – Optional link for documents stored in a knowledge base.

### Key Fields (excerpt)

```rust
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

pub struct Session {
    pub id: i32,
    pub user_id: i32,
    pub started_at: NaiveDateTime,
    pub last_active: NaiveDateTime,
    pub knowledge_base_id: i32,
}

pub struct Message {
    pub id: i32,
    pub session_id: i32,
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub timestamp: NaiveDateTime,
}
```

The schema is automatically migrated by Diesel when the server starts. For custom extensions, add new tables to `models.rs` and run `diesel migration generate <name>`.
