# Localization (i18n) 🟡 BETA

> **How the platform stores translations, who resolves the language, and how to
> add a key or a whole language.**

The platform has one catalogue, one `Locale` type and three consumers: the
server (`botcore::i18n`), the suite shell (`/suite/js/i18n.js`) and the
login/cloud pages (`/cloud/js/i18n-lite.js`). This chapter covers the catalogue
and the tooling; the two sub-pages cover the
[server surface](./server.md) and the [client runtime](./client.md).

---

## Where the catalogues live

```
botlib/locales/
├── en/          ← the source of truth (every key is authored here first)
│   ├── admin.ftl
│   ├── apps.ftl
│   ├── bot-templates.ftl
│   ├── common.ftl
│   ├── email.ftl
│   ├── errors.ftl
│   ├── ui-*.ftl          ← 15 topic files split out of the old `ui.ftl`
│   └── … (41 topic files)
├── pt-BR/
├── es/
├── fr/
├── de/
├── ja/
├── ko/
└── zh-CN/
```

The `ui-*` files are one family per surface, and every locale carries the same
set: `ui-shell` (navigation, dashboard, cards, generic controls), `ui-navigation`
(settings nav and guided tours), `ui-chat`, `ui-drive`, `ui-productivity`
(tasks, calendar, meet, mail, research, paper), `ui-settings`, `ui-auth`,
`ui-admin`, `ui-sources`, `ui-compliance`, `ui-attendant`, `ui-crm`,
`ui-business` (billing, products, tickets), `ui-apps` (office/business bundles)
and `ui-apps-runtime` (sheet, canvas, project, dashboards). Each file stays well
under the 450-line limit; the loader does not care about the split, so a new
topic file needs no registration.

One directory per locale, named with a BCP-47 code (`pt-BR`, `zh-CN`). The
directory name *is* the locale: `botlib::i18n::available_locales()` lists the
directories it loaded, so **a locale exists if and only if its directory
exists** — there is no second list to keep in sync
(`botlib/src/i18n/bundle.rs`, `botlib/src/i18n/mod.rs`).

### How the loader merges a locale directory

`LocaleBundle::load` (`botlib/src/i18n/translation_parser.rs`) walks the locale
directory, parses **every** `*.ftl` file with `TranslationFile::parse`, and
`merge`s each result into one flat `HashMap<String, String>`. That is the whole
merge rule:

