# CSS Customization

The **gbtheme** CSS files define the visual style of the bot UI. They are split into three layers to make them easy to extend.

## Files

| File | Role |
|------|------|
| `main.css` | Core layout, typography, and global variables. |
| `components.css` | Styles for reusable UI components (buttons, cards, modals). |
| `responsive.css` | Media queries for mobile, tablet, and desktop breakpoints. |

## CSS Variables (in `main.css`)

```css
:root {
  --primary-color: #2563eb;
  --secondary-color: #64748b;
  --background-color: #ffffff;
  --text-color: #1e293b;
  --border-radius: 8px;
  --spacing-unit: 8px;
}
```

Changing a variable updates the entire theme without editing individual rules.

## Extending the Theme

1. **Add a new variable** – Append to `:root` and reference it in any selector.
2. **Override a component** – Duplicate the selector in `components.css` after the original definition; the later rule wins.
3. **Create a dark mode** – Add a `@media (prefers-color-scheme: dark)` block that redefines the variables.

```css
@media (prefers-color-scheme: dark) {
  :root {
    --primary-color: #3b82f6;
    --background-color: #111827;
    --text-color: #f9fafb;
  }
}
```

## Best Practices

* Keep the file size small – avoid large image data URIs; store images in `assets/`.
* Use `rem` units for font sizes; they scale with the root `font-size`.
* Limit the depth of nesting; flat selectors improve performance.

All CSS files are loaded in `index.html` in the order: `main.css`, `components.css`, `responsive.css`.
