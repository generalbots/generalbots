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

## Component Styling Guide

### Message Bubbles

Customize chat message appearance:

```css
/* User messages */
.message-user {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 12px 16px;
  border-radius: 18px 18px 4px 18px;
  max-width: 70%;
  margin-left: auto;
}

/* Bot messages */
.message-bot {
  background: #f7fafc;
  color: #2d3748;
  padding: 12px 16px;
  border-radius: 18px 18px 18px 4px;
  max-width: 70%;
  border: 1px solid #e2e8f0;
}

/* Typing indicator */
.typing-indicator {
  display: inline-flex;
  padding: 16px;
  background: #edf2f7;
  border-radius: 18px;
}

.typing-indicator span {
  height: 8px;
  width: 8px;
  background: #718096;
  border-radius: 50%;
  margin: 0 2px;
  animation: typing 1.4s infinite;
}
```

### Input Field

Style the message input area:

```css
.input-container {
  padding: 16px;
  background: white;
  border-top: 1px solid #e2e8f0;
}

.input-wrapper {
  display: flex;
  align-items: center;
  background: #f7fafc;
  border: 2px solid #e2e8f0;
  border-radius: 24px;
  padding: 8px 16px;
  transition: all 0.2s;
}

.input-wrapper:focus-within {
  border-color: var(--primary-color);
  background: white;
  box-shadow: 0 0 0 3px rgba(66, 153, 225, 0.1);
}

.message-input {
  flex: 1;
  border: none;
  background: transparent;
  outline: none;
  font-size: 16px;
}

.send-button {
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 50%;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: transform 0.2s;
}

.send-button:hover {
  transform: scale(1.1);
}

.send-button:active {
  transform: scale(0.95);
}
```

### Buttons

Consistent button styling:

```css
/* Primary button */
.btn-primary {
  background: var(--primary-color);
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 8px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary:hover {
  filter: brightness(110%);
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

/* Secondary button */
.btn-secondary {
  background: transparent;
  color: var(--primary-color);
  border: 2px solid var(--primary-color);
  padding: 8px 18px;
  border-radius: 8px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary:hover {
  background: var(--primary-color);
  color: white;
}

/* Icon button */
.btn-icon {
  background: transparent;
  border: none;
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-icon:hover {
  background: rgba(0, 0, 0, 0.05);
}
```

## Animation Library

### Entrance Animations

```css
@keyframes slideInUp {
  from {
    transform: translateY(20px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes scaleIn {
  from {
    transform: scale(0.95);
    opacity: 0;
  }
  to {
    transform: scale(1);
    opacity: 1;
  }
}

/* Apply animations */
.message {
  animation: slideInUp 0.3s ease-out;
}

.modal {
  animation: scaleIn 0.2s ease-out;
}
```

### Loading States

```css
/* Spinner */
.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid #e2e8f0;
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Skeleton loader */
.skeleton {
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: loading 1.5s infinite;
}

@keyframes loading {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}
```

## Responsive Design Patterns

### Mobile-First Approach

```css
/* Base mobile styles */
.container {
  padding: 16px;
  width: 100%;
}

/* Tablet and up */
@media (min-width: 768px) {
  .container {
    padding: 24px;
    max-width: 768px;
    margin: 0 auto;
  }
}

/* Desktop */
@media (min-width: 1024px) {
  .container {
    padding: 32px;
    max-width: 1024px;
  }
}

/* Wide screens */
@media (min-width: 1440px) {
  .container {
    max-width: 1280px;
  }
}
```

### Touch-Friendly Styles

```css
/* Increase touch targets on mobile */
@media (pointer: coarse) {
  button, a, input, select {
    min-height: 44px;
    min-width: 44px;
  }
  
  .btn-primary, .btn-secondary {
    padding: 12px 24px;
    font-size: 16px;
  }
}

/* Disable hover effects on touch devices */
@media (hover: none) {
  .btn-primary:hover {
    filter: none;
    box-shadow: none;
  }
}
```

## Theme Variants

### Dark Mode

```css
@media (prefers-color-scheme: dark) {
  :root {
    --primary-color: #60a5fa;
    --secondary-color: #94a3b8;
    --background-color: #0f172a;
    --text-color: #f1f5f9;
    --border-color: #334155;
  }
  
  .message-bot {
    background: #1e293b;
    color: #f1f5f9;
    border-color: #334155;
  }
  
  .input-wrapper {
    background: #1e293b;
    border-color: #334155;
  }
}
```

### High Contrast

```css
@media (prefers-contrast: high) {
  :root {
    --primary-color: #0066cc;
    --text-color: #000000;
    --background-color: #ffffff;
  }
  
  * {
    border-width: 2px !important;
  }
  
  button:focus, input:focus {
    outline: 3px solid #000000 !important;
    outline-offset: 2px !important;
  }
}
```

## Performance Tips

1. **Use CSS Variables**: Change themes by updating variables, not entire stylesheets
2. **Minimize Specificity**: Keep selectors simple for faster parsing
3. **Avoid Deep Nesting**: Maximum 3 levels deep
4. **Use Transform/Opacity**: For animations instead of layout properties
5. **Lazy Load Non-Critical CSS**: Load theme variations on demand

## Browser Compatibility

```css
/* Provide fallbacks for older browsers */
.gradient-bg {
  background: #3b82f6; /* Fallback */
  background: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%);
}

/* Use @supports for progressive enhancement */
@supports (backdrop-filter: blur(10px)) {
  .modal-backdrop {
    backdrop-filter: blur(10px);
  }
}
```

## See Also

- [Theme Structure](./structure.md) - File organization
- [Chapter 4: User Interface](../chapter-04-gbui/README.md) - Applying themes to templates
- [Chapter 6: BASIC](../chapter-06-gbdialog/README.md) - Dynamic theme switching

## Next Step

Return to [Chapter 5 Overview](./README.md) or continue to [Chapter 6: BASIC Dialogs](../chapter-06-gbdialog/README.md).
