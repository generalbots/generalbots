# Suite Apps — Documentation Coverage Status 🟡 BETA

This page tracks which Suite applications have documentation in the botbook and their current status.

## Coverage Overview

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 340" font-family="system-ui, sans-serif">
  <!-- Background -->
  <rect width="600" height="340" fill="#0d1117" rx="12"/>

  <!-- Title -->
  <text x="300" y="35" text-anchor="middle" fill="#e6edf3" font-size="18" font-weight="bold">Suite Documentation Coverage</text>

  <!-- Donut chart -->
  <g transform="translate(160,180)">
    <!-- Total arc (missing = 20 apps, documented = 21 apps, total = 41) -->
    <!-- Missing segment: 20/41 = 48.8% → 175.6° -->
    <!-- Documented segment: 21/41 = 51.2% → 184.4° -->

    <!-- Missing (red) -->
    <circle cx="0" cy="0" r="90" fill="none" stroke="#f85149" stroke-width="36"
      stroke-dasharray="153.3 412.7" stroke-dashoffset="0" transform="rotate(-90)"/>
    <!-- Documented (green) -->
    <circle cx="0" cy="0" r="90" fill="none" stroke="#3fb950" stroke-width="36"
      stroke-dasharray="160.4 405.6" stroke-dashoffset="-153.3" transform="rotate(-90)"/>

    <!-- Center text -->
    <text x="0" y="-8" text-anchor="middle" fill="#e6edf3" font-size="32" font-weight="bold">51%</text>
    <text x="0" y="16" text-anchor="middle" fill="#8b949e" font-size="13">covered</text>
  </g>

  <!-- Legend -->
  <g transform="translate(330,120)">
    <rect x="0" y="0" width="14" height="14" rx="3" fill="#3fb950"/>
    <text x="22" y="12" fill="#e6edf3" font-size="13">Documented (21 apps)</text>

    <rect x="0" y="28" width="14" height="14" rx="3" fill="#f85149"/>
    <text x="22" y="40" fill="#e6edf3" font-size="13">Missing (20 apps)</text>

    <rect x="0" y="56" width="14" height="14" rx="3" fill="#8b949e"/>
    <text x="22" y="68" fill="#e6edf3" font-size="13">Total: 41 apps in suite</text>
  </g>

  <!-- Bar summary -->
  <g transform="translate(330,210)">
    <text x="0" y="0" fill="#8b949e" font-size="11" text-transform="uppercase">By Priority</text>

    <rect x="0" y="12" width="80" height="12" rx="3" fill="#f85149" opacity="0.3"/>
    <rect x="0" y="12" width="60" height="12" rx="3" fill="#f85149"/>
    <text x="66" y="22" fill="#e6edf3" font-size="11">High: 3 missing</text>

    <rect x="0" y="32" width="80" height="12" rx="3" fill="#d29922" opacity="0.3"/>
    <rect x="0" y="32" width="53" height="12" rx="3" fill="#d29922"/>
    <text x="66" y="42" fill="#e6edf3" font-size="11">Medium: 11 missing</text>

    <rect x="0" y="52" width="80" height="12" rx="3" fill="#3fb950" opacity="0.3"/>
    <rect x="0" y="52" width="47" height="12" rx="3" fill="#3fb950"/>
    <text x="66" y="62" fill="#e6edf3" font-size="11">Low: 6 missing</text>
  </g>
</svg>

## App Status Table

