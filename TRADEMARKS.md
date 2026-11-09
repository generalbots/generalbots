# Trademarks and Commercial Names

This document lists the names, brands and marks that appear in the General Bots
project — its own names and the third-party names it refers to — and explains
how they may be used.

It exists so that nobody reading or redistributing this project is left with the
impression that General Bots owns, is sponsored by, is affiliated with, or is
endorsed by any of the companies listed here (or vice versa).

> **This is not legal advice.** It is an inventory and a statement of intent.
> If you need a legal opinion about trademark use in your jurisdiction, consult
> a qualified lawyer.

---

## 1. General Bots marks

The following names identify this project and its components. They are used
throughout the source code, the documentation (BotBook), the user interface and
the release artifacts.

| Name | Refers to |
|------|-----------|
| **General Bots** | The project as a whole, and the copyright holder named in [`LICENSE`](./LICENSE) |
| **GeneralBots** / **generalbots** | Alternate spellings used in URLs, package names and organization names |
| **BotServer** | The backend service (`botserver/`) — HTTP/WebSocket API, BASIC engine, channels |
| **BotUI** | The web front end (`botui/`) — suite, cloud and login surfaces |
| **BotApp** | The desktop shell (`botapp/`, Tauri) |
| **BotBook** | The documentation set (`botbook/`) |
| **BotModels** | The model/anomaly service and its Python sidecar (`pythonbox`) |
| **BotVibe** | The project/VM provisioning subsystem |
| **BotDrive** | The Drive/object-storage subsystem |
| **BotDocs / BotSheet / BotSlides** | The document, spreadsheet and presentation editors |

The project is distributed under the **MIT License** (see [`LICENSE`](./LICENSE)).
The MIT license grants copyright permissions; **it does not grant any trademark
or brand license**. You may redistribute the code, but you may not use the
General Bots names or logos to imply that your fork, product or service is the
official General Bots project, or that it is endorsed by it.

If you fork the project, please make the fork clearly distinguishable (for
example, keep the source attribution but use a different product name and
logo).

---

## 2. Third-party marks

All names below are the property of their respective owners. They are used in
this project **descriptively and nominatively** — to identify a component that
is bundled, integrated, supported or referenced — and never as a claim of
ownership, partnership, sponsorship or endorsement.

The absence of a name from this list does not mean that no trademark exists;
it only means that the name is not currently used in this repository. The
absence of a `™` or `®` symbol next to a name also means nothing: many owners
omit the symbol, and this document does not assert registration status either
way.

### 2.1 Bundled infrastructure components

These are shipped as binaries or archives (see `botserver/3rdparty.toml`) and
run alongside or beneath General Bots.

| Name | Owner / project | Role in General Bots |
|------|-----------------|----------------------|
| **MinIO**, **MinIO Client (mc)** | MinIO, Inc. | S3-compatible object storage ("Drive") |
| **PostgreSQL**, **libpq** | PostgreSQL Global Development Group | Relational database |
| **Valkey** | Valkey contributors (Linux Foundation) | In-memory cache (Redis-compatible) |
| **Redis** | Redis Ltd. | Protocol/API compatibility reference only |
| **Caddy** | Caddy project (Stack Holdings / Light Code Labs) | Reverse proxy and TLS |
| **Stalwart** | Stalwart Labs | Mail server |
| **Zitadel** | Zitadel AG | Identity provider / SSO |
| **Forgejo**, **Forgejo Actions Runner** | Forgejo contributors (Codeberg e.V.) | Git hosting and CI runner ("ALM") |
| **CoreDNS** | The Kubernetes Authors / CNCF | DNS server |
| **LiveKit** | LiveKit, Inc. | Real-time audio/video ("Meet") |
| **Qdrant** | Qdrant Solutions GmbH | Vector database |
| **NocoDB** | NocoDB, Inc. | Database UI ("table editor") |
| **Roundcube** | The Roundcube Project | Webmail |
| **Vector** | Datadog, Inc. | Log aggregation |
| **InfluxDB** | InfluxData, Inc. | Time-series database |
| **HashiCorp Vault** | HashiCorp, Inc. | Secrets management |
| **Garage** | Garage contributors (Deuxfleurs) | S3-compatible storage (optional) |
| **Skytable** | Skytable contributors | Database (optional) |
| **Incus** | Incus contributors (Linux Containers) | Container/VM runtime |
| **LXD** | Canonical Ltd. | Container runtime (compatibility note only) |
| **Ceph** | Red Hat, Inc. / Ceph Foundation | Storage (reference/integration) |
| **Docker** | Docker, Inc. | Container tooling (developer workflows) |
| **Kubernetes** | The Linux Foundation | Deployment target |
| **Keycloak** | Red Hat, Inc. | Identity integration (supported alternative) |
| **Auth0** | Okta, Inc. | Identity integration (supported alternative) |
| **Okta** | Okta, Inc. | Identity integration (supported alternative) |
| **Nginx** | F5, Inc. | Reverse proxy (deployment reference) |
| **Apache** | The Apache Software Foundation | HTTP server / project ecosystem |
| **Elasticsearch** | Elastic N.V. | Search integration |
| **RabbitMQ** | Broadcom Inc. (VMware) | Message broker integration |
| **Prometheus** | The Linux Foundation / CNCF | Metrics integration |
| **Grafana** | Grafana Labs | Observability integration |
| **Terraform** | HashiCorp, Inc. | Infrastructure provisioning (deployment reference) |

