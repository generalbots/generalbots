# Research - AI Search

> **Your intelligent research assistant**

![Research Flow](../../assets/suite/research-flow.svg)

---

## Overview

Research is the AI-powered search and discovery app in General Bots Suite. Find information from the web, your documents, and databases using natural language. Research understands your questions, finds relevant sources, and presents organized answers with citations.

---

## Interface Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Research                                      [History] [Settings] [×] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  🔍 Ask anything...                                       [→]   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Search in:  [● All] [○ Web] [○ Documents] [○ Database]                │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  📋 ANSWER                                                      │   │
│  │  ─────────                                                       │   │
│  │                                                                 │   │
│  │  Renewable energy sources include solar, wind, hydroelectric,   │   │
│  │  geothermal, and biomass. Solar energy has seen the fastest     │   │
│  │  growth, with global capacity increasing 25% annually. Wind     │   │
│  │  power is the second largest source of renewable electricity.   │   │
│  │                                                                 │   │
│  │  Key Statistics (2024):                                         │   │
│  │  • Solar: 1,200 GW global capacity                              │   │
│  │  • Wind: 900 GW global capacity                                 │   │
│  │  • Hydro: 1,400 GW global capacity                              │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  📚 SOURCES (5)                                    [Show More]  │   │
│  │  ─────────────                                                   │   │
│  │  1. Energy Report 2024.pdf - Company KB           [📄 View]     │   │
│  │  2. IEA World Energy Outlook - iea.org            [🔗 Open]     │   │
│  │  3. Renewable Growth Statistics - energy.gov      [🔗 Open]     │   │
│  │  4. Internal Policy Document.docx - Company KB    [📄 View]     │   │
│  │  5. Climate Action Report - unfccc.int            [🔗 Open]     │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  [📋 Copy Answer] [📄 Export] [💬 Ask Follow-up] [🔄 New Search]        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Features

### Basic Search

Just type your question in natural language:

**Examples of questions you can ask:**

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           EXAMPLE QUERIES                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  📊 Business Questions                                                  │
│  • "What are our sales numbers for Q1 2025?"                           │
│  • "Who are our top 10 customers by revenue?"                          │
│  • "What's our refund policy?"                                         │
│                                                                         │
│  📚 Knowledge Questions                                                 │
│  • "How does photosynthesis work?"                                     │
│  • "What are the main causes of climate change?"                       │
│  • "Explain blockchain technology"                                     │
│                                                                         │
│  🔍 Research Questions                                                  │
│  • "Compare React vs Vue for web development"                          │
│  • "What are the latest trends in AI?"                                 │
│  • "Find studies about remote work productivity"                       │
│                                                                         │
│  📋 Document Questions                                                  │
│  • "What does our employee handbook say about PTO?"                    │
│  • "Find the budget approval process"                                  │
│  • "What were the action items from last month's meeting?"             │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### Search Sources

Choose where to search:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Search in:                                                             │
│                                                                         │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      │
│  │     ● All        │  │     ○ Web        │  │   ○ Documents    │      │
│  │  ───────────     │  │  ───────────     │  │  ───────────     │      │
│  │  Search all      │  │  Search the      │  │  Search your     │      │
│  │  available       │  │  internet for    │  │  knowledge       │      │
│  │  sources         │  │  public info     │  │  base files      │      │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘      │
│                                                                         │
│  ┌──────────────────┐  ┌──────────────────┐                            │
│  │   ○ Database     │  │   ○ Custom       │                            │
│  │  ───────────     │  │  ───────────     │                            │
│  │  Query your      │  │  Select          │                            │
│  │  structured      │  │  specific        │                            │
│  │  data            │  │  sources         │                            │
│  └──────────────────┘  └──────────────────┘                            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

| Source | What It Searches | Best For |
|--------|------------------|----------|
| **All** | Everything available | General questions |
| **Web** | Internet/public websites | Current events, general knowledge |
| **Documents** | Your uploaded files (.gbkb) | Company policies, internal info |
| **Database** | Connected databases | Business data, reports |
| **Custom** | Specific sources you choose | Focused research |

---

### AI-Powered Answers

Research doesn't just find links—it reads, understands, and summarizes:

```
You: What is our vacation policy?

Research: 📋 ANSWER
          ─────────
          
          According to your Employee Handbook (2024), the vacation 
          policy is as follows:
          
          Full-Time Employees:
          • 0-2 years: 15 days per year
          • 2-5 years: 20 days per year
          • 5+ years: 25 days per year
          
          Key Points:
          • Vacation must be requested 2 weeks in advance
          • Up to 5 days can be carried over to next year
          • Unused days over the limit are forfeited
          • Manager approval required for requests over 5 consecutive days
          
          📚 SOURCE: Employee_Handbook_2024.pdf, Page 23-24
```

