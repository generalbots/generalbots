# GET Keyword

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

The command automatically handles text extraction from PDF and DOCX files, converting them to plain UTF‑8 text.  
If the request fails or the file cannot be found, an error message is returned.

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
- Automatically detects file type and performs extraction for supported formats (PDF, DOCX, TXT).  
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