### 2.2 Local inference and model runtimes

| Name | Owner / project | Role |
|------|-----------------|------|
| **llama.cpp** | The llama.cpp contributors (Georgi Gerganov et al.) | Local LLM inference server |
| **CUDA**, **NVIDIA** | NVIDIA Corporation | GPU runtime for local inference |
| **Llama** | Meta Platforms, Inc. | Model family served through llama.cpp |
| **GPT-OSS** | OpenAI | Open-weight model bundled for local inference |
| **BGE (BGE Small EN)** | Beijing Academy of Artificial Intelligence (BAAI) | Embedding model |
| **DeepSeek**, **DeepSeek R1 Distill Qwen** | DeepSeek (Hangzhou DeepSeek Artificial Intelligence Co., Ltd.) | Model family |
| **Qwen** | Alibaba Group (Alibaba Cloud) | Model family |
| **GLM** | Zhipu AI | Model family |
| **Kimi** | Moonshot AI | Model family |
| **MiniMax** | MiniMax | Model family |
| **Yi** | 01.AI | Model family |
| **Step** | StepFun | Model family |
| **Doubao** | ByteDance | Model family |
| **Mistral** | Mistral AI | Model family / provider |
| **Ollama** | Ollama | Local model runtime (optional) |

The bundled model weights remain under the license terms of their publishers;
those terms are separate from the MIT license of this project.

### 2.3 Hosted AI providers and APIs

General Bots talks to these services through their HTTP APIs. Integration means
only that the API is supported — it is **not** a partnership, certification or
endorsement.

| Name | Owner |
|------|-------|
| **OpenAI**, **GPT**, **ChatGPT** | OpenAI, L.L.C. |
| **Anthropic**, **Claude** | Anthropic PBC |
| **Google**, **Gemini**, **Vertex AI** | Google LLC |
| **Amazon Web Services (AWS)**, **Amazon Bedrock** | Amazon.com, Inc. |
| **Kiro** | Amazon Web Services |
| **Groq** | Groq, Inc. |
| **Mistral** | Mistral AI |
| **Cohere** | Cohere Inc. |
| **DeepSeek** | DeepSeek |
| **Qwen**, **Alibaba Cloud** | Alibaba Group |
| **GLM** | Zhipu AI |
| **Kimi** | Moonshot AI |
| **MiniMax** | MiniMax |
| **Yi** | 01.AI |
| **Doubao** | ByteDance |
| **DeepInfra** | DeepInfra |
| **Cerebras** | Cerebras Systems |
| **Fireworks AI** | Fireworks AI, Inc. |
| **Together AI** | Together Computer, Inc. |
| **OpenRouter** | OpenRouter, Inc. |
| **Requesty** | Requesty |
| **Hugging Face** | Hugging Face, Inc. |

Any provider not listed here that is reachable through an OpenAI-compatible
endpoint is used on the same descriptive basis.

### 2.4 Messaging, social and communication channels

The channel integrations send and receive messages through the official APIs of
these platforms.

