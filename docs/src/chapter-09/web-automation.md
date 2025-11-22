# Web Automation

The web automation module enables BotServer to interact with websites, extract content, and perform automated browser tasks.

## Overview

Web automation features allow bots to:
- Crawl and index website content
- Extract structured data from web pages
- Automate form submissions
- Capture screenshots
- Monitor website changes
- Perform headless browser operations

## Configuration

Enable web automation in `config.csv`:

```csv
webAutomationEnabled,true
browserTimeout,30000
maxCrawlDepth,3
userAgent,BotServer/1.0
```

## Features

### Website Crawling

The `ADD_WEBSITE` keyword triggers web crawling:

```basic
ADD_WEBSITE "https://example.com"
```

This will:
1. Launch headless browser
2. Navigate to the URL
3. Extract text content
4. Follow internal links (respecting robots.txt)
5. Index content in vector database
6. Make content searchable via `FIND` keyword

### Content Extraction

Extract specific data from web pages:

```basic
url = "https://news.example.com"
content = GET url
headlines = EXTRACT_CSS content, "h2.headline"
```

### Form Automation

Submit forms programmatically:

```basic
NAVIGATE "https://example.com/contact"
FILL_FIELD "name", customer_name
FILL_FIELD "email", customer_email
FILL_FIELD "message", inquiry_text
CLICK_BUTTON "submit"
result = GET_PAGE_TEXT()
```

### Screenshot Capture

Capture visual representations:

```basic
NAVIGATE "https://example.com/dashboard"
screenshot = CAPTURE_SCREENSHOT()
SAVE_FILE screenshot, "dashboard.png"
```

### Change Monitoring

Monitor websites for updates:

```basic
SET_MONITOR "https://example.com/status", "hourly"
ON "website_changed" DO
    changes = GET_CHANGES()
    SEND_MAIL admin_email, "Website Updated", changes
END ON
```

## Crawler Configuration

### Crawl Rules

Control crawler behavior:

| Setting | Description | Default |
|---------|-------------|---------|
| `maxDepth` | Maximum crawl depth | 3 |
| `maxPages` | Maximum pages to crawl | 100 |
| `crawlDelay` | Delay between requests (ms) | 1000 |
| `respectRobots` | Honor robots.txt | true |
| `followRedirects` | Follow HTTP redirects | true |
| `includeImages` | Extract image URLs | false |
| `includePDFs` | Process PDF links | true |

### Selector Strategies

Extract content using CSS selectors:

```basic
' Extract specific elements
titles = EXTRACT_CSS page, "h1, h2, h3"
paragraphs = EXTRACT_CSS page, "p"
links = EXTRACT_CSS page, "a[href]"
images = EXTRACT_CSS page, "img[src]"
```

Or XPath expressions:

```basic
' XPath extraction
prices = EXTRACT_XPATH page, "//span[@class='price']"
reviews = EXTRACT_XPATH page, "//div[@class='review-text']"
```

## Browser Automation

### Navigation

Control browser navigation:

```basic
NAVIGATE "https://example.com"
WAIT_FOR_ELEMENT "#content"
SCROLL_TO_BOTTOM()
BACK()
FORWARD()
REFRESH()
```

### Interaction

Interact with page elements:

```basic
CLICK "#login-button"
TYPE "#username", user_credentials
SELECT "#country", "USA"
CHECK "#agree-terms"
UPLOAD_FILE "#document", "report.pdf"
```

### Waiting Strategies

Wait for specific conditions:

```basic
WAIT_FOR_ELEMENT "#results"
WAIT_FOR_TEXT "Loading complete"
WAIT_FOR_URL "success"
WAIT_SECONDS 3
```

## Data Processing

### Structured Data Extraction

Extract structured data from pages:

```basic
products = EXTRACT_TABLE "#product-list"
FOR EACH product IN products
    SAVE_TO_DB product.name, product.price, product.stock
NEXT
```

