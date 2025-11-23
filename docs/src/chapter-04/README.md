# Chapter 04: UI Customization

BotServer provides basic UI customization through configuration parameters in the `config.csv` file. While there's no dedicated `.gbtheme` package type, you can customize the appearance of the web interface using theme parameters.

## Overview

The web interface theming system allows you to customize:
- Brand colors (primary and secondary)
- Logo image
- Application title
- Logo text

These customizations are applied dynamically to the web interface and broadcast to all connected clients in real-time.

## Theme Configuration

Theme settings are configured in your bot's `config.csv` file located in the `.gbot` directory:

```
templates/your-bot.gbai/your-bot.gbot/config.csv
```

### Available Theme Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `theme-color1` | Primary theme color | `#0d2b55` |
| `theme-color2` | Secondary theme color | `#fff9c2` |
| `theme-logo` | Logo image URL | `https://example.com/logo.svg` |
| `theme-title` | Browser tab title | `My Custom Bot` |
| `theme-logo-text` | Text displayed with logo | `Company Name` |
| `Theme Color` | Simple color name (alternative) | `green`, `purple`, `indigo` |

## Implementation Details

### How Theme Changes Work

1. **Configuration Loading**: When a bot loads, it reads theme parameters from `config.csv`
2. **Drive Monitoring**: The `DriveMonitor` watches for changes to the configuration file
3. **Broadcasting**: Theme changes are broadcast to all connected web clients via WebSocket
4. **Dynamic Application**: The web interface applies theme changes without requiring a page refresh

### Theme Event Structure

When theme parameters change, the system broadcasts a `change_theme` event:

```json
{
  "event": "change_theme",
  "data": {
    "color1": "#0d2b55",
    "color2": "#fff9c2",
    "logo_url": "https://example.com/logo.svg",
    "title": "Custom Title",
    "logo_text": "Company Name"
  }
}
```

## Web Interface Structure

The BotServer web interface consists of:

### Main Directories
- `web/desktop/` - Desktop web application
  - `chat/` - Chat interface components
  - `css/` - Stylesheets
  - `js/` - JavaScript files
  - `public/` - Static assets
- `web/html/` - Simplified HTML interface

### Key Files
- `index.html` - Main application entry point
- `chat/chat.js` - Chat interface logic with theme handling
- `account.html` - User account management
- `settings.html` - Bot settings interface

## Customization Examples

### Example 1: Corporate Branding

```csv
name,value
theme-color1,#003366
theme-color2,#FFD700
theme-logo,https://company.com/logo.png
theme-title,Corporate Assistant
theme-logo-text,ACME Corp
```

### Example 2: Simple Color Theme

```csv
name,value
Theme Color,teal
```

### Example 3: Educational Institution

```csv
name,value
theme-color1,#1e3a5f
theme-color2,#f0f0f0
theme-logo,https://university.edu/seal.svg
theme-title,Campus Assistant
theme-logo-text,State University
```

## Dark Mode Support

The web interface includes built-in dark mode support with CSS data attributes:

- Light mode: `[data-theme="light"]`
- Dark mode: `[data-theme="dark"]`

The interface automatically adjusts colors, backgrounds, and contrast based on the user's theme preference.

## Limitations

Current theming capabilities are limited to:
- Color customization (2 colors)
- Logo replacement
- Title and text changes

Advanced customization like:
- Custom CSS injection
- Layout modifications
- Component replacement
- Font changes

...are not currently supported through configuration. For these changes, you would need to modify the web interface source files directly.

## Best Practices

1. **Use Web-Safe Colors**: Ensure your color choices have sufficient contrast for accessibility
2. **Logo Format**: Use SVG for logos when possible for better scaling
3. **Logo Hosting**: Host logos on reliable CDNs or your own servers
4. **Title Length**: Keep titles concise to avoid truncation in browser tabs
5. **Test Changes**: Verify theme changes work across different browsers and devices

## Real-Time Updates

One of the key features is real-time theme updates. When you modify the `config.csv` file:

1. Save your changes to `config.csv`
2. The system detects the change automatically
3. All connected clients receive the theme update
4. The interface updates without requiring a refresh

This makes it easy to experiment with different themes and see results immediately.

## Next Steps

- See [Theme Structure](./structure.md) for details on how themes are applied
- See [Web Interface](./web-interface.md) for understanding the UI components
- See [CSS Customization](./css.md) for advanced styling options
- See [HTML Templates](./html.md) for modifying the interface structure