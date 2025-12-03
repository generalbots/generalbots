# AI Search Template

The AI Search template provides an intelligent document search bot that uses AI to answer questions based on your uploaded documents. It combines vector search with large language models for accurate, context-aware responses.

## Topic: AI-Powered Document Search & Q&A

This template is perfect for:
- Knowledge base assistants
- Document-based customer support
- Internal documentation search
- FAQ automation with source documents

## The Code

```basic
REM AI Search - Intelligent Document Q&A
REM Uses RAG (Retrieval Augmented Generation) for accurate answers

' Add search tools
ADD TOOL "search-documents"
ADD TOOL "summarize-document"

' Use the knowledge base
USE KB "ai-search.gbkb"

' Set up the AI context
SET CONTEXT "document-search" AS "You are a helpful document search assistant. Answer questions based on the documents in your knowledge base. Always cite your sources when possible. If the answer is not in the documents, say so clearly."

' Clear and add suggestions
CLEAR SUGGESTIONS
ADD SUGGESTION "search" AS "Search documents"
ADD SUGGESTION "summary" AS "Get document summary"
ADD SUGGESTION "help" AS "How to use"

BEGIN TALK
**AI Search Assistant** 🔍

I can help you find information in your documents using AI-powered search.

**What I can do:**
• Search across all uploaded documents
• Answer questions with context
• Summarize long documents
• Find specific information quickly

Just ask me a question or describe what you're looking for.
END TALK

BEGIN SYSTEM PROMPT
You are an AI document search assistant with access to a knowledge base of documents.

When answering questions:
1. Search the knowledge base for relevant information
2. Provide accurate answers based on the documents
3. Cite the source document when possible
4. If information isn't found, clearly state that
5. Offer to search for related topics

Be concise but thorough. Always prioritize accuracy over speed.
END SYSTEM PROMPT
```

## Sample Dialogs

These conversations show how the AI Search template works in real-world scenarios.

### Dialog 1: Document Search Query

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🔍</div>
    <div class="wa-header-info">
      <div class="wa-header-name">AI Search</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">AI Search</div>
      <p><strong>AI Search Assistant</strong> 🔍</p>
      <p>I can help you find information in your documents using AI-powered search.</p>
      <p>Just ask me a question or describe what you're looking for.</p>
      <div class="wa-time">10:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>How do I reset the TV to factory settings?</p>
      <div class="wa-time">10:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Based on the LG 42LB5800 manual, here's how to reset to factory settings:</p>
      <p><strong>Option 1 - From Menu:</strong></p>
      <p>1. Press the Settings button on your remote</p>
      <p>2. Navigate to Option → Initial Setting</p>
      <p>3. Select "Reset to Initial Settings"</p>
      <p>4. Confirm when prompted</p>
      <p><strong>Option 2 - Master Reset:</strong></p>
      <p>Press and hold the power button for 10 seconds while unplugged, then plug back in.</p>
      <p>📄 <em>Source: 42LB5800 User Manual, Page 45</em></p>
      <div class="wa-time">10:01</div>
    </div>
  </div>
</div>

### Dialog 2: Information Not Found

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🔍</div>
    <div class="wa-header-info">
      <div class="wa-header-name">AI Search</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>What's the warranty period for the refrigerator?</p>
      <div class="wa-time">11:15 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>I couldn't find specific warranty information for a refrigerator in the current knowledge base.</p>
      <p>The documents I have access to include:</p>
      <p>• LG 42LB5800 TV Manual</p>
      <p>• General product documentation</p>
      <p>Would you like me to:</p>
      <p>1. Search for general warranty policies?</p>
      <p>2. Look for related product information?</p>
      <div class="wa-time">11:15</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Search for general warranty policies</p>
      <div class="wa-time">11:16 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Here's what I found about warranty policies:</p>
      <p>📋 <strong>Standard Warranty Coverage:</strong></p>
      <p>• Parts and labor: 1 year from purchase date</p>
      <p>• Display panel: 2 years</p>
      <p>• Registration required for extended coverage</p>
      <p>📄 <em>Source: Product Documentation, Section 8</em></p>
      <div class="wa-time">11:16</div>
    </div>
  </div>
</div>

