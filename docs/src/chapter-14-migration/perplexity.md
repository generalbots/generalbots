# Perplexity Migration Guide

Migrating from Perplexity to General Bots for AI-powered search and knowledge retrieval.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Overview

Perplexity is an AI-powered search assistant that answers questions with web citations. General Bots provides equivalent and expanded capabilities through its knowledge base, RAG system, and LLM integration—with the advantage of using your own documents, self-hosting, and full customization.

## Why Migrate

| Aspect | Perplexity | General Bots |
|--------|------------|--------------|
| Hosting | Cloud only | Self-hosted |
| Pricing | $20/month Pro | No subscription |
| Knowledge Source | Web search | Your documents + optional web |
| Customization | None | Full BASIC scripting |
| Data Privacy | Queries logged | Complete privacy |
| API Access | Limited | Full REST API |
| Multi-channel | Web only | Web, WhatsApp, Teams, etc. |
| Automation | None | Full workflow automation |
| Integration | None | Any system via API |

## Feature Comparison

### Search and Q&A

| Perplexity Feature | General Bots Equivalent |
|--------------------|------------------------|
| Web search | `USE WEBSITE` + `LLM` |
| Document Q&A (Pro) | `USE KB` + `LLM` |
| Citation generation | RAG with sources |
| Focus modes | `SET CONTEXT` |
| Collections (Pro) | Multiple `.gbkb` folders |
| File upload | Knowledge base indexing |

### What Perplexity Does

1. Searches the web for relevant information
2. Synthesizes answers from multiple sources
3. Provides citations and links
4. Allows follow-up questions

### What General Bots Does

1. Searches your private knowledge base
2. Optionally fetches web content
3. Synthesizes answers with full context
4. Provides source references
5. Allows conversation and follow-ups
6. Automates actions based on answers
7. Deploys to any channel

## Migration Approach

### Step 1: Build Your Knowledge Base

Instead of relying on web search, create a curated knowledge base:

```
my-bot.gbkb/
├── company/
│   ├── policies.pdf
│   ├── procedures.md
│   └── org-chart.pdf
├── products/
│   ├── catalog.pdf
│   ├── specifications.xlsx
│   └── pricing.csv
├── support/
│   ├── faq.md
│   ├── troubleshooting.md
│   └── known-issues.md
└── industry/
    ├── regulations.pdf
    └── best-practices.md
```

### Step 2: Configure RAG

Enable retrieval-augmented generation:

```basic
' Load knowledge collections
USE KB "company"
USE KB "products"
USE KB "support"

' Set assistant behavior
SET CONTEXT "You are a knowledgeable assistant. Answer questions based on the provided documents. Always cite your sources."

' Handle questions
TALK "What would you like to know?"
HEAR question
answer = LLM question
TALK answer
```

### Step 3: Add Web Search (Optional)

For real-time information, add website sources:

```basic
USE KB "internal-docs"
USE WEBSITE "https://docs.example.com"
USE WEBSITE "https://industry-news.com"

answer = LLM "What are the latest updates on " + topic
```

## Recreating Perplexity Features

### Focus Modes

**Perplexity Focus: Academic**

```basic
SET CONTEXT "You are an academic research assistant. Provide scholarly, well-cited responses based on peer-reviewed sources and academic literature. Be precise and thorough."

USE KB "research-papers"
USE KB "academic-journals"

answer = LLM question
```

**Perplexity Focus: Writing**

```basic
SET CONTEXT "You are a professional writing assistant. Help with content creation, editing, and improving text. Focus on clarity, style, and engagement."

answer = LLM "Help me write: " + topic
```

**Perplexity Focus: Code**

```basic
SET CONTEXT "You are an expert programmer. Provide accurate, well-documented code examples. Explain your reasoning and suggest best practices."

USE KB "code-documentation"
USE KB "api-references"

answer = LLM question
```

### Collections

**Perplexity Collections** organize related searches.

**General Bots equivalent:**

```basic
' Create specialized search contexts
WEBHOOK "search-products"
    USE KB "products"
    SET CONTEXT "You are a product specialist."
    answer = LLM body.query
END WEBHOOK

WEBHOOK "search-support"
    USE KB "support"
    SET CONTEXT "You are a support technician."
    answer = LLM body.query
END WEBHOOK

WEBHOOK "search-legal"
    USE KB "legal"
    SET CONTEXT "You are a legal advisor. Always include disclaimers."
    answer = LLM body.query
END WEBHOOK
```

### Pro Search (Deep Research)

**Perplexity Pro Search** performs multi-step research.

**General Bots equivalent:**

```basic
' Deep research workflow
PARAM topic AS string

DESCRIPTION "Perform comprehensive research on a topic"

SET CONTEXT "You are a research analyst. Conduct thorough analysis with multiple perspectives."

USE KB "all-documents"

' Step 1: Initial analysis
initial = LLM "Provide an overview of: " + topic

' Step 2: Deep dive
details = LLM "Now provide detailed analysis with specific examples for: " + topic

' Step 3: Alternative perspectives
alternatives = LLM "What are alternative viewpoints or counterarguments regarding: " + topic

' Step 4: Synthesis
WITH research_prompt
    .instruction = "Synthesize a comprehensive report"
    .overview = initial
    .details = details
    .alternatives = alternatives
END WITH

final_report = LLM "Create a comprehensive report combining: " + research_prompt

TALK final_report
```

