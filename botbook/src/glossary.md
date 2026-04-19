# Glossary

Quick lookup for General Bots terms. If you're lost, start here.

---

## A

**A2A Protocol** - Agent-to-Agent Protocol. Enables bots to communicate and delegate tasks to each other in multi-agent systems. Messages include request, response, broadcast, and delegate types.

**ADD BOT** - BASIC keyword to add a bot to the current session with triggers, tools, or schedules.

**Argon2** - Password hashing algorithm used for secure credential storage. Makes brute-force attacks computationally infeasible.

**Auto-Bootstrap** - The automatic first-run process that installs and configures all dependencies: PostgreSQL, cache, storage, and LLM servers.

---

## B

**BASIC** - The scripting language for General Bots dialogs. Inspired by the 1964 language, simplified for conversational AI. Powers all `.bas` scripts with keywords like TALK, HEAR, and LLM.

**BM25** - Best Match 25. Sparse retrieval algorithm for keyword-based search. Used in hybrid RAG alongside dense (semantic) search.

**BOOK** - BASIC keyword to schedule calendar appointments.

**Bot Memory** - Persistent storage scoped to a single bot, shared across all users. Access with `SET BOT MEMORY` and `GET BOT MEMORY`.

**Bot Package** - A folder ending in `.gbai` containing everything to run a bot: scripts, documents, and configuration.

**BotSession** - The active conversation between user and bot. Tracks state, history, and context. Persists to database, cached for speed.

**Bootstrap** - Initial setup process that installs all dependencies automatically on first launch.

**BROADCAST TO BOTS** - BASIC keyword to send a message to all bots in the current session.

---

## C

**Cache** - In-memory storage component for sessions, temporary data, and semantic caching. Provides sub-millisecond access times.

**Collection** - A folder of documents in `.gbkb/` that becomes searchable knowledge. Each subfolder is a separate collection.

**Chunking** - The process of splitting documents into smaller pieces for embedding and retrieval. Default chunk size is optimized for context windows.

**config.csv** - The configuration file for each bot. Simple key-value pairs in CSV format. Lives in the `.gbot/` folder.

**Context** - Information available to the LLM during a conversation. Includes history, knowledge base results, and user-provided context via `SET CONTEXT`.

**Context Compaction** - Automatic summarization of older conversation history to fit within token limits while preserving important information.

**CREATE DRAFT** - BASIC keyword to compose and save an email draft to the user's mailbox.

**CREATE TASK** - BASIC keyword to create a task with assignee and due date.

---

## D

**DELEGATE TO BOT** - BASIC keyword to send a task to another bot and optionally wait for a response.

**Dense Search** - Semantic search using vector embeddings. Finds content by meaning rather than exact keywords.

**Dialog** - A `.bas` script defining conversation flow. Contains BASIC code with keywords like TALK and HEAR.

**Drive** - Built-in S3-compatible object storage. Stores documents, templates, and uploads. Auto-installed during bootstrap.

---

## E

**Embedding** - Text converted to numerical vectors for similarity search. Similar meanings produce similar vectors.

**Embedding Model** - Neural network that generates embeddings. Default is BGE, replaceable with any GGUF-compatible model.

**Episodic Memory** - Summaries of past conversations stored for long-term context. Automatically generated when conversations end.

**Event Handler** - BASIC code triggered by events. Use `ON` keyword with triggers like `"login"`, `"email"`, or cron expressions.

---

## F

**FIND** - BASIC keyword to search database tables with filter criteria. Returns matching records.

**FOR EACH** - BASIC keyword for iterating over collections and query results.

---

## G

**.gbai** - "General Bot AI" package folder. Contains the entire bot. Example: `support.gbai/` becomes the bot at `/support`.

**.gbdialog** - Subfolder containing BASIC scripts. Must include `start.bas` as the entry point. Tools go in `tools/` subdirectory.

**.gbdrive** - File storage configuration subfolder. Maps to Drive buckets for document management.

**.gbkb** - "Knowledge Base" subfolder. Each subdirectory becomes a searchable collection with automatic indexing.

**.gbot** - Configuration subfolder containing `config.csv` with bot settings.

**.gbtheme** - Optional UI customization subfolder for CSS, images, and HTML templates.

