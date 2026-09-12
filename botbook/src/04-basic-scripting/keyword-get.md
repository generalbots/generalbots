# GET Keyword 🟡 BETA

The **GET** keyword retrieves content from a specified source — either a remote URL or a local file stored in the bot’s configured storage system.  
It is used to fetch data dynamically during script execution.

---

## Syntax

```basic
variable = GET "source"
```

---

## Parameters

- `"source"` — The location of the content to retrieve.  
  This can be:
  - An HTTP/HTTPS URL (e.g., `"https://api.example.com/data"`)
  - A relative path to a file stored in the bot's drive bucket or local storage.
- `variable` — The variable that will receive the fetched content.

---

## Description

`GET` performs a read operation from the specified source.  
If the source is a URL, the bot sends an HTTP GET request and retrieves the response body.  
If the source is a file path, the bot reads the file content directly from its configured storage (e.g., drive component or local filesystem).

The command automatically handles text extraction from documents in Drive, converting them to plain UTF‑8 text: PDF is decoded in memory, plain-text formats are decoded directly, and office formats are converted by the shared document processor.  
If the request fails, the format is unsupported or the file cannot be found, an error message that names the reason is returned.

| Format | Handling | Size limit |
|--------|----------|------------|
| `pdf` | `pdf-extract`, in memory | 500 MB |
| `docx`, `doc`, `xlsx`, `xls` | document processor | 100 MB |
| `pptx` | document processor | 200 MB |
| `txt`, `md`, `csv`, `json`, `xml`, `html` | decoded as UTF-8 | 10–1024 MB per format |
| `rtf` | document processor | 50 MB |
| anything else | read as UTF-8; binary content is refused by name | — |

A document above its limit is refused before conversion, with both sizes in the message (for example `inbox/report.pdf: PDF document is 600000000 bytes, above the 524288000 byte limit`), and an unknown binary format reports `unsupported document format '.bin'` instead of a bare UTF-8 error. This is what lets an inbound Telegram document be summarised or classified by the LLM (#1331).

This keyword is essential for integrating external APIs, reading stored documents, and dynamically loading data into scripts.

---

## Example

```basic
' Fetch data from a remote API
GET "https://api.example.com/users" INTO RESPONSE
PRINT RESPONSE

' Read a local file from the bot’s storage
GET "reports/summary.txt" INTO CONTENT
TALK CONTENT
```

---

## Implementation Notes

- Implemented in Rust under `src/file/mod.rs` and `src/web_automation/crawler.rs`.  
- Uses the `reqwest` library for HTTP requests with timeout and error handling.  
- Detects the format from the extension and performs the extraction listed above; the office-format path stages the object in a temporary file because the processor reads from a path, and always removes it afterwards.  
- Validates paths to prevent directory traversal or unsafe access.  
- Runs in a separate thread to avoid blocking the main engine.

---

## Related Keywords

- [`FIND`](keyword-find.md) — Searches for data within the current context.  
- [`FORMAT`](keyword-format.md) — Formats retrieved data for display.  
- [`PRINT`](keyword-print.md) — Outputs data to the console or chat.

---

## Summary

`GET` is a versatile keyword for retrieving external or stored content.  
It enables bots to access APIs, read documents, and integrate dynamic data sources seamlessly within BASIC scripts.
