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

The system automatically picks up any theme placed under `@/templates/…` when the bot’s configuration (`.gbtheme` entry in `config.csv`) points to the folder name.