**General Bots** - Open-source enterprise conversational AI platform. Combines LLMs with structured dialogs, knowledge bases, and multi-channel support.

**GET** - BASIC keyword to retrieve data from APIs, files, or session variables.

**GET BOT MEMORY** - BASIC keyword to retrieve persistent bot-level data.

**GET USER MEMORY** - BASIC keyword to retrieve cross-session user data accessible from any bot.

**GraphQL** - Query language for APIs. Supported via the `GRAPHQL` keyword for complex data retrieval.

---

## H

**HEAR** - BASIC keyword to wait for and capture user input. `name = HEAR` stores the response in a variable.

**Hot Reload** - Automatic reloading of BASIC scripts when files change. No restart needed.

**Hybrid Search** - RAG approach combining dense (semantic) and sparse (keyword) retrieval using Reciprocal Rank Fusion.

**HTMX** - Frontend library used for dynamic UI updates without full page reloads.

---

## I

**INSERT** - BASIC keyword to add records to database tables.

**Intent** - What the user wants to accomplish. Detected from natural language via LLM classification.

---

## K

**Keyword** - A BASIC command like TALK, HEAR, or LLM. About 50+ available. Written in uppercase by convention.

**Knowledge Base (KB)** - Documents searchable by the bot. Organized in folders under `.gbkb/`. Activate with `USE KB "foldername"`.

---

## L

**LiveKit** - WebRTC platform used for video meetings in General Bots.

**LLM** - Large Language Model. The AI that powers natural conversation. Supports OpenAI, Anthropic, Groq, and local models via llama.cpp.

**llama.cpp** - C++ library for running LLM inference locally. Used for self-hosted model deployment.

**Local-First** - Architecture principle where everything runs locally by default. No cloud dependencies required.

---

## M

**MCP** - Model Context Protocol. Standard format for defining tools that LLMs can call. Supported alongside OpenAI function format.

**Memory** - Data persistence system with four scopes: User Memory (cross-bot), Bot Memory (per-bot), Session Memory (temporary), and Episodic Memory (conversation summaries).

**Model Routing** - Dynamic selection of LLM models based on task requirements. Use `USE MODEL "fast"`, `"quality"`, `"code"`, or `"auto"`.

**Multi-Agent** - Architecture where multiple specialized bots collaborate on complex tasks.

**Multi-Channel** - Same bot works across WhatsApp, Telegram, Teams, Web, and other channels without modification.

---

## N

**No Forms** - General Bots philosophy since 2017: people should converse, not fill forms. Conversations replace traditional UI forms.

---

## O

**ON** - BASIC keyword to define event handlers for triggers, schedules, or webhooks.

**OIDC** - OpenID Connect. Authentication protocol handled by the Directory service (Zitadel).

---

## P

**Package Manager** - Built-in system that installs bot packages. Drop a `.gbai` folder and it's automatically loaded.

**PARAM** - Declares tool parameters. `PARAM name, email` means the tool needs these inputs. LLM collects them automatically.

**PostgreSQL** - The database for General Bots. Stores users, sessions, messages, and bot configuration. Auto-installed and auto-configured.

**POST** - BASIC keyword to make HTTP POST requests to external APIs.

**Pragmatismo** - Brazilian software company that created and maintains General Bots.

---

## Q

**Qdrant** - Vector database for semantic search at scale. Optional component for large knowledge bases.

---

## R

**RAG** - Retrieval-Augmented Generation. Pattern where relevant documents are retrieved and provided to the LLM as context.

**Reranking** - Optional LLM-based scoring of search results for improved relevance. Adds latency but improves quality.

**Rhai** - Rust scripting engine that powers the BASIC interpreter. Sandboxed and safe.

**RRF** - Reciprocal Rank Fusion. Algorithm for combining rankings from multiple search methods in hybrid RAG.

**RUN PYTHON / JAVASCRIPT / BASH** - BASIC keywords to execute code in sandboxed environments.

---

## S

**SAVE** - BASIC keyword to write data to CSV files or database tables.

**Script** - A `.bas` file with BASIC code. `start.bas` is the entry point; other scripts are tools or utilities.

**Semantic Cache** - Caching system that matches similar (not just identical) queries to reuse LLM responses.