- Every file in the directory contributes; the filename carries no meaning.
- `merge` is `HashMap::extend`, so when the same key is defined twice the last
  write wins — and directory iteration order is the OS's, not the repository's.
  A duplicate key is therefore *non-deterministic*, which is why the linter
  rejects duplicates outright (see [Tooling](#tooling)).
- Missing files are not an error: a locale simply has fewer keys and resolves
  the rest through English.

**Splitting a large catalogue is just adding a file.** Create
`botlib/locales/en/settings.ftl`, move the `settings-*` keys into it, and the
loader picks it up — no code change, no registry. That is how the old 2,466-line
`ui.ftl` became the fifteen `ui-*.ftl` topic files, one per surface (#2087 asked
for exactly this split), and how `admin.ftl`, `email.ftl` and `suite-*.ftl` were
carved out before them.

### Embedded vs. filesystem loading

`botlib` compiles the catalogue into the binary with `rust-embed`
(`#[folder = "locales"]` in `translation_parser.rs`, feature `i18n`, enabled by
default in `botlib/Cargo.toml` and by `botserver`). Two consequences:

| Mode | When | Behaviour |
|------|------|-----------|
| Embedded (`feature = "i18n"`) | Normal `botserver` build | `load_embedded()` merges every embedded path that starts with the locale code and ends in `.ftl` |
| Filesystem (`not(feature = "i18n")`) | Library builds without the feature | `load()` reads the directory passed to `i18n::init()` |

Because embedding happens at **compile time**, editing an `.ftl` file changes
nothing until the binary is rebuilt. `botcore::i18n::default_locales_path()`
still resolves a directory (`./locales`, `../botlib/locales`, `../locales`) for
the CLI and for builds without the feature; `botserver` calls
`i18n::init_i18n(default_locales_path())` once at boot
(`botserver/src/main_module/init.rs`).

---

## The supported Fluent subset

The parser is **not** a Fluent implementation. It is a flat `key = value`
reader with two extensions, and the catalogue must stay inside that subset.

### Messages and placeholders

```ftl
greeting = Hello, { $name }!
bot-you-said = You said: { $message }
```

- `key = value`, one message per entry. `#` starts a comment line.
- Keys are kebab-case (`^[a-z0-9]+(-[a-z0-9]+)*$`, enforced by the linter).
- A line without `=` continues the previous value. The parser starts a
  multi-line entry when the value has more `{` than `}` (`is_multiline_start`),
  which is how plural blocks survive.
- Placeholders are `{ $name }`; both `{ $name }` and `{$name}` are substituted
  (`I18nBundle::interpolate`).

### Plural / select blocks

```ftl
admin-group-member-count = { $count ->
    [one] { $count } member
   *[other] { $count } members
}
```

`I18nBundle::handle_plurals` finds the block, `matching_brace` finds the `}`
that closes it (brace depth, quote-aware — *not* the first `}`, which used to
truncate every plural at the nested `{ $count }`), and `select_plural_form`
picks a variant:

| Count | Variant selected |
|-------|------------------|
| `0` | `[zero]` if present, else `*[other]` |
| `1` | `[one]` if present, else `*[other]` |
| anything else | `*[other]` |

### What is *not* supported

Do not use these — they will render as literal text:

- **Functions**: `NUMBER()`, `DATETIME()`, `LIST()`, `PLATFORM()`.
- **Attributes**: `message.attr = …` / `{ $var.attr }`.
- **Message references and terms**: `{ other-message }`, `-brand-name`.
- **CLDR categories beyond `zero`/`one`**: `[two]`, `[few]` and `[many]` are
  never selected; they fall through to `*[other]`. Russian/Arabic-style plural
  rules are therefore approximated, not honoured.
- **Escaped braces / raw text blocks** (`{ "" }`).

### What happens when an argument is missing

Three distinct outcomes, all in `botlib/src/i18n/bundle.rs`:

| Situation | Result |
|-----------|--------|
| Key not in the locale **and** not in English | `[the-key]` — the bracketed key, logged as a warning |
| Key translated, but one placeholder's argument is absent from the map | that placeholder stays literal, e.g. `Hello, { $name }!` |
| The whole message is rendered with **no** argument map (`i18n::get`) | `strip_placeholders` removes `{ … }` chunks that contain `$` |

The third case is why plural messages must always be rendered through
`translate`/`t_args` with their `$count` argument: without arguments the block
is stripped rather than selected, and raw variant text can leak through. Use
`i18n::translate(locale, key, &[("count", "5")])`, never `i18n::get`.

---

## Coverage today

`en` is the source and the fallback. Coverage in the other locales is uneven —
`es` and `pt-BR` track English closely, while `de`, `fr`, `ja`, `ko` and
`zh-CN` still miss a large share of keys and resolve them through English.
Never quote numbers from this page: run the report.

```bash
python3 scripts/i18n_report.py            # per-locale + per-app coverage
python3 scripts/i18n_report.py --json
python3 scripts/i18n_report.py --app chat
```

A missing key is not a build error; it is a silent English fallback. That is
deliberate (the UI must never show `[key]` to an end user) but it means the
report, not the build, is how you find gaps.

---

## Tooling

Four scripts in `scripts/`, sharing one catalog reader (`i18n_lint.py`):

| Script | Purpose |
|--------|---------|
| `i18n_lint.py` | The gate. `--check` exits `1` on violations **beyond the baseline** (`.github/i18n-baseline.json`, which may only shrink). `--json` for machines, `--write-baseline` to record current debt. |
| `i18n_extract.py` | Translator handoff: emits the keys a locale is missing or whose placeholders disagree with `en`, as JSON or CSV. Never rewrites existing translations. |
| `i18n_report.py` | Coverage: keys, missing vs `en`, duplicates, plural blocks per locale; used keys and gaps per app. Reports, does not gate. |
| `i18n_pseudo.py` | Builds a pseudo-locale (`⟦elongated⟧` values) under `/tmp/gb-i18n-pseudo` for overflow testing. Placeholders and variant labels are preserved; it is deliberately never installed under `botlib/locales`, or it would appear in the language picker. |

Lint violation kinds: `DUPLICATE`, `CONFLICT` (same key, different values),
`BAD_KEY`, `UNBALANCED` braces, `PLURAL_NO_OTHER`, `MISSING` (a key the UI asks
for that `en` does not define), `PLACEHOLDER_MISMATCH`.

The `MISSING` scan reads every `*.html`/`*.js` under `botui/ui` **except two
subtrees that do not consume this catalogue**: `vendor/` (Monaco and other
third-party bundles call their own `localize()` with their own keys) and
`wizard/` (the setup wizard ships its own dictionaries under
`botui/ui/wizard/i18n/<locale>.js` and uses no `data-i18n`). Scanning them
reported 239 phantom gaps — 195 wizard, 44 vendor — that no translator could
ever close. With the scope fixed the baseline is empty: the catalogue has no
duplicate, conflicting, unbalanced or missing key.

### The CI job

`.github/workflows/ci.yml` defines the `i18n` job ("i18n catalog gate"), which
runs on GitHub-hosted runners and does exactly two things:

```yaml
- run: python3 scripts/i18n_lint.py --check
- run: python3 scripts/i18n_pseudo.py --check
```

`i18n_extract.py` and `i18n_report.py` are developer tools, not gates.

Run them locally before pushing:

```bash
python3 scripts/i18n_lint.py --check        # must exit 0
python3 scripts/i18n_pseudo.py --check      # must print "pseudo-locale check: OK"
```

---

## Recipe: add a key

1. Author it in English first, in the topic file that matches the feature:

   ```ftl
   # botlib/locales/en/settings.ftl
   settings-language-changed = Language changed
   settings-items-selected = { $count ->
       [one] { $count } item selected
      *[other] { $count } items selected
   }
   ```

2. Add the same key to each locale you support (at minimum `pt-BR`; the linter
   reports the rest through `i18n_report.py`):

   ```bash
   python3 scripts/i18n_extract.py --locale pt-BR --format csv --output /tmp/pt-BR.csv
   ```

3. **No Rust change is required.** `get_translations_json`
   (`botcore/src/i18n/translations/mod.rs`) serves the union of its explicit
   key list and `botlib::i18n::message_ids()`, i.e. every key authored in the
   English `.ftl` files. A key that exists only in an `.ftl` file still reaches
   the browser.

4. Use it in markup or JS:

   ```html
   <span data-i18n="settings-language-changed"></span>
   <span data-i18n="settings-items-selected" data-i18n-params='{"count": 3}'></span>
   ```

   ```javascript
   el.textContent = window.gbT("settings-language-changed", "Language changed");
   ```

5. Gate it:

   ```bash
   python3 scripts/i18n_lint.py --check
   ```

---

## Recipe: add a language

`xx` is the new BCP-47 code (e.g. `it`, `pt-PT`, `ar`).

```bash
mkdir -p botlib/locales/xx
cp botlib/locales/en/*.ftl botlib/locales/xx/
# translate the values in place — keep keys, placeholders and [one]/[other] labels
python3 scripts/i18n_report.py            # xx appears with its coverage
python3 scripts/i18n_lint.py --check      # no duplicates, no placeholder mismatch
```

Then:

1. **Rebuild.** The catalogue is embedded with `rust-embed`, so `available_locales()`
   only grows after a compile.
2. **Picker label** — add the native name to `NATIVE_NAMES` in
   `botui/ui/cloud/js/i18n-switcher.js` (`it: "Italiano"`); without it the
   switcher shows the raw code.
3. **Early-boot fallback** — `BUNDLED_LOCALES` in
   `botserver/crates/botcore/src/i18n/mod.rs` is used only before the bundles
   load (early boot, unit tests). Add the code for consistency.
4. Nothing else. The client fetches `/api/i18n/locales` and adopts whatever the
   server reports, so no frontend list needs editing.

Right-to-left languages need no extra file: `i18n.js` and `i18n-lite.js` carry
an `RTL_LANGUAGES` list (`ar`, `he`, `fa`, `ur`) and set `<html dir="rtl">`.

---

## What is deliberately not localized (yet)

- **Bot prompt files.** `load_system_prompt_for_channel`
  (`botserver/src/core/bot/ws/message.rs`) reads `PROMPT-{CHANNEL}.md`, then
  `PROMPT.md`/`prompt.md`/`PROMPT.txt`. There is **no locale dimension** in the
  file names: a `PROMPT-pt-BR.md` is never read. The *language of the answer* is
  not left to the authored text either — the loader appends the
  response-language contract
  (`RESPONSE_LANGUAGE_INSTRUCTION`: "Always answer in the user's language …",
  skipped when the authored prompt already states it), so a custom bot without
  any language instruction still answers in the language the user wrote.
- **Suggestion and switcher labels.** `ADD SUGGESTION … AS "<key>"` and
  `ADD SWITCHER … AS "<key>"` accept a catalogue key: the runtime resolves it
  for the session locale (session `context_data.locale` → bot `locale`
  configuration → `en`) and keeps an unknown string exactly as authored, so
  custom bots keep their own wording. The shipped templates under
  `bottemplates/**/start.bas` use `bot-suggest-*` keys from
  `botlib/locales/<locale>/bot-templates.ftl`.
- **Log lines.** Only human-facing CLI and UI text is translated; log output
  stays English so it remains greppable (`botcore/src/i18n/cli.rs`).
- **Machine-readable payloads.** JSON keys, provider codes and
  OpenAI-compatible error envelopes keep their shape; only the human-readable
  text is localized.
- **Channel identity data.** WhatsApp/Telegram give a phone or chat id, not a
  `users` row, so those adapters resolve the *bot's* configured locale with
  `botlib::i18n::locale_from_config` instead of a per-person preference.

---

## See also

- [i18n Server Surface](./server.md) — routes, negotiation, `LocalizedError`
- [i18n Client Runtime](./client.md) — `window.i18n`, attributes, formatters
- [i18n API](../../08-rest-api-tools/i18n-api.md) — the HTTP contract
- [Settings - User Preferences](../apps/settings.md) — the language picker
