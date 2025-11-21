# General Bots Desktop - Component Guide

## 🎨 UI Components Overview

This document provides a comprehensive guide to all UI components used in the General Bots Desktop application, including their structure, styling, and usage examples.

---

## 📐 Layout Components

### 1. Float Header

The main navigation header with glass morphism effect.

**Structure:**
```html
<header class="float-header" role="banner">
  <div class="header-left">
    <!-- Logo and branding -->
  </div>
  <div class="header-right">
    <!-- Theme selector, apps menu, avatar -->
  </div>
</header>
```

**CSS Variables:**
- `--header-bg`: Background color with transparency
- `--header-border`: Border color
- `--header-height`: Height (default: 64px)

**Styling:**
```css
.float-header {
  background: var(--header-bg);
  backdrop-filter: blur(20px) saturate(180%);
  border-bottom: 1px solid var(--header-border);
  box-shadow: var(--shadow-sm);
}
```

---

### 2. Logo Wrapper

Clickable logo with hover effects.

**Structure:**
```html
<button class="logo-wrapper" onclick="window.location.reload()">
  <div class="logo-icon" role="img"></div>
  <span class="logo-text">General Bots</span>
</button>
```

**States:**
- Default: Glass background with subtle shadow
- Hover: Accent border, scale transform
- Active: Maintains hover state

**CSS:**
```css
.logo-wrapper {
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  transition: all var(--transition-fast);
}

.logo-wrapper:hover {
  background: var(--bg-hover);
  border-color: var(--accent-color);
  transform: scale(1.02);
}
```

---

### 3. Main Content Area

Container for dynamically loaded sections.

**Structure:**
```html
<main id="main-content" role="main">
  <div id="section-container">
    <div id="section-chat" class="section">
      <!-- Chat content -->
    </div>
  </div>
</main>
```

**Behavior:**
- Only one section visible at a time
- Sections cached after first load
- Smooth fade transitions between sections

---

## 🎯 Interactive Components

### 4. Icon Button

Circular button with icon (used for apps menu, theme toggle).

**Structure:**
```html
<button class="icon-button" aria-label="Description">
  <svg width="24" height="24" viewBox="0 0 24 24">
    <!-- Icon path -->
  </svg>
</button>
```

**Variants:**
- `.apps-button`: Apps menu trigger
- With hover lift effect
- With focus ring for accessibility

**CSS:**
```css
.icon-button {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-full);
  background: var(--glass-bg);
  border: 1px solid var(--border-color);
}

.icon-button:hover {
  background: var(--bg-hover);
  border-color: var(--accent-color);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}
```

---

### 5. Theme Dropdown

Select element for theme switching.

**Structure:**
```html
<select class="theme-dropdown" id="themeDropdown">
  <option value="default">🎨 Default</option>
  <option value="cyberpunk">🌃 Cyberpunk</option>
  <!-- More themes -->
</select>
```

**Features:**
- Auto-populated by ThemeManager
- Saves selection to localStorage
- Instant theme application
- Custom styled options

**CSS:**
```css
.theme-dropdown {
  padding: 8px 16px;
  background: var(--glass-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  backdrop-filter: blur(10px);
}

.theme-dropdown:focus {
  border-color: var(--accent-color);
  box-shadow: 0 0 0 3px var(--accent-light);
}
```

---

### 6. Apps Dropdown Menu

Popup menu for application switching.

**Structure:**
```html
<nav class="apps-dropdown" id="appsDropdown" role="menu">
  <div class="apps-dropdown-title">Applications</div>
  <div class="app-grid">
    <a class="app-item" href="#chat" role="menuitem">
      <div class="app-icon">💬</div>
      <span>Chat</span>
    </a>
    <!-- More apps -->
  </div>
</nav>
```

**States:**
- Hidden: `opacity: 0`, `pointer-events: none`
- Visible: `.show` class added
- Item hover: Background highlight
- Active item: Accent background and border

**CSS:**
```css
.apps-dropdown {
  position: absolute;
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
  transition: all var(--transition-smooth);
}

.apps-dropdown.show {
  opacity: 1;
  transform: translateY(0) scale(1);
  pointer-events: all;
}

.app-item {
  padding: var(--space-md);
  border-radius: var(--radius-lg);
  border: 1px solid transparent;
}

.app-item:hover {
  background: var(--bg-hover);
  border-color: var(--border-color);
  transform: translateY(-2px);
}

.app-item.active {
  background: var(--accent-light);
  border-color: var(--accent-color);
}
```

