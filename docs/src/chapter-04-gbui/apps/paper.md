# Paper - AI Writing

> **Your intelligent document editor**

![Paper Flow](../../assets/suite/paper-flow.svg)

---

## Overview

Paper is the AI-powered writing app in General Bots Suite. Create documents, reports, letters, and more with help from your AI assistant. Paper understands context, suggests improvements, and helps you write faster and better.

---

## Interface Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Paper                              [Share] [Export ▼] [⚙️]       [×]   │
├──────────────┬──────────────────────────────────────────────────────────┤
│              │  ┌─────────────────────────────────────────────────────┐ │
│  DOCUMENTS   │  │ B  I  U  S  │ H1 H2 H3 │ • ≡ ☐ │ 🔗 📷 📊 │ ✨ AI │ │
│  ───────────  │  └─────────────────────────────────────────────────────┘ │
│              │                                                          │
│  📄 Untitled │  ┌─────────────────────────────────────────────────────┐ │
│  📄 Report   │  │                                                     │ │
│  📄 Notes    │  │  Quarterly Report                                   │ │
│  📄 Letter   │  │  ═══════════════════                                │ │
│              │  │                                                     │ │
│  ───────────  │  │  Executive Summary                                  │ │
│  📁 Projects │  │  ──────────────────                                  │ │
│    📄 Plan   │  │  This quarter showed significant growth across      │ │
│    📄 Budget │  │  all business units. Revenue increased by 15%       │ │
│              │  │  compared to the previous quarter, driven by        │ │
│  ───────────  │  │  strong performance in the enterprise segment.     │ │
│              │  │                                                     │ │
│  [+ New Doc] │  │  Key Highlights                                     │ │
│              │  │  ──────────────                                      │ │
│              │  │  • Revenue: $2.4M (+15%)                            │ │
│              │  │  • New customers: 47                                │ │
│              │  │  • Customer retention: 94%                          │ │
│              │  │                                                     │ │
│              │  │  |                                                  │ │
│              │  │                                                     │ │
│              │  └─────────────────────────────────────────────────────┘ │
├──────────────┴──────────────────────────────────────────────────────────┤
│  Words: 156  │  Characters: 892  │  Reading time: 1 min  │  Saved ✓    │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Features

### Creating a New Document

**Method 1: Click New Document**

1. Click **+ New Doc** in the left sidebar
2. Start typing immediately
3. Document auto-saves as you work

**Method 2: From Template**

1. Click **+ New Doc**
2. Select **From Template**
3. Choose a template:

```
┌─────────────────────────────────────────────────────────────────┐
│  Choose a Template                                        [×]   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │             │  │   ═══════   │  │    ┌───┐    │             │
│  │    Blank    │  │   Report    │  │    │   │    │             │
│  │             │  │   ───────   │  │    Letter   │             │
│  │             │  │   • • •     │  │    └───┘    │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Meeting   │  │   ☐ ☐ ☐    │  │     📧      │             │
│  │    Notes    │  │  Checklist  │  │    Email    │             │
│  │   ───────   │  │   ☐ ☐ ☐    │  │   Template  │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Project   │  │   Resume    │  │   Invoice   │             │
│  │  Proposal   │  │    / CV     │  │             │             │
│  │             │  │             │  │             │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Method 3: Ask the Bot**

```
You: Create a new document for meeting notes
Bot: ✅ Created new document: "Meeting Notes"
     
     I've set up a template with:
     • Date and attendees section
     • Agenda
     • Discussion points
     • Action items
     
     [Open Document]