| Name | Owner |
|------|-------|
| **WhatsApp**, **WhatsApp Business** | Meta Platforms, Inc. |
| **Messenger**, **Facebook**, **Facebook Pages**, **Instagram**, **Threads** | Meta Platforms, Inc. |
| **Telegram** | Telegram Messenger LLP |
| **Slack** | Slack Technologies, LLC (Salesforce) |
| **Discord** | Discord Inc. |
| **Microsoft Teams** | Microsoft Corporation |
| **Zoom** | Zoom Communications, Inc. |
| **Google Meet** | Google LLC |
| **Twilio**, **Twilio SMS** | Twilio Inc. |
| **LinkedIn**, **LinkedIn Ads** | LinkedIn Corporation (Microsoft) |
| **X**, **Twitter** | X Corp. |
| **TikTok** | ByteDance Ltd. |
| **WeChat** | Tencent Holdings Ltd. |
| **YouTube**, **YouTube Analytics** | Google LLC |
| **Pinterest** | Pinterest, Inc. |
| **Reddit** | Reddit, Inc. |
| **Snapchat** | Snap Inc. |
| **Bluesky** | Bluesky Social PBC |
| **Mastodon** | Mastodon gGmbH |
| **Twitch** | Twitch Interactive, Inc. (Amazon) |
| **Signal** | Signal Messenger LLC |
| **Spotify** | Spotify AB |
| **Last.fm** | Last.fm Ltd. (Paramount) |
| **Blogger** | Google LLC |
| **Beehiiv** | Beehiiv, Inc. |
| **Luma** | Luma AI, Inc. |

Using these integrations requires your own developer accounts and credentials
with each platform, and is subject to each platform's own terms of service.

### 2.5 Documents, file formats and office suites

General Bots reads and writes the office formats listed below. Compatibility is
implemented from the published specifications; no Microsoft or LibreOffice code
is copied into this project.

| Name | Owner / standard | Relevance |
|------|------------------|-----------|
| **Microsoft Office**, **Word**, **Excel**, **PowerPoint**, **Microsoft 365**, **Office 365** | Microsoft Corporation | `.docx`, `.xlsx`, `.pptx` compatibility |
| **Office Open XML (OOXML)** | ECMA-376 / ISO/IEC 29500 (Ecma International, ISO) | File format standard |
| **OpenDocument (ODF)** | OASIS | `.odt`, `.ods` compatibility |
| **LibreOffice** | The Document Foundation | Alternative office suite interoperability |
| **PDF** | Adobe Inc. (ISO 32000) | Document export |
| **DOCX / XLSX / PPTX / ODT / ODS** | see OOXML and ODF above | File extensions |

"Word", "Excel", "PowerPoint" and "Office" are used only to say that a file of
that format can be opened or produced. This project is **not** a Microsoft
product, is **not** endorsed by Microsoft, and does not ship Microsoft code.

### 2.6 Operating systems, platforms and runtimes

| Name | Owner | Role |
|------|-------|------|
| **Windows**, **Microsoft** | Microsoft Corporation | Supported desktop/server platform |
| **Linux** | Linus Torvalds (mark administered by the Linux Mark Institute) | Supported platform |
| **Ubuntu** | Canonical Ltd. | Supported Linux distribution |
| **Debian** | Software in the Public Interest, Inc. | Supported Linux distribution |
| **macOS**, **Apple** | Apple Inc. | Supported desktop platform |
| **iOS** | Apple Inc. | Client platform (reference) |
| **Android** | Google LLC | Client platform (reference) |
| **Google Chrome**, **Chrome** | Google LLC | Browser and automation target |
| **Rust** | The Rust Foundation | Implementation language |
| **Node.js** | OpenJS Foundation | Build and tooling runtime |
| **Tauri** | The Tauri contributors | Desktop application shell |
| **htmx (HTMX)** | htmx contributors | Front-end interaction layer |
| **Diesel** | Diesel contributors | Rust ORM |
| **Rhai** | Rhai contributors | BASIC scripting engine |
| **WebKit / WebView** | Apple Inc. / respective platform owners | Embedded browser engine |
| **Intel** | Intel Corporation | Supported CPU architecture |
| **AMD** | Advanced Micro Devices, Inc. | Supported CPU/GPU architecture |
| **Samsung**, **Qualcomm**, **Huawei**, **Xiaomi** | respective owners | Client/hardware compatibility references |

### 2.7 Business software integrations (catalog)

General Bots ships an integration catalog (`botserver/src/apps/integration_catalog/`)
whose entries name third-party services. Every name below is a trademark or
service mark of its respective owner; the list is descriptive of supported
integrations only.

