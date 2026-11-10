# Internationalization API 🟡 BETA

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
  "locales": ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"],
  "default": "en"
}
```

The list comes from the loaded catalogue (`botlib/locales/<code>/`), so it grows
by adding a locale directory — there is no second list to maintain. See
[Localization (i18n)](../07-user-interface/i18n/README.md).

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

BotServer resolves the response language with one helper (`botcore::i18n::locale_from_headers`), in this order:

1. **Explicit** — `X-Locale: pt-BR` header (public/unauthenticated contexts). An unsupported value is ignored, not rejected.
2. **User preference** — the locale stored in `user_preferences` (key `locale`) for authenticated requests, and for background work (emails, notifications, digests) the recipient's preference — never the process locale.
3. **`Accept-Language`** — exact match, then language match, then fallback. Entries with `q=0` are treated as "not acceptable" and skipped.
4. **Default** — `en`.

**Example `Accept-Language` header:**
```
Accept-Language: pt-BR,pt;q=0.9,en;q=0.8
```

This requests Brazilian Portuguese first, then any Portuguese variant, then English.

---

## Locale Preference

### Get Preference

**`GET /api/i18n/preference`** — returns the locale this user picked (`{"locale": "pt-BR"}`), so another device or a server-rendered email can reuse it.

### Set Preference

**`POST /api/i18n/preference`** with `{"locale": "pt-BR"}`. Only a locale the catalogue ships is accepted; an unsupported value returns `400` with the `available` list.

---

## Localized Error Contract

User-facing API errors carry a stable, machine-readable `code`, a localized
`message`, and the legacy `error` string (same text) so existing clients keep
working:

```json
{
  "code": "error-bot-not-found",
  "message": "Bot não encontrado.",
  "error": "Bot não encontrado."
}
```

`details` (optional) carries structured context. Machine-readable payloads —
OpenAI-compatible `{"error": {"message": ...}}` envelopes, provider codes, JSON
keys — keep their shape; only the human-readable text is localized.

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

- [Localization (i18n)](../07-user-interface/i18n/README.md) — catalogue, Fluent subset, tooling
- [i18n Server Surface](../07-user-interface/i18n/server.md) — resolver and error contract
- [i18n Client Runtime](../07-user-interface/i18n/client.md) — `window.i18n` and attributes
- [Admin API](./admin-api.md) — System configuration
- [Organizations API](./organizations-api.md) — Organization settings
