# Console Mode

The BotServer console mode provides a text-based interface for monitoring your bot's operation directly in the terminal.

## Starting Console Mode

```bash
# Start BotServer with console UI
./botserver --console
```

## Console Interface

The console displays real-time information about your running BotServer instance:

```
╔════════════════════════════════════════════════════════════╗
║                    BotServer Console                       ║
╠════════════════════════════════════════════════════════════╣
║ Status: Running                                            ║
║ Uptime: 2h 34m 12s                                         ║
║ Port: 8080                                                 ║
║                                                            ║
║ Components:                                                ║
║   PostgreSQL: ✓ Connected                                  ║
║   Valkey:     ✓ Connected                                  ║
║   Storage:    ✓ Connected                                  ║
║   Vectors:    ✓ Connected                                  ║
║                                                            ║
║ Active Sessions: 12                                        ║
║ Messages Today: 1,234                                      ║
║                                                            ║
║ Press 'q' to quit, 'r' to refresh                          ║
╚════════════════════════════════════════════════════════════╝
```

## Console Features

### Status Overview
- Server status (Running/Stopped)
- Uptime counter
- Port information
- Component health checks

### Session Information
- Count of active sessions
- Daily message count
- Recent activity indicators

### Component Status
Real-time status of all components:
- Database connectivity
- Cache status
- Storage availability
- Vector database status

## Keyboard Controls

| Key | Action |
|-----|--------|
| `q` | Quit console mode |
| `r` | Force refresh display |
| `c` | Clear console |
| `h` | Show help |

## Console Output

The console provides basic logging output:

```
[2024-01-15 10:23:45] Server started on port 8080
[2024-01-15 10:23:46] Database connected
[2024-01-15 10:23:47] Cache initialized
[2024-01-15 10:23:48] Storage mounted
[2024-01-15 10:24:01] New session: abc123
[2024-01-15 10:24:15] Message processed
```

## Using Console Mode

### Development
Console mode is useful during development to monitor:
- Component initialization
- Connection status
- Error messages
- Session activity

### Production
In production, console mode can help with:
- Quick status checks
- Basic monitoring
- Troubleshooting connection issues

## Limitations

Console mode provides basic monitoring only. For detailed analytics:
- Check PostgreSQL directly for session data
- Use system logs for detailed error information
- Monitor Valkey for cache statistics
- Review application logs for debugging

## Terminal Requirements

- Supports any terminal with basic text output
- UTF-8 support recommended for box drawing
- Minimum 80 columns width recommended
- Works over SSH connections

## Tips

- Console mode is read-only - it doesn't accept bot commands
- For interactive bot testing, use the web interface at http://localhost:8080
- Console refreshes automatically every few seconds
- Output is buffered for performance

## Troubleshooting

### Console Not Updating
- Check terminal compatibility
- Ensure proper permissions
- Verify components are running

### Display Issues
- Try a different terminal emulator
- Check terminal encoding (should be UTF-8)
- Resize terminal window if text is cut off

## Summary

Console mode provides a simple, lightweight way to monitor BotServer status without needing a web browser. It's ideal for quick checks and basic monitoring, but for full functionality, use the web interface.
