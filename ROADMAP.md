# General Bots Roadmap

## 🎯 Vision: The Free Open Source Enterprise AI Suite

**General Bots = Office Suite + Multi-LLM AI + Research Engine + Security AI + Autonomous Agents**

All of these capabilities, **completely FREE and open source**.

---

## 🏆 What We Have Today

### ✅ Core Platform (Production Ready)

| Feature | Status | Description |
|---------|--------|-------------|
| **Conversational AI** | ✅ Complete | Multi-turn dialogs with any LLM provider |
| **Multi-LLM Support** | ✅ Complete | Connect to any LLM API (local or cloud) |
| **Knowledge Base (RAG)** | ✅ Complete | Document indexing and semantic search |
| **BASIC + LLM Scripting** | ✅ Complete | Simple programming for everyone |
| **Tool/Function Calling** | ✅ Complete | MCP and custom tool support |
| **Multi-Channel Messaging** | ✅ Complete | WhatsApp, Telegram, Web, SMS |
| **Email Integration** | ✅ Complete | Send, receive, and process emails |
| **File Storage (.gbdrive)** | ✅ Complete | Cloud-native file management |
| **Document Processing** | ✅ Complete | PDF, Word, Excel, images |
| **Scheduling & Jobs** | ✅ Complete | Cron-based automation |
| **Web UI (HTMX)** | ✅ Complete | Modern, responsive interface |
| **REST API** | ✅ Complete | Full API for integrations |
| **Database (PostgreSQL)** | ✅ Complete | Enterprise-grade storage |
| **Vector Search (Qdrant)** | ✅ Complete | Semantic similarity search |
| **Template System** | ✅ Complete | Pre-built business applications |

### ✅ BASIC Keywords Implemented

| Category | Keywords |
|----------|----------|
| **Dialog** | TALK, HEAR, WAIT, PRINT |
| **Memory** | SET, GET, SET BOT MEMORY, GET BOT MEMORY |
| **AI** | LLM, SET CONTEXT, USE KB, USE TOOL |
| **Data** | SAVE, FIND, FILTER, AGGREGATE, JOIN, MERGE |
| **HTTP** | GET, POST, PUT, PATCH, DELETE HTTP, GRAPHQL, SOAP |
| **Files** | READ, WRITE, COPY, MOVE, UPLOAD, DOWNLOAD |
| **Email** | SEND MAIL, CREATE DRAFT |
| **Control** | FOR EACH, WHILE/WEND, IF/THEN/ELSE, SWITCH/CASE |
| **Procedures** | SUB, FUNCTION, CALL, RETURN |
| **Events** | ON, WEBHOOK, SET SCHEDULE |
| **Social** | POST TO (Instagram, Facebook, LinkedIn) |

### ✅ Templates Ready

| Template | Category | Purpose |
|----------|----------|---------|
| Employee Management | HR | Full employee CRUD |
| IT Helpdesk | IT | Ticket management |
| Sales Pipeline | CRM | Deal tracking |
| Contact Directory | CRM | Contact management |
| Default | Core | Starter template |
| Announcements | Comms | Company news |

---

## 🚀 What We're Building

### Phase 1: Marketing Automation (Q1 2025)

**Goal:** Complete inbound marketing and lead generation platform

| Feature | Target | Description |
|---------|--------|-------------|
| **Landing Pages** | 🔄 In Progress | CREATE SITE keyword for landing pages |
| **Lead Capture Forms** | 📋 Planned | Embedded forms with validation |
| **Lead Scoring** | 📋 Planned | AI-powered lead qualification |
| **Email Campaigns** | 📋 Planned | Drip campaigns with templates |
| **Social Media Posting** | 📋 Planned | POST TO Instagram, Facebook, LinkedIn |
| **Analytics Dashboard** | 📋 Planned | Conversion tracking and ROI |
| **A/B Testing** | 📋 Planned | Landing page optimization |
| **CRM Integration** | ✅ Complete | Pipeline and contact management |

#### Landing Page Plan (CREATE SITE Enhancement)

