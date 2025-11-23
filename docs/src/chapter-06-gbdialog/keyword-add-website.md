# ADD WEBSITE Keyword

**Syntax**

```
ADD WEBSITE "https://example.com"
```

**Parameters**

- `"url"` – A valid HTTP or HTTPS URL pointing to a website that should be added to the conversation context.

**Description**

`ADD WEBSITE` validates the provided URL and, when the `web_automation` feature is enabled, launches a headless browser to crawl the site, extract its textual content, and index it into a vector‑DB collection associated with the current user. The collection name is derived from the URL and the bot's identifiers. After indexing, the website becomes a knowledge source that can be queried by `FIND` or `LLM` calls.

If the feature is not compiled, the keyword returns an error indicating that web automation is unavailable.

**Example**

```basic
ADD WEBSITE "https://en.wikipedia.org/wiki/General_Bots"
TALK "Website added. You can now search its content with FIND."