```

---

### Formatting Text

Use the toolbar or keyboard shortcuts to format your text:

```
┌───────────────────────────────────────────────────────────────────────┐
│  B   I   U   S   │  H1  H2  H3  │  •  ≡  ☐  │  🔗  📷  📊  │  ✨ AI  │
│  ▲   ▲   ▲   ▲      ▲   ▲   ▲      ▲  ▲  ▲      ▲   ▲   ▲       ▲    │
│  │   │   │   │      │   │   │      │  │  │      │   │   │       │    │
│  │   │   │   │      │   │   │      │  │  │      │   │   │       │    │
│Bold │   │Strike    │   │   │    Bullet│ │    Link│   │       AI │    │
│   Italic │         │ Heading │      │Checklist │Image│      Help │    │
│       Underline    │   2     │   Numbered  │       Table            │
│                   Heading 1 Heading 3     List                       │
└───────────────────────────────────────────────────────────────────────┘
```

| Format | Toolbar | Shortcut |
|--------|---------|----------|
| **Bold** | B | `Ctrl+B` |
| *Italic* | I | `Ctrl+I` |
| Underline | U | `Ctrl+U` |
| ~~Strikethrough~~ | S | `Ctrl+Shift+S` |
| Heading 1 | H1 | `Ctrl+1` |
| Heading 2 | H2 | `Ctrl+2` |
| Heading 3 | H3 | `Ctrl+3` |
| Bullet list | • | `Ctrl+Shift+8` |
| Numbered list | ≡ | `Ctrl+Shift+7` |
| Checklist | ☐ | `Ctrl+Shift+9` |
| Link | 🔗 | `Ctrl+K` |

---

### AI Writing Assistant ✨

Paper's AI assistant helps you write better and faster. Click the **✨ AI** button or type `/` to access AI features.

**Generate Content**

```
You: /write an introduction about renewable energy
Bot: Here's a draft introduction:

     "Renewable energy has emerged as one of the most 
     significant solutions to our global climate challenges.
     As fossil fuel reserves dwindle and environmental 
     concerns grow, solar, wind, and hydroelectric power
     offer sustainable alternatives that can power our
     future while protecting our planet."
     
     [Insert] [Regenerate] [Edit]
```

**Improve Writing**

1. Select text you want to improve
2. Click **✨ AI** → **Improve**
3. Choose an option:

```
┌─────────────────────────────────────┐
│  ✨ AI Assistant                    │
├─────────────────────────────────────┤
│                                     │
│  What would you like to do?         │
│                                     │
│  [📝 Improve writing      ]         │
│  [✂️  Make it shorter     ]         │
│  [📖 Make it longer       ]         │
│  [🎯 Make it clearer      ]         │
│  [💼 Make it professional ]         │
│  [😊 Make it friendly     ]         │
│  [🔧 Fix grammar          ]         │
│  [🌐 Translate            ]         │
│                                     │
└─────────────────────────────────────┘
```

**Summarize**

```
You: /summarize this document
Bot: 📋 Document Summary:
     
     This quarterly report covers company performance
     from January to March 2025. Key points:
     
     • Revenue grew 15% to $2.4M
     • 47 new customers acquired
     • 94% customer retention rate
     • Enterprise segment drove growth
     
     The report recommends expanding the sales team
     and investing in product development.
```

**Generate Ideas**

```
You: /brainstorm topics for a blog post about productivity
Bot: 💡 Here are 10 blog post ideas:
     
     1. "5 Morning Habits of Highly Productive People"
     2. "The Pomodoro Technique: A Complete Guide"
     3. "How to Eliminate Distractions While Working"
     4. "Time Blocking: Schedule Your Way to Success"
     5. "The 2-Minute Rule: Small Tasks, Big Impact"
     6. "Digital Minimalism for Better Focus"
     7. "Why Multitasking is Killing Your Productivity"
     8. "Building a Productive Workspace at Home"
     9. "The Science of Taking Breaks"
     10. "Productivity Apps That Actually Work"
     
     Would you like me to expand on any of these?