```basic
' Create a landing page with AI
CREATE SITE "promo-jan" WITH TEMPLATE "landing-page" USING PROMPT "
  Create a landing page for our January promotion.
  Product: Enterprise AI Suite
  Offer: 30% discount for early adopters
  CTA: Schedule a demo
"

' Capture leads from the landing page
ON FORM SUBMIT "promo-jan"
  SAVE "leads.csv", name, email, phone, source
  SEND MAIL email, "Welcome!", "Thank you for your interest..."
  ADD TO CAMPAIGN email, "nurture-sequence"
END ON
```

### Phase 2: Social Media Integration (Q1 2025)

**Goal:** Unified social media management

| Feature | Target | Description |
|---------|--------|-------------|
| **POST TO Instagram** | 📋 Planned | Post images and stories |
| **POST TO Facebook** | 📋 Planned | Posts, stories, and pages |
| **POST TO LinkedIn** | 📋 Planned | Articles and updates |
| **POST TO Twitter/X** | 📋 Planned | Tweets and threads |
| **Content Calendar** | 📋 Planned | Schedule posts in advance |
| **Engagement Tracking** | 📋 Planned | Likes, comments, shares |
| **AI Content Generation** | 📋 Planned | LLM-powered post creation |

#### Social Media Keywords Plan

```basic
' Post to Instagram
POST TO INSTAGRAM image, "Check out our new feature! #AI #Automation"

' Post to multiple platforms
POST TO "instagram,facebook,linkedin" image, caption

' Schedule a post
POST TO INSTAGRAM AT "2025-02-01 10:00" image, caption

' Get engagement metrics
metrics = GET INSTAGRAM METRICS "post-id"
TALK "Likes: " + metrics.likes + ", Comments: " + metrics.comments
```

### Phase 3: Enterprise Office Suite (Q2 2025)

**Goal:** Complete office productivity replacement

| Feature | Target | Description |
|---------|--------|-------------|
| **Calendar Integration** | 🔄 In Progress | Event management |
| **Task Management** | 🔄 In Progress | To-do lists and projects |
| **Contact Management** | ✅ Complete | Directory and CRM |
| **Meeting Scheduling** | 📋 Planned | Booking and availability |
| **Video Calls** | 📋 Planned | WebRTC integration |
| **Real-time Collaboration** | 📋 Planned | Shared documents |
| **Spreadsheet Engine** | 📋 Planned | Excel-compatible |
| **Document Editor** | 📋 Planned | Word-compatible |

### Phase 4: AI Autonomy (Q2 2025)

**Goal:** Autonomous agent capabilities

| Feature | Target | Description |
|---------|--------|-------------|
| **Autonomous Agents** | 📋 Planned | Self-directing AI workflows |
| **Multi-Step Planning** | 📋 Planned | Complex task decomposition |
| **Self-Correcting Workflows** | 📋 Planned | Error recovery |
| **Memory Persistence** | 📋 Planned | Long-term memory |
| **Goal Decomposition** | 📋 Planned | Break down objectives |

### Phase 5: Security Suite (Q3 2025)

**Goal:** Enterprise security and compliance

| Feature | Target | Description |
|---------|--------|-------------|
| **AI Content Filtering** | 📋 Planned | Content moderation |
| **Threat Detection** | 📋 Planned | Security monitoring |
| **Compliance Automation** | 📋 Planned | GDPR, LGPD, SOC2 |
| **Audit Logging** | ✅ Complete | Full activity tracking |
| **Data Loss Prevention** | 📋 Planned | Sensitive data protection |
| **Access Control** | ✅ Complete | Role-based permissions |

### Phase 6: Research & Discovery (Q4 2025)

**Goal:** Deep research capabilities

| Feature | Target | Description |
|---------|--------|-------------|
| **Web Search Integration** | 📋 Planned | Real-time web search |
| **Citation Generation** | 📋 Planned | Academic references |
| **Source Verification** | 📋 Planned | Fact-checking |
| **Knowledge Graphs** | 📋 Planned | Entity relationships |
| **Academic Search** | 📋 Planned | Papers and research |

