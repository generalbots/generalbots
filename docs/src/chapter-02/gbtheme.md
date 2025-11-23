# .gbtheme UI Theming

The `.gbtheme` package provides simple CSS-based theming for the bot's UI interface.

## What is .gbtheme?

`.gbtheme` is a simplified theming system that uses CSS files to customize the bot's appearance. No complex HTML templates or JavaScript required - just CSS.

## Theme Structure

A theme is simply one or more CSS files in the `.gbtheme` folder:

```
botname.gbtheme/
  default.css       # Main theme file
  dark.css         # Alternative theme
  holiday.css      # Seasonal theme
```

## Using Themes

### Default Theme

Place a `default.css` file in your `.gbtheme` folder:

```css
/* default.css */
:root {
  --primary-color: #0d2b55;
  --secondary-color: #fff9c2;
  --background: #ffffff;
  --text-color: #333333;
  --font-family: 'Inter', sans-serif;
}

.chat-container {
  background: var(--background);
  color: var(--text-color);
}

.bot-message {
  background: var(--primary-color);
  color: white;
}

.user-message {
  background: var(--secondary-color);
  color: var(--text-color);
}
```

### Changing Themes Dynamically

Use the BASIC keyword to switch themes at runtime:

```basic
' Switch to dark theme
CHANGE THEME "dark"

' Switch back to default
CHANGE THEME "default"

' Seasonal theme
IF month = 12 THEN
  CHANGE THEME "holiday"
END IF
```

## CSS Variables

The bot interface uses CSS custom properties that themes can override:

| Variable | Description | Default |
|----------|-------------|---------|
| `--primary-color` | Main brand color | `#0d2b55` |
| `--secondary-color` | Accent color | `#fff9c2` |
| `--background` | Page background | `#ffffff` |
| `--text-color` | Main text | `#333333` |
| `--font-family` | Typography | `system-ui` |
| `--border-radius` | Element corners | `8px` |
| `--spacing` | Base spacing unit | `16px` |
| `--shadow` | Box shadows | `0 2px 4px rgba(0,0,0,0.1)` |

## Simple Examples

### Minimal Theme

```css
/* minimal.css */
:root {
  --primary-color: #000000;
  --secondary-color: #ffffff;
}
```

### Corporate Theme

```css
/* corporate.css */
:root {
  --primary-color: #1e3a8a;
  --secondary-color: #f59e0b;
  --background: #f8fafc;
  --text-color: #1e293b;
  --font-family: 'Roboto', sans-serif;
  --border-radius: 4px;
}
```

### Dark Theme

```css
/* dark.css */
:root {
  --primary-color: #60a5fa;
  --secondary-color: #34d399;
  --background: #0f172a;
  --text-color: #e2e8f0;
}

body {
  background: var(--background);
  color: var(--text-color);
}
```

## Best Practices

1. **Keep it simple** - Just override CSS variables
2. **Use one file** - Start with a single `default.css`
3. **Test contrast** - Ensure text is readable
4. **Mobile-first** - Design for small screens
5. **Performance** - Keep file size small

## Theme Switching in Scripts

```basic
' User preference
preference = GET USER "theme_preference"
IF preference <> "" THEN
  CHANGE THEME preference
END IF

' Theme selection based on user preferences
' System handles theme switching automatically
```

## Integration with config.csv

You can set the default theme in your bot's configuration:

```csv
name,value
theme,default
theme-color1,#0d2b55
theme-color2,#fff9c2
```

These values are available as CSS variables but the `.css` file takes precedence.

## No Build Process Required

Unlike complex theming systems, `.gbtheme`:
- No webpack or build tools
- No preprocessors needed
- No template engines
- Just plain CSS files
- Hot reload on change

## Migration from Complex Themes

If migrating from a complex theme system:

1. **Extract colors** - Find your brand colors
2. **Create CSS** - Map to CSS variables
3. **Test interface** - Verify appearance
4. **Remove complexity** - Delete unused assets

The bot's default UI handles layout and functionality - themes just customize appearance.