```

---

### Document Organization

**Folders**

Organize your documents into folders:

1. Right-click in the sidebar
2. Select **New Folder**
3. Name your folder
4. Drag documents into it

```
┌──────────────────────┐
│  DOCUMENTS           │
│  ───────────          │
│  📄 Quick Notes      │
│  📄 Ideas            │
│                      │
│  📁 Work             │
│     📄 Report Q1     │
│     📄 Report Q2     │
│     📄 Presentation  │
│                      │
│  📁 Personal         │
│     📄 Goals 2025    │
│     📄 Journal       │
│                      │
│  📁 Archive          │
└──────────────────────┘
```

**Search Documents**

Find documents quickly:

1. Press `Ctrl+P` or click the search icon
2. Type document name or content
3. Press Enter to open

```
┌─────────────────────────────────────────┐
│  🔍 Search documents...                 │
├─────────────────────────────────────────┤
│                                         │
│  Recent:                                │
│  📄 Quarterly Report           2h ago   │
│  📄 Meeting Notes - May 15     1d ago   │
│  📄 Project Proposal           3d ago   │
│                                         │
│  All documents matching "report":       │
│  📄 Quarterly Report                    │
│  📄 Annual Report 2024                  │
│  📄 Expense Report Template             │
│                                         │
└─────────────────────────────────────────┘
```

---

### Collaboration

**Share a Document**

1. Click **Share** button
2. Enter email addresses
3. Set permissions
4. Click **Send**

```
┌─────────────────────────────────────────┐
│  Share Document                   [×]   │
├─────────────────────────────────────────┤
│                                         │
│  Share with:                            │
│  ┌─────────────────────────────────┐    │
│  │ sarah@company.com               │    │
│  └─────────────────────────────────┘    │
│  [+ Add more people]                    │
│                                         │
│  Permission: [Can edit        ▼]        │
│              ┌────────────────┐         │
│              │ Can edit       │         │
│              │ Can comment    │         │
│              │ Can view       │         │
│              └────────────────┘         │
│                                         │
│  ☐ Notify people via email              │
│                                         │
│  ───────────────────────────────        │
│                                         │
│  Or share via link:                     │
│  ┌─────────────────────────────────┐    │
│  │ https://paper.bot/doc/abc123   │    │
│  └─────────────────────────────────┘    │
│  [Copy Link]                            │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │            Share                │    │
│  └─────────────────────────────────┘    │
│                                         │
└─────────────────────────────────────────┘
```

**Permissions Explained**

| Permission | Can View | Can Comment | Can Edit |
|------------|----------|-------------|----------|
| **View** | ✅ | ❌ | ❌ |
| **Comment** | ✅ | ✅ | ❌ |
| **Edit** | ✅ | ✅ | ✅ |

---

### Export Options

Export your documents to different formats:

1. Click **Export ▼**
2. Choose a format:

| Format | Best For |
|--------|----------|
| **PDF** | Printing, sharing final versions |
| **Word (.docx)** | Editing in Microsoft Word |
| **Markdown (.md)** | Technical documentation |
| **Plain Text (.txt)** | Simple text without formatting |
| **HTML** | Web publishing |

```
┌─────────────────────────────────────────┐
│  Export Document                  [×]   │
├─────────────────────────────────────────┤
│                                         │
│  Export as:                             │
│                                         │
│  ┌────────────┐  ┌────────────┐         │
│  │    PDF     │  │    Word    │         │
│  │   📄       │  │    📝      │         │
│  └────────────┘  └────────────┘         │
│                                         │
│  ┌────────────┐  ┌────────────┐         │
│  │  Markdown  │  │    Text    │         │
│  │    #       │  │    Aa      │         │
│  └────────────┘  └────────────┘         │
│                                         │
│  Options:                               │
│  ☑ Include headers and footers          │
│  ☐ Include comments                     │
│  ☑ Include page numbers                 │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │           Export                │    │
│  └─────────────────────────────────┘    │
│                                         │
└─────────────────────────────────────────┘
```

---

### Version History

Paper automatically saves versions of your document:

1. Click **⚙️** → **Version History**
2. See all saved versions
3. Click to preview
4. Restore if needed

```
┌─────────────────────────────────────────┐
│  Version History                  [×]   │
├─────────────────────────────────────────┤
│                                         │
│  ● Current version                      │
│    Today, 3:45 PM                       │
│                                         │
│  ○ Today, 2:30 PM                       │
│    Added executive summary              │
│                                         │
│  ○ Today, 11:15 AM                      │
│    Initial draft                        │
│                                         │
│  ○ Yesterday, 4:00 PM                   │
│    Created document                     │
│                                         │
│  ─────────────────────────────────      │
│                                         │
│  [Preview]  [Restore This Version]      │
│                                         │
└─────────────────────────────────────────┘
```

---

## Keyboard Shortcuts

### Text Formatting

| Shortcut | Action |
|----------|--------|
| `Ctrl+B` | Bold |
| `Ctrl+I` | Italic |
| `Ctrl+U` | Underline |
| `Ctrl+Shift+S` | Strikethrough |
| `Ctrl+1` | Heading 1 |
| `Ctrl+2` | Heading 2 |
| `Ctrl+3` | Heading 3 |
| `Ctrl+0` | Normal text |

### Lists & Structure

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+7` | Numbered list |
| `Ctrl+Shift+8` | Bullet list |
| `Ctrl+Shift+9` | Checklist |
| `Tab` | Indent |
| `Shift+Tab` | Outdent |

### Editing

