# i18n Client Runtime 🟡 BETA

> **`window.i18n` / `window.gbT`, the `data-i18n*` attributes, the formatters,
> and why the cloud and login pages load a second, smaller loader.**

Two loaders exist, on purpose:

| Surface | File | Loaded by |
|---------|------|-----------|
| Suite (port 3000) | `botui/ui/suite/js/i18n.js` | `botui/ui/suite/desktop.html`, with `app-i18n.js` and `i18n-format.js` |
| Login (port 5000) and cloud (port 4000) | `botui/ui/cloud/js/i18n-lite.js` | every `botui/ui/login/*.html` and `botui/ui/cloud/*.html` page, with `i18n-switcher.js` |

---

## Why `i18n-lite.js` exists

`/suite/js/i18n.js` is served by the suite router and assumes the desktop shell
(window manager, `productConfig`, the suite asset tree). The login router serves
`/js/*` from `login/js` and falls back to `cloud/js`, so the suite bundle is
simply **not reachable** from a login or cloud page. Rather than duplicate the
catalogue, `cloud/js/i18n-lite.js` implements the same API contract against the
same endpoints with a smaller footprint, and lives in `cloud/js/` so one copy
covers both routers (see its header comment).

The two loaders differ in these ways:

| Feature | `i18n.js` (suite) | `i18n-lite.js` (login/cloud) |
|---------|-------------------|------------------------------|
| Cache key | `gb-i18n-cache-<locale>-<build>` (build from `/api/product`) | `gb-i18n-lite-<locale>` (no build stamp) |
| Persists the choice to the server (`POST /api/i18n/preference`) | Yes, best effort | No — `localStorage` only |
| Reads the server-side preference on boot | Yes, when `localStorage` is empty | No |
| `data-i18n-label` (for `<option label>` / `<optgroup label>`) | No | Yes |
| `formatRelativeTime` | Yes | No |
| Sends `Accept-Language` on catalogue fetches | No (plain `Accept`) | Yes |

Both keep a one-hour TTL, both fall back to English when a locale has no
entries, and both expose the same `window.i18n` surface plus `window.gbT`.

---

## `window.i18n`

```javascript
window.i18n.t(key, params)              // key itself when missing (and a console warning)
window.i18n.tOr(key, fallback, params)  // fallback when the key is missing
window.i18n.init()                      // idempotent; both loaders self-invoke it
window.i18n.setLocale(locale)           // switch, re-render in place, announce
window.i18n.getLocale()                 // active locale, e.g. "pt-BR"
window.i18n.getAvailableLocales()       // from /api/i18n/locales
window.i18n.translatePage(root?)        // translate a subtree (default: document)
window.i18n.translateElement(el)        // translate one node
window.i18n.clearCache()                // drop every gb-i18n-cache-* entry (suite)
```

`window.gbT` is `tOr` under a short name, for call sites that build DOM strings
outside a translated element (window chrome, launcher, toasts):

```javascript
el.textContent = window.gbT("action-retry", "Retry");
```

**`t` versus `tOr`.** `t` returns the key when the catalogue has no entry —
correct for `data-i18n` nodes, where the authored English text stays in place
because the runtime skips the write when the value equals the key. `tOr` is for
generated strings, where leaking an identifier would be a visible bug. The rule
of thumb: markup uses attributes, JavaScript uses `gbT`/`tOr`.

---

## The `data-i18n*` attribute list

| Attribute | Writes to | Supported by |
|-----------|-----------|--------------|
| `data-i18n="key"` | `textContent` | both |
| `data-i18n-html="key"` | `innerHTML` (opt-in; catalogue content only) | both |
| `data-i18n-placeholder="key"` | `placeholder` attribute | both |
| `data-i18n-title="key"` | `title` attribute | both |
| `data-i18n-aria-label="key"` | `aria-label` attribute | both |
| `data-i18n-value="key"` | `value` attribute | both |
| `data-i18n-label="key"` | `label` attribute | `i18n-lite.js` only |
| `data-i18n-params='{"count":3}'` | arguments for the key on that element (JSON) | both |
| `data-i18n-document-title="key"` | `document.title`, read from `<html>` | both |

```html
<h2 data-i18n="settings-title"></h2>
<input data-i18n-placeholder="chat-placeholder" />
<button data-i18n-title="action-delete" data-i18n-aria-label="action-delete"></button>
<span data-i18n="admin-group-member-count" data-i18n-params='{"count": 3}'></span>
```

