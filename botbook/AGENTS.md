# Maintaining BotBook

This book documents software that changes every week. The job is not to write
prose about General Bots; it is to make every sentence traceable to a file in
this repository, and to remove what cannot be traced.

The rule that governs everything below: **a claim that cannot be verified
against the code is deleted, not softened.**

## The one mechanism to understand first

mdBook renders **only the pages listed in `src/SUMMARY.md`**. A markdown file
that exists in the repository but is absent from the table of contents is never
converted to HTML, so:

- it does not appear in the sidebar, and
- every link pointing at it returns the 404 fallback for real readers.

Checking links against the filesystem therefore proves nothing. A previous
review pass reported "0 dead links" while 497 links pointed at 209 pages that
mdBook never published. Always check against `SUMMARY.md`:

```bash
python3 scripts/docs_link_check.py     # MISSING and UNPUBLISHED, exit 1 on failure
python3 scripts/docs_summary_sync.py --check   # every page listed in SUMMARY.md
```

## Workflow

1. **Open a tracking issue** with the scope and the numbers you measured, so a
   long review stays reviewable. Close it only with evidence.
2. **Locate the authority** for each claim (table below). Read the file.
3. **Write from the source**, not from the existing page. Existing prose is the
   thing under review.
4. **Run the guards** (all of them, from the repository root).
5. **Build and inspect** the rendered page, not just the markdown.
6. **Commit per issue** with the measured numbers in the message.

## Where the truth lives

| Claim | Authority |
|---|---|
| Which apps exist, their titles/categories | `botserver/src/apps/registry.rs` |
| What an app's UI actually offers | `botui/ui/suite/<app>/` — the `.html` plus `modules/*.js` |
| Dependency versions (Postgres, Valkey, Vault, …) | `botserver/3rdparty.toml` |
| Product version | `Cargo.toml` at the repository root |
| Available BASIC keywords | `botserver/src/basic/keywords/` |
| Retrieval behaviour and configuration keys | `botserver/src/core/bot/kb_context/` |
| Crate layout and responsibilities | `botserver/crates/*/` |
| Live hostnames and routing | Caddy config on the `proxy` container |

If a page quotes a port, a flag name, a model name or a default value and you
cannot reproduce it from one of these, treat the sentence as wrong.

## Investigation techniques that actually resolve pages

- **Read the app's own strings.** Grep the app's HTML/JS for button labels,
  empty-state copy and menu names. That vocabulary is the real UI; do not invent
  menus that match the general idea of the app.
- **Look for consumers before documenting a capability.** A hybrid-search
  engine, a tuner or a config key with no callers outside its own crate is dead
  code. Documenting it as a feature sends users to settings that do nothing —
  say it is unused instead.
- **Confirm a feature gate gates something.** A flag read into a variable that
  nothing branches on is not a gate.
- **Prefer the manifest to prose** for anything versioned, then sync with
  `scripts/docs_component_versions.py`.
- **Check artwork the way you check prose.** Screen mockups carry dates and
  version badges that go stale: verify a calendar grid has the right number of
  days and that its weekdays line up, and verify version labels against the
  manifest.
- **Cut unsupported specificity.** Model names, chunk sizes, top-k values,
  thresholds and "the system uses X internally" are the most commonly fabricated
  sentences in this book. If the number is not in the code, the sentence goes.
- **Fabricated keywords and endpoints.** Search for the keyword or route before
  documenting it (`ADD_SELECTION` was documented for months and exists nowhere).

## Conventions

**Maturity marker in every app page title.** The title carries the status, so a
reader knows before opening the page:

| Marker | Meaning | Examples |
|---|---|---|
| `🟢 GA` | shipped and supported | Chat, Drive, Vibe |
| `🟡 PREVIEW (advanced)` | usable, still changing | Mail, Sheet |
| `🟡 PREVIEW — IN TEST` | under active development | Docs, Slides |
| `🟡 PREVIEW` | everything else in the catalog | most apps |

**Screens.** One SVG per app in `src/assets/suite/`, referenced relatively with
descriptive alt text on its own line. Do not leave an asset orphaned: an SVG
that no page references is invisible to readers.

**Links.** Relative, to a unique real path. Never point at `README.md` directly
— the book cover is not published; link to `./introduction.md` or the chapter
overview instead.

**No ASCII mockups where an SVG exists.** Prefer `src/assets/suite/*.svg`.

**Style.** Formal language, no slang. Name the mechanism and the trade-off.
Where a capability is not implemented, say so in a short table rather than
leaving the reader to discover it.

## Guards

Run from the repository root. All five must pass before committing.

