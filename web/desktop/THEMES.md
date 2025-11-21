# General Bots Desktop - Theme System Documentation

## Overview

The General Bots Desktop application uses a modern, flexible theme system based on CSS custom properties (CSS variables) with HSL color format. This system allows for easy customization and dynamic theme switching without page reloads.

## Architecture

### 1. **HSL-Based Theme Variables**

All themes use HSL (Hue, Saturation, Lightness) format for colors, which provides several advantages:
- Easy color manipulation
- Better accessibility control
- Smooth color transitions
- Alpha transparency support

### 2. **Two-Layer System**

The theme system uses a two-layer approach:

#### Layer 1: Base Theme Variables (HSL Format)
Defined in theme files (`public/themes/*.css`) or `css/app.css`:

```css
:root {
  --background: 0 0% 100%;           /* HSL: white */
  --foreground: 222 47% 11%;         /* HSL: dark gray */
  --primary: 217 91% 60%;            /* HSL: blue */
  --card: 0 0% 98%;                  /* HSL: light gray */
  /* ... more variables */
}
```

#### Layer 2: Working CSS Variables
Automatically derived in `css/app.css`:

```css
:root {
  /* Convert HSL to usable CSS colors */
  --primary-bg: hsl(var(--background));
  --primary-fg: hsl(var(--foreground));
  --accent-color: hsl(var(--primary));
  
  /* With alpha transparency */
  --glass-bg: hsla(var(--background) / 0.7);
  --accent-light: hsla(var(--primary) / 0.1);
}
```

## Available Theme Variables

### Core Theme Variables (HSL Format)

These are the base variables that themes should override:

| Variable | Purpose | Example Value |
|----------|---------|---------------|
| `--background` | Main background color | `0 0% 100%` |
| `--foreground` | Main text color | `222 47% 11%` |
| `--card` | Card background | `0 0% 98%` |
| `--card-foreground` | Card text | `222 47% 11%` |
| `--popover` | Popup background | `0 0% 100%` |
| `--popover-foreground` | Popup text | `222 47% 11%` |
| `--primary` | Primary accent color | `217 91% 60%` |
| `--primary-foreground` | Text on primary | `0 0% 100%` |
| `--secondary` | Secondary elements | `214 32% 91%` |
| `--secondary-foreground` | Text on secondary | `222 47% 11%` |
| `--muted` | Muted backgrounds | `214 32% 91%` |
| `--muted-foreground` | Muted text | `215 16% 47%` |
| `--accent` | Accent elements | `214 32% 91%` |
| `--accent-foreground` | Text on accent | `222 47% 11%` |
| `--destructive` | Error/danger color | `0 84% 60%` |
| `--destructive-foreground` | Text on destructive | `0 0% 98%` |
| `--border` | Border color | `214 32% 91%` |
| `--input` | Input background | `214 32% 91%` |
| `--ring` | Focus ring color | `217 91% 60%` |
| `--radius` | Border radius | `0.5rem` |

### Chart Colors

For data visualization:

| Variable | Purpose |
|----------|---------|
| `--chart-1` | Primary chart color |
| `--chart-2` | Secondary chart color |
| `--chart-3` | Tertiary chart color |
| `--chart-4` | Quaternary chart color |
| `--chart-5` | Quinary chart color |

### Working Variables (Derived)

These are automatically calculated and should **not** be overridden in theme files:

- **Layout**: `--primary-bg`, `--primary-fg`, `--secondary-bg`, `--secondary-fg`
- **Glass Effects**: `--glass-bg`, `--glass-border`, `--glass-shadow`
- **Text**: `--text-primary`, `--text-secondary`, `--text-tertiary`, `--text-muted`
- **Interactive**: `--accent-color`, `--accent-hover`, `--accent-light`, `--accent-gradient`
- **Borders**: `--border-color`, `--border-light`, `--border-dark`
- **States**: `--bg-hover`, `--bg-active`, `--bg-disabled`
- **Components**: `--user-message-bg`, `--bot-message-bg`, `--sidebar-bg`, etc.
- **Status**: `--success-color`, `--warning-color`, `--error-color`, `--info-color`
- **Shadows**: `--shadow-sm`, `--shadow-md`, `--shadow-lg`, `--shadow-xl`

