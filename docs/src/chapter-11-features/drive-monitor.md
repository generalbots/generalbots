# Drive Monitor

The Drive Monitor is a real-time file synchronization system that watches for changes in bot storage buckets and automatically updates the database and runtime configuration.

## Overview

DriveMonitor provides hot-reloading capabilities for bot configurations by continuously monitoring file changes in object storage. When files are modified, added, or removed, the system automatically detects changes through ETags and file comparison, updates the database with new configurations, recompiles scripts and tools, refreshes knowledge bases, and broadcasts theme changes to connected clients.

## Architecture

```
┌─────────────────┐
│  Object Storage │ (S3-compatible)
│     Buckets     │
└────────┬────────┘
         │ Poll every 30s
         ▼
┌─────────────────┐
│  Drive Monitor  │
│   - Check ETags │
│   - Diff files  │
└────────┬────────┘
         │ Changes detected
         ▼
┌─────────────────────────┐
│   Process Updates       │
│ - Compile scripts (.bas)│
│ - Update KB (.gbkb)     │
│ - Refresh themes        │
│ - Update database       │
└─────────────────────────┘
```

## Implementation

### Core Components

The DriveMonitor is implemented in `src/drive/drive_monitor/mod.rs` with the following structure:

```rust
pub struct DriveMonitor {
    state: Arc<AppState>,
    bucket_name: String,
    file_states: Arc<RwLock<HashMap<String, FileState>>>,
    bot_id: Uuid,
    kb_manager: Arc<KnowledgeBaseManager>,
    work_root: PathBuf,
    is_processing: Arc<AtomicBool>,
}
```

### Monitoring Process

The monitoring process begins with initialization when a bot is mounted, at which point a DriveMonitor instance is created and spawned. Every 30 seconds, the monitor polls for changes in `.gbdialog` files containing scripts and tools, `.gbkb` collections containing knowledge base documents, `.gbtheme` files for UI themes, and `.gbot/config.csv` for bot configuration.

Change detection uses ETags to efficiently identify file modifications without downloading entire files. When changes are detected, different file types trigger specific handlers. Scripts are compiled to AST, knowledge base files are indexed and embedded, themes are broadcast to WebSocket clients, and config changes trigger bot settings reload.

### File Type Handlers

#### Script Files (.bas)

The script handler compiles BASIC scripts to AST format for efficient execution. It stores the compiled version in the database for persistence and updates the tool registry if the script defines callable tools.

#### Knowledge Base Files (.gbkb)

The knowledge base handler downloads new and modified documents from storage. It processes text extraction to prepare content for indexing, generates embeddings using the configured embedding model, and updates the vector database for semantic search functionality.

#### Theme Files (.gbtheme)

The theme handler detects CSS and JavaScript changes in theme packages. It broadcasts updates to all connected WebSocket clients and triggers UI refresh without requiring a full page reload.

## Usage

The DriveMonitor is automatically started when a bot is mounted:

```rust
// In BotOrchestrator::mount_bot
let drive_monitor = Arc::new(DriveMonitor::new(
    state.clone(), 
    bucket_name, 
    bot_id
));
let _handle = drive_monitor.clone().spawn().await;
```

## Configuration

No explicit configuration is needed since the monitor automatically uses the bot's storage bucket name, creates work directories as needed, and manages its own file state cache internally.

## Performance Considerations

The polling interval of 30 seconds balances responsiveness with resource usage to avoid overwhelming the storage backend. Concurrent processing uses atomic flags to prevent overlapping operations that could cause race conditions. The caching system maintains an ETag cache to minimize unnecessary downloads when files haven't changed. Batching ensures that multiple file changes detected in a single poll cycle are processed together efficiently.

## Error Handling

The monitor includes robust error handling that continues operation even if individual file processing fails. Errors are logged for debugging while maintaining overall service availability. Isolated error boundaries prevent cascading failures that could take down the entire monitoring system.

## Monitoring and Debugging

Enable debug logging to see monitor activity:

```bash
RUST_LOG=botserver::drive::drive_monitor=debug cargo run
```

Log output includes change detection events showing which files were modified, file processing status as each file is handled, compilation results for script files, and database update confirmations when changes are persisted.

## Best Practices

Keep related files organized in their appropriate directories such as `.gbdialog` for scripts, `.gbkb` for knowledge base content, and `.gbtheme` for UI customizations. The monitor tracks changes but doesn't maintain history, so use git or another version control system to track file revisions. For knowledge base documents larger than 10MB, consider splitting them into smaller files for better processing performance. During active development, the 30-second polling delay can be avoided by restarting the bot to force immediate reprocessing.

## Limitations

The system is not truly real-time due to the 30-second polling interval, meaning changes aren't reflected instantly. There is no conflict resolution mechanism, so concurrent modifications follow a last-write-wins policy. Memory usage for the file state cache is minimal since only ETags are stored rather than full file contents.

## Future Enhancements

Planned improvements include WebSocket notifications from the storage layer for instant updates without polling, configurable polling intervals per file type to allow more frequent checks for critical files, differential sync for large knowledge bases to reduce processing time, and multi-version support for A/B testing different bot configurations.