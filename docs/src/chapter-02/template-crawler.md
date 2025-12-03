# Web Crawler Template (crawler.gbai)

A General Bots template for automated web crawling and content extraction for knowledge base population.

---

## Overview

The Crawler template enables your bot to automatically fetch, parse, and index web content. It's designed for building knowledge bases from websites, monitoring web pages for changes, and extracting structured data from online sources.

## Features

- **Automated Web Scraping** - Fetch and parse web pages automatically
- **Document Mode** - Answer questions based on crawled content
- **Configurable Depth** - Control how many pages to crawl
- **Content Indexing** - Automatically add content to knowledge base
- **LLM Integration** - Use AI to understand and summarize crawled content

---

## Package Structure

```
crawler.gbai/
├── crawler.gbkb/          # Knowledge base for crawled content
│   └── docs/              # Indexed documents
└── crawler.gbot/
    └── config.csv         # Crawler configuration
```

---

## Configuration

Configure the crawler in `crawler.gbot/config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `Website` | Target URL to crawl | `https://pragmatismo.com.br/` |
| `website Max Documents` | Maximum pages to crawl | `2` |
| `Answer Mode` | How to respond to queries | `document` |
| `Theme Color` | UI theme color | `purple` |
| `LLM Provider` | AI provider for processing | `openai` |

### Example config.csv

```csv
name,value
Website,https://pragmatismo.com.br/
website Max Documents,2
Answer Mode,document
Theme Color,purple
LLM Provider,openai
```

---

## How It Works

1. **Initialization** - Bot reads the target website from configuration
2. **Crawling** - Fetches pages starting from the root URL
3. **Extraction** - Parses HTML and extracts meaningful text content
4. **Indexing** - Stores content in the knowledge base for RAG
5. **Q&A** - Users can ask questions about the crawled content

---

## Usage

### Basic Setup

1. Copy the template to your bot's packages directory:

```bash
cp -r templates/crawler.gbai /path/to/your/bot/packages/
```

2. Edit `crawler.gbot/config.csv` with your target website:

```csv
name,value
Website,https://your-website.com/
website Max Documents,10
Answer Mode,document
```

3. Deploy and the bot will automatically crawl the configured site.

### Querying Crawled Content

Once crawled, users can ask questions naturally:

- "What services does the company offer?"
- "Tell me about the pricing"
- "Summarize the about page"
- "What are the main features?"

### Answer Modes

| Mode | Behavior |
|------|----------|
| `document` | Answers strictly based on crawled content |
| `hybrid` | Combines crawled content with general knowledge |
| `summary` | Provides concise summaries of relevant pages |

---

## Sample Conversation

```
User: What does this company do?

Bot:  Based on the website content:
      
      Pragmatismo provides enterprise AI solutions 
      including:
      
      • General Bots platform
      • Custom AI development
      • Conversational AI consulting
      • Integration services
      
      📄 Source: pragmatismo.com.br/about

User: Do they have any case studies?

Bot:  Yes! According to the website, they have worked 
      with several enterprise clients:
      
      • Banking sector automation
      • Healthcare chatbots
      • E-commerce customer service
      
      📄 Source: pragmatismo.com.br/cases

User: How can I contact them?

Bot:  Contact information from the website:
      
      📧 Email: contact@pragmatismo.com.br
      📍 Location: São Paulo, Brazil
      🌐 Website: pragmatismo.com.br
      
      📄 Source: pragmatismo.com.br/contact
```

---

## Advanced Configuration

### Limiting Crawl Scope

Control which pages are crawled:

```csv
name,value
Website,https://example.com/docs/
website Max Documents,50
Website Include Pattern,/docs/*
Website Exclude Pattern,/docs/archive/*
```

### Scheduling Recrawls

Set up periodic recrawling to keep content fresh:

```csv
name,value
Website Refresh Schedule,0 0 * * 0
```

This example recrawls every Sunday at midnight.

### Authentication

For sites requiring authentication:

```csv
name,value
Website Auth Type,basic
Website Username,user
Website Password,secret
```

---

## Customization

### Creating Custom Crawl Logic

Create a BASIC dialog for custom crawling:

```basic
' custom-crawl.bas
urls = ["https://site1.com", "https://site2.com", "https://site3.com"]

FOR EACH url IN urls
    content = GET url
    
    IF content THEN
        SAVE "crawled_pages.csv", url, content, NOW()
        SET CONTEXT content
    END IF
NEXT

TALK "Crawled " + UBOUND(urls) + " pages successfully."
```

### Processing Crawled Content

Use LLM to process and structure crawled data:

```basic
' process-crawled.bas
pages = FIND "crawled_pages.csv"

FOR EACH page IN pages
    summary = LLM "Summarize this content in 3 bullet points: " + page.content
    
    WITH processed
        url = page.url
        summary = summary
        processed_at = NOW()
    END WITH
    
    SAVE "processed_content.csv", processed
NEXT
```

### Extracting Structured Data

Extract specific information from pages:

```basic
' extract-products.bas
SET CONTEXT "You are a data extraction assistant. Extract product information as JSON."

page_content = GET "https://store.example.com/products"

products = LLM "Extract all products with name, price, and description as JSON array: " + page_content

SAVE "products.json", products
```

---

## Integration Examples

### With Knowledge Base

```basic
' Add crawled content to KB
content = GET "https://docs.example.com/api"

IF content THEN
    USE KB "api-docs.gbkb"
    ADD TO KB content, "API Documentation"
END IF
```

### With Notifications

```basic
' Monitor for changes
previous = GET BOT MEMORY "last_content"
current = GET "https://news.example.com"

IF current <> previous THEN
    SEND EMAIL "admin@company.com", "Website Changed", "The monitored page has been updated."
    SET BOT MEMORY "last_content", current
END IF
```

---

## Best Practices

1. **Respect robots.txt** - Only crawl pages allowed by the site's robots.txt
2. **Rate limiting** - Don't overwhelm target servers with requests
3. **Set reasonable limits** - Start with low `Max Documents` values
4. **Monitor content quality** - Review crawled content for accuracy
5. **Keep content fresh** - Schedule periodic recrawls for dynamic sites
6. **Handle errors gracefully** - Implement retry logic for failed requests

---

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| No content indexed | Invalid URL | Verify the Website URL is accessible |
| Partial content | Max Documents too low | Increase the limit in config |
| Stale answers | Content not refreshed | Set up scheduled recrawls |
| Authentication errors | Missing credentials | Add auth settings to config |
| Timeout errors | Slow target site | Increase timeout settings |

---

## Limitations

- JavaScript-rendered content may not be fully captured
- Some sites block automated crawlers
- Large sites may take significant time to fully crawl
- Dynamic content may require special handling

---

## Use Cases

- **Documentation Bots** - Index product docs for support
- **Competitive Intelligence** - Monitor competitor websites
- **News Aggregation** - Collect news from multiple sources
- **Research Assistants** - Build knowledge bases from academic sources
- **FAQ Generators** - Extract FAQs from help sites

---

## Related Templates

- [AI Search](./template-ai-search.md) - AI-powered document search
- [Talk to Data](./template-talk-to-data.md) - Natural language data queries
- [Law](./template-law.md) - Legal document processing with similar RAG approach

---

## See Also

- [Templates Reference](./templates.md) - Full template list
- [Template Samples](./template-samples.md) - Example conversations
- [gbkb Reference](../chapter-03/README.md) - Knowledge base guide