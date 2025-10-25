# .gbtheme UI Theming

The `.gbtheme` package contains user interface customization files for web and other frontend interfaces.

## What is .gbtheme?

`.gbtheme` defines the visual appearance and user experience:
- CSS stylesheets for styling
- HTML templates for structure
- JavaScript for interactivity
- Assets like images and fonts

## Theme Structure

A typical theme package contains:

```
theme-name.gbtheme/
├── web/
│   ├── index.html          # Main template
│   ├── chat.html          # Chat interface
│   └── login.html         # Authentication
├── css/
│   ├── main.css           # Primary styles
│   ├── components.css     # UI components
│   └── responsive.css     # Mobile styles
├── js/
│   ├── app.js            # Application logic
│   └── websocket.js      # Real-time communication
└── assets/
    ├── images/
    ├── fonts/
    └── icons/
```

## Web Interface

The main web interface consists of:

### HTML Templates
- `index.html`: Primary application shell
- `chat.html`: Conversation interface
- Component templates for reusable UI

### CSS Styling
- Color schemes and typography
- Layout and responsive design
- Animation and transitions
- Dark/light mode support

### JavaScript
- WebSocket communication
- UI state management
- Event handling
- API integration

## Theme Variables

Themes can use CSS custom properties for easy customization:

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

## Responsive Design

Themes should support:
- **Desktop**: Full-featured interface
- **Tablet**: Adapted layout and interactions
- **Mobile**: Touch-optimized experience
- **Accessibility**: Screen reader and keyboard support

## Theme Switching

Multiple themes can be provided:
- Light and dark variants
- High contrast for accessibility
- Brand-specific themes
- User-selected preferences

## Customization Points

Key areas for theme customization:
- Color scheme and branding
- Layout and component arrangement
- Typography and spacing
- Animation and micro-interactions
- Iconography and imagery