**Semantic Search** - Finding content by meaning rather than exact keywords. Powered by embeddings and vector similarity.

**SEND MAIL** - BASIC keyword to send emails with optional HTML and attachments.

**Session** - Active conversation state between user and bot. Expires after inactivity (default 30 minutes).

**Session Memory** - Temporary storage for the current conversation. Access with `SET` and `GET`.

**SET** - BASIC keyword to store values in session variables or update database records.

**SET BOT MEMORY** - BASIC keyword to store persistent bot-level data.

**SET CONTEXT** - BASIC keyword to add information to the LLM context. Influences all subsequent responses.

**SET SCHEDULE** - BASIC keyword for cron-based task scheduling. Accepts natural language like `"every monday at 9am"`.

**SET USER MEMORY** - BASIC keyword to store cross-session user data accessible from any bot.

**Sparse Search** - Keyword-based search using algorithms like BM25. Excels at exact matches and rare terms.

**SSE** - Server-Sent Events. Used for real-time streaming of LLM responses.

**Stalwart** - Email server component providing IMAP/SMTP/JMAP support.

**Suite** - The complete General Bots workspace application with Chat, Drive, Tasks, Mail, Calendar, and other apps.

**SWITCH** - BASIC keyword for multi-way conditional branching.

---

## T

**TALK** - BASIC keyword to send messages to the user. Supports text, markdown, and multimedia.

**Template** - Pre-built bot configuration in the `templates/` folder. Copy and modify to create new bots.

**Token** - Unit of text for LLMs. Roughly 4 characters. Context windows are measured in tokens.

**Tool** - A `.bas` file the LLM can call automatically. Define with `PARAM` declarations and a `DESCRIPTION`. Place in the `tools/` folder.

**TRANSFER CONVERSATION** - BASIC keyword to hand off the entire conversation to another bot.

---

## U

**UPDATE** - BASIC keyword to modify existing database records.

**USE KB** - BASIC keyword to activate a knowledge base for semantic search. `USE KB "policies"` makes the policies collection searchable.

**USE MODEL** - BASIC keyword to switch LLM models. Options: `"fast"`, `"quality"`, `"code"`, or `"auto"`.

**USE TOOL** - BASIC keyword to enable a tool for LLM use. The AI determines when to call it.

**User Memory** - Persistent storage scoped to a user, accessible across all bots and sessions.

---

## V

**Vault** - HashiCorp Vault. Secrets management service for storing credentials securely. Only `VAULT_*` environment variables are used.

**Vector** - Mathematical representation of meaning. Similar meanings produce similar vectors.

**Vector Database** - Database optimized for storing and searching embeddings. Qdrant is the default option.

---

## W

**WAIT** - BASIC keyword to pause execution for a specified duration.

**WEBHOOK** - BASIC keyword to create HTTP endpoints that trigger bot actions.

**WebSocket** - Real-time connection for chat. Enables instant messaging without polling. Path: `/ws`.

---

## Z

**Zitadel** - Identity and access management service. Handles authentication, users, and permissions.

---

## Package Extensions

| Extension | Purpose |
|-----------|---------|
| `.gbai` | Complete bot package |
| `.gbdialog` | BASIC scripts |
| `.gbkb` | Knowledge base documents |
| `.gbot` | Bot configuration |
| `.gbtheme` | UI customization |
| `.gbdrive` | File storage mapping |
| `.bas` | BASIC script file |

---

## Common Confusions

**"Do I need containers?"** - No. botserver installs everything directly or in optional LXC containers.

**"What database?"** - PostgreSQL, automatically installed and configured.

**"What about scaling?"** - Single server handles 1000+ concurrent users. Scale by running multiple instances.

**"Is BASIC really BASIC?"** - Inspired by BASIC, not strict implementation. Simplified and focused on conversations.

**"Can I use TypeScript/Python/etc?"** - BASIC handles conversation logic. Use `RUN PYTHON/JAVASCRIPT` for code execution, or integrate via REST API.

**"Is it production-ready?"** - Yes. Used in production since 2016, current Rust version since 2023.

---

<div align="center">
  <img src="./assets/general-bots-logo.svg" alt="General Bots" width="200">
</div>