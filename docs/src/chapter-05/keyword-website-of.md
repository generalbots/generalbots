# WEBSITE_OF Keyword

**Syntax**

```
WEBSITE_OF "search-term"
```

**Parameters**

- `"search-term"` – The term to search for using a headless browser (e.g., a query string).

**Description**

`WEBSITE_OF` performs a web search for the given term using a headless Chromium instance (via the `headless_chrome` crate). It navigates to DuckDuckGo, enters the search term, and extracts the first non‑advertisement result URL. The keyword returns the URL as a string, which can then be used with `ADD_WEBSITE` or other keywords.

**Example**

```basic
SET url = WEBSITE_OF "GeneralBots documentation"
ADD_WEBSITE url
TALK "Added the top result as a knowledge source."
```

The script searches for “GeneralBots documentation”, retrieves the first result URL, adds it as a website KB, and notifies the user.

**Implementation Notes**

- The keyword runs the browser actions in a separate thread with its own Tokio runtime.
- If no results are found, the keyword returns the string `"No results found"`.
- Errors during navigation or extraction are logged and cause a runtime error.
- The search is performed on DuckDuckGo to avoid reliance on proprietary APIs.