## Creating a New Theme

### Step 1: Create Theme File

Create a new CSS file in `public/themes/yourtheme.css`:

```css
:root {
  /* Theme Name: Your Theme */
  
  /* Base colors */
  --background: 240 10% 10%;           /* Dark blue-gray */
  --foreground: 0 0% 95%;              /* Light text */
  
  /* Card colors */
  --card: 240 10% 15%;
  --card-foreground: 0 0% 95%;
  
  /* Popup colors */
  --popover: 240 10% 10%;
  --popover-foreground: 0 0% 95%;
  
  /* Primary accent (main brand color) */
  --primary: 280 80% 60%;              /* Purple */
  --primary-foreground: 0 0% 100%;
  
  /* Secondary elements */
  --secondary: 240 10% 20%;
  --secondary-foreground: 0 0% 95%;
  
  /* Muted/subtle elements */
  --muted: 240 10% 25%;
  --muted-foreground: 240 5% 60%;
  
  /* Accent highlights */
  --accent: 320 80% 60%;               /* Pink accent */
  --accent-foreground: 0 0% 100%;
  
  /* Destructive/error states */
  --destructive: 0 85% 60%;
  --destructive-foreground: 0 0% 98%;
  
  /* Borders and inputs */
  --border: 240 10% 20%;
  --input: 240 10% 20%;
  --ring: 280 80% 60%;                 /* Focus ring matches primary */
  
  /* Border radius */
  --radius: 0.5rem;
  
  /* Chart colors */
  --chart-1: 280 80% 60%;              /* Purple */
  --chart-2: 320 80% 60%;              /* Pink */
  --chart-3: 200 80% 60%;              /* Cyan */
  --chart-4: 140 80% 60%;              /* Green */
  --chart-5: 40 80% 60%;               /* Orange */
}
```

### Step 2: Register Theme

Add your theme to `js/theme-manager.js`:

```javascript
const themes = [
  // ... existing themes
  { id: "yourtheme", name: "🎨 Your Theme", file: "yourtheme.css" }
];
```

### Step 3: Test Your Theme

1. Reload the application
2. Open the theme dropdown in the header
3. Select your theme from the list
4. Verify all UI elements look correct

## Theme Best Practices

### 1. **Contrast Ratios**

Ensure sufficient contrast between text and backgrounds:
- Normal text: Minimum 4.5:1 contrast ratio
- Large text: Minimum 3:1 contrast ratio
- Interactive elements: Minimum 3:1 contrast ratio

### 2. **Color Harmony**

Use complementary or analogous colors:
- **Monochromatic**: Different shades of the same hue
- **Analogous**: Colors next to each other on the color wheel
- **Complementary**: Colors opposite on the color wheel
- **Triadic**: Three colors evenly spaced on the color wheel

### 3. **Accessibility**

- Test with screen readers
- Ensure keyboard navigation works
- Provide sufficient color contrast
- Don't rely on color alone to convey information

### 4. **HSL Values**

Format: `H S% L%` (without commas, without `hsl()`)
- **Hue (H)**: 0-360 (color wheel position)
- **Saturation (S)**: 0-100% (color intensity)
- **Lightness (L)**: 0-100% (brightness)

Examples:
```css
--primary: 217 91% 60%;    /* Bright blue */
--primary: 217 50% 60%;    /* Desaturated blue */
--primary: 217 91% 30%;    /* Dark blue */
```

### 5. **Consistency**

Maintain consistent:
- Saturation levels across related colors
- Lightness values for similar elements
- Hue relationships (complementary, analogous, etc.)

## Using Themes in Components

### Basic Usage

```css
.my-component {
  background: var(--primary-bg);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
}
```

### With Transparency

