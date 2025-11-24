# Theme Structure

The **gbtheme** package is simply CSS files that style the bot's UI. Themes don't include HTML or JavaScript - they only control appearance.

```
theme-name.gbtheme/
├── default.css            # Main theme file (required)
├── dark.css              # Optional dark mode variant
├── print.css             # Optional print styles
└── assets/              # Optional theme resources
    ├── images/
    ├── fonts/
    └── icons/
```

### Design Principles

* **CSS-only theming** – Themes are pure CSS files, no HTML or JavaScript modifications
* **CSS Variables** – Use CSS custom properties for colors, spacing, and other values
* **Responsive design** – Use media queries within your CSS for mobile-first layouts
* **Asset locality** – Optional `assets/` folder for theme-specific images, fonts, and icons

### Creating Your Theme

1. Create a `.gbtheme` folder in your bot package
2. Add a `default.css` file with your styles
3. Override CSS variables to change colors and spacing
4. Add optional assets like fonts or background images

The system automatically picks up any theme placed under `@/templates/…` when the bot's configuration (`.gbtheme` entry in `config.csv`) points to the folder name.

## Theme Loading Process

1. **Discovery**: Bot looks for theme folder in `work/{bot_name}/{bot_name}.gbtheme/`
2. **Validation**: Checks for required files (at least one CSS file)
3. **Registration**: Theme becomes available in theme selector
4. **Activation**: User selects theme or bot loads default
5. **Hot Reload**: Changes apply immediately without restart

## File Organization Best Practices

### CSS File Options

You can have multiple CSS files in your theme:

```
mybot.gbtheme/
├── default.css       # Main theme (loaded automatically)
├── dark.css         # Dark mode variant
├── mobile.css       # Mobile-specific overrides
└── print.css        # Print media styles
```

Or keep everything in a single file - your choice!

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

### Step 1: Create Theme Folder

```bash
mkdir -p work/mybot/mybot.gbtheme
```

### Step 2: Create Your CSS

Create `default.css` with CSS variables:

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

### Step 3: Style Components

Add your component styles in the same file:

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

## Using the Theme

Once you've created your CSS file, the bot will automatically load it. You can switch between themes using BASIC:

```basic
' Switch to a different theme
CHANGE THEME "dark"

' Back to default
CHANGE THEME "default"
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

## Theme Selection

Themes are switched via BASIC commands, not JavaScript. The system handles the CSS file swapping automatically.

## Accessibility Considerations

- Maintain WCAG 2.1 AA contrast ratios
- Support high contrast mode
- Include focus indicators
- Test with screen readers

## Advanced Personalization Options

### Beyond CSS Theming

While themes handle visual styling, you have more options for deeper UI customization:

1. **Extend default.gbui** - The UI templates in `.gbui` packages can be modified:
   - Copy the default UI templates to your bot's `.gbui` folder
   - Modify the HTML structure to fit your needs
   - Add custom components and layouts
   - The system will use your UI instead of the default

2. **Create Your Own UI Type** - Build a completely custom interface:
   - Design your own UI framework
   - Implement custom WebSocket handlers
   - Create unique interaction patterns
   - Full control over the user experience

### Join the Community

**We encourage you to contribute!** The General Bots project welcomes:

- **UI Improvements** - Submit pull requests with better default UIs
- **Theme Collections** - Share your creative themes
- **Custom UI Types** - Develop new interaction paradigms
- **Documentation** - Help improve these guides

### Using General Bots as a Foundation

General Bots is designed to be a starting point for your own projects:

```
Fork the project → Customize the UI → Build your product
```

You can:
- Use it as a base for commercial products
- Create industry-specific bot interfaces
- Develop specialized UI frameworks
- Build on top of the core engine

The architecture is intentionally modular - take what you need, replace what you don't.

### Getting Started with UI Development

1. **Study the default.gbui** - Understand the current structure
2. **Fork the repository** - Create your own version
3. **Experiment freely** - The UI layer is independent
4. **Share your work** - Help others learn from your innovations

Remember: The UI is just HTML/CSS/JS talking to the bot via WebSocket. You have complete freedom to reimagine how users interact with your bot!

## See Also

- [CSS Customization](./css.md) - Detailed CSS guide
- [Chapter 4: User Interface](../chapter-04-gbui/README.md) - UI templates
- [Chapter 6: BASIC](../chapter-06-gbdialog/README.md) - Theme switching in dialogs
- [GitHub Repository](https://github.com/GeneralBots/botserver) - Contribute to the project

## Next Step

Continue to [CSS Customization](./css.md) for detailed styling techniques.