| Script | Checks | Exit code |
|---|---|---|
| `scripts/docs_link_check.py` | every internal link resolves to a published page | 1 on any broken link |
| `scripts/docs_summary_sync.py --check` | every page is registered in `SUMMARY.md` | 1 when pages are missing |
| `scripts/docs_component_versions.py --check` | documented versions match `3rdparty.toml` | 1 on drift |
| `scripts/docs_suite_apps_status.py` | regenerates the Suite Apps status page from the registry | 0 (writes the page) |
| `scripts/docs_suite_svg_audit.py` | text inside the canvas, plus version claims in the SVG screens | 1 on drift |
| `scripts/docs_app_screens.py --check` | every app screen extracts from the registry | 1 on a failed app |

`docs_summary_sync.py` without `--check` appends the missing entries to
`SUMMARY.md`; `docs_suite_svg_wire.py` wires orphaned screens into their pages.

## Regenerating the app screens

Do not redraw an app screen by hand. `scripts/docs_app_screens.py` builds each
one from the app's own markup, so a screen that disagrees with the product is a
generator bug rather than an artwork task.

```bash
python3 scripts/docs_app_screens.py                # rewrite all 73 screens
python3 scripts/docs_app_screens.py tasks drive    # rewrite a subset
python3 scripts/docs_app_screens.py --report       # extraction summary only
```

Its two inputs are authoritative and must not be replaced with copies:

1. **`botserver/src/apps/registry.rs`** — the app list the suite renders. It is
   read directly; there is no committed catalogue to drift from it.
2. **`botui/ui/suite/<app>/`** markup, following `hx-get` partials, and
   **`botlib/locales/en/ui.ftl`** for every `data-i18n` label.

Conventions the generator follows, and that a replacement must keep:

- **Row and card bodies stay neutral.** Region names, tabs, filters, columns and
  actions are real; record payloads are skeleton blocks. Never invent records.
- **`display:none` is not absent.** Suite apps ship shells hidden and reveal
  them on hydration, so those containers are the real interface.
- **Modals are not the surface.** `TRANSIENT` excludes dialogs, masks and
  drawers: an app is documented by its shell, not by a dialog.
- **A conversation surface keeps its declared thread** instead of falling back
  to recovered script markup.

After regenerating, run the guards above and review the diff: a screen that
loses real labels has regressed, even if it still parses.

## Build and read the result

```bash
cargo install mdbook          # or unpack a prebuilt release into /tmp
cd botbook && mdbook build    # output in botbook/book/ (git-ignored)
find book -name '*.html' | wc -l
```

Open the pages you changed. A rendered page with the wrong title, a duplicated
heading or a broken screen tells you more than the markdown diff does.

## Publishing to docs.generalbots.org

The site is static, served by Caddy inside the `proxy` container from
`/opt/gbo/data/websites/docs.generalbots.org`. There is no CI for the book; the
build is produced locally and swapped in.

```bash
# 1. build
cd botbook && mdbook build
tar -czf /tmp/docs-book.tar.gz -C book .

# 2. upload and swap (SRV1_HOST/SRV1_PASS from ~/.prod)
sshpass -p "$SRV1_PASS" ssh root@"$SRV1_HOST" "cat > /tmp/docs-book.tar.gz" < /tmp/docs-book.tar.gz
incus file push /tmp/docs-book.tar.gz proxy/tmp/
# stage, assert the page count, then move it into place — never extract over the live root

# 3. verify the deployed bytes are the bytes you built
sha256sum book/index.html
curl -s https://docs.generalbots.org/ | sha256sum
```

Caddy's site block for this host is in `/opt/gbo/conf/config` inside the
`proxy` container. Validate before reloading, always:

```bash
incus exec proxy -- /opt/gbo/bin/caddy validate --adapter caddyfile --config /opt/gbo/conf/config
incus exec proxy -- /opt/gbo/bin/caddy reload  --adapter caddyfile --config /opt/gbo/conf/config
```

**Do not trust a 200 alone.** The site block ends with
`try_files {path} {path}/ /index.html`, so a missing page returns the home page
with status 200. Compare the `<title>` of the page you requested against the
home page, or compare checksums.

## Failure modes this review actually found

Keeping these in mind is cheaper than rediscovering them.

1. **Pages written but never published** — 209 of them, because `SUMMARY.md` was
   never extended. Old build: 441 HTML files. Current: 546.
2. **A link check that measured the filesystem** instead of the published book,
   so the breakage above looked like success.
3. **Fabricated implementation detail** — BGE embeddings, 500-token chunks, HNSW
   parameters, top-k and thresholds that appear nowhere in the code.
4. **Dead code documented as a feature** — a BM25 tuner and a query decomposer
   with no consumers.
5. **Wrong stack claims** — the roadmap said Actix-Web and SQLx; the server is
   Axum and Diesel.
6. **Stale versions in prose and in artwork** — Vault 1.15.4 vs 1.18.2, Rust
   1.78 vs 1.97, a `v6.4.0` badge against 6.3.1.
7. **Empty stubs** — files containing a single heading, referenced from a
   chapter index. Deleted.
8. **A vhost pointing at a renamed directory**, which took the whole site down
   with a Caddy 404 that a status-code-only check would have accepted.
