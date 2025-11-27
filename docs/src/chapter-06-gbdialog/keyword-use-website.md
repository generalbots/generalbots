# USE WEBSITE Keyword

**Syntax**

```
USE WEBSITE "https://example.com"
```

**Parameters**

- `"url"` – A valid HTTP or HTTPS URL pointing to a website that should be made available in the conversation context.

**Description**

`USE WEBSITE` operates in two distinct modes:

1. **Preprocessing Mode** (Script Compilation): When found in a BASIC script during compilation, it registers the website for background crawling. The crawler service will fetch, extract, and index the website's content into a vector database collection. This ensures the website content is ready before any conversation starts.

2. **Runtime Mode** (Conversation Execution): During a conversation, `USE WEBSITE` associates an already-crawled website collection with the current session, making it available for queries via `FIND` or `LLM` calls. This behaves similarly to `USE KB` - it's a session-scoped association.

If a website hasn't been registered during preprocessing, the runtime execution will fail with an appropriate error message.

**Example**

```basic
' In script preprocessing, this registers the website for crawling
USE WEBSITE "https://docs.example.com"

' During conversation, this makes the crawled content available
USE WEBSITE "https://docs.example.com"
FIND "deployment procedures"
TALK "I found information about deployment procedures in the documentation."
```

**Preprocessing Behavior**

When the script is compiled:
- The URL is validated
- The website is registered in the `website_crawls` table
- The crawler service picks it up and indexes the content
- Status can be: pending (0), crawled (1), or failed (2)

**Runtime Behavior**

When executed in a conversation:
- Checks if the website has been crawled
- Associates the website collection with the current session
- Makes the content searchable via `FIND` and available to `LLM`

**With LLM Integration**

```basic
USE WEBSITE "https://company.com/policies"
question = HEAR "What would you like to know about our policies?"
FIND question
answer = LLM "Based on the search results, provide a clear answer"
TALK answer
```

**Related Keywords**

- [CLEAR WEBSITES](./keyword-clear-websites.md) - Remove all website associations from session
- [USE KB](./keyword-use-kb.md) - Similar functionality for knowledge base files
- [FIND](./keyword-find.md) - Search within loaded websites and KBs
- [LLM](./keyword-llm.md) - Process search results with AI