Parameter substitution is textual: `{ $count }` and `{$count}` both become the
value. Only the attributes listed above are recognised — a `data-i18n-*` name
invented elsewhere does nothing.

### Dynamically injected DOM

The suite loader re-translates on `htmx:afterSwap` and `htmx:afterSettle`, and
watches `document.body` with a `MutationObserver` for added nodes. The lite
loader watches mutations too. Code that renders its own DOM still needs to call
`window.i18n.translatePage(node)` if it replaces markup wholesale, or listen for
`localeChanged` and re-render.

---

## The `localeChanged` event

Both loaders dispatch a `CustomEvent` on `window` after a locale switch **and**
after the first successful load (once the DOM pass has run):

```javascript
window.addEventListener("localeChanged", (event) => {
  const locale = event.detail.locale; // "pt-BR"
  rerenderMyOwnDom();
});
```

The suite loader dispatches both `localeChanged` and `i18n:localeChanged` (same
detail). Existing consumers include the start menu, status bar, command palette,
sidebar, toasts, spotlight, tour overlay, about page, recycle bin and the cloud
language switcher — all of them re-render strings they built themselves instead
of forcing a page reload.

`setLocale` also updates `<html lang="pt-BR">` (full BCP-47 tag, so fonts and
assistive technology can tell `pt-BR` from `pt-PT`) and `<html dir>`: `rtl` for
`ar`, `he`, `fa`, `ur`, `ltr` otherwise.

---

## Formatters

`botui/ui/suite/js/i18n-format.js` attaches `Intl`-based helpers to
`window.i18n`, so no UI code needs to hardcode `$`, `.` or `MM/DD/YYYY`:

```javascript
window.i18n.formatDate(value, options?)          // Intl.DateTimeFormat
window.i18n.formatNumber(value, options?)        // Intl.NumberFormat
window.i18n.formatCurrency(value, currency?)     // style: "currency", default USD
window.i18n.formatRelativeTime(value, unit)      // Intl.RelativeTimeFormat, numeric: "auto"
```

Every helper is guarded: a bad value or an unsupported locale returns the input
as a string instead of throwing. `i18n-lite.js` ships `formatDate`,
`formatNumber` and `formatCurrency` (same guards) but not
`formatRelativeTime`.

---

## Locale detection and caching (suite)

On boot `i18n.js`:

1. Reads `/api/product` for the build version → cache key
   `gb-i18n-cache-<locale>-<build>`, so a deploy that changes the catalogue
   invalidates every client cache without a manual version bump.
2. Fetches `/api/i18n/locales` and adopts that list (the bundled list is only a
   first-paint fallback).
3. Picks the locale: an explicit `localStorage["gb-locale"]` wins; otherwise it
   asks `GET /api/i18n/preference` (the choice made on another device); otherwise
   the browser language; otherwise `en`.
4. Loads the dictionary from cache (1 h TTL) or `GET /api/i18n/<locale>`.
5. Translates the page, installs the observers, then announces `localeChanged`.

`setLocale(locale)` writes `gb-locale`, POSTs the preference (best effort — an
unauthenticated visitor still gets the switch), reloads the dictionary,
re-translates, updates `<html lang/dir>` and dispatches the event.

The settings page (`botui/ui/suite/settings/main/07_appearance.js`) calls
`window.i18n.setLocale` directly and shows a toast, so switching language needs
no reload. `botui/ui/suite/js/translations.js` is only a compatibility shim that
delegates to `i18n.js` for the one caller that still loads it — there is a
single cache and a single locale source.

---

## The language switcher (login / cloud)

`botui/ui/cloud/js/i18n-switcher.js` mounts a `<select>` at every
`[data-i18n-switcher]` element; if a page declares none and has a
`.mgmt-topbar-actions` container, the switcher is appended there automatically.
Labels come from its `NATIVE_NAMES` map (`pt-BR` → "Português (Brasil)"), so a
new locale needs an entry there or the raw code is shown. The control stays in
sync through the `localeChanged` listener.

---

## See also

- [Localization (i18n)](./README.md) — catalogue and tooling
- [i18n Server Surface](./server.md) — the endpoints these loaders call
- [i18n API](../../08-rest-api-tools/i18n-api.md) — the HTTP reference
