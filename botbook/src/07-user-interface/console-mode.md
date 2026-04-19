# Console Mode

The botserver console mode provides a text-based interface for monitoring your bot's operation directly in the terminal.

## Starting Console Mode

```bash
# Start botserver with console UI
./botserver --console
```

## Console Interface

The console displays real-time information about your running botserver instance:

```
╔════════════════════════════════════════════════════════════╗
║                    botserver Console                       ║
╠════════════════════════════════════════════════════════════╣
║ Status: Running                                            ║
║ Uptime: 2h 34m 12s                                         ║
║ Port: 8080                                                 ║
║                                                            ║
║ Components:                                                ║
║   PostgreSQL: ✓ Connected                                  ║
║   Cache:      ✓ Connected                                  ║
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

The status overview displays the server's current state including whether it is running or stopped, an uptime counter showing how long the server has been active, the port the server is listening on, and health checks for all connected components.

### Session Information

Session information provides visibility into current activity with a count of active sessions, the total number of messages processed today, and recent activity indicators that show when the last interactions occurred.

### Component Status

Real-time status monitoring covers all infrastructure components including database connectivity to PostgreSQL, cache service status, storage availability for file operations, and vector database connection status for semantic search functionality.

## Keyboard Controls

| Key | Action |
|-----|--------|
| `q` | Quit console mode |
| `r` | Force refresh display |
| `c` | Clear console |
| `h` | Show help |

## Console Output

The console provides basic logging output showing timestamped events as they occur:

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

Console mode is particularly useful during development for monitoring component initialization, tracking connection status, observing error messages as they occur, and watching session activity in real time.

### Production

In production environments, console mode helps with quick status checks when you need immediate visibility, basic monitoring of system health, and troubleshooting connection issues without accessing the web interface.

## Limitations

Console mode provides basic monitoring only and is not intended for detailed analytics. For comprehensive data analysis, query PostgreSQL directly for session data. System logs contain detailed error information for debugging. The cache service provides its own statistics interface. Application logs offer the most complete picture for troubleshooting complex issues.

## Terminal Requirements

Console mode supports any terminal with basic text output capabilities. UTF-8 support is recommended to properly render box drawing characters. A minimum width of 80 columns is recommended for optimal display. The console works over SSH connections, making it suitable for remote server monitoring.

## Tips

Console mode operates in read-only fashion and does not accept bot commands. For interactive bot testing, use the web interface available at http://localhost:9000. The display refreshes automatically every few seconds to show current status. Output is buffered for performance to avoid slowing down the server during high activity periods.

## Troubleshooting

### Console Not Updating

If the console stops updating, check terminal compatibility with your emulator, ensure the process has proper permissions to write to the terminal, and verify that all components are running and responsive.

### Display Issues

Display problems can often be resolved by trying a different terminal emulator. Check that your terminal encoding is set to UTF-8 for proper character rendering. If text appears cut off, resize the terminal window to provide adequate width for the display.

## Summary

Console mode provides a simple, lightweight way to monitor botserver status without needing a web browser. It's ideal for quick checks and basic monitoring, but for full functionality including interactive bot testing and detailed analytics, use the web interface.