### Citation and Sources

**Perplexity** shows numbered citations with links.

**General Bots** provides source references through RAG:

```basic
USE KB "documents"

SET CONTEXT "When answering, always cite which document your information comes from. Format citations as [Source: document name]."

answer = LLM question
TALK answer
```

## What You Gain

### Private Knowledge Base

Your proprietary documents stay private:

```basic
USE KB "confidential-data"
USE KB "internal-reports"

' All queries against your own data
' Nothing sent to external search engines
answer = LLM sensitive_question
```

### Custom AI Behavior

Fine-tune responses for your specific needs:

```basic
SET CONTEXT "You are the customer service assistant for Acme Corp.
- Always be friendly and professional
- If you don't know something, offer to connect with a human
- Never discuss competitor products
- Emphasize our satisfaction guarantee"

answer = LLM customer_question
```

### Multi-Channel Deployment

Access your AI assistant anywhere:

```basic
' Same knowledge base, any channel
' Web chat, WhatsApp, Teams, Slack, SMS, Email

TALK "How can I help you?"
HEAR question
USE KB "company-knowledge"
answer = LLM question
TALK answer
```

### Automation Beyond Q&A

Take action based on queries:

```basic
USE KB "products"

TALK "What are you looking for?"
HEAR query

answer = LLM query

' If user wants to order, take action
IF CONTAINS(LOWER(query), "order") OR CONTAINS(LOWER(query), "buy") THEN
    TALK "Would you like me to start an order?"
    HEAR confirm AS BOOLEAN
    IF confirm THEN
        CREATE TASK "Follow up on order inquiry" DUE DATEADD(NOW(), 1, "day")
        SEND MAIL TO "sales@company.com" SUBJECT "Order Inquiry" BODY "Customer asked: " + query
    END IF
END IF

TALK answer
```

### API for Integration

Create search APIs for your applications:

```basic
WEBHOOK "search"

USE KB params.collection
SET CONTEXT params.context

answer = LLM params.query

WITH response
    .answer = answer
    .query = params.query
    .timestamp = NOW()
END WITH
```

Call from any application:

```
POST /api/search
{
  "collection": "products",
  "context": "You are a product expert",
  "query": "What's the best option for enterprise?"
}
```

## Migration Checklist

### Pre-Migration

- [ ] Identify information sources you frequently search
- [ ] Gather documents to build knowledge base
- [ ] Determine required focus modes/contexts
- [ ] Plan deployment channels
- [ ] Set up General Bots environment

### Migration

- [ ] Organize documents into .gbkb collections
- [ ] Create context configurations
- [ ] Build specialized search endpoints
- [ ] Test with common queries
- [ ] Configure multi-channel access

### Post-Migration

- [ ] Compare answer quality
- [ ] Train team on new interface
- [ ] Monitor and refine contexts
- [ ] Add automation workflows
- [ ] Expand knowledge base as needed

## Example: Complete Search Assistant

```basic
' search-assistant.bas
' A Perplexity-like search experience with General Bots

' Load knowledge bases
USE KB "company-docs"
USE KB "product-info"
USE KB "industry-knowledge"

' Configure AI behavior
SET CONTEXT "You are an intelligent search assistant. 
Provide accurate, well-sourced answers. 
When citing information, mention the source document.
If you're uncertain, acknowledge the limitations.
Be concise but comprehensive."

' Main conversation loop
TALK "Hello! I can search our knowledge base and help answer your questions. What would you like to know?"

LOOP
    HEAR query
    
    IF LOWER(query) = "exit" OR LOWER(query) = "quit" THEN
        TALK "Goodbye!"
        EXIT LOOP
    END IF
    
    ' Generate response with sources
    answer = LLM query
    TALK answer
    
    ' Offer follow-up
    TALK "Would you like to know more about any aspect of this?"
LOOP
```

## Best Practices

**Curate your knowledge base.** Quality documents produce better answers than random web search.

**Use specific contexts.** Tailor the AI's behavior for different use cases rather than using generic settings.

**Iterate on prompts.** Refine your `SET CONTEXT` instructions based on the quality of responses.

**Combine sources strategically.** Mix internal documents with curated external sources for comprehensive coverage.

**Add automation.** Go beyond Q&A—let your assistant take actions, create tasks, and integrate with workflows.

## See Also

- [Knowledge Base](../chapter-03/README.md) - Building effective KBs
- [USE KB](../chapter-06-gbdialog/keyword-use-kb.md) - Knowledge base keyword
- [SET CONTEXT](../chapter-06-gbdialog/keyword-set-context.md) - AI configuration
- [Platform Comparison](./comparison-matrix.md) - Full feature comparison