# Locale catalogues

One directory per locale. The directory name **is** the locale: the loader
lists the directories it finds, so a locale exists if and only if its directory
exists. There is no second list to keep in sync.

```
botlib/locales/
├── en/        ← the source of truth: every key is authored here first
├── it/        ru/   pt-BR/   es/   fr/   de/   ja/   ko/   zh-CN/
```

Italian (`it`), Russian (`ru`) and Korean (`ko`) are full catalogues: `it` and
`ru` ship all English topic files and `ko` has no gaps against `en` (issue
#2970). `locale_meta.rs` supplies their picker names, direction and CLDR
formats; the picker itself lists them in `language-selector.html` and
`02_language.html`.

Each directory holds topic files (`admin.ftl`, `common.ftl`, `errors.ftl`,
`suite-*.ftl`, and the fifteen `ui-*.ftl` files split out of the old
`ui.ftl`). The filename carries no meaning to the loader: it parses every
`*.ftl` in the directory and merges them into one flat map, so **adding a topic
file needs no code change and no registration**.

## Rules

1. **English is the source of truth.** A key is added to `en/` first, in the
   topic file that owns its surface. Other locales never invent keys.
2. **Kebab-case keys**, one definition per line: `^[a-z0-9]+(-[a-z0-9]+)*$`.
   The linter rejects anything else as `BAD_KEY`.
3. **No duplicate keys.** The merge is a `HashMap::extend`, and directory
   iteration order is the OS's, so a duplicate resolves *non-deterministically*.
   The linter rejects `DUPLICATE` and `CONFLICT` outright.
4. **No `data-i18n` key without an `en` definition.** A missing key renders as
   the raw key in the UI.
5. **Plurals are CLDR Fluent blocks** with an explicit `*[other]` branch:
   ```
   tasks-count = { $count ->
       [zero] No tasks
       [one] { $count } task
      *[other] { $count } tasks
   }
   ```
   Locales without a plural distinction (ja, ko, zh-CN) keep only `*[other]`;
   the fallback is part of the contract, not an accident.
6. **Keep every file under 450 lines** (repository limit). Split by surface,
   not by size: a new `ui-<surface>.ftl` is a better answer than a 600-line
   `ui.ftl`.
7. **No secrets, hosts or personal data** in a catalogue.

## Adding a key

```bash
# 1. author it in English, in the owning topic file
$EDITOR botlib/locales/en/ui-drive.ftl

# 2. translate it in every shipped locale that has the file
$EDITOR botlib/locales/pt-BR/ui-drive.ftl
#    … and es, fr, de, ja, ko, zh-CN

# 3. gate
python3 scripts/i18n_lint.py --check     # must exit 0
```

`python3 scripts/i18n_extract.py --locale pt-BR` prints exactly the keys a
locale is missing (or whose placeholders disagree with `en`), as JSON or CSV —
that is the translator handoff, and it never rewrites existing translations.

## Adding a locale

1. `mkdir botlib/locales/<bcp-47>` and copy the English topic files.
2. Translate the values; keep the keys, the placeholders and the plural
   branches unchanged.
3. Add the presentation metadata (native name, direction, CLDR date and number
   patterns) in `botserver/crates/botcore/src/i18n/locale_meta.rs`. Without an
   entry the locale still ships — the API lists it with the English formats and
   `ltr` — but the picker shows the bare code.
4. `python3 scripts/i18n_lint.py --check` and
   `python3 scripts/i18n_pseudo.py --check`.

## Tooling

| Command | Purpose |
|---------|---------|
| `python3 scripts/i18n_lint.py --check` | The gate: duplicates, conflicts, bad keys, unbalanced braces, plurals without `*[other]`, keys the UI uses but `en` lacks, placeholder drift. Runs in CI (`.github/workflows/ci.yml`, job `i18n`). |
| `python3 scripts/i18n_report.py` | Coverage: keys, missing vs `en`, duplicates, plural blocks per locale, plus used/missing keys per app. Reports only. |
| `python3 scripts/i18n_extract.py --locale <code>` | Translator handoff (JSON/CSV). |
| `python3 scripts/i18n_pseudo.py` | Builds a pseudo-locale under `/tmp/gb-i18n-pseudo` for overflow testing. Never installed here — it would appear in the language picker. |

The gate's baseline (`.github/i18n-baseline.json`) is **shrink-only**: a
violation it records passes, a new one fails the build. It is empty today.

## Translation coverage (measured 2026-11-27)

`en` defines 12,855 keys. The per-locale gaps are recorded here so they are
explicit rather than silent; they are **not** gated yet, and
`scripts/i18n_report.py` is the command that measures them.

| Locale | Keys | Missing vs `en` | Where |
|--------|------|-----------------|-------|
| `en`, `it`, `ru`, `pt-BR`, `ko` | 12,855 | 0 | — |
| `es` | 12,772 | 83 | `errors` 30, `ui-chat` 20, `ui-productivity` 12, `suite-apps-extra` 6, `ui-apps-runtime` 5 |
| `fr`, `de`, `ja`, `zh-CN` | 12,583 | 272 each | `ui-apps-runtime` 194, `errors` 30, `ui-chat` 20, `ui-productivity` 12, `suite-apps-extra` 6 |

Italian and Russian were added and Korean's 272 gaps closed under issue #2970;
English fallbacks still cover the remaining gaps in the other locales. The
background for that work is the i18n epic (#2096) and its content issue
(#2095). A key with no translation falls back to English at runtime, which is
why a gap is invisible in the UI until someone reads the locale.