| Shortcut | Action |
|----------|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+C` | Copy |
| `Ctrl+X` | Cut |
| `Ctrl+V` | Paste |
| `Ctrl+A` | Select all |
| `Ctrl+F` | Find |
| `Ctrl+H` | Find and replace |

### Navigation

| Shortcut | Action |
|----------|--------|
| `Ctrl+P` | Quick open document |
| `Ctrl+S` | Save (auto-saves anyway) |
| `Ctrl+N` | New document |
| `Ctrl+W` | Close document |
| `Escape` | Close dialog/menu |

### AI Features

| Shortcut | Action |
|----------|--------|
| `/` | Open AI command menu |
| `Ctrl+Shift+A` | AI improve selection |
| `Ctrl+Shift+G` | Generate content |

---

## Tips & Tricks

### Writing Tips

💡 **Use headings** to organize your document - makes it scannable

💡 **Write first, edit later** - don't let perfectionism slow you down

💡 **Use AI to overcome writer's block** - ask for ideas or outlines

💡 **Break long paragraphs** into shorter ones for readability

### Productivity Tips

💡 **Use templates** for recurring documents (reports, meeting notes)

💡 **Learn keyboard shortcuts** - much faster than clicking

💡 **Use `/` commands** for quick AI assistance

💡 **Set up folders** to keep documents organized

### AI Tips

💡 **Be specific** when asking AI for help - better prompts = better results

💡 **Use "Make it shorter"** for concise professional writing

💡 **Ask for multiple versions** and pick the best one

💡 **Use AI to check grammar** before sharing important documents

---

## Troubleshooting

### Document not saving

**Possible causes:**
1. Internet connection lost
2. Browser storage full
3. Session expired

**Solution:**
1. Check internet connection
2. Copy your text as backup (`Ctrl+A`, `Ctrl+C`)
3. Refresh the page
4. Log in again if prompted
5. Paste your text back if needed

---

### Formatting not working

**Possible causes:**
1. Text not selected
2. Format not supported in current context
3. Browser compatibility issue

**Solution:**
1. Select the text first, then apply formatting
2. Try a different format
3. Use keyboard shortcuts instead of toolbar
4. Try a different browser

---

### AI features not responding

**Possible causes:**
1. AI service temporarily unavailable
2. Network timeout
3. Request too long

**Solution:**
1. Wait a few seconds and try again
2. Try a shorter text selection
3. Refresh the page
4. Check if other AI features work

---

### Can't share document

**Possible causes:**
1. No sharing permissions
2. Invalid email address
3. Document not saved

**Solution:**
1. Check if you're the document owner
2. Verify email addresses are correct
3. Wait for document to save (check status bar)
4. Contact administrator if sharing is restricted

---

### Export fails

**Possible causes:**
1. Document too large
2. Special characters causing issues
3. Browser blocking download

**Solution:**
1. Try exporting a smaller section first
2. Remove any unusual characters or images
3. Check browser download settings
4. Try a different export format

---

## BASIC Integration

Control Paper from your bot dialogs:

### Create a Document

```basic
doc = CREATE DOCUMENT "Project Notes"
doc.content = "Meeting notes from " + TODAY

SAVE DOCUMENT doc
TALK "Document created: " + doc.id
```

### Generate Content with AI

```basic
HEAR topic AS TEXT "What should I write about?"

content = GENERATE TEXT "Write a brief introduction about " + topic

doc = CREATE DOCUMENT topic
doc.content = content
SAVE DOCUMENT doc

TALK "I've created a document about " + topic
TALK "Here's a preview:"
TALK LEFT(content, 200) + "..."
```

### Export a Document

```basic
HEAR docName AS TEXT "Which document should I export?"

doc = FIND DOCUMENT docName

IF doc IS NOT NULL THEN
    pdf = EXPORT DOCUMENT doc AS "PDF"
    TALK "Here's your PDF:"
    SEND FILE pdf
ELSE
    TALK "Document not found"
END IF
```

### Search Documents

```basic
HEAR query AS TEXT "What are you looking for?"

results = SEARCH DOCUMENTS query

IF COUNT(results) > 0 THEN
    TALK "I found " + COUNT(results) + " documents:"
    FOR EACH doc IN results
        TALK "- " + doc.title
    NEXT
ELSE
    TALK "No documents found matching '" + query + "'"
END IF
```

### Summarize a Document

```basic
HEAR docName AS TEXT "Which document should I summarize?"

doc = FIND DOCUMENT docName

IF doc IS NOT NULL THEN
    summary = SUMMARIZE doc.content
    TALK "Summary of '" + doc.title + "':"
    TALK summary
ELSE
    TALK "Document not found"
END IF
```

---

## See Also

- [Drive App](./drive.md) - Store and organize files
- [Mail App](./mail.md) - Email your documents
- [Research App](./research.md) - Research topics for your writing
- [How To: Add Documents to Knowledge Base](../how-to/add-kb-documents.md)