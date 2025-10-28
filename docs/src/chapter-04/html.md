# HTML Templates

The **gbtheme** HTML files provide the markup for the bot’s UI. They are deliberately minimal to allow easy customization.

## index.html

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>GeneralBots Chat</title>
  <link rel="stylesheet" href="css/main.css">
  <link rel="stylesheet" href="css/components.css">
  <link rel="stylesheet" href="css/responsive.css">
</head>
<body>
  <header class="app-header">
    <h1>GeneralBots</h1>
  </header>

  <main id="chat-container"></main>

  <footer class="app-footer">
    <p>&copy; 2025 GeneralBots</p>
  </footer>

  <script src="js/websocket.js"></script>
  <script src="js/app.js"></script>
</body>
</html>
```

*Loads the CSS layers and the JavaScript modules.*

## chat.html

```html
<div class="chat-window">
  <div id="messages" class="messages"></div>
  <div class="input-area">
    <input id="user-input" type="text" placeholder="Type a message…" autocomplete="off"/>
    <button id="send-btn">Send</button>
  </div>
  <div id="typing-indicator" class="typing">…</div>
</div>
```

*Used by `app.js` to render the conversation view.*

## login.html

```html
<div class="login-form">
  <h2>Login</h2>
  <input id="username" type="text" placeholder="Username"/>
  <input id="password" type="password" placeholder="Password"/>
  <button id="login-btn">Login</button>
</div>
```

*Optional page displayed when the bot requires authentication.*

## Customization Tips

* Replace the `<header>` content with your brand logo.
* Add additional `<meta>` tags (e.g., Open Graph) in `index.html`.
* Insert extra `<script>` tags for analytics or feature flags.
* Use the `assets/` folder to store images referenced via `<img src="assets/images/logo.png">`.

All HTML files are located under the theme’s `web/` directory and are referenced by the server based on the `.gbtheme` configuration in `config.csv`.