| # | App | In Suite | Doc Exists | Status | Priority |
|---|-----|:--------:|:----------:|--------|----------|
| 1 | **analytics** | ✅ | ✅ | 🟢 Done | — |
| 2 | **billing** | ✅ | ✅ | 🟢 Done | — |
| 3 | **calendar** | ✅ | ✅ | 🟢 Done | — |
| 4 | **chat** | ✅ | ✅ | 🟢 Done | — |
| 5 | **crm** | ✅ | ✅ | 🟢 Done | — |
| 6 | **dashboards** | ✅ | ✅ | 🟢 Done | — |
| 7 | **designer** | ✅ | ✅ | 🟢 Done | — |
| 8 | **drive** | ✅ | ✅ | 🟢 Done | — |
| 9 | **mail** | ✅ | ✅ | 🟢 Done | — |
| 10 | **meet** | ✅ | ✅ | 🟢 Done | — |
| 11 | **paper** | ✅ | ✅ | 🟢 Done | — |
| 12 | **player** | ✅ | ✅ | 🟢 Done | — |
| 13 | **products** | ✅ | ✅ | 🟢 Done | — |
| 14 | **research** | ✅ | ✅ | 🟢 Done | — |
| 15 | **sources** | ✅ | ✅ | 🟢 Done | — |
| 16 | **suite** | ✅ | ✅ | 🟢 Done | — |
| 17 | **tasks** | ✅ | ✅ | 🟢 Done | — |
| 18 | **tickets** | ✅ | ✅ | 🟢 Done | — |
| 19 | **vibe** | ✅ | ✅ | 🟢 Done | — |
| 20 | **compliance** | — | ✅ | 🟢 Done | — |
| 21 | **compliance-api** | — | ✅ | 🟢 Done | — |
| 22 | **o365** | ✅ | ✅ | 🟢 Done | — |
| 22 | **admin** | ✅ | ❌ | 🔴 Missing | 🔴 High |
| 23 | **settings** | ✅ | ❌ | 🔴 Missing | 🔴 High |
| 24 | **monitoring** | ✅ | ❌ | 🔴 Missing | 🔴 High |
| 25 | **attendant** | ✅ | ❌ | 🔴 Missing | 🟡 Medium |
| 26 | **docs** | ✅ | ❌ | 🔴 Missing | 🟡 Medium |
| 27 | **sheet** | ✅ | ❌ | 🔴 Missing | 🟡 Medium |
| 28 | **slides** | ✅ | ❌ | 🔴 Missing | 🟡 Medium |
| 29 | **canvas** | ✅ | ❌ | 🔴 Missing | 🟡 Medium |
| 30 | **campaigns** | ✅ | ❌ | 🔴 Missing | 🟡 Medium |
| 31 | **goals** | ✅ | ❌ | 🔴 Missing | 🟡 Medium |
| 32 | **project** | ✅ | ❌ | 🔴 Missing | 🟡 Medium |
| 33 | **people** | ✅ | ❌ | 🔴 Missing | 🟡 Medium |
| 34 | **learn** | ✅ | ❌ | 🔴 Missing | 🟢 Low |
| 35 | **social** | ✅ | ❌ | 🔴 Missing | 🟢 Low |
| 36 | **workspace** | ✅ | ❌ | 🔴 Missing | 🟢 Low |
| 37 | **browser** | ✅ | ❌ | 🔴 Missing | 🟢 Low |
| 38 | **terminal** | ✅ | ❌ | 🔴 Missing | 🟢 Low |
| 39 | **video** | ✅ | ❌ | 🔴 Missing | 🟢 Low |
| 40 | **templates** | ✅ | ❌ | 🔴 Missing | 🟢 Low |
| 41 | **tools** | ✅ | ❌ | 🔴 Missing | 🟢 Low |
| 42 | **lists** | ✅ | ❌ | 🔴 Missing | 🟢 Low |
| 43 | **about** | ✅ | ❌ | 🔴 Missing | ⚪ Infra |
| 44 | **auth** | ✅ | ❌ | 🔴 Missing | ⚪ Infra |