**Productivity and collaboration**
Notion · Linear · Airtable · Dropbox · Gmail · Microsoft Outlook · Outlook
Calendar · OneDrive · SharePoint · Box · Trello · Confluence · ClickUp ·
monday.com · Todoist · Asana · Jira · Basecamp · Smartsheet · Motion · Grain ·
Descript · Raindrop · Typeform · Jotform · Cal.com · Calendly · Google Forms ·
Google Workspace · Google Workspace Admin · Google Photos · Readwise · Canva ·
Arena · n8n · Workday · Zendesk · Intercom · Plain

**CRM, marketing and sales**
Salesforce · HubSpot · Microsoft Dynamics · Apollo · lemlist · Loops.so ·
Streak · Mailchimp · Ahrefs · Google Ads · Crunchbase · Greenhouse · Ashby ·
Rippling · ClickFunnels · Canny · PhantomBuster · Attio

**Commerce, ERP and logistics**
Shopify · WooCommerce · BigCommerce · Squarespace · Webflow · HighLevel ·
Printful · Whop · QuickBooks · Xero Accounting · Lightspeed X · Square ·
Paddle · TOTVS Protheus · SAP Business One · SAP · FedEx · UPS · DHL ·
iFood · Rappi · Uber Eats · Mercado Livre · Mercado Pago · Lalamove · Loggi

**Finance and banking**
Stripe · PayPal · Coinbase · Wise · You Need a Budget (YNAB) · MoonPay · Qik ·
Robinhood · Mercury · Ramp · Expensify · Plaid

**Developer, data and observability**
GitHub · GitLab · Cursor · AWS · Google Cloud · Microsoft Azure · Vercel ·
Supabase · Netlify · Cloudflare · Snowflake · Databricks · PostHog ·
Amplitude · Sentry · Splunk · Resend · Postmark · Algolia · Upstash Redis ·
Hex · Mintlify · Devin · Pydantic Logfire

**Lifestyle and consumer**
Fitbit · Philips Hue · Eight Sleep · NewsCatcher

**Payment rails and fiscal documents (Brazil)**
Pix · Boleto · NF-e · NFC-e

> **Pix** is a payment arrangement and brand of the **Banco Central do Brasil**.
> **Boleto** (boleto bancário) is a Brazilian payment instrument; the name is
> used generically. **NF-e / NFC-e** are Brazilian electronic invoice models
> defined by the tax authorities. Their use here is descriptive of supported
> file formats and payment methods only.

---

## 3. How these names are used

1. **Descriptive use only.** A name appears because the project integrates with,
   bundles, supports or documents that thing. It does not describe origin,
   sponsorship or approval.
2. **No logos or brand assets.** General Bots does not redistribute third-party
   logos or wordmarks as part of its own branding. Logos of third parties that
   appear in documentation belong to their owners and are shown only to identify
   the corresponding component.
3. **No implied affiliation.** Nothing in this repository, its documentation, its
   website or its marketing should be read as a statement that General Bots is
   affiliated with, certified by, partnered with or endorsed by any of the
   companies listed above.
4. **No endorsement in the other direction.** The mention of a third-party
   product does not mean that its owner endorses General Bots.
5. **Respect the owners' guidelines.** If you reuse a name from this list in your
   own material, follow that owner's trademark guidelines. Many of them publish
   explicit rules for nominative and referential use.
6. **Correct capitalization is intentional.** Names are written as their owners
   write them (for example `PostgreSQL`, not `Postgres`; `WhatsApp`, not
   `Whatsapp`; `GitHub`, not `Github`; `monday.com`, not `Monday`). Please
   preserve that spelling.

---

## 4. Licenses are separate from trademarks

Trademark rights and copyright licenses are different things:

- The **MIT license** in [`LICENSE`](./LICENSE) covers the General Bots source
  code and grants no rights in any name or logo.
- Bundled third-party binaries (MinIO, Caddy, Stalwart, Zitadel, Forgejo,
  Vault, Qdrant, LiveKit, PostgreSQL, Valkey, Garage, Skytable, and others) are
  distributed under **their own licenses**, some of which are copyleft. Their
  notices must be preserved. See `botserver/3rdparty.toml`, the per-project
  license files and `LICENSES-desktop.md` where present.
- Model weights have their own licenses, set by their publishers.

If a bundled component's license conflicts with the MIT license of this
project, the component's license governs that component.

---

## 5. Reporting a concern

If you own a mark listed here and believe it is used incorrectly, or if you
believe a mark is missing, please open an issue in the repository. Corrections
to spelling, attribution or wording are welcome and will be applied promptly.

---

*Last updated: 2026-11-09*