### Content Cleaning

Clean extracted content:

```basic
raw_text = GET_PAGE_TEXT()
clean_text = REMOVE_HTML(raw_text)
clean_text = REMOVE_SCRIPTS(clean_text)
clean_text = NORMALIZE_WHITESPACE(clean_text)
```

## Performance Optimization

### Caching

Cache crawled content:

```basic
IF NOT IN_CACHE(url) THEN
    content = CRAWL_URL(url)
    CACHE_SET(url, content, "1 hour")
ELSE
    content = CACHE_GET(url)
END IF
```

### Parallel Processing

Process multiple URLs concurrently:

```basic
urls = ["url1", "url2", "url3"]
results = PARALLEL_CRAWL(urls, max_workers=5)
```

## Security Considerations

### Authentication

Handle authenticated sessions:

```basic
LOGIN "https://example.com/login", username, password
cookie = GET_COOKIE("session")
' Use cookie for subsequent requests
NAVIGATE "https://example.com/dashboard"
```

### Rate Limiting

Respect rate limits:

```basic
CONFIGURE_CRAWLER(
    rate_limit = 10,  ' requests per second
    user_agent = "BotServer/1.0",
    timeout = 30000
)
```

### Content Filtering

Filter inappropriate content:

```basic
content = CRAWL_URL(url)
IF CONTAINS_INAPPROPRIATE(content) THEN
    LOG_WARNING "Inappropriate content detected"
    SKIP_URL(url)
END IF
```

## Error Handling

Handle common web automation errors:

```basic
TRY
    content = CRAWL_URL(url)
CATCH "timeout"
    LOG "Page load timeout: " + url
    RETRY_WITH_DELAY(5000)
CATCH "404"
    LOG "Page not found: " + url
    MARK_AS_BROKEN(url)
CATCH "blocked"
    LOG "Access blocked, might need CAPTCHA"
    USE_PROXY()
END TRY
```

## Integration with Knowledge Base

Automatically index crawled content:

```basic
ADD_WEBSITE "https://docs.example.com"
' Content is automatically indexed

' Later, search the indexed content
answer = FIND "installation guide"
TALK answer
```

## Monitoring and Logging

Track automation activities:

```basic
START_MONITORING()
result = CRAWL_URL(url)
metrics = GET_METRICS()
LOG "Pages crawled: " + metrics.page_count
LOG "Time taken: " + metrics.duration
LOG "Data extracted: " + metrics.data_size
```

## Best Practices

1. **Respect robots.txt**: Always honor website crawling rules
2. **Use appropriate delays**: Don't overwhelm servers
3. **Handle errors gracefully**: Implement retry logic
4. **Cache when possible**: Reduce redundant requests
5. **Monitor performance**: Track crawling metrics
6. **Secure credentials**: Never hardcode passwords
7. **Test selectors**: Verify CSS/XPath selectors work
8. **Clean data**: Remove unnecessary HTML/scripts
9. **Set timeouts**: Prevent infinite waiting
10. **Log activities**: Maintain audit trail

## Limitations

- JavaScript-heavy sites may require additional configuration
- CAPTCHA-protected sites need manual intervention
- Some sites block automated access
- Large-scale crawling requires distributed setup
- Dynamic content may need special handling

## Troubleshooting

### Page Not Loading
- Check network connectivity
- Verify URL is accessible
- Increase timeout values
- Check for JavaScript requirements

### Content Not Found
- Verify CSS selectors are correct
- Check if content is dynamically loaded
- Wait for elements to appear
- Use browser developer tools to test

### Access Denied
- Check user agent settings
- Verify authentication credentials
- Respect rate limits
- Consider using proxies

## Implementation

The web automation module is located in `src/web_automation/` and uses:
- Headless browser engine for rendering
- HTML parsing for content extraction
- Request throttling for rate limiting
- Vector database for content indexing