## Coverage by Category

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 360" font-family="system-ui, sans-serif">
  <rect width="600" height="360" fill="#0d1117" rx="12"/>
  <text x="300" y="30" text-anchor="middle" fill="#e6edf3" font-size="16" font-weight="bold">Coverage by App Category</text>

  <!-- Category bars -->
  <!-- Core (chat, drive, tasks, mail) = 4/4 = 100% -->
  <text x="20" y="70" fill="#8b949e" font-size="12">Core</text>
  <rect x="100" y="58" width="390" height="16" rx="4" fill="#1f2937"/>
  <rect x="100" y="58" width="390" height="16" rx="4" fill="#3fb950"/>
  <text x="500" y="70" fill="#3fb950" font-size="12" font-weight="bold">100%</text>
  <text x="105" y="70" fill="#0d1117" font-size="10" font-weight="bold">4/4</text>

  <!-- Productivity (calendar, meet, paper, research, products) = 5/5 = 100% -->
  <text x="20" y="100" fill="#8b949e" font-size="12">Productivity</text>
  <rect x="100" y="88" width="390" height="16" rx="4" fill="#1f2937"/>
  <rect x="100" y="88" width="390" height="16" rx="4" fill="#3fb950"/>
  <text x="500" y="100" fill="#3fb950" font-size="12" font-weight="bold">100%</text>
  <text x="105" y="100" fill="#0d1117" font-size="10" font-weight="bold">5/5</text>

  <!-- Office (docs, sheet, slides) = 0/3 = 0% -->
  <text x="20" y="130" fill="#8b949e" font-size="12">Office</text>
  <rect x="100" y="118" width="390" height="16" rx="4" fill="#1f2937"/>
  <rect x="100" y="118" width="0" height="16" rx="4" fill="#f85149"/>
  <text x="500" y="130" fill="#f85149" font-size="12" font-weight="bold">0%</text>
  <text x="105" y="130" fill="#8b949e" font-size="10">0/3</text>

  <!-- CRM/Sales (crm, billing, tickets) = 3/3 = 100% -->
  <text x="20" y="160" fill="#8b949e" font-size="12">CRM / Sales</text>
  <rect x="100" y="148" width="390" height="16" rx="4" fill="#1f2937"/>
  <rect x="100" y="148" width="390" height="16" rx="4" fill="#3fb950"/>
  <text x="500" y="160" fill="#3fb950" font-size="12" font-weight="bold">100%</text>
  <text x="105" y="160" fill="#0d1117" font-size="10" font-weight="bold">3/3</text>

  <!-- Dev Tools (designer, vibe, sources) = 3/3 = 100% -->
  <text x="20" y="190" fill="#8b949e" font-size="12">Dev Tools</text>
  <rect x="100" y="178" width="390" height="16" rx="4" fill="#1f2937"/>
  <rect x="100" y="178" width="390" height="16" rx="4" fill="#3fb950"/>
  <text x="500" y="190" fill="#3fb950" font-size="12" font-weight="bold">100%</text>
  <text x="105" y="190" fill="#0d1117" font-size="10" font-weight="bold">3/3</text>

  <!-- Analytics (analytics, dashboards, monitoring) = 2/3 = 67% -->
  <text x="20" y="220" fill="#8b949e" font-size="12">Analytics</text>
  <rect x="100" y="208" width="390" height="16" rx="4" fill="#1f2937"/>
  <rect x="100" y="208" width="260" height="16" rx="4" fill="#d29922"/>
  <text x="500" y="220" fill="#d29922" font-size="12" font-weight="bold">67%</text>
  <text x="105" y="220" fill="#0d1117" font-size="10" font-weight="bold">2/3</text>

  <!-- Media (player, video, canvas) = 1/3 = 33% -->
  <text x="20" y="250" fill="#8b949e" font-size="12">Media</text>
  <rect x="100" y="238" width="390" height="16" rx="4" fill="#1f2937"/>
  <rect x="100" y="238" width="130" height="16" rx="4" fill="#d29922"/>
  <text x="500" y="250" fill="#d29922" font-size="12" font-weight="bold">33%</text>
  <text x="105" y="250" fill="#0d1117" font-size="10" font-weight="bold">1/3</text>

  <!-- Admin/System (admin, settings, monitoring, auth, about) = 0/5 = 0% -->
  <text x="20" y="280" fill="#8b949e" font-size="12">Admin / System</text>
  <rect x="100" y="268" width="390" height="16" rx="4" fill="#1f2937"/>
  <rect x="100" y="268" width="0" height="16" rx="4" fill="#f85149"/>
  <text x="500" y="280" fill="#f85149" font-size="12" font-weight="bold">0%</text>
  <text x="105" y="280" fill="#8b949e" font-size="10">0/5</text>

  <!-- Social/Community (social, campaigns, lists, goals, projects, people, learn, workspace) = 0/8 = 0% -->
  <text x="20" y="310" fill="#8b949e" font-size="12">Social / Other</text>
  <rect x="100" y="298" width="390" height="16" rx="4" fill="#1f2937"/>
  <rect x="100" y="298" width="0" height="16" rx="4" fill="#f85149"/>
  <text x="500" y="310" fill="#f85149" font-size="12" font-weight="bold">0%</text>
  <text x="105" y="310" fill="#8b949e" font-size="10">0/8</text>

  <!-- Legend -->
  <rect x="20" y="335" width="10" height="10" rx="2" fill="#3fb950"/>
  <text x="35" y="344" fill="#8b949e" font-size="10">100%</text>
  <rect x="80" y="335" width="10" height="10" rx="2" fill="#d29922"/>
  <text x="95" y="344" fill="#8b949e" font-size="10">33-67%</text>
  <rect x="160" y="335" width="10" height="10" rx="2" fill="#f85149"/>
  <text x="175" y="344" fill="#8b949e" font-size="10">0%</text>
</svg>

## Missing Documentation — Details

### 🔴 High Priority (Critical for users)

