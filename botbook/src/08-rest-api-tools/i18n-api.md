# Internationalization API

> **Retrieve available locales and translated strings for the BotServer UI.**

---

## Base URL

```
/api/i18n
```

## Authentication

This API does not require authentication. Endpoints are public.

---

## Overview

The Internationalization (i18n) API provides access to translated UI strings. BotServer supports multiple locales and automatically negotiates the best match based on the `Accept-Language` header. This API exposes the underlying locale data for client-side use.

---

## Endpoints

### Get Available Locales

**`GET /api/i18n/locales`**

List all available locales supported by the system.

**Response:**
```json
{
  "locales": ["en", "pt-BR", "es", "fr"],
  "default": "en"
}
```

---

### Get Translations for Locale

**`GET /api/i18n/:locale`**

Retrieve all translated strings for a specific locale. If the requested locale is not found, falls back to the default locale (`en`).

**Path Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| locale | string | Yes | Locale identifier (e.g., `en`, `pt-BR`, `es`) |

**Response:**
```json
{
  "locale": "pt-BR",
  "translations": {
    "app.title": "BotServer",
    "login.button": "Entrar",
    "login.username": "Usuário",
    "login.password": "Senha",
    "chat.placeholder": "Digite sua mensagem...",
    "chat.send": "Enviar",
    "settings.title": "Configurações",
    "users.title": "Usuários",
    "bots.title": "Bots"
  }
}
```

**Fallback Response (unknown locale):**
```json
{
  "locale": "en",
  "translations": {
    "app.title": "BotServer",
    "login.button": "Login",
    "login.username": "Username",
    "login.password": "Password",
    "chat.placeholder": "Type your message...",
    "chat.send": "Send",
    "settings.title": "Settings",
    "users.title": "Users",
    "bots.title": "Bots"
  }
}
```

---

## Locale Negotiation

BotServer automatically negotiates the best locale using the `Accept-Language` header. The negotiation follows these rules:

1. **Exact match** — `pt-BR` matches `pt-BR`
2. **Language match** — `pt-PT` matches `pt` (any regional variant)
3. **Fallback** — Any unknown locale falls back to `en`

**Example `Accept-Language` header:**
```
Accept-Language: pt-BR,pt;q=0.9,en;q=0.8
```

This requests Brazilian Portuguese first, then any Portuguese variant, then English.

---

## Examples

### Get All Available Locales

```bash
curl -X GET http://localhost:8080/api/i18n/locales
```

### Get Portuguese Translations

```bash
curl -X GET http://localhost:8080/api/i18n/pt-BR
```

### Get English Translations

```bash
curl -X GET http://localhost:8080/api/i18n/en
```

### Use in Frontend

```javascript
// Fetch translations for the user's locale
const response = await fetch('/api/i18n/pt-BR');
const { locale, translations } = await response.json();

// Apply translations to the UI
document.querySelectorAll('[data-i18n]').forEach(el => {
  const key = el.getAttribute('data-i18n');
  if (translations[key]) {
    el.textContent = translations[key];
  }
});
```

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 404 | Not Found (if i18n is not initialized) |

---

## See Also

- [BotUI Development](../07-user-interface/web-interface.md) — Frontend interface
- [Admin API](./admin-api.md) — System configuration
- [Organizations API](./organizations-api.md) — Organization settings