---

### Follow-Up Questions

Continue your research with follow-up questions:

```
You: What are the main benefits of solar energy?

Research: [Shows answer about solar energy benefits...]

You: How does it compare to wind energy?

Research: 📋 COMPARING SOLAR VS WIND ENERGY
          ──────────────────────────────────
          
          Based on your previous question, here's a comparison:
          
          ┌────────────────┬─────────────────┬─────────────────┐
          │ Factor         │ Solar           │ Wind            │
          ├────────────────┼─────────────────┼─────────────────┤
          │ Installation   │ Rooftops, land  │ Open areas      │
          │ Consistency    │ Daylight hours  │ Variable        │
          │ Maintenance    │ Lower           │ Moderate        │
          │ Cost/kWh       │ $0.03-0.06      │ $0.02-0.05      │
          │ Lifespan       │ 25-30 years     │ 20-25 years     │
          └────────────────┴─────────────────┴─────────────────┘
          
          Both are excellent renewable options. Solar is better for
          individual buildings, while wind is more efficient at scale.
```

---

### Source Citations

Every answer includes citations so you can verify:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  📚 SOURCES                                                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Internal Documents:                                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 📄 Employee_Handbook_2024.pdf                                   │   │
│  │    Location: Company KB / HR / Policies                         │   │
│  │    Relevant pages: 23-24                                        │   │
│  │    [View Document] [Copy Citation]                              │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  Web Sources:                                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ 🔗 Department of Energy - Renewable Energy Basics               │   │
│  │    URL: energy.gov/renewable-energy                             │   │
│  │    Retrieved: May 15, 2025                                      │   │
│  │    [Open Link] [Copy Citation]                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### Research History

Access your previous searches:

1. Click **History** in the top right
2. Browse or search past queries
3. Click to revisit any search

```
┌─────────────────────────────────────────┐
│  Research History               [×]     │
├─────────────────────────────────────────┤
│  🔍 Search history...                   │
├─────────────────────────────────────────┤
│                                         │
│  TODAY                                  │
│  ─────                                  │
│  • "vacation policy"           2:30 PM  │
│  • "Q1 sales report"          11:15 AM  │
│  • "competitor analysis"       9:45 AM  │
│                                         │
│  YESTERDAY                              │
│  ─────────                              │
│  • "renewable energy trends"   4:20 PM  │
│  • "project timeline"          2:00 PM  │
│                                         │
│  LAST WEEK                              │
│  ─────────                              │
│  • "budget approval process"            │
│  • "customer feedback summary"          │
│  • "marketing strategy 2025"            │
│                                         │
│  [Clear History]                        │
│                                         │
└─────────────────────────────────────────┘
```

---

### Export Results

Save your research for later use:

1. Click **📄 Export**
2. Choose format:

| Format | Best For |
|--------|----------|
| **PDF** | Sharing, printing |
| **Markdown** | Documentation |
| **Word** | Reports, editing |
| **Copy to Paper** | Continue writing |

```
┌─────────────────────────────────────────┐
│  Export Research                  [×]   │
├─────────────────────────────────────────┤
│                                         │
│  Include:                               │
│  ☑ Answer                               │
│  ☑ Sources with citations               │
│  ☐ Search query                         │
│  ☐ Timestamp                            │
│                                         │
│  Format:                                │
│  ┌─────────┐  ┌─────────┐              │
│  │   PDF   │  │  Word   │              │
│  └─────────┘  └─────────┘              │
│  ┌─────────┐  ┌─────────┐              │
│  │Markdown │  │  Paper  │              │
│  └─────────┘  └─────────┘              │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │          Export                 │    │
│  └─────────────────────────────────┘    │
│                                         │
└─────────────────────────────────────────┘
```

---

### Advanced Search

Use operators for more precise searches:

| Operator | Example | What It Does |
|----------|---------|--------------|
| `""` | `"exact phrase"` | Find exact match |
| `AND` | `solar AND wind` | Both terms required |
| `OR` | `solar OR wind` | Either term |
| `NOT` | `energy NOT nuclear` | Exclude term |
| `site:` | `site:company.com` | Search specific site |
| `type:` | `type:pdf` | Search specific file type |
| `date:` | `date:2025` | Filter by date |
| `in:` | `in:documents` | Search specific source |

**Examples:**

```
"quarterly report" AND sales date:2025
```
Finds documents with exact phrase "quarterly report" AND the word "sales" from 2025

```
project proposal NOT draft type:pdf
```
Finds PDF files about project proposals, excluding drafts

