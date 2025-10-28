# GET Keyword

**Syntax**

```
GET "url-or-path"
```

**Parameters**

- `"url-or-path"` – Either an HTTP/HTTPS URL (e.g., `https://api.example.com/data`) or a relative path to an object stored in the configured MinIO bucket.

**Description**

`GET` fetches the content from the specified location.

- If the argument starts with `http://` or `https://`, the keyword performs an HTTP GET request using a timeout‑protected `reqwest` client. The response must have a successful status code; otherwise a runtime error is raised.
- If the argument does not start with a scheme, it is treated as a path inside the bot’s MinIO bucket. The keyword reads the object, automatically handling PDF extraction when the file ends with `.pdf`. The content is returned as a UTF‑8 string.

The fetched content can be stored in a variable or passed to other keywords such as `TALK` or `FORMAT`.

**Example (HTTP)**

```basic
SET data = GET "https://api.example.com/users"
TALK "Received data: " + data
```

**Example (Bucket file)**

```basic
SET report = GET "reports/summary.txt"
TALK "Report content:\n" + report
```

**Security**

The implementation validates the path to prevent directory traversal (`..`) and other unsafe patterns. Invalid or unsafe paths cause a runtime error.

**Implementation Notes**

- The request runs in a separate thread with its own Tokio runtime to avoid blocking the main engine.
- Network timeouts are set to 30 seconds; connection timeouts to 10 seconds.
- For bucket access, the keyword ensures the bucket exists before attempting to read.