### Dialog 3: Document Summary Request

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🔍</div>
    <div class="wa-header-info">
      <div class="wa-header-name">AI Search</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Can you summarize the TV manual for me?</p>
      <div class="wa-time">14:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📺 <strong>LG 42LB5800 Smart TV - Summary</strong></p>
      <p><strong>Key Features:</strong></p>
      <p>• 42" Full HD LED Display (1920x1080)</p>
      <p>• Smart TV with webOS platform</p>
      <p>• Built-in WiFi and Ethernet</p>
      <p>• 3 HDMI ports, 3 USB ports</p>
      <p><strong>Main Sections:</strong></p>
      <p>1. Installation & Setup (pg 5-20)</p>
      <p>2. Remote Control Guide (pg 21-30)</p>
      <p>3. Smart Features & Apps (pg 31-50)</p>
      <p>4. Picture & Sound Settings (pg 51-70)</p>
      <p>5. Troubleshooting (pg 71-90)</p>
      <p>Would you like details on any specific section?</p>
      <div class="wa-time">14:30</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `ADD TOOL` | Register search and summary tools |
| `USE KB` | Connect to the knowledge base |
| `SET CONTEXT` | Define the AI's role and behavior |
| `ADD SUGGESTION` | Create quick action buttons |
| `BEGIN TALK/END TALK` | Multi-line welcome message |
| `BEGIN SYSTEM PROMPT/END SYSTEM PROMPT` | Define AI behavior rules |

## How It Works

1. **Knowledge Base Connection**: `USE KB` loads your documents for semantic search
2. **Tool Registration**: `ADD TOOL` enables search and summarization capabilities
3. **Context Setting**: `SET CONTEXT` tells the AI how to behave
4. **User Query**: User asks a question in natural language
5. **RAG Process**: System searches documents, retrieves relevant chunks
6. **AI Response**: LLM generates answer based on retrieved context

## Template Structure

```
ai-search.gbai/
├── ai-search.gbdialog/
│   ├── start.bas          # Main entry point
│   └── qr.bas             # QR code handler
├── ai-search.gbdrive/
│   └── manuals/           # Folder for PDF documents
│       └── 42LB5800.pdf   # Example manual
├── ai-search.gbkb/
│   └── docs/              # Knowledge base documents
│       └── README.md      # KB documentation
└── ai-search.gbot/
    └── config.csv         # Bot configuration
```

## Customization Ideas

### Add Document Categories

```basic
ADD SUGGESTION "manuals" AS "📚 Product Manuals"
ADD SUGGESTION "policies" AS "📋 Policies"
ADD SUGGESTION "tutorials" AS "🎓 Tutorials"

HEAR category

SWITCH category
    CASE "manuals"
        USE KB "manuals.gbkb"
    CASE "policies"
        USE KB "policies.gbkb"
    CASE "tutorials"
        USE KB "tutorials.gbkb"
END SWITCH
```

### Add Source Citations

```basic
SET CONTEXT "search-with-citations" AS "Always include the document name and page number when citing information. Format: [Document Name, Page X]"
```

### Add Search Filters

```basic
PARAM search_query AS STRING LIKE "how to reset" DESCRIPTION "What to search for"
PARAM doc_type AS STRING LIKE "manual" DESCRIPTION "Type of document: manual, policy, guide"

DESCRIPTION "Search documents with optional type filter"

IF doc_type <> "" THEN
    results = FIND "documents.csv", "type = '" + doc_type + "'"
    ' Search within filtered results
ELSE
    ' Search all documents
END IF
```

### Add Follow-up Questions

```basic
TALK "Here's what I found about your question..."
TALK response

TALK "Would you like me to:"
ADD SUGGESTION "more" AS "Tell me more"
ADD SUGGESTION "related" AS "Show related topics"
ADD SUGGESTION "new" AS "Ask new question"
HEAR followup

IF followup = "more" THEN
    ' Provide more detail
ELSE IF followup = "related" THEN
    ' Show related topics
END IF
```

## Best Practices

1. **Organize Documents**: Keep documents in logical folders within `.gbdrive`
2. **Update Regularly**: Re-index knowledge base when documents change
3. **Clear Context**: Set a specific context to improve answer relevance
4. **Handle Missing Info**: Always gracefully handle cases where info isn't found
5. **Cite Sources**: Configure the AI to cite document sources for credibility

## Related Templates

- [talk-to-data.md](./talk-to-data.md) - Query structured data with natural language
- [crawler.md](./crawler.md) - Crawl websites to build knowledge bases

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:600px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-chat::after{content:'';display:table;clear:both}
.wa-message{clear:both;margin-bottom:10px;max-width:85%;position:relative}
.wa-message.user{float:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;border-radius:8px 0 8px 8px;margin-left:40px}
.wa-message.bot{float:left}
.wa-message.bot .wa-bubble{background-color:#fff;border-radius:0 8px 8px 8px;margin-right:40px}
.wa-bubble{padding:8px 12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
.wa-message.user .wa-time{color:#61a05e}
.wa-sender{font-size:12px;font-weight:600;color:#06cf9c;margin-bottom:2px}
.wa-status.read::after{content:'✓✓';color:#53bdeb;margin-left:4px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>