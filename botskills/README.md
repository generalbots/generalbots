# botskills

Curated collection of LLM agent skills for General Bots, organized like
`bottemplates/`: each skill is a self-contained folder with a `SKILL.md`
front-matter manifest (name, description, instructions) plus optional
support scripts and references.

Every skill was selected from public skill repositories, classified by
domain, and trimmed of test fixtures. Sources are identified per skill in
the catalog below and pinned by commit at the bottom of this file.

## Layout

```
botskills/
├── automation/      Browser and task automation
├── documentation/   Docs and technical writing
├── engineering/     Debugging, verification, optimization
├── frontend/        UI frameworks and schema validation
├── integration/     MCP servers and skill authoring
├── operations/      Monitoring and incident tooling
├── research/        Web search, scraping, docs lookup
├── testing/         Unit and web application testing
│   # content-generation pack (#2063)
├── office/          Word, Excel, PowerPoint, PDF, conversion
├── design/          Brand, layout, diagrams, icons, colour, type
├── data/            Schema, queries, pipelines, analysis
├── content/         Copy, SEO, social, scripts, localisation
├── quality/         Testing, review, profiling, security, release
├── business/        Finance, tax, contracts, HR, compliance, support
└── media/           Image, audio, video, captions, thumbnails
```

## Catalog — classified, source identified

| Category | Skill | Purpose | Source |
|----------|-------|---------|--------|
| automation | agent-browser | Browser automation for testing, form filling, screenshots, data extraction | pedronauck/skills (curated) |
| documentation | documentation-writer | Diátaxis-style technical documentation authoring | pedronauck/skills (curated) |
| engineering | systematic-debugging | Root-cause-first workflow for any bug, test failure, or unexpected behavior | pedronauck/skills (curated) |
| engineering | verification-before-completion | Mandatory verification pass before claiming work complete | pedronauck/skills (curated) |
| engineering | [extreme-software-optimization](https://jeffreys-skills.md/skills/extreme-software-optimization) | Performance and binary-size optimization techniques | Jeffrey's Skills (subscription only — listed by name/link, files not included) |
| frontend | next-best-practices | Next.js conventions: RSC boundaries, data patterns, async APIs, metadata | pedronauck/skills (curated) |
| frontend | shadcn | Building and styling UI components with shadcn/ui and Radix primitives | pedronauck/skills (curated) |
| frontend | zod | Zod schema validation patterns for type safety and parsing | pedronauck/skills (curated) |
| integration | mcp-builder | Creating high-quality MCP servers with well-designed tools | Prat011/awesome-llm-skills |
| integration | skill-creator | Authoring new SKILL.md skills with validation and packaging scripts | Prat011/awesome-llm-skills |
| operations | sentry-cli | Querying issues, events, and releases from the command line | pedronauck/skills (curated) |
| research | context7 | Up-to-date library documentation lookup | pedronauck/skills (curated) |
| research | exa-web-search-free | Free AI web and code search via Exa MCP (no API key) | pedronauck/skills (curated) |
| research | firecrawl | Web scraping and crawling for context gathering | pedronauck/skills (curated) |
| research | last30days | Recency research: what was said about a topic in the last 30 days across Reddit, X, YouTube, Hacker News, Polymarket, GitHub and the web | mvanhorn/last30days-skill |
| testing | vitest | Fast unit testing with Jest-compatible API, mocking, and coverage | pedronauck/skills (curated) |
| testing | webapp-testing | Playwright toolkit for local web app testing, screenshots, and console logs | Prat011/awesome-llm-skills |

## Content-generation pack (#2063)

The issue's tables sum 96 unique names, all of which ship here: 11 already
existed in the original categories (`systematic-debugging`,
`verification-before-completion`, `next-best-practices`, `shadcn`, `zod`,
`mcp-builder`, `skill-creator`, `context7`, `exa-web-search-free`, `firecrawl`,
`last30days`) and 85 are new packages in the seven added categories, for 101
packages in total together with the five pre-existing skills outside the
issue's list (`agent-browser`, `documentation-writer`, `sentry-cli`, `vitest`,
`webapp-testing`). All are authored in this repository (`source: in-repo`)
with the same `SKILL.md` contract:

```
---
name: docx
description: |
  Create and edit Word documents: styles, TOC, page setup, headers...
version: 1.0.0
source: in-repo
---

# docx
## Workflow
## Quality checks
## Output
```

Install them all at once (the endpoint stores `enabled = TRUE`):

```bash
scripts/install-skills.sh --token "$TOKEN"          # --dry-run to preview
```

The prompt advertises every enabled skill as one `name: description` line and
the model pulls the method with `skills.find {query}` — bodies never enter the
system prompt. See "Runtime contract" below.

| Category | Skills |
|----------|--------|
| office | docx, xlsx, pptx, pdf, html-to-pdf, docx-track-changes, spreadsheet-modeling, mail-merge, document-a11y, office-convert |
| design | canvas-design, brand-guidelines, figma-workflow, diagrams, svg-icons, color-systems, typography, presentation-design, image-editing, logo-identity |
| frontend | react-best-practices, next-best-practices, tailwind, shadcn, zod, web-accessibility, web-performance, state-management, forms-validation |
| data | postgres, sqlalchemy, prisma, api-design, auth-sessions, queues-jobs, caching, migrations, etl-pipelines, pandas-analysis |
| content | copywriting, seo-on-page, blog-outline, newsletter, social-posts, ad-copy, video-script, podcast-script, translation, brand-tone |
| research | exa-web-search-free, firecrawl, context7, last30days, research-synthesis, fact-checking, citations, competitive-analysis, pricing-research, news-briefing |
| integration | mcp-builder, skill-creator, webhooks, slack-workflows, gmail-workflows, calendar-scheduling, crm-actions, web-scraping |
| quality | tdd, code-review, refactoring, performance-profiling, security-review, logging-observability, release-management (systematic-debugging and verification-before-completion already live under `engineering/`) |
| business | finance-reporting, invoicing-brazil, contracts-review, hr-onboarding, kyc-verification, compliance-iso27001, tax-brazil, procurement, customer-support, analytics-kpis |
| media | image-generation, audio-generation, video-generation, voice-tts, subtitles-captions, thumbnails, memes, gif-creator, music-generation, upscaling-restoration |

