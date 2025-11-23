# Glossary

Quick lookup for BotServer terms. If you're lost, start here.

## A

**Auto-Bootstrap** - The magical first run that installs everything automatically. PostgreSQL, cache, storage - all configured without you lifting a finger.

**Argon2** - The Fort Knox of password hashing. Makes brute-force attacks computationally infeasible. We use it everywhere passwords are stored.

## B

**BASIC** - Yes, that programming language from 1964. We brought it back because `TALK "Hello"` beats `await ctx.send()` any day. Powers all conversation scripts.

**Bot Package** - A folder ending in `.gbai` containing everything to run a bot. Scripts, documents, config. That's it. Copy folder = deploy bot.

**BotSession** - The conversation between user and bot. Remembers everything - who you are, what you said, where you left off. Persists to database, cached for speed.

## C

**Collection** - A folder of documents in `.gbkb/` that becomes searchable knowledge. Drop PDFs in `policies/`, bot answers policy questions. Zero configuration.

**Context** - What the LLM knows right now. Includes conversation history, active knowledge bases, loaded tools. Limited by token window (usually 4-8k tokens).

**config.csv** - The only config file you need. Simple key-value pairs in CSV format. Opens in Excel. Lives in `.gbot/` folder.

## D

**Dialog** - A `.bas` script defining conversation flow. Not complex state machines - just simple BASIC code like `TALK "Hi"` and `answer = HEAR`.

**Drive** - Built-in S3-compatible storage (SeaweedFS). Stores documents, templates, uploads. Auto-installed during bootstrap. No AWS account needed.

## E

**Embedding** - Text converted to numbers for similarity search. "dog" and "puppy" have similar embeddings. BGE model by default, replaceable with any GGUF model.

**Event Handler** - BASIC code that runs when something happens. `ON "login"` runs at login. `ON "email"` when email arrives. `ON "0 8 * * *"` at 8 AM daily.

## G

**.gbai** - "General Bot AI" folder. Contains entire bot. Example: `support.gbai/` becomes bot at `/support`. No manifest files, no build process.

**.gbdialog** - Subfolder with BASIC scripts. Must contain `start.bas` as entry point. Tools go in `tools/` subdirectory.

**.gbkb** - "Knowledge Base" subfolder. Each subdirectory becomes a searchable collection. PDFs, Word docs, text files - all automatically indexed.

**.gbot** - Configuration subfolder. Contains single `config.csv` file with bot settings. Missing values use defaults.

**.gbtheme** - Optional UI customization. CSS files, images, HTML templates. Most bots don't need this.

## H

**HEAR** - BASIC keyword to get user input. `name = HEAR` waits for user to type, stores response in variable.

**Hot Reload** - Change scripts while bot runs. Edit file, bot uses new version immediately. No restart needed (unless changing config).

## K

**Knowledge Base (KB)** - Documents the bot can search. Organized in folders under `.gbkb/`. Use with `USE KB "foldername"` in scripts.

**Keyword** - BASIC command like TALK, HEAR, LLM. About 40 total. All caps by convention, not requirement.

## L

**LLM** - Large Language Model (ChatGPT, Claude, Llama). The AI brain that powers natural conversation understanding.

**Local Mode** - Run everything on your machine. LLM runs locally (llama.cpp), no internet required. Slower but private.

## M

**MCP** - Model Context Protocol. Standard format for defining tools that LLMs can call. Alternative to OpenAI function format.

**Memory** - Two types: Session (temporary, per conversation) and Bot (permanent, across all sessions). `SET` for session, `SET BOT MEMORY` for permanent.

## P

**Package Manager** - Built-in system that installs/manages components. Handles PostgreSQL, cache, storage, vector DB, LLM server. All automatic.

**PARAM** - Defines tool parameters. `PARAM name, email, date` means tool needs these three inputs. LLM collects them automatically.

**PostgreSQL** - The database. Stores users, sessions, messages, bot config. Auto-installed, auto-configured, auto-migrated.

## Q

**Qdrant** - Vector database for semantic search. Stores document embeddings. Finds similar content even with different words. Optional but recommended.

## R

**Rhai** - The scripting engine powering BASIC interpreter. Rust-based, sandboxed, safe. You never see it directly.

## S

**Script** - A `.bas` file with BASIC code. `start.bas` runs first. Can call others with `RUN "other.bas"`.

**Semantic Search** - Finding by meaning, not keywords. "vacation policy" finds "time off guidelines". Powered by embeddings and vector similarity.

**Session** - See BotSession. The container for a conversation.

**Session Token** - Random string identifying a session. Stored in browser localStorage for web, passed as header for API. Unguessable for security.

## T

**TALK** - BASIC keyword to send message. `TALK "Hello world"` displays that text to user.

**Template** - Example bot in `templates/` folder. Copy, modify, deploy. `default.gbai` is minimal starter, `announcements.gbai` shows advanced features.

**Tool** - A `.bas` file the LLM can call. Has PARAM definitions and DESCRIPTION. Put in `tools/` folder. AI figures out when to use it.

**Token** - Unit of text for LLMs. Roughly 4 characters. GPT-3.5 handles 4k tokens, GPT-4 handles 8k-128k. Includes prompt + response.

## U

**USE KB** - BASIC command to activate knowledge base. `USE KB "policies"` makes policies folder searchable for that session.

**USE TOOL** - BASIC command to enable a tool. `USE TOOL "send-email"` lets LLM send emails when appropriate.

## V

**Valkey** - Redis-compatible cache (fork after Redis license change). Stores session data for fast access. Auto-installed during bootstrap.

**Vector** - Mathematical representation of meaning. "Cat" might be [0.2, 0.8, -0.3, ...]. Similar meanings have similar vectors.

**Vector Collection** - See Collection. Folder of documents converted to vectors for semantic search.

## W

**WebSocket** - Real-time connection for chat. Enables instant messaging without polling. Path: `/ws` on same port as HTTP.

## Symbols

**.bas** - File extension for BASIC scripts. Plain text files with BASIC code.

**.csv** - Configuration format. Simple, editable in Excel, no JSON parsing needed.

**.env** - Environment variables file. Auto-generated during bootstrap with credentials and settings.

## Common Confusions

**"Do I need containers?"** - No. BotServer installs everything directly or in LXC containers.

**"What database?"** - PostgreSQL, auto-installed, auto-configured.

**"What about scaling?"** - Single server handles 1000+ concurrent users. Scale by running multiple instances.

**"Is BASIC really BASIC?"** - Inspired by BASIC, not strict implementation. Simpler, focused on conversations.

**"Can I use TypeScript/Python/etc?"** - No. BASIC only for conversation logic. Extend core in Rust if needed.

**"Is it production-ready?"** - Yes. Used in production since 2016 (earlier versions), current Rust version since 2023.