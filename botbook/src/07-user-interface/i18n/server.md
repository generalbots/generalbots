# i18n Server Surface 🟡 BETA

> **The HTTP endpoints, the single locale resolver, and the localized error
> contract.**

All of this lives in `botserver/crates/botcore/src/i18n/` and is mounted once,
from `botserver/src/main_module/routes/feature_merge_office.rs`, by merging
`configure_i18n_routes()` (`i18n/routes.rs`).

```
botcore/src/i18n/
├── mod.rs          Locale re-export, locale_from_headers, RequestLocale, init
├── routes.rs       /api/i18n/*
├── error.rs        LocalizedError
├── cli.rs          --lang / LC_ALL for CLI text
└── translations/   explicit key list + FTL union
```

There is exactly **one** `Locale` type: `botlib::i18n::Locale`
(`botlib/src/i18n/locale.rs`), re-exported by `botcore::i18n`. The old
duplicate implementation and the hard-coded `AVAILABLE_LOCALES` are gone; the
available set now comes from the loaded catalogue
(`botcore/src/i18n/mod.rs`). `BUNDLED_LOCALES` survives only as a
before-initialisation fallback.

---

## Endpoints

### `GET /api/i18n/locales`

Lists the locales the catalogue actually ships, plus the default and the
presentation metadata a language picker and a formatter need.

```json
{
  "locales": ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"],
  "default": "en",
  "details": [
    {
      "code": "en",
      "native_name": "English",
      "dir": "ltr",
      "date_format": "MM/dd/yyyy",
      "number_format": "#,##0.###"
    }
  ]
}
```

No authentication. `locales` is the flat list of codes the two client loaders
already read; `details` describes the same locales, in the same order, with:

| Field | Meaning |
|-------|---------|
| `code` | BCP-47 code, identical to the catalogue directory name |
| `native_name` | The language's own name (`Português (Brasil)`, `日本語`), so the picker is readable in every locale |
| `dir` | `ltr` or `rtl`; the shell sets the document direction from it |
| `date_format` | CLDR date pattern for `Intl.DateTimeFormat` (`dd/MM/yyyy`) |
| `number_format` | CLDR decimal pattern for `Intl.NumberFormat` (`#.##0,###`) |

The table lives in `botcore/src/i18n/locale_meta.rs`. It decorates the
catalogue, it does not define it: a locale directory added without an entry
still appears in `locales` and `details`, with its code as the native name, the
English formats and `ltr`. RTL is decided by the language subtag (`ar`, `he`,
`fa`, `ur`, …), so the first right-to-left locale needs no code change either.

### `GET /api/i18n/:locale`

Returns one flat dictionary for that locale.

```json
{ "locale": "pt-BR", "translations": { "action-retry": "Tentar novamente", "…": "…" } }
```

- The path segment is parsed with `Locale::new`; an unparsable value falls back
  to the default locale rather than erroring (`routes.rs`,
  `handle_get_translations`).
