# Web Interface

The **gbtheme** web interface provides the front‑end experience for end users. It consists of three core HTML pages and a set of JavaScript modules that handle real‑time communication with the bot server.

## Pages

| Page | Purpose |
|------|---------|
| `index.html` | Application shell, loads the main JavaScript bundle and displays the navigation bar. |
| `chat.html` | Primary conversation view – shows the chat transcript, input box, and typing indicator. |
| `login.html` | Simple authentication screen used when the bot is configured with a login flow. |

## JavaScript Modules

* **app.js** – Initializes the WebSocket connection, routes incoming bot messages to the UI, and sends user input (`TALK`) back to the server.
* **websocket.js** – Low‑level wrapper around the browser’s `WebSocket` API, handling reconnection logic and ping/pong keep‑alive.

## Interaction Flow

1. **Load** – `index.html` loads `app.js`, which creates a `WebSocket` to `ws://<host>/ws`.
2. **Handshake** – The server sends a `HELLO` message containing bot metadata (name, version).
3. **User Input** – When the user presses *Enter* in the chat input, `app.js` sends a `TALK` JSON payload.
4. **Bot Response** – The server streams `MESSAGE` events; `app.js` appends them to the chat window.
5. **Typing Indicator** – While the LLM processes, the server sends a `TYPING` event; the UI shows an animated ellipsis.

## Customization Points

* **CSS Variables** – Override colors, fonts, and spacing in `css/main.css` (`:root { --primary-color: … }`).
* **HTML Layout** – Replace the `<header>` or `<footer>` sections in `index.html` to match branding.
* **JS Hooks** – Add custom event listeners in `app.js` (e.g., analytics on `MESSAGE` receipt).

All files are located under the theme’s `web/` and `js/` directories as described in the [Theme Structure](./structure.md).
