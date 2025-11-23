# Chapter 04: .gbui User Interface Reference

The `.gbui` (General Bots User Interface) file format provides HTML-based templates for bot interfaces. These files create conversational experiences that work across web, desktop, and mobile platforms.

## Available Templates

General Bots includes two built-in UI templates located in `ui/desktop/`:

### 1. default.gbui
Full-featured interface with multiple applications:
- Chat, Drive, Tasks, and Mail integration
- Theme selector with dark mode
- Keyboard shortcuts (Alt+1 through Alt+4)
- Responsive design for all screen sizes
- WebSocket real-time communication

### 2. single.gbui  
Minimalist chat-only interface:
- Clean, focused conversation view
- Fast loading (< 50KB total)
- Auto dark mode support
- Perfect for embedding or kiosks
- Mobile-optimized touch targets

## How .gbui Files Work

Each .gbui file is a complete HTML document with:
- **Structure**: Standard HTML5 markup
- **Styling**: Embedded CSS or theme integration  
- **Behavior**: JavaScript for WebSocket communication
- **Responsiveness**: Media queries for different screens

The bot server automatically serves the appropriate .gbui file based on:
- User device (desktop/mobile detection)
- Configuration in config.csv
- URL parameters (e.g., `?ui=single`)

## Choosing a Template

### Use default.gbui when:
- Building a full workspace application
- Users need file management (Drive)
- Task tracking is required
- Email integration is needed
- Desktop is the primary platform

### Use single.gbui when:
- Creating a simple chatbot
- Embedding in existing websites
- Optimizing for mobile devices
- Building kiosk interfaces
- Minimizing load time is critical

## Creating Custom Templates

To create your own .gbui file:

1. Copy an existing template as a starting point
2. Modify the HTML structure
3. Adjust CSS styles  
4. Update JavaScript behavior
5. Save to `ui/desktop/custom.gbui`

The bot automatically detects new .gbui files on restart.

## Other UI Files

Besides .gbui templates, the ui folder contains:

- `index.html` - Original HTML interface (kept for compatibility)
- `account.html` - User account settings page
- `settings.html` - Application settings page
- Supporting folders: `/css`, `/js`, `/chat`, `/drive`, `/tasks`, `/mail`

## Console Mode

For terminal users, General Bots also provides a console interface (not a .gbui file):
- Text-based UI using terminal capabilities
- Keyboard-driven navigation
- Works over SSH connections
- See [Console Mode](./console-mode.md) for details

## See Also

### Documentation
- [default.gbui](./default-gbui.md) - Full desktop interface details
- [single.gbui](./single-gbui.md) - Minimalist chat interface  
- [Console Mode](./console-mode.md) - Terminal-based interface
- [Chapter 5: CSS Theming](../chapter-05-gbtheme/README.md) - Style your interfaces
- [Chapter 6: BASIC Dialogs](../chapter-06-gbdialog/README.md) - Connect conversations to UI

### Further Reading - Blog Posts
- [No Forms](https://pragmatismo.cloud/blog/no-forms) - Why conversational UI is the future
- [Beyond Chatbots](https://pragmatismo.cloud/blog/beyond-chatbots) - Rich interaction patterns

### Next Chapter
Continue to [Chapter 5: CSS Theming](../chapter-05-gbtheme/README.md) to learn how to style your .gbui interfaces.