- `translations` is the union of the explicit Rust key list
  (`translations/translation_keys_actions.rs`, `…_common.rs`, `…_tours.rs`) and
  `botlib::i18n::message_ids()` — every key authored in `botlib/locales/en/*.ftl`,
  deduplicated. A key that exists only in an `.ftl` file is still served (#1853).
- Values are rendered **without arguments**, so plural blocks are stripped, not
  selected. The endpoint is a dictionary for the client; per-message arguments
  are applied in the browser (`data-i18n-params`) or server-side with
  `translate`.
- A key missing everywhere is served as `[the-key]` and logged as a warning.

### `GET /api/i18n/preference`

Returns the locale the caller picked: `{ "locale": "pt-BR" }`.

Requires a bearer token. The user id comes from the token claims
(`shared::tenant::user_id_from_claims`); without one the response is `401` with
the localized body for `error-auth-token-missing`.

The value lives in the generic `user_preferences` table under
`preference_key = 'locale'` — **not** in a `users.locale` column, which does not
exist. `locale_for_user(conn, user_id)` (`routes.rs`) is the reusable reader for
background work (emails, notifications, scheduled jobs): they must resolve the
recipient's preference explicitly and never use the process locale.

### `POST /api/i18n/preference`

```json
{ "locale": "pt-BR" }
```

| Case | Response |
|------|----------|
| Saved | `200` `{ "locale": "pt-BR" }` |
| No/invalid token | `401` `error-auth-token-missing` |
| Unparsable locale | `400` `error-locale-invalid` |
| Parsable but not shipped | `400` `error-locale-unsupported` + `"available": [...]` |
| Database failure | `500` `error-locale-persist-failed` |

The write is an `INSERT … ON CONFLICT (user_id, preference_key) DO UPDATE`, so a
user has one row per key. Only locales the catalogue ships are accepted, so a
typo cannot pin someone to a language with no translations.

---

## The locale resolver

One function decides the language of a request:
`botcore::i18n::locale_from_headers` (`i18n/mod.rs`).

```
X-Locale  →  Accept-Language  →  en
```

1. **`X-Locale`** — an explicit pin, honoured only if the catalogue ships a
   matching locale (`matches` ignores a missing region, so `pt` selects
   `pt-BR`). An unsupported value is ignored, not rejected. This is for
   public/unauthenticated contexts where there is no user row to consult.
2. **`Accept-Language`** — parsed into `(language, q)` pairs, sorted by `q`,
   with `q=0` entries dropped ("not acceptable", RFC 9110). Negotiation is
   exact language+region, then language-only, then the default.
3. **`en`** — the fallback.

`RequestLocale` is the axum extractor wrapping that resolver
(`FromRequestParts`), so a handler asks for `RequestLocale` instead of parsing
headers. Its rejection is `Infallible`: locale resolution never fails a request.

> **Not implemented:** a `?locale=` query parameter. The resolver reads headers
> only. (An older revision of the [i18n API page](../../08-rest-api-tools/i18n-api.md)
> mentioned it.)

### Resolution order for a *person* (not a request)

| Context | Signal |
|---------|--------|
| Browser request | `X-Locale`, then `Accept-Language`, then `en` |
| Authenticated handler needing the user's own choice | `locale_for_user(conn, user_id)`, then the header resolver |
| Email / notification / scheduled job | the recipient's `user_preferences` value, then `en` |
| WhatsApp / Telegram / Teams message | the bot's `locale` config via `botlib::i18n::locale_from_config`, then `en` |
| CLI text (`--help`, `security`, shutdown banner) | `--lang <code>` / `--lang=<code>`, then `LC_ALL`, `LC_MESSAGES`, `LANG`, then `en` |

---

## The `LocalizedError` contract

`botcore::i18n::error` (`error.rs`) is the only shape user-facing errors should
take:

```json
{
  "code": "error-http-404",
  "message": "Bot não encontrado.",
  "error": "Bot não encontrado.",
  "details": { "trace_id": "…" }
}
```

| Field | Meaning |
|-------|---------|
| `code` | Stable, machine-readable identifier (`error-http-404`, `error-locale-unsupported`, …). Clients branch on this. |
| `message` | Localized human text, resolved with the request locale. |
| `error` | The same localized text, kept for clients that still read the legacy field. |
| `details` | Optional structured context, omitted when absent. |

Constructors: `LocalizedError::new`, `with_args`, `not_found`, `validation`,
`internal`, `unauthorized`, `forbidden`, `rate_limited`, each with
`with_details`. For handlers that do not build the struct, `error_body` and
`error_body_with_args` return the same JSON.

Rules:

- Never return a raw `e.to_string()` to a client — that is what
  `log_and_sanitize` and this contract exist to prevent.
- Keep `code` stable across releases; translate `message`, never the code.
- Do not reshape machine-readable payloads (JSON keys, provider error
  envelopes, OpenAI-compatible `{"error": {"message": …}}` bodies) — localize
  only their human-readable text.

---

## Translating from Rust

```rust
use botcore::i18n::{t, t_args, Locale, translate};

let locale = Locale::new("pt-BR").unwrap_or_default();
let plain = t(&locale, "action-retry");                        // no arguments
let count = t_args(&locale, "admin-group-member-count", &[("count", "3")]);
```

`translate` is `botlib::i18n::translate`, re-exported, so library crates that
cannot depend on `botcore` (email, channel adapters, the CLI) use the same
catalogue and fallback chain.

Initialization: `botserver` calls `i18n::init_i18n(default_locales_path())`
once at boot (`main_module/init.rs`); a failure is logged and the process keeps
running with keys visible. Code paths that run *before* the bootstrap call
`ensure_initialized()` (`botcore/src/i18n/mod.rs`), which loads the catalogue on
demand.

---

## See also

- [Localization (i18n)](./README.md) — catalogue layout and the Fluent subset
- [i18n Client Runtime](./client.md) — what consumes these endpoints
- [i18n API](../../08-rest-api-tools/i18n-api.md) — the HTTP reference
