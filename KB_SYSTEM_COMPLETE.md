# 🧠 Knowledge Base (KB) System - Complete Implementation

## Overview

The KB system allows `.bas` tools to **dynamically add/remove Knowledge Bases to conversation context** using `ADD_KB` and `CLEAR_KB` keywords. Each KB is a vectorized folder that gets queried by the LLM during conversation.

## 🏗️ Architecture

```
work/
  {bot_name}/
    {bot_name}.gbkb/          # Knowledge Base root
      circular/               # KB folder 1
        document1.pdf
        document2.md
        vectorized/           # Auto-generated vector index
      comunicado/             # KB folder 2
        announcement1.txt
        announcement2.pdf
        vectorized/
      geral/                  # KB folder 3
        general1.md
        vectorized/
```

## 📊 Database Tables (Already Exist!)

### From Migration 6.0.2 - `kb_collections`
```sql
kb_collections
  - id (uuid)
  - bot_id (uuid)
  - name (text)                    -- e.g., "circular", "comunicado"
  - folder_path (text)             -- "work/bot/bot.gbkb/circular"
  - qdrant_collection (text)       -- "bot_circular"
  - document_count (integer)
```

### From Migration 6.0.2 - `kb_documents`
```sql
kb_documents
  - id (uuid)
  - bot_id (uuid)
  - collection_name (text)         -- References kb_collections.name
  - file_path (text)
  - file_hash (text)
  - indexed_at (timestamptz)
```

### NEW Migration 6.0.7 - `session_kb_associations`
```sql
session_kb_associations
  - id (uuid)
  - session_id (uuid)              -- Current conversation
  - bot_id (uuid)
  - kb_name (text)                 -- "circular", "comunicado", etc.
  - kb_folder_path (text)          -- Full path to KB
  - qdrant_collection (text)       -- Qdrant collection to query
  - added_at (timestamptz)
  - added_by_tool (text)           -- Which .bas tool added this KB
  - is_active (boolean)            -- true = active in session
```

## 🔧 BASIC Keywords

### `ADD_KB kbname`

**Purpose**: Add a Knowledge Base to the current conversation session

**Usage**:
```bas
' Static KB name
ADD_KB "circular"

' Dynamic KB name from variable
kbname = LLM "Return one word: circular, comunicado, or geral based on: " + subject
ADD_KB kbname

' Multiple KBs in one tool
ADD_KB "circular"
ADD_KB "geral"
```

**What it does**:
1. Checks if KB exists in `kb_collections` table
2. If not found, creates entry with default path
3. Inserts/updates `session_kb_associations` with `is_active = true`
4. Logs which tool added the KB
5. KB is now available for LLM queries in this session

**Example** (from `change-subject.bas`):
```bas
PARAM subject as string
DESCRIPTION "Called when someone wants to change conversation subject."

kbname = LLM "Return one word circular, comunicado or geral based on: " + subject
ADD_KB kbname

TALK "You have chosen to change the subject to " + subject + "."
```

### `CLEAR_KB [kbname]`

**Purpose**: Remove Knowledge Base(s) from current session

**Usage**:
```bas
' Remove specific KB
CLEAR_KB "circular"
CLEAR_KB kbname

' Remove ALL KBs
CLEAR_KB
```

**What it does**:
1. Sets `is_active = false` in `session_kb_associations`
2. KB no longer included in LLM prompt context
3. If no argument, clears ALL active KBs

**Example**:
```bas
' Switch from one KB to another
CLEAR_KB "circular"
ADD_KB "comunicado"

' Start fresh conversation with no context
CLEAR_KB
TALK "Context cleared. What would you like to discuss?"
```

## 🤖 Prompt Engine Integration

### How Bot Uses Active KBs

When building the LLM prompt, the bot:

1. **Gets Active KBs for Session**:
```rust
let active_kbs = get_active_kbs_for_session(&conn_pool, session_id)?;
// Returns: Vec<(kb_name, kb_folder_path, qdrant_collection)>
// Example: [("circular", "work/bot/bot.gbkb/circular", "bot_circular")]
```

2. **Queries Each KB's Vector DB**:
```rust
for (kb_name, _path, qdrant_collection) in active_kbs {
    let results = qdrant_client.search_points(
        qdrant_collection,
        user_query_embedding,
        limit: 5
    ).await?;
    
    // Add results to context
    context_docs.extend(results);
}
```

3. **Builds Enriched Prompt**:
```
System: You are a helpful assistant.

Context from Knowledge Bases:
[KB: circular]
- Document 1: "Circular 2024/01 - New policy regarding..."
- Document 2: "Circular 2024/02 - Update on procedures..."

[KB: geral]
- Document 3: "General information about company..."

User: What's the latest policy update?