---

## 📁 Template Expansion Plan

### 50 Templates Target

| Category | Count | Status |
|----------|-------|--------|
| HR & People | 5 | 1 ✅ |
| IT & Support | 5 | 1 ✅ |
| CRM & Sales | 6 | 2 ✅ |
| Finance | 6 | 📋 Planned |
| Operations | 5 | 📋 Planned |
| Healthcare | 5 | 📋 Planned |
| Education | 4 | 📋 Planned |
| Real Estate | 4 | 📋 Planned |
| Events | 4 | 📋 Planned |
| Nonprofit | 5 | 📋 Planned |
| **Marketing** | 6 | 📋 Planned |

### New Marketing Templates

| # | Template | Folder | Key Files |
|---|----------|--------|-----------|
| 51 | Landing Page Builder | `marketing/landing-pages.gbai` | `start.bas`, `create-page.bas`, `capture-lead.bas` |
| 52 | Email Campaigns | `marketing/campaigns.gbai` | `start.bas`, `create-campaign.bas`, `send-campaign.bas` |
| 53 | Lead Nurturing | `marketing/nurturing.gbai` | `start.bas`, `add-to-sequence.bas`, `nurture-jobs.bas` |
| 54 | Social Media Manager | `marketing/social.gbai` | `start.bas`, `post-content.bas`, `schedule-post.bas` |
| 55 | Analytics Dashboard | `marketing/analytics.gbai` | `start.bas`, `track-conversion.bas`, `report.bas` |
| 56 | A/B Testing | `marketing/ab-testing.gbai` | `start.bas`, `create-test.bas`, `analyze-results.bas` |

---

## 🔧 Technical Improvements

### New Keywords Needed

#### Marketing & Social Media

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `POST TO` | `POST TO "instagram" image, caption` | Post to social platforms |
| `GET METRICS` | `metrics = GET INSTAGRAM METRICS "id"` | Get engagement data |
| `CREATE LANDING PAGE` | `CREATE LANDING PAGE "name" WITH template` | Build landing pages |
| `ADD TO CAMPAIGN` | `ADD TO CAMPAIGN email, "campaign-name"` | Add to email sequence |
| `TRACK CONVERSION` | `TRACK CONVERSION "campaign", "event"` | Track marketing events |

#### Classic BASIC Functions (Priority)

| Function | Syntax | Description |
|----------|--------|-------------|
| `LEN` | `length = LEN(string)` | String length |
| `LEFT` | `result = LEFT(string, n)` | Left substring |
| `RIGHT` | `result = RIGHT(string, n)` | Right substring |
| `MID` | `result = MID(string, start, length)` | Middle substring |
| `TRIM` | `result = TRIM(string)` | Remove whitespace |
| `UCASE` | `result = UCASE(string)` | Uppercase |
| `LCASE` | `result = LCASE(string)` | Lowercase |
| `REPLACE` | `result = REPLACE(string, old, new)` | Replace substring |
| `SPLIT` | `array = SPLIT(string, delimiter)` | Split into array |
| `VAL` | `number = VAL(string)` | String to number |
| `STR` | `string = STR(number)` | Number to string |
| `ROUND` | `result = ROUND(number, decimals)` | Round number |
| `ABS` | `result = ABS(number)` | Absolute value |
| `NOW` | `datetime = NOW()` | Current date/time |
| `TODAY` | `date = TODAY()` | Current date |
| `DATEADD` | `date = DATEADD(date, n, "day")` | Add to date |
| `DATEDIFF` | `days = DATEDIFF(date1, date2, "day")` | Date difference |
| `YEAR` | `year = YEAR(date)` | Extract year |
| `MONTH` | `month = MONTH(date)` | Extract month |
| `DAY` | `day = DAY(date)` | Extract day |
| `ISNULL` | `result = ISNULL(value)` | Check if null |
| `ARRAY` | `arr = ARRAY(1, 2, 3)` | Create array |
| `UBOUND` | `size = UBOUND(array)` | Array size |
| `SORT` | `sorted = SORT(array)` | Sort array |
| `UNIQUE` | `distinct = UNIQUE(array)` | Remove duplicates |
| `MAX` | `maximum = MAX(array)` | Maximum value |
| `MIN` | `minimum = MIN(array)` | Minimum value |