| App | Description | UI Location | What to Document |
|-----|-------------|-------------|------------------|
| **admin** | Full admin console with operations, billing, compliance dashboards | `admin/index.html` | User management, roles, groups, DNS, organization settings, onboarding |
| **settings** | User profile, security, notifications, API keys | `settings/index.html` | Profile config, 2FA, notification prefs, integrations, API keys, data privacy |
| **monitoring** | System health, metrics, logs, alerts, services | `monitoring/index.html` | Health checks, metrics visualization, log viewer, alert config, resource usage |

### 🟡 Medium Priority (Productivity features)

| App | Description | UI Location | What to Document |
|-----|-------------|-------------|------------------|
| **attendant** | Human agent console for bot-transferred conversations | `attendant/index.html` | Queue management, conversation handoff, agent workflows |
| **docs** | WYSIWYG document editor (Google Docs-like) | `docs/docs.html` | Rich text editing, formatting, print, collaboration |
| **sheet** | Spreadsheet editor | `sheet/sheet.html` | Cell editing, formulas, formatting, import/export |
| **slides** | Presentation editor (PowerPoint-like) | `slides/slides.html` | Slide creation, templates, transitions, presenter mode |
| **canvas** | Collaborative whiteboard | `canvas/canvas.html` | Drawing tools, shapes, collaboration, export |
| **campaigns** | Multi-channel marketing campaigns | `campaigns/campaigns.html` | Email/WhatsApp/Social campaigns, templates, lists |
| **goals** | OKR goals management | `goals/goals.html` | Objectives, key results, periods, tracking |
| **project** | Project management with Gantt | `project/project.html` | Projects, Gantt charts, dependencies, resources |
| **people** | Contacts and directory | `people/people.html` | Contact CRUD, groups, directory search |

### 🟢 Low Priority (Nice to have)

| App | Description | UI Location |
|-----|-------------|-------------|
| **learn** | E-learning platform | `learn/learn.html` |
| **social** | Social feed / community | `social/social.html` |
| **workspace** | Notion-style workspace | `workspace/workspace.html` |
| **browser** | Built-in web browser | `browser/browser.html` |
| **terminal** | Command-line terminal | `terminal/terminal.html` |
| **video** | Video editor | `video/video.html` |
| **templates** | Content templates | `templates/templates.html` |
| **tools** | Security & compliance UI | `tools/security.html` |
| **lists** | Marketing lists | `lists/lists.html` |

## Roadmap

<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 200" font-family="system-ui, sans-serif">
  <rect width="600" height="200" fill="#0d1117" rx="12"/>
  <text x="300" y="28" text-anchor="middle" fill="#e6edf3" font-size="15" font-weight="bold">Documentation Roadmap</text>

  <!-- Timeline -->
  <line x1="40" y1="100" x2="560" y2="100" stroke="#30363d" stroke-width="2"/>

  <!-- Done marker -->
  <circle cx="80" cy="100" r="8" fill="#3fb950"/>
  <text x="80" y="80" text-anchor="middle" fill="#3fb950" font-size="11" font-weight="bold">Done</text>
  <text x="80" y="125" text-anchor="middle" fill="#8b949e" font-size="10">21 apps</text>

  <!-- In Progress marker -->
  <circle cx="240" cy="100" r="8" fill="#d29922"/>
  <text x="240" y="80" text-anchor="middle" fill="#d29922" font-size="11" font-weight="bold">Next</text>
  <text x="240" y="125" text-anchor="middle" fill="#8b949e" font-size="10">admin, settings,</text>
  <text x="240" y="138" text-anchor="middle" fill="#8b949e" font-size="10">monitoring</text>

  <!-- Planned marker -->
  <circle cx="400" cy="100" r="8" fill="#f85149"/>
  <text x="400" y="80" text-anchor="middle" fill="#f85149" font-size="11" font-weight="bold">Planned</text>
  <text x="400" y="125" text-anchor="middle" fill="#8b949e" font-size="10">Office, Media,</text>
  <text x="400" y="138" text-anchor="middle" fill="#8b949e" font-size="10">Social, Other</text>

  <!-- Future marker -->
  <circle cx="540" cy="100" r="8" fill="#8b949e"/>
  <text x="540" y="80" text-anchor="middle" fill="#8b949e" font-size="11">Later</text>
  <text x="540" y="125" text-anchor="middle" fill="#8b949e" font-size="10">Infra, Low</text>

  <!-- Progress bar -->
  <rect x="40" y="165" width="520" height="8" rx="4" fill="#1f2937"/>
  <rect x="40" y="165" width="268" height="8" rx="4" fill="#3fb950"/>
  <text x="300" y="190" text-anchor="middle" fill="#8b949e" font-size="11">21 / 41 apps documented (51%)</text>
</svg>