Media and ML execution stays in BotModels (see #2062); the media skills
describe the request shape and the verification, not a local model stack.

### Runtime contract

- `POST /api/sources/skills` parses the front-matter (including `description: |`
  block scalars), hashes the document and stores it with `enabled = TRUE`.
- The prompt lists every enabled skill as `name: description` — no `LIMIT`, no
  body excerpt — so the block grows by one short line per skill.
- `skills.list` returns names, descriptions and the enabled flag.
- `skills.find {query}` returns the matching package bodies (bounded), which is
  how the model gets the method without a giant system prompt.

## Why the original sixteen

Selection criteria, in order:

1. **Fit to the General Bots workflow** — skills that map to real daily work
   in this repo: Rust + Rhai development, HTMX frontends, browser testing via
   CDP, MinIO/Vault/PostgreSQL operations, and CI/CD discipline.
2. **No external accounts required** — every skill works out of the box;
   the only network-dependent ones (context7, exa, firecrawl, Sentry) degrade
   to documented manual procedures when the service is unreachable.
3. **One skill per concern** — no overlapping duplicates were imported
   (for example, the four web-search candidates were reduced to the two
   non-overlapping winners: exa for search, firecrawl for scraping).

### Note on `last30days`

`last30days` is the one skill with a real external-account requirement. It runs
keyless at a reduced floor (its own engine's `available_sources`, minus
login-backed sources); the full set — TikTok, Instagram, X, and the optional
Brave/Perplexity/OpenRouter backends — needs credentials, 15 optional keys in
total. It was accepted because criterion 2 is met *degraded*, not met in full,
and because its concern is genuinely distinct from criterion 3's other research
skills: those answer "what does this mean", this answers "what did people say
recently". It overlaps `exa` only for general web search.

Two behaviours worth knowing before enabling it, both documented by the author:

- **Browser cookies are opt-in.** Reads require `FROM_BROWSER` or explicit
  setup consent; `--preflight` is a permission inspector that does not read
  cookie values.
- **It talks to third-party APIs.** `api.scrapecreators.com`, `api.x.com`,
  `hn.algolia.com`, `gamma-api.polymarket.com`, and optionally
  `api.openai.com` / `api.perplexity.ai`. Its SKILL.md states it never posts,
  likes, or modifies content on any platform. Given General Bots' sovereignty
  posture, decide deliberately before enabling it for a given bot.

The 14MB `assets/` directory (demo images and audio) was excluded as media, not
skill behaviour.

## Selection from the source repos

- **pedronauck/skills** contributes 12 of 23 curated skills (community and
  deprecated trees were reviewed; nothing met criterion 1 without heavy
  modification).
- **Prat011/awesome-llm-skills** contributes 3 of 29 skills (the remainder are
  platform-specific SaaS integrations — Notion, Slack, invoice pipelines —
  outside General Bots scope).
- **mvanhorn/last30days-skill** contributes 1 of 1 skill, MIT-licensed, imported
  with `assets/` trimmed and the licence retained as `LICENSE.txt`.
- Test fixtures shipped inside upstream skills (`test-pressure-*.md`,
  `test-academic.md`) were removed; they are evaluation artifacts, not
  skill content.

### Considered and rejected

**1N3/Sn1per** (backlog #1484) was evaluated and **not** imported. Three
independent reasons, any one sufficient:

1. **It is not a skill.** No `SKILL.md`, no agent/skill/MCP manifest of any
   kind — it is a 6.7MB self-hosted offensive-security application (`bin/`,
   `sniper`, `Dockerfile`, `wordlists/`, `templates/`). There is nothing to
   import into this layout.
2. **Its EULA forbids what this library does.** Not an OSI licence but a
   six-term custom EULA, including *"You agree not to create any product or
   service from any par of the Code from this Project, paid or free"* (3) and
   *"You agree not to re-license the Code"* (4). Term 3 rules out shipping it
   in General Bots, which sells paid plans; term 4 conflicts with this
   repository's MIT licensing.
3. **It is offensive-security tooling** — recon, scanning and exploitation.
   General Bots sells CRM, Chat, Mail and Drive to schools and small business.

If offensive-security capability is wanted in the library, the correct form is a
General-Bots-authored skill documenting authorised pentest methodology, scoped
and defensive — not a vendored platform.

## Source pins

| Repository | Commit |
|------------|--------|
| https://github.com/pedronauck/skills | `233da80` (2026-09-13) |
| https://github.com/Prat011/awesome-llm-skills | `35e1ea2` (2026-09-13) |
| https://github.com/mvanhorn/last30days-skill | `625f5b1` (2026-10-07) |

The two pre-existing upstream repositories had no SPDX license declared at pin
time; the skills carry their own `LICENSE.txt` where the upstream author
provided one (Apache-2.0 for the Anthropic-derived skills).
`last30days-skill` declares **MIT** (Copyright 2026 Matt Van Horn), verified
against its `LICENSE` at the pin above. Re-verify licenses before commercial
redistribution.

## Usage

Skills are plain folders — copy the category/skill directory into the agent's
skill discovery path, or reference `SKILL.md` directly. See
`bottemplates/` for the analogous bot package convention and the General
Bots documentation for `.gbdialog` tool integration.