#### Error Handling

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `ON ERROR GOTO` | `ON ERROR GOTO handler` | Error handler |
| `TRY...CATCH` | `TRY ... CATCH e ... END TRY` | Structured errors |
| `THROW` | `THROW "error message"` | Raise error |

### Infrastructure

| Item | Status | Description |
|------|--------|-------------|
| Clustering | 📋 Planned | Multi-node deployment |
| Edge Deployment | 📋 Planned | Run on edge devices |
| Offline Mode | 📋 Planned | Local-only operation |
| Mobile App | 📋 Planned | Native mobile client |
| Desktop App | 🔄 Tauri | Desktop wrapper |

---

## 🤝 Community Goals

### Documentation

- [ ] Complete keyword reference (all keywords)
- [ ] Video tutorials for each template
- [ ] Interactive playground
- [ ] Cookbook with recipes
- [ ] Localization (10 languages)

### Ecosystem

- [ ] Plugin marketplace
- [ ] Template sharing hub
- [ ] Community templates
- [ ] Integration directory
- [ ] Certification program

---

## 💡 Why General Bots?

### The Problem

Enterprise software costs thousands per user per year:
- Office Suite: $10-60/user/month
- AI Assistant: $20-30/user/month
- Marketing Automation: $50-300/month
- CRM: $25-150/user/month
- **Total:** $100-500/user/month

For 100 users = **$120,000-600,000/year**

### The Solution

General Bots provides the same capabilities:
- **Cost:** $0 (open source)
- **Data ownership:** 100% yours
- **Customization:** Unlimited
- **AI provider:** Your choice
- **Deployment:** Anywhere

### The Difference

| Aspect | Enterprise SaaS | General Bots |
|--------|-----------------|--------------|
| Cost | $$$$$ | Free |
| Data | Their cloud | Your control |
| Vendor lock-in | High | None |
| Customization | Limited | Unlimited |
| AI Models | Fixed | Any provider |
| Open Source | No | Yes (AGPL) |

---

## 📊 Success Metrics

### 2025 Goals

| Metric | Target |
|--------|--------|
| GitHub Stars | 10,000 |
| Active Deployments | 5,000 |
| Community Templates | 100 |
| Contributors | 50 |
| Documentation Pages | 500 |
| Languages Supported | 10 |

---

## 🗺️ How to Contribute

### Code Contributions

1. Pick an item from this roadmap
2. Open an issue to discuss
3. Submit a PR with tests
4. Get reviewed and merged

### Non-Code Contributions

- Write documentation
- Create templates
- Report bugs
- Answer questions
- Translate docs
- Share on social media

### Priority Areas

1. **Marketing Keywords** - POST TO, tracking, campaigns
2. **Classic BASIC Functions** - LEN, LEFT, RIGHT, MID, etc.
3. **Templates** - Create business templates
4. **Documentation** - Write guides and tutorials
5. **Localization** - Translate to more languages

---

## 📅 Release Schedule

| Version | Date | Focus |
|---------|------|-------|
| v5.0 | Q1 2025 | Marketing automation, Social media |
| v5.1 | Q2 2025 | Office suite, Agent capabilities |
| v5.2 | Q3 2025 | Security features |
| v5.3 | Q4 2025 | Research features |
| v6.0 | Q1 2026 | Enterprise complete |

---

## 🌟 The Dream

**"Every organization, regardless of size or budget, deserves enterprise-grade AI capabilities."**

General Bots makes this possible by providing:

1. **Free software** - No licensing costs
2. **Open source** - Full transparency
3. **Self-hosted** - Your data, your servers
4. **Extensible** - Add what you need
5. **Community-driven** - Built together

Join us in democratizing enterprise AI.

---

*Last updated: 2025*

*"BASIC for AI, AI for Everyone"*