---

### 7. User Avatar

User profile button with gradient background.

**Structure:**
```html
<button class="user-avatar" id="userAvatar">
  <span>U</span>
</button>
```

**Features:**
- Gradient background (uses accent colors)
- Scale animation on hover
- Circular shape
- Customizable initial

**CSS:**
```css
.user-avatar {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-full);
  background: var(--accent-gradient);
  color: white;
  font-weight: 700;
}

.user-avatar:hover {
  transform: scale(1.1);
  box-shadow: var(--shadow-md);
}
```

---

## 🎴 Content Components

### 8. Glass Panel

Container with glass morphism effect.

**Usage:**
```html
<div class="glass-panel">
  <!-- Content -->
</div>
```

**CSS:**
```css
.glass-panel {
  background: var(--glass-bg);
  backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
}
```

**Use Cases:**
- Floating cards
- Overlays
- Modals
- Sidebars

---

### 9. Card Component

Standard card container for content.

**Usage:**
```html
<div class="card">
  <h3>Card Title</h3>
  <p>Card content...</p>
</div>
```

**CSS:**
```css
.card {
  background: hsl(var(--card));
  color: hsl(var(--card-foreground));
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: var(--space-lg);
  box-shadow: var(--shadow-sm);
}

.card:hover {
  box-shadow: var(--shadow-md);
  border-color: var(--accent-color);
}
```

---

### 10. Loading Overlay

Full-screen loading indicator.

**Structure:**
```html
<div class="loading-overlay" id="loadingOverlay">
  <div class="loading-spinner"></div>
</div>
```

**States:**
- Visible: Default state on page load
- Hidden: `.hidden` class added after initialization

**CSS:**
```css
.loading-overlay {
  position: fixed;
  inset: 0;
  background: var(--primary-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: var(--z-modal);
}

.loading-overlay.hidden {
  opacity: 0;
  visibility: hidden;
  pointer-events: none;
}

.loading-spinner {
  width: 48px;
  height: 48px;
  border: 4px solid var(--border-color);
  border-top-color: var(--accent-color);
  border-radius: var(--radius-full);
  animation: spin 0.8s linear infinite;
}
```

---

### 11. Connection Status

WebSocket connection indicator.

**Structure:**
```html
<div class="connection-status disconnected">
  Disconnected
</div>
```

**States:**
- `.disconnected`: Red, visible
- `.connecting`: Yellow, visible
- `.connected`: Green, hidden (auto-fade)

**CSS:**
```css
.connection-status {
  position: fixed;
  top: 72px;
  left: 50%;
  transform: translateX(-50%);
  padding: 8px 16px;
  border-radius: var(--radius-lg);
  font-size: 13px;
  font-weight: 500;
  z-index: var(--z-fixed);
  opacity: 0;
}

.connection-status.disconnected {
  background: var(--error-color);
  color: white;
  opacity: 1;
}

.connection-status.connecting {
  background: var(--warning-color);
  color: white;
  opacity: 1;
}

.connection-status.connected {
  background: var(--success-color);
  color: white;
  opacity: 0; /* Auto-hide when connected */
}
```

---

## 🔘 Button Components

### 12. Primary Button

Main action button with accent color.

**Usage:**
```html
<button class="button-primary">
  Submit
</button>
```

**CSS:**
```css
.button-primary {
  background: var(--accent-color);
  color: hsl(var(--primary-foreground));
  border: none;
  padding: 10px 20px;
  border-radius: var(--radius-md);
  font-weight: 600;
  cursor: pointer;
}

.button-primary:hover {
  background: var(--accent-hover);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}
```

---

### 13. Secondary Button

Alternative button style.

**Usage:**
```html
<button class="button-secondary">
  Cancel
</button>
```

**CSS:**
```css
.button-secondary {
  background: var(--secondary-bg);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  padding: 10px 20px;
  border-radius: var(--radius-md);
  font-weight: 600;
}

.button-secondary:hover {
  background: var(--bg-hover);
  border-color: var(--accent-color);
}
```

---

## 📱 Responsive Components

### Mobile Adaptations

**Breakpoints:**
```css
/* Mobile (≤480px) */
@media (max-width: 480px) {
  :root {
    --header-height: 56px;
  }
  
  .icon-button {
    width: 36px;
    height: 36px;
  }
  
  .logo-text {
    display: none; /* Hide on small screens */
  }
}

/* Tablet (≤768px) */
@media (max-width: 768px) {
  .apps-dropdown {
    width: calc(100vw - 32px);
    max-width: 280px;
  }
}
```

---

## 🎭 Animation Utilities

### Fade In

```css
.fade-in {
  animation: fadeIn var(--transition-smooth) ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
```

### Slide In

```css
.slide-in {
  animation: slideIn var(--transition-smooth) ease-out;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(-20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}
```

### Spin (for loaders)

```css
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
```

---

## ♿ Accessibility Features

### Focus Styles

All interactive elements have visible focus indicators:

```css
*:focus-visible {
  outline: 2px solid var(--accent-color);
  outline-offset: 2px;
}
```

### Screen Reader Text

```css
.visually-hidden {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}
```

### ARIA Labels

All interactive components include proper ARIA labels:

```html
<button aria-label="Open applications menu" aria-expanded="false">
  <svg aria-hidden="true">...</svg>
</button>
```

---

## 📊 Z-Index Layers

Consistent z-index hierarchy:

```css
:root {
  --z-dropdown: 1000;   /* Dropdowns */
  --z-sticky: 1020;     /* Sticky header */
  --z-fixed: 1030;      /* Fixed elements */
  --z-modal-backdrop: 1040; /* Modal backdrop */
  --z-modal: 1050;      /* Modals */
  --z-popover: 1060;    /* Popovers */
  --z-tooltip: 1070;    /* Tooltips */
}
```

---

## 🎨 Color System

### Theme Colors

```css
/* Light backgrounds */
--primary-bg: hsl(var(--background));
--secondary-bg: hsl(var(--card));

/* Text colors */
--text-primary: hsl(var(--foreground));
--text-secondary: hsl(var(--muted-foreground));

/* Interactive colors */
--accent-color: hsl(var(--primary));
--accent-hover: hsl(var(--primary) / 0.9);
--accent-light: hsla(var(--primary) / 0.1);

/* Status colors */
--success-color: hsl(142 76% 36%);
--warning-color: hsl(38 92% 50%);
--error-color: hsl(var(--destructive));
```

---

## 📏 Spacing System

Consistent spacing scale:

```css
:root {
  --space-xs: 4px;
  --space-sm: 8px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;
  --space-2xl: 48px;
}
```

**Usage:**
```css
.component {
  padding: var(--space-md);
  margin-bottom: var(--space-lg);
  gap: var(--space-sm);
}
```

---

## 🔄 Border Radius

Scalable border radius system:

```css
:root {
  --radius: 0.5rem; /* Base radius from theme */
  --radius-sm: calc(var(--radius) * 0.5);
  --radius-md: var(--radius);
  --radius-lg: calc(var(--radius) * 1.5);
  --radius-xl: calc(var(--radius) * 2);
  --radius-2xl: calc(var(--radius) * 3);
  --radius-full: 9999px;
}
```

---

## 🖼️ Shadow System

Elevation through shadows:

```css
:root {
  --shadow-sm: 0 1px 2px 0 hsla(var(--foreground) / 0.05);
  --shadow-md: 0 4px 6px -1px hsla(var(--foreground) / 0.1);
  --shadow-lg: 0 10px 15px -3px hsla(var(--foreground) / 0.1);
  --shadow-xl: 0 20px 25px -5px hsla(var(--foreground) / 0.1);
}
```

---

## 🎬 Transition System

Consistent animation timing:

```css
:root {
  --transition-fast: 150ms cubic-bezier(0.4, 0, 0.2, 1);
  --transition-smooth: 300ms cubic-bezier(0.4, 0, 0.2, 1);
  --transition-slow: 500ms cubic-bezier(0.4, 0, 0.2, 1);
}
```

**Usage:**
```css
.component {
  transition: all var(--transition-smooth);
}
```

---

## 📝 Component Checklist

When creating new components:

- [ ] Use theme variables for all colors
- [ ] Include hover/focus/active states
- [ ] Add ARIA labels and roles
- [ ] Test keyboard navigation
- [ ] Ensure responsive behavior
- [ ] Use consistent spacing
- [ ] Apply appropriate z-index
- [ ] Add smooth transitions
- [ ] Test with all themes
- [ ] Verify accessibility

---

## 🔗 Related Documentation

- [THEMES.md](THEMES.md) - Theme system details
- [README.md](README.md) - General documentation
- [MDN Web Docs](https://developer.mozilla.org) - Web standards reference

---

**Component Library Version:** 1.0  
**Last Updated:** 2024  
**Maintained by:** General Bots Team