---
name: web-scraping
description: |
  Scrape respectfully with pagination, rate limits and robots awareness. Use for structured data
  extraction.
version: 1.0.0
source: in-repo
---

# web-scraping

## Workflow

1. Check robots.txt and the site's terms; stop if disallowed.
2. Identify yourself with a clear user agent and throttle requests.
3. Paginate with the documented next cursor, not by guessing offsets.
4. Extract structured fields with stable selectors; keep the source URL per record.
5. Handle empty/anti-bot pages explicitly instead of returning blank rows.
6. Store raw HTML samples for audit when the extraction is disputed.

## Quality checks

- No request burst beyond the declared rate.
- Every record carries its source URL and retrieval time.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
