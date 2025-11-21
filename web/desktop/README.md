# General Bots Desktop

A modern, themeable desktop web application for AI-powered workspace featuring Chat, Drive, Tasks, and Mail modules.

## 🎨 Features

- **Modern UI/UX**: Glass morphism effects, smooth animations, and responsive design
- **Theme System**: 19+ built-in themes with HSL-based customization
- **Modular Architecture**: Pluggable sections (Chat, Drive, Tasks, Mail)
- **Real-time Communication**: WebSocket-based chat with LiveKit integration
- **Accessibility**: ARIA labels, keyboard navigation, screen reader support
- **Performance**: Lazy loading, section caching, optimized animations
- **Responsive**: Works on desktop, tablet, and mobile devices

## 🚀 Quick Start

### Prerequisites

- Modern web browser (Chrome 88+, Firefox 89+, Safari 14+)
- Web server (Apache, Nginx, or development server)

### Installation

1. Clone or download the project
2. Serve the `web/desktop` directory through a web server
3. Navigate to `http://localhost/desktop/` (or your server URL)

### Development Server

```bash
# Using Python
cd web/desktop
python -m http.server 8000

# Using Node.js
npx http-server -p 8000

# Using PHP
php -S localhost:8000
```

Then open `http://localhost:8000` in your browser.

## 📁 Project Structure

```
desktop/
├── index.html              # Main entry point
├── README.md              # This file
├── THEMES.md              # Theme system documentation
│
├── css/
│   └── app.css            # Core styles and theme bridge
│
├── js/
│   ├── theme-manager.js   # Theme switching logic
│   └── layout.js          # Section loading and navigation
│
├── public/
│   └── themes/            # Theme CSS files
│       ├── orange.css
│       ├── cyberpunk.css
│       ├── retrowave.css
│       └── ... (16 more themes)
│
├── chat/
│   ├── chat.html          # Chat UI
│   ├── chat.css           # Chat styles
│   └── chat.js            # Chat logic
│
├── drive/
│   ├── drive.html         # Drive UI
│   ├── drive.css          # Drive styles
│   └── drive.js           # Drive logic
│
├── tasks/
│   ├── tasks.html         # Tasks UI
│   ├── tasks.css          # Tasks styles
│   └── tasks.js           # Tasks logic
│
└── mail/
    ├── mail.html          # Mail UI
    ├── mail.css           # Mail styles
    └── mail.js            # Mail logic
```

## 🎨 Theme System

The application uses a sophisticated HSL-based theme system that allows for dynamic theme switching without page reloads.

### Using Themes

1. Click the theme dropdown in the header
2. Select from 19+ available themes
3. Theme preference is saved to localStorage

### Available Themes

- **Default** - Clean, modern light theme
- **Orange** - Office-inspired orange palette
- **Cyberpunk** - Neon cyberpunk aesthetic
- **Retrowave** - 80s synthwave vibes
- **Vapor Dream** - Vaporwave aesthetic
- **Y2K Glow** - Y2K-era design
- **3D Bevel** - Classic 3D beveled look
- **Arcade Flash** - Retro arcade style
- **Disco Fever** - 70s disco aesthetic
- **Grunge Era** - 90s grunge style
- **Jazz Age** - Art deco inspired
- **Mellow Gold** - Warm, mellow tones
- **Mid Century Modern** - 50s/60s design
- **Polaroid Memories** - Vintage photo aesthetic
- **Saturday Cartoons** - Bright, playful colors
- **Seaside Postcard** - Beach-inspired palette
- **Typewriter** - Classic typewriter look
- **Xerox UI** - Office copier aesthetic
- **XTree Gold** - DOS file manager tribute

### Creating Custom Themes

See [THEMES.md](THEMES.md) for detailed documentation on creating and customizing themes.

## ⌨️ Keyboard Shortcuts

- **Alt + 1** - Switch to Chat
- **Alt + 2** - Switch to Drive
- **Alt + 3** - Switch to Tasks
- **Alt + 4** - Switch to Mail
- **Esc** - Close open menus/dropdowns
- **Enter/Space** - Activate focused element

## 🧩 Modules

### Chat

Real-time chat interface with:
- WebSocket communication
- Markdown support
- Voice input (optional)
- Message history
- Typing indicators
- Connection status

### Drive

File management system with:
- File upload/download
- Folder navigation
- File preview
- Search functionality
- Sorting and filtering

### Tasks

Task management with:
- Create/edit/delete tasks
- Task prioritization
- Due dates
- Categories/tags
- Completion tracking

### Mail

Email interface with:
- Inbox/sent/drafts
- Compose messages
- Rich text editing
- Attachments
- Search and filters

## 🎯 Architecture

### Theme System

The theme system uses a two-layer architecture:

1. **Base HSL Variables** - Defined in theme files
2. **Working Variables** - Automatically derived in `app.css`

Example:
```css
/* Base variable (in theme file) */
--primary: 217 91% 60%;

/* Working variable (auto-derived) */
--accent-color: hsl(var(--primary));
--accent-light: hsla(var(--primary) / 0.1);
```

### Section Loading

