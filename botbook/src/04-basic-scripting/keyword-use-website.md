# USE WEBSITE Keyword

**Syntax**

```basic
USE WEBSITE "https://example.com"

USE WEBSITE "https://example.com" REFRESH "1d"
```

**Parameters**

- `"url"` – A valid HTTP or HTTPS URL pointing to a website that should be made available in the conversation context.
- `"refresh"` – (Optional) How often to recrawl the website. Supports: `"1d"` (1 day), `"1w"` (1 week), `"1m"` (1 month), `"1y"` (1 year). Defaults to `"1m"`.

**Description**

`USE WEBSITE` operates in two distinct modes:

1. **Preprocessing Mode** (Script Compilation): When found in a BASIC script during compilation, it registers the website for background crawling. The crawler service will fetch, extract, and index the website's content into a vector database collection. The crawl happens immediately on first compile, then recurs based on the REFRESH interval.

2. **Runtime Mode** (Conversation Execution): During a conversation, `USE WEBSITE` associates an already-crawled website collection with the current session, making it available for queries via `FIND` or `LLM` calls. This behaves similarly to `USE KB` - it's a session-scoped association.

If a website hasn't been registered during preprocessing, the runtime execution will auto-register it for crawling.

**Refresh Interval Behavior**

- **Smart Interval Selection**: If the same URL is registered multiple times with different REFRESH intervals, the **shortest interval** is always used
- **Default**: If no REFRESH is specified, defaults to `"1m"` (1 month)
- **Formats Supported**:
  - `"1d"` = 1 day
  - `"1w"` = 1 week
  - `"1m"` = 1 month (default)
  - `"1y"` = 1 year
  - Custom: `"3d"`, `"2w"`, `"6m"`, etc.

**Examples**

Basic usage with default 1-month refresh:
```basic
USE WEBSITE "https://docs.example.com"
```

High-frequency website (daily refresh):
```basic
USE WEBSITE "https://news.example.com" REFRESH "1d"
```

Stable documentation (monthly refresh):
```basic
USE WEBSITE "https://api.example.com/docs" REFRESH "1m"
```

Multiple registrations - shortest interval wins:
```basic
USE WEBSITE "https://example.com" REFRESH "1w"
USE WEBSITE "https://example.com" REFRESH "1d"
' Final refresh interval: 1d (shortest)
```

**Runtime Example**

```basic
USE WEBSITE "https://company.com/policies" REFRESH "1w"
question = HEAR "What would you like to know about our policies?"
FIND question
answer = LLM "Based on the search results, provide a clear answer"
TALK answer
```

**Preprocessing Behavior**

When the script is compiled:
- The URL is validated
- The website is registered in the `website_crawls` table with the specified refresh policy
- The crawler service immediately starts crawling the website
- Subsequent crawls are scheduled based on the REFRESH interval
- Status can be: pending (0), crawled (1), or failed (2)

**Runtime Behavior**

When executed in a conversation:
- Checks if the website has been registered and crawled
- If not registered, auto-registers with default 1-month refresh
- Associates the website collection with the current session
- Makes the content searchable via `FIND` and available to `LLM`

**Database Schema**

The `website_crawls` table stores:
- `refresh_policy` - User-configured refresh interval (e.g., "1d", "1w", "1m")
- `expires_policy` - Internal representation in days
- `next_crawl` - Timestamp for next scheduled crawl
- `crawl_status` - 0=pending, 1=success, 2=processing, 3=error

**Related Keywords**

- [CLEAR WEBSITES](./keyword-clear-websites.md) - Remove all website associations from session
- [USE KB](./keyword-use-kb.md) - Similar functionality for knowledge base files
- [FIND](./keyword-find.md) - Search within loaded websites and KBs
- [LLM](./keyword-llm.md) - Process search results with AI