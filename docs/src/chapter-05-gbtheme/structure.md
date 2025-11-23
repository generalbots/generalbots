# Theme Structure

The **gbtheme** package follows a conventional layout that separates concerns between markup, styling, scripts, and assets.

```
theme-name.gbtheme/
├── web/
│   ├── index.html          # Main application shell
│   ├── chat.html          # Conversation UI
│   └── login.html         # Authentication page
├── css/
│   ├── main.css           # Core styles
│   ├── components.css     # UI component styling
│   └── responsive.css     # Media‑query breakpoints
├── js/
│   ├── app.js            # Front‑end logic, WebSocket handling
│   └── websocket.js      # Real‑time communication layer
└── assets/
    ├── images/
    ├── fonts/
    └── icons/
```

### Design Principles

* **Separation of concerns** – HTML defines structure, CSS defines appearance, JS defines behavior.
* **Custom properties** – `css/variables.css` (included in `main.css`) provides theme colors, spacing, and radius that can be overridden per‑bot.
* **Responsive** – `responsive.css` uses mobile‑first breakpoints (`@media (min-width: 768px)`) to adapt the layout.
* **Asset locality** – All images, fonts, and icons are stored under `assets/` to keep the theme self‑contained and portable.

### Extending a Theme

1. Duplicate an existing theme folder (e.g., `default.gbtheme` → `mybrand.gbtheme`).
2. Edit `css/main.css` to change colors via the `:root` variables.
3. Replace `web/index.html` header/footer with brand‑specific markup.
4. Add new icons to `assets/icons/` and reference them in the HTML.

The system automatically picks up any theme placed under `@/templates/…` when the bot's configuration (`.gbtheme` entry in `config.csv`) points to the folder name.

## Theme Loading Process

1. **Discovery**: Bot looks for theme folder in `work/{bot_name}/{bot_name}.gbtheme/`
2. **Validation**: Checks for required files (at least one CSS file)
3. **Registration**: Theme becomes available in theme selector
4. **Activation**: User selects theme or bot loads default
5. **Hot Reload**: Changes apply immediately without restart

## File Organization Best Practices

### CSS Organization

```
css/
├── variables.css      # Theme variables and colors
├── reset.css         # Browser normalization
├── base.css          # Typography and base styles
├── layout.css        # Grid and structure
├── components/       # Component-specific styles
│   ├── buttons.css
│   ├── messages.css
│   ├── inputs.css
│   └── cards.css
└── utilities.css     # Helper classes
```

### Asset Management

```
assets/
├── images/
│   ├── logo.svg      # Vector graphics preferred
│   ├── bg.webp       # Modern formats for performance
│   └── icons/        # Icon set
├── fonts/
│   └── custom.woff2  # Web fonts if needed
└── data/
    └── theme.json    # Theme metadata
```

## Creating a Custom Theme

### Step 1: Copy Base Theme

```bash
cp -r work/default/default.gbtheme work/mybot/mybot.gbtheme
```

### Step 2: Customize Variables

Edit `css/variables.css`:

```css
:root {
  /* Brand Colors */
  --brand-primary: #your-color;
  --brand-secondary: #your-color;
  
  /* Semantic Colors */
  --color-success: #10b981;
  --color-warning: #f59e0b;
  --color-error: #ef4444;
  
  /* Typography */
  --font-family: 'Inter', system-ui, sans-serif;
  --font-size-base: 16px;
  --line-height: 1.5;
  
  /* Spacing Scale */
  --space-xs: 0.25rem;
  --space-sm: 0.5rem;
  --space-md: 1rem;
  --space-lg: 2rem;
  --space-xl: 4rem;
}
```

### Step 3: Apply Brand Styles

Override components in `css/components.css`:

```css
/* Custom message bubbles */
.message-user {
  background: var(--brand-primary);
  color: white;
  border-radius: 18px 18px 4px 18px;
}

.message-bot {
  background: #f3f4f6;
  border: 1px solid #e5e7eb;
  border-radius: 18px 18px 18px 4px;
}
```

## Theme Inheritance

Themes can extend other themes:

```css
/* In mybot.gbtheme/css/main.css */
@import url('../../../default.gbtheme/css/main.css');

/* Override specific variables */
:root {
  --primary-color: #ff6b6b;
}
```

## Performance Optimization

### CSS Loading Strategy

1. **Critical CSS**: Inline essential styles in HTML
2. **Async Loading**: Load non-critical CSS asynchronously
3. **Minification**: Minify CSS for production
4. **Purging**: Remove unused CSS rules

### Asset Optimization

- Use SVG for logos and icons
- Implement lazy loading for images
- Serve WebP with fallbacks
- Enable gzip compression

## Theme Switching

Dynamic theme switching without page reload:

```javascript
// Theme manager automatically handles this
ThemeManager.switchTheme('dark');
```

## Accessibility Considerations

- Maintain WCAG 2.1 AA contrast ratios
- Support high contrast mode
- Include focus indicators
- Test with screen readers

## See Also

- [CSS Customization](./css.md) - Detailed CSS guide
- [Chapter 4: User Interface](../chapter-04-gbui/README.md) - UI templates
- [Chapter 6: BASIC](../chapter-06-gbdialog/README.md) - Theme switching in dialogs

## Next Step

Continue to [CSS Customization](./css.md) for detailed styling techniques.