Sections are loaded dynamically:

1. User clicks app icon or uses keyboard shortcut
2. `layout.js` loads HTML, CSS, and JS for the section
3. Section content is cached for fast switching
4. Alpine.js components are initialized (for Drive, Tasks, Mail)
5. Chat uses custom WebSocket logic

### State Management

- Theme preference: `localStorage.getItem('gb-theme')`
- Section cache: In-memory JavaScript object
- WebSocket connections: Managed per section

## 🔧 Configuration

### Theme Configuration

Edit `js/theme-manager.js` to add/remove themes:

```javascript
const themes = [
  { id: "default", name: "🎨 Default", file: null },
  { id: "mytheme", name: "🌟 My Theme", file: "mytheme.css" }
];
```

### Section Configuration

Edit `js/layout.js` to add/remove sections:

```javascript
const sections = {
  chat: "chat/chat.html",
  mysection: "mysection/mysection.html"
};
```

## 🎨 Styling Guidelines

### Using Theme Variables

Always use theme variables for colors:

```css
/* ✅ Good */
.my-component {
  background: var(--primary-bg);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

/* ❌ Bad */
.my-component {
  background: #ffffff;
  color: #000000;
  border: 1px solid #cccccc;
}
```

### Responsive Breakpoints

```css
/* Mobile */
@media (max-width: 480px) { }

/* Tablet */
@media (max-width: 768px) { }

/* Desktop */
@media (min-width: 769px) { }
```

## 🧪 Testing

### Browser Testing

Test on:
- Chrome/Edge (latest)
- Firefox (latest)
- Safari (latest)
- Mobile browsers

### Theme Testing

1. Switch to each theme
2. Navigate to all sections
3. Test interactive elements
4. Verify contrast ratios
5. Check accessibility

### Accessibility Testing

- Use screen reader (NVDA, JAWS, VoiceOver)
- Navigate with keyboard only
- Test with high contrast mode
- Verify ARIA labels

## 🐛 Troubleshooting

### Themes Not Loading

1. Check browser console for errors
2. Verify theme file exists in `public/themes/`
3. Clear browser cache
4. Check theme file syntax

### Sections Not Switching

1. Check console for JavaScript errors
2. Verify section files exist
3. Check network tab for failed requests
4. Clear localStorage

### WebSocket Connection Issues

1. Check server is running
2. Verify WebSocket URL
3. Check browser console
4. Test network connectivity

## 📊 Performance

### Optimization Techniques

- **Lazy Loading**: Sections loaded on demand
- **Caching**: Section HTML/CSS/JS cached after first load
- **CSS Variables**: Fast theme switching without reflow
- **Debouncing**: Input handlers debounced
- **Animations**: GPU-accelerated with `transform` and `opacity`

### Performance Metrics

- Initial load: < 1s
- Theme switch: < 100ms
- Section switch: < 200ms (cached), < 500ms (first load)
- Animation: 60 FPS target

## 🔒 Security

### Best Practices

- No inline event handlers
- CSP-friendly code
- XSS protection in chat
- Sanitized user input
- Secure WebSocket connections (WSS in production)

### Content Security Policy

Recommended CSP header:

```
Content-Security-Policy: 
  default-src 'self'; 
  script-src 'self' https://cdnjs.cloudflare.com https://cdn.jsdelivr.net https://unpkg.com; 
  style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; 
  font-src https://fonts.gstatic.com;
```

## 🤝 Contributing

### Adding a New Theme

1. Create `public/themes/mytheme.css`
2. Define HSL variables (see THEMES.md)
3. Add to `js/theme-manager.js`
4. Test thoroughly
5. Submit pull request

### Adding a New Section

1. Create directory: `mysection/`
2. Add files: `mysection.html`, `mysection.css`, `mysection.js`
3. Register in `js/layout.js`
4. Add icon to apps menu in `index.html`
5. Test integration
6. Update documentation

## 📝 API Reference

### ThemeManager

```javascript
// Initialize
ThemeManager.init();

// Load theme
ThemeManager.loadTheme('cyberpunk');

// Subscribe to changes
ThemeManager.subscribe((data) => {
  console.log(data.themeId, data.themeName);
});

// Get themes
const themes = ThemeManager.getAvailableThemes();
```

### Layout Manager

```javascript
// Switch section
window.switchSection('chat');

// Get current section
const section = window.location.hash.substring(1);
```

## 🔄 Version History

### v1.0.0 (Current)
- Initial release
- 19 built-in themes
- 4 core modules (Chat, Drive, Tasks, Mail)
- HSL-based theme system
- Keyboard shortcuts
- Accessibility improvements

## 📄 License

See project root for license information.

## 🙋 Support

For issues, questions, or contributions:
- Check documentation in THEMES.md
- Review browser console for errors
- Test with different browsers/themes
- Contact General Bots team

## 🌟 Acknowledgments

- **shadcn/ui** - Theme variable inspiration
- **Alpine.js** - Reactive components
- **GSAP** - Smooth animations
- **LiveKit** - Real-time communication
- **marked** - Markdown parsing

---

**Built with ❤️ by the General Bots Team**