```css
.my-glass-component {
  background: var(--glass-bg);
  backdrop-filter: blur(10px);
  border: 1px solid var(--glass-border);
}
```

### State Changes

```css
.my-button {
  background: var(--accent-color);
  color: hsl(var(--primary-foreground));
}

.my-button:hover {
  background: var(--accent-hover);
}

.my-button:active {
  background: var(--bg-active);
}
```

### Custom Transparency

```css
.my-overlay {
  background: hsla(var(--background) / 0.9);
  color: hsl(var(--foreground));
}

.my-highlight {
  background: hsla(var(--primary) / 0.15);
}
```

## Built-in Themes

The application includes these pre-built themes:

1. **Default** - Clean, modern light theme
2. **Orange** - Office-inspired orange theme
3. **Cyberpunk** - Neon cyberpunk aesthetic
4. **Retrowave** - 80s synthwave vibes
5. **Vapor Dream** - Vaporwave aesthetic
6. **Y2K Glow** - Y2K-era design
7. **3D Bevel** - Classic 3D beveled look
8. **Arcade Flash** - Retro arcade style
9. **Disco Fever** - 70s disco aesthetic
10. **Grunge Era** - 90s grunge style
11. **Jazz Age** - Art deco inspired
12. **Mellow Gold** - Warm, mellow tones
13. **Mid Century Modern** - 50s/60s design
14. **Polaroid Memories** - Vintage photo aesthetic
15. **Saturday Cartoons** - Bright, playful colors
16. **Seaside Postcard** - Beach-inspired palette
17. **Typewriter** - Classic typewriter look
18. **Xerox UI** - Office copier aesthetic
19. **XTree Gold** - DOS file manager tribute

## Dark Mode Support

The system automatically detects system dark mode preference:

```css
@media (prefers-color-scheme: dark) {
  :root:not([data-theme]) {
    /* Automatically applied dark theme variables */
    --background: 222 47% 11%;
    --foreground: 213 31% 91%;
    /* ... */
  }
}
```

Themes can override this by setting their own values.

## Troubleshooting

### Theme Not Loading

1. Check browser console for errors
2. Verify file path in `js/theme-manager.js`
3. Ensure CSS file is in `public/themes/` directory
4. Check CSS syntax (no `hsl()` wrapper needed)

### Colors Look Wrong

1. Verify HSL format: `H S% L%` (spaces, not commas)
2. Check contrast ratios
3. Test with different system preferences (light/dark mode)
4. Clear browser cache

### Theme Not Persisting

Themes are saved to `localStorage` with key `gb-theme`. Check:
1. localStorage is enabled
2. No browser extensions blocking storage
3. Clear localStorage and try again

## API Reference

### ThemeManager

```javascript
// Initialize theme system
ThemeManager.init();

// Load specific theme
ThemeManager.loadTheme('cyberpunk');

// Subscribe to theme changes
ThemeManager.subscribe((data) => {
  console.log(`Theme changed to: ${data.themeName}`);
});

// Get available themes
const themes = ThemeManager.getAvailableThemes();
```

## Performance Considerations

- CSS custom properties update instantly
- No page reload required
- Themes are cached after first load
- Minimal performance impact
- Works with backdrop-filter and other modern CSS

## Browser Support

- Chrome/Edge 88+
- Firefox 89+
- Safari 14+
- All modern browsers with CSS custom property support

## Contributing

To contribute a new theme:

1. Create theme file following the structure above
2. Test thoroughly across all sections (Chat, Drive, Tasks, Mail)
3. Ensure accessibility standards are met
4. Add to theme registry in `theme-manager.js`
5. Document any special features or requirements

## Resources

- [HSL Color Picker](https://hslpicker.com/)
- [Coolors Palette Generator](https://coolors.co/)
- [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)
- [MDN CSS Custom Properties](https://developer.mozilla.org/en-US/docs/Web/CSS/--*)

---

**Version:** 1.0  
**Last Updated:** 2024  
**Maintained by:** General Bots Team