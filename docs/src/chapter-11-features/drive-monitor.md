# Drive Monitor

The Drive Monitor is a real-time file synchronization system that watches for changes in bot storage buckets and automatically updates the database and runtime configuration.

## Overview

DriveMonitor provides hot-reloading capabilities for bot configurations by continuously monitoring file changes in object storage. When files are modified, added, or removed, the system automatically:

- Detects changes through ETags and file comparison
- Updates the database with new configurations  
- Recompiles scripts and tools
- Refreshes knowledge bases
- Broadcasts theme changes to connected clients

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

1. **Initialization**: When a bot is mounted, a DriveMonitor instance is created and spawned
2. **Polling**: Every 30 seconds, the monitor checks for changes in:
   - `.gbdialog` files (scripts and tools)
   - `.gbkb` collections (knowledge base documents)
   - `.gbtheme` files (UI themes)
   - `.gbot/config.csv` (bot configuration)

3. **Change Detection**: Uses ETags to detect file modifications efficiently
4. **Processing**: Different file types trigger specific handlers:
   - Scripts → Compile to AST
   - Knowledge base → Index and embed documents
   - Themes → Broadcast updates to WebSocket clients
   - Config → Reload bot settings

### File Type Handlers

#### Script Files (.bas)
- Compiles BASIC scripts to AST
- Stores compiled version in database
- Updates tool registry if applicable

#### Knowledge Base Files (.gbkb)
- Downloads new/modified documents
- Processes text extraction
- Generates embeddings
- Updates vector database

#### Theme Files (.gbtheme)
- Detects CSS/JS changes
- Broadcasts updates to connected clients
- Triggers UI refresh without page reload

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

No explicit configuration needed - the monitor automatically:
- Uses the bot's storage bucket name
- Creates work directories as needed
- Manages its own file state cache

## Performance Considerations

- **Polling Interval**: 30 seconds (balance between responsiveness and resource usage)
- **Concurrent Processing**: Uses atomic flags to prevent overlapping operations
- **Caching**: Maintains ETag cache to minimize unnecessary downloads
- **Batching**: Processes multiple file changes in a single cycle

## Error Handling

The monitor includes robust error handling:
- Continues operation even if individual file processing fails
- Logs errors for debugging while maintaining service availability
- Prevents cascading failures through isolated error boundaries

## Monitoring and Debugging

Enable debug logging to see monitor activity:

```bash
RUST_LOG=botserver::drive::drive_monitor=debug cargo run
```

Log output includes:
- Change detection events
- File processing status
- Compilation results
- Database update confirmations

## Best Practices

1. **File Organization**: Keep related files in appropriate directories (.gbdialog, .gbkb, etc.)
2. **Version Control**: The monitor tracks changes but doesn't maintain history - use git for version control
3. **Large Files**: For knowledge base documents > 10MB, consider splitting into smaller files
4. **Development**: During development, the 30-second delay can be avoided by restarting the bot

## Limitations

- **Not Real-time**: 30-second polling interval means changes aren't instant
- **No Conflict Resolution**: Last-write-wins for concurrent modifications
- **Memory Usage**: Keeps file state in memory (minimal for ETags)

## Future Enhancements

Planned improvements include:
- WebSocket notifications from storage layer for instant updates
- Configurable polling intervals per file type
- Differential sync for large knowledge bases
- Multi-version support for A/B testing