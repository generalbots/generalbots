# Chapter 05: gbtheme CSS Theming

The `.gbtheme` package provides CSS-based styling for your bot's user interface. Themes control colors, fonts, layouts, and visual effects across all .gbui templates.

## Quick Start

To theme your bot, create a `.gbtheme` folder in your bot package and add a CSS file:

```
mybot.gbai/
└── mybot.gbtheme/
    └── default.css      # Your theme file
```

The theme loads automatically when the bot starts.

## How Themes Work

Themes are standard CSS files that override the default styles. The system loads them in this order:

1. System default styles
2. Your theme CSS file
3. Any inline style overrides

## Available Theme Files

You can create multiple theme files for different purposes:

- `default.css` - Main theme (loaded automatically)
- `dark.css` - Dark mode variant
- `print.css` - Print-friendly styles
- Custom theme names for special occasions

## Theme Elements to Style

### Core Components
- `.chat-container` - Main chat window
- `.message` - All messages
- `.message-user` - User messages
- `.message-bot` - Bot responses
- `.chat-input` - Input field
- `.send-button` - Send button

### UI Elements
- `.sidebar` - Side navigation
- `.header` - Top bar
- `.footer` - Bottom bar
- `.card` - Content cards
- `.button` - All buttons
- `.dialog` - Modal dialogs

### States and Interactions
- `.typing-indicator` - Typing animation
- `.loading` - Loading states
- `.error` - Error messages
- `.success` - Success messages
- `:hover` - Hover effects
- `:focus` - Focus states

## Switching Themes

Use the `CHANGE THEME` command in your BASIC scripts:

```basic
' Switch to dark theme
CHANGE THEME "dark"

' Back to default
CHANGE THEME "default"
```

## Theme Best Practices

1. **Start Simple**: Begin with colors and fonts, add complexity gradually
2. **Test Responsiveness**: Ensure your theme works on mobile devices
3. **Maintain Readability**: Keep sufficient contrast between text and backgrounds
4. **Use CSS Variables**: Define colors once, reuse throughout
5. **Test Dark Mode**: Many users prefer dark interfaces

## Pre-built Themes

BotServer includes example themes you can use as starting points:

- **default** - Clean, modern interface
- **3dbevel** - Retro Windows 95 style
- **minimal** - Simplified, distraction-free
- **corporate** - Professional business look
- **playful** - Colorful, fun design

## File Structure

Keep your theme files organized:

```
mybot.gbtheme/
├── default.css         # Main theme
├── dark.css           # Dark variant
├── mobile.css         # Mobile-specific styles
└── assets/            # Images, fonts
    ├── logo.png
    └── fonts/
```

## Loading Order

Themes are applied in a specific order to ensure proper cascading:

1. Base system styles (always loaded)
2. Theme file specified in config or `default.css`
3. Media queries for responsive design
4. User preference overrides (if any)

## Dynamic Theme Changes

Themes can be changed at runtime without restarting:

```basic
' Change theme based on time of day
hour = HOUR(NOW())
IF hour >= 18 OR hour < 6 THEN
  CHANGE THEME "dark"
ELSE
  CHANGE THEME "default"
END IF
```

## Troubleshooting

### Theme Not Loading
- Check file is named correctly (e.g., `default.css`)
- Verify `.gbtheme` folder is in the right location
- Restart the bot after adding new theme files

### Styles Not Applying
- Check CSS syntax is valid
- Use browser developer tools to inspect elements
- Verify selectors match the HTML structure
- Clear browser cache if changes aren't visible

### Performance Issues
- Minimize complex animations
- Optimize image sizes
- Avoid too many web fonts
- Use CSS transforms instead of position changes

## Summary

The `.gbtheme` system keeps styling simple and separate from bot logic. Just drop a CSS file in the `.gbtheme` folder and your bot gets a new look. Focus on the essentials - colors, fonts, and spacing - and let the default styles handle the rest.

## See Also

- [Chapter 4: .gbui Interface](../chapter-04-gbui/README.md) - User interface templates
- [Chapter 2: Packages](../chapter-02/README.md) - Package structure
- [Chapter 6: BASIC Dialogs](../chapter-06-gbdialog/README.md) - Using CHANGE THEME command
- [Chapter 8: Configuration](../chapter-08-config/README.md) - Theme configuration options