# generate-summary.bas Template

The `update-summary.bas` template demonstrates how to create scheduled tasks that automatically update bot memory with fresh content summaries.

## Overview

This template shows how to:
1. Schedule a recurring task using `SET SCHEDULE`
2. Retrieve documents from the knowledge base
3. Generate summaries using the LLM
4. Store processed content in bot memory for quick access

## Example: Update Summary Script

From the announcements bot's `update-summary.bas`:

```basic
SET SCHEDULE "59 * * * *"

text = GET "announcements.gbkb/news/news.pdf"
resume = LLM "In a few words, resume this: " + text

SET BOT MEMORY "resume", resume

text1 = GET "announcements.gbkb/auxiliom/auxiliom.pdf"
SET BOT MEMORY "auxiliom", text1

text2 = GET "announcements.gbkb/toolbix/toolbix.pdf"
SET BOT MEMORY "toolbix", text2
```

## Breaking Down the Script

### Scheduling with Cron Expression

```basic
SET SCHEDULE "59 * * * *"
```

This schedules the script to run at 59 minutes past every hour. The cron format is:
- Minute (0-59)
- Hour (0-23)
- Day of month (1-31)
- Month (1-12)
- Day of week (0-6)

Common schedule patterns:
- `"0 9 * * *"` - Daily at 9:00 AM
- `"0 */6 * * *"` - Every 6 hours
- `"*/30 * * * *"` - Every 30 minutes
- `"0 0 * * 1"` - Weekly on Monday at midnight

### Retrieving Documents

```basic
text = GET "announcements.gbkb/news/news.pdf"
```

The `GET` keyword retrieves files from the bot's knowledge base stored in drive storage. The path is relative to the bot's bucket.

### Generating Summaries with LLM

```basic
resume = LLM "In a few words, resume this: " + text
```

The `LLM` keyword sends a prompt to the language model and returns the response. Here it's creating a concise summary of the document.

### Storing in Bot Memory

```basic
SET BOT MEMORY "resume", resume
```

Bot memories are persistent key-value pairs that survive across sessions. They're perfect for storing pre-processed content that needs quick access.

## Use Cases

### Daily News Digest

```basic
SET SCHEDULE "0 6 * * *"  ' Daily at 6 AM

news = GET "knowledge/daily-news.txt"
summary = LLM "Create a brief summary of today's key points: " + news

SET BOT MEMORY "daily_digest", summary
```

### Weekly Report Generation

```basic
SET SCHEDULE "0 9 * * 1"  ' Monday at 9 AM

data = GET "reports/weekly-data.csv"
analysis = LLM "Analyze this weekly data and highlight trends: " + data

SET BOT MEMORY "weekly_report", analysis
```

### Content Freshness Check

```basic
SET SCHEDULE "0 */4 * * *"  ' Every 4 hours

doc = GET "policies/current-policy.pdf"
extract = LLM "Extract the version and date from this document: " + doc

SET BOT MEMORY "policy_version", extract
```

## Best Practices

1. **Schedule Wisely**: Avoid scheduling resource-intensive tasks too frequently
2. **Handle Errors**: Check if documents exist before processing
3. **Optimize Prompts**: Use clear, concise prompts for better LLM responses
4. **Memory Management**: Use meaningful keys for bot memories
5. **Test First**: Run the script manually before scheduling

## Integration with Start Dialog

The memories set by scheduled tasks can be used in the start dialog:

```basic
' In start.bas
resume = GET BOT MEMORY "resume"
SET CONTEXT "summary" AS resume
TALK resume
```

## Monitoring Scheduled Tasks

Scheduled tasks run in the background. Check logs for execution status:
- Successful runs are logged at INFO level
- Errors are logged at ERROR level
- Schedule changes are logged when the script runs

## Common Patterns

### Conditional Updates

```basic
SET SCHEDULE "0 * * * *"  ' Hourly

current = GET "data/current.json"
stored = GET BOT MEMORY "last_data"

IF current <> stored THEN
    summary = LLM "Summarize changes in: " + current
    SET BOT MEMORY "last_data", current
    SET BOT MEMORY "change_summary", summary
END IF
```

### Multiple Document Processing

```basic
SET SCHEDULE "0 0 * * *"  ' Daily at midnight

docs = ["doc1.pdf", "doc2.pdf", "doc3.pdf"]
combined = ""

FOR EACH doc IN docs
    content = GET "knowledge/" + doc
    combined = combined + "\n---\n" + content
NEXT

summary = LLM "Create an executive summary: " + combined
SET BOT MEMORY "daily_summary", summary
```

### Time-Based Content

```basic
SET SCHEDULE "0 7 * * *"  ' Daily at 7 AM

hour = HOUR(NOW())
greeting = ""

IF hour < 12 THEN
    greeting = "Good morning! "
ELSE
    greeting = "Good day! "
END IF

news = GET "updates/daily.txt"
brief = LLM "Summarize in one sentence: " + news

SET BOT MEMORY "daily_greeting", greeting + brief
```

## Limitations

- Scripts run asynchronously - don't expect immediate updates
- Long-running scripts may timeout
- Memory updates are not instantly visible to active sessions
- Schedule syntax must be valid cron format

## Summary

The update-summary pattern enables bots to maintain fresh, pre-processed content without manual intervention. By combining `SET SCHEDULE`, `GET`, `LLM`, and `SET BOT MEMORY`, you can create intelligent bots that stay current with changing information and provide instant access to summarized content.