```
budget in:documents site:finance
```
Searches documents in the finance folder for budget information

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `/` | Focus search box |
| `Enter` | Search |
| `Ctrl+Enter` | Search in new tab |
| `Escape` | Clear search / close panel |
| `↑` / `↓` | Navigate results |
| `Ctrl+C` | Copy answer |
| `Ctrl+S` | Save/export results |
| `H` | Open history |
| `Tab` | Cycle through sources |
| `1-5` | Jump to source N |

---

## Tips & Tricks

### Better Search Results

💡 **Be specific** - "Q1 2025 sales revenue by region" works better than "sales"

💡 **Use natural language** - Ask questions like you would ask a colleague

💡 **Try different phrasings** - If results aren't great, rephrase your question

💡 **Use follow-ups** - Build on previous searches for deeper research

### Finding Documents

💡 **Mention the document type** - "Find the PDF about vacation policy"

💡 **Reference dates** - "Meeting notes from last Tuesday"

💡 **Name departments** - "HR policies about sick leave"

### Web Research

💡 **Be current** - Add "2025" or "latest" for recent information

💡 **Compare sources** - Research shows multiple sources for verification

💡 **Check citations** - Click through to verify important information

---

## Troubleshooting

### No results found

**Possible causes:**
1. Query too specific
2. Information not in knowledge base
3. Typo in search terms

**Solution:**
1. Try broader search terms
2. Search "All" sources instead of one
3. Check spelling
4. Try different phrasing
5. Upload relevant documents to knowledge base

---

### Wrong or irrelevant results

**Possible causes:**
1. Ambiguous query
2. Outdated documents in KB
3. Source selection too broad

**Solution:**
1. Be more specific in your question
2. Use quotes for exact phrases
3. Select specific source (Documents only, Web only)
4. Use advanced operators

---

### Sources not loading

**Possible causes:**
1. Document was moved or deleted
2. Web page no longer available
3. Permission issues

**Solution:**
1. Check if document exists in Drive
2. Try opening the web link directly
3. Ask administrator about permissions
4. Use cached/saved version if available

---

### Search is slow

**Possible causes:**
1. Searching many sources
2. Large knowledge base
3. Complex query

**Solution:**
1. Select specific source instead of "All"
2. Be more specific to narrow results
3. Wait for indexing to complete (if recent uploads)
4. Check network connection

---

### AI answer seems incorrect

**Possible causes:**
1. Outdated information in sources
2. AI misinterpreted question
3. Conflicting information in sources

**Solution:**
1. Always verify with cited sources
2. Rephrase your question
3. Ask for clarification: "Are you sure about X?"
4. Check multiple sources for accuracy

---

## BASIC Integration

Use Research in your bot dialogs:

### Basic Search

```basic
HEAR question AS TEXT "What would you like to know?"

result = SEARCH question

TALK result.answer

TALK "Sources:"
FOR EACH source IN result.sources
    TALK "- " + source.title
NEXT
```

### Search Specific Sources

```basic
' Search only documents
result = SEARCH "vacation policy" IN "documents"

' Search only web
result = SEARCH "latest AI news" IN "web"

' Search specific knowledge base
result = SEARCH "product specs" IN "products.gbkb"
```

### Research with Follow-up

```basic
TALK "What would you like to research?"
HEAR topic AS TEXT

result = SEARCH topic
TALK result.answer

HEAR followUp AS TEXT "Any follow-up questions? (or 'done')"

WHILE followUp <> "done"
    result = SEARCH followUp WITH CONTEXT result
    TALK result.answer
    HEAR followUp AS TEXT "Any more questions? (or 'done')"
WEND

TALK "Research complete!"
```

### Export Research

```basic
HEAR query AS TEXT "What should I research?"

result = SEARCH query

' Export as PDF
pdf = EXPORT RESEARCH result AS "PDF"
SEND FILE pdf

' Or copy to Paper
doc = CREATE DOCUMENT "Research: " + query
doc.content = result.answer + "\n\nSources:\n" + result.citations
SAVE DOCUMENT doc

TALK "Research saved to Paper"
```

### Automated Research Report

```basic
topics = ["market trends", "competitor analysis", "customer feedback"]

report = ""
FOR EACH topic IN topics
    result = SEARCH topic + " 2025"
    report = report + "## " + topic + "\n\n"
    report = report + result.answer + "\n\n"
NEXT

doc = CREATE DOCUMENT "Weekly Research Report"
doc.content = report
SAVE DOCUMENT doc

TALK "Research report created with " + COUNT(topics) + " topics"
```

---

## See Also

- [Paper App](./paper.md) - Write documents based on your research
- [Drive App](./drive.md) - Upload documents to knowledge base
- [Chat App](./chat.md) - Ask quick questions
- [How To: Add Documents to Knowledge Base](../how-to/add-kb-documents.md)