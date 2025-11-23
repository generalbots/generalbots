# Chapter 04: gbtheme Reference

Themes control how your bot looks in the web interface. A theme is simply a CSS file that changes colors, fonts, and styles.

## Quick Start

1. Create a `.gbtheme` folder in your bot package
2. Add a CSS file (like `default.css` or `3dbevel.css`)
3. The theme loads automatically when the bot starts

## Theme Structure

```
mybot.gbai/
└── mybot.gbtheme/
    ├── default.css      # Main theme
    ├── 3dbevel.css      # Retro Windows 95 style
    └── dark.css         # Dark mode variant
```

## The 3D Bevel Theme

The `3dbevel.css` theme gives your bot a classic Windows 95 look with 3D beveled edges:

```css
/* Everything uses monospace font for that retro feel */
body, .card, .popover, .input, .button, .menu, .dialog {
  font-family: 'IBM Plex Mono', 'Courier New', monospace !important;
  background: #c0c0c0 !important;
  color: #000 !important;
  border-radius: 0 !important;  /* No rounded corners */
  box-shadow: none !important;
}

/* 3D bevel effect on panels */
.card, .popover, .menu, .dialog {
  border: 2px solid #fff !important;           /* Top/left highlight */
  border-bottom: 2px solid #404040 !important; /* Bottom shadow */
  border-right: 2px solid #404040 !important;  /* Right shadow */
  padding: 8px !important;
  background: #e0e0e0 !important;
}

/* Buttons with 3D effect */
.button, button, input[type="button"], input[type="submit"] {
  background: #e0e0e0 !important;
  color: #000 !important;
  border: 2px solid #fff !important;
  border-bottom: 2px solid #404040 !important;
  border-right: 2px solid #404040 !important;
  padding: 4px 12px !important;
  font-weight: bold !important;
}

/* Input fields look recessed */
input, textarea, select {
  background: #fff !important;
  color: #000 !important;
  border: 2px solid #404040 !important;  /* Reversed for inset look */
  border-bottom: 2px solid #fff !important;
  border-right: 2px solid #fff !important;
}

/* Classic scrollbars */
::-webkit-scrollbar {
  width: 16px !important;
  background: #c0c0c0 !important;
}
::-webkit-scrollbar-thumb {
  background: #404040 !important;
  border: 2px solid #fff !important;
  border-bottom: 2px solid #404040 !important;
  border-right: 2px solid #404040 !important;
}

/* Blue hyperlinks like Windows 95 */
a {
  color: #0000aa !important;
  text-decoration: underline !important;
}
```

## How Themes Work

1. **CSS Variables**: Themes use CSS custom properties for colors
2. **Class Targeting**: Style specific bot UI elements
3. **Important Rules**: Override default styles with `!important`
4. **Font Stacks**: Provide fallback fonts for compatibility

## Creating Your Own Theme

Start with this template:

```css
/* Basic color scheme */
:root {
  --primary: #007bff;
  --background: #ffffff;
  --text: #333333;
  --border: #dee2e6;
}

/* Chat container */
.chat-container {
  background: var(--background);
  color: var(--text);
}

/* Messages */
.message-user {
  background: var(--primary);
  color: white;
}

.message-bot {
  background: var(--border);
  color: var(--text);
}

/* Input area */
.chat-input {
  border: 1px solid var(--border);
  background: var(--background);
}
```

## Switching Themes

Use the `CHANGE THEME` keyword in your BASIC scripts:

```basic
' Switch to retro theme
CHANGE THEME "3dbevel"

' Back to default
CHANGE THEME "default"

' Seasonal themes
month = MONTH(NOW())
IF month = 12 THEN
  CHANGE THEME "holiday"
END IF
```

## Common Theme Elements

### Message Bubbles
```css
.message {
  padding: 10px;
  margin: 5px;
  border-radius: 10px;
}
```

### Suggestion Buttons
```css
.suggestion-button {
  background: #f0f0f0;
  border: 1px solid #ccc;
  padding: 8px 16px;
  margin: 4px;
  cursor: pointer;
}
```

### Input Field
```css
.chat-input {
  width: 100%;
  padding: 10px;
  font-size: 16px;
}
```

## Theme Best Practices

1. **Test on Multiple Browsers**: Ensure compatibility
2. **Use Web-Safe Fonts**: Or include font files
3. **High Contrast**: Ensure readability
4. **Mobile Responsive**: Test on different screen sizes
5. **Keep It Simple**: Don't overcomplicate the CSS

## File Naming

- `default.css` - Loaded automatically as main theme
- `dark.css` - Dark mode variant
- `3dbevel.css` - Special theme (Windows 95 style)
- `[name].css` - Any custom theme name

## Loading Order

1. System default styles
2. Theme CSS file
3. Inline style overrides (if any)

The theme system keeps styling separate from bot logic, making it easy to change the look without touching the code.