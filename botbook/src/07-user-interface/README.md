# Chapter 7: User Interface

Customize your bot's appearance and build complete productivity suites.

## Overview

Themes control colors, fonts, logos, and overall visual presentation of your bot interface.

## Quick Start

```csv
# In config.csv
name,value
theme-color1,#0d2b55
theme-color2,#fff9c2
theme-title,My Bot
theme-logo,https://example.com/logo.svg
```

## Theme Structure

```
mybot.gbai/
└── mybot.gbtheme/
    └── style.css
```

## Configuration Options

| Setting | Description | Example |
|---------|-------------|---------|
| `theme-color1` | Primary color | `#0d2b55` |
| `theme-color2` | Secondary color | `#fff9c2` |
| `theme-title` | Bot name in header | `My Assistant` |
| `theme-logo` | Logo URL | `https://...` |

## CSS Customization

Create `style.css` in your `.gbtheme` folder:

```css
:root {
  --primary: #0d2b55;
  --secondary: #fff9c2;
}

.chat-header {
  background: var(--primary);
}

.user-message {
  background: var(--secondary);
}
```

## Chapter Contents

- [Theme Structure](./structure.md) - File organization
- [CSS Customization](./css.md) - Styling reference

## See Also

- [UI Reference](../07-user-interface/README.md) - Interface options
- [.gbot Configuration](../10-configuration-deployment/README.md) - All settings