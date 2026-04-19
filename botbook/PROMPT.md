# botbook Development Guide

**Version:** 6.2.0
**Purpose:** Documentation for General Bots (mdBook)

---

## CRITICAL: Keyword Naming Rules

**Keywords NEVER use underscores. Always use spaces.**

### Correct Syntax
```basic
SEND MAIL to, subject, body, attachments
GENERATE PDF template, data, output
MERGE PDF files, output
DELETE "url"
ON ERROR RESUME NEXT
SET BOT MEMORY key, value
KB STATISTICS
```

### WRONG - Never Use Underscores
```basic
SEND_MAIL          ' WRONG!
GENERATE_PDF       ' WRONG!
DELETE_HTTP        ' WRONG!
```

### Keyword Mappings
| Write This | NOT This |
|------------|----------|
| `SEND MAIL` | `SEND_MAIL` |
| `GENERATE PDF` | `GENERATE_PDF` |
| `MERGE PDF` | `MERGE_PDF` |
| `DELETE` | `DELETE_HTTP` |
| `SET HEADER` | `SET_HEADER` |
| `FOR EACH` | `FOR_EACH` |

---

## OFFICIAL ICONS - MANDATORY

**NEVER generate icons with LLM. Use official SVG icons from `botui/ui/suite/assets/icons/`**

### Usage in Documentation

```markdown
<!-- Reference icons in docs -->
![Chat](../assets/icons/gb-chat.svg)

<!-- With HTML for sizing -->
<img src="../assets/icons/gb-analytics.svg" alt="Analytics" width="24">
```

---

## STRUCTURE

```
botbook/
├── book.toml          # mdBook configuration
├── src/
│   ├── SUMMARY.md     # Table of contents
│   ├── README.md      # Introduction
│   ├── 01-introduction/   # Quick Start
│   ├── 02-templates/      # Package System
│   ├── 03-knowledge-base/ # Knowledge Base
│   ├── 06-gbdialog/       # BASIC Dialogs
│   ├── 08-config/         # Configuration
│   ├── 10-rest/           # REST API
│   └── assets/            # Images, diagrams
├── i18n/              # Translations
└── book/              # Generated output
```

---

## DOCUMENTATION RULES

```
- All documentation MUST match actual source code
- Extract real keywords from botserver/src/basic/keywords/
- Use actual examples from botserver/templates/
- Version numbers must be 6.2.0
- No placeholder content - only verified features
```

---

## NO ASCII DIAGRAMS - MANDATORY

**NEVER use ASCII art diagrams. ALL diagrams must be SVG.**

### Prohibited ASCII Patterns
```
+-------+
|  Box  |
+-------+
```

### What to Use Instead

| Instead of... | Use... |
|---------------|--------|
| ASCII box diagrams | SVG diagrams in `assets/` |
| ASCII flow charts | SVG with arrows and boxes |
| ASCII directory trees | Markdown tables |

---

## SVG DIAGRAM GUIDELINES

All SVGs must support light/dark modes:

```xml
<style>
  .title-text { fill: #1E1B4B; }
  .main-text { fill: #334155; }

  @media (prefers-color-scheme: dark) {
    .title-text { fill: #F1F5F9; }
    .main-text { fill: #E2E8F0; }
  }
</style>
```

---

## CONVERSATION EXAMPLES

Use WhatsApp-style HTML format for bot interactions:

```html
<div class="wa-chat">
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Hello! How can I help?</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I want to enroll</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
</div>
```

---

## SOURCE CODE REFERENCES

| Topic | Source Location |
|-------|-----------------|
| BASIC Keywords | `botserver/src/basic/keywords/` |
| Database Models | `botserver/src/core/shared/models.rs` |
| API Routes | `botserver/src/core/urls.rs` |
| Configuration | `botserver/src/core/config/` |
| Templates | `botserver/templates/` |

---

## BUILDING BOTSERVER

**CRITICAL: ALWAYS USE DEBUG BUILD DURING DEVELOPMENT**

```bash
# CORRECT - Use debug build (FAST)
cargo build

# WRONG - NEVER use --release during development (SLOW)
# cargo build --release  # DO NOT USE!

# Run debug server
cargo run
```

**Why Debug Build:**
- Debug builds compile in ~30 seconds
- Release builds take 5-10 minutes with LTO
- Debug builds are sufficient for development and testing
- Only use `--release` for production deployment

---

## BUILDING DOCUMENTATION

```bash
# Install mdBook
cargo install mdbook

# Build documentation
cd botbook && mdbook build

# Serve locally with hot reload
mdbook serve --open
```

---

## TESTING PROCEDURES

### Tool Testing Workflow

**CRITICAL: NO STOP UNTIL NO MORE ERRORS IN TOOLS**

When testing bot tools, follow this sequential process WITHOUT STOPPING:

```
1. Test Tool #1
   ├─ Fill form one field at a time (if multi-step form)
   ├─ Verify NO ERRORS in output
   ├─ Check Result types are NOT visible (no "core::result::Result<..." strings)
   ├─ Verify database save (if applicable)
   ├─ IF ERRORS FOUND: FIX THEM IMMEDIATELY, RE-TEST SAME TOOL
   ├─ ONLY move to next tool when CURRENT tool has ZERO errors

2. Test Tool #2
   └─ (repeat process - DO NOT STOP if errors found)

3. Continue until ALL tools tested with ZERO errors
```

**IMPORTANT:**
- Do NOT stop testing to summarize or ask questions
- Do NOT move to next tool if current tool has errors
- Fix errors immediately, rebuild, re-test same tool
- Only proceed when current tool is completely error-free

### Error Patterns to Watch For

**CRITICAL ERRORS (Must Fix Before Proceeding):**
- `core::result::Result<alloc::string::String, alloc::string::String>` in output
- `invalid input syntax for type timestamp`
- `Some("Desculpe, houve um erro...")`
- Empty balloon messages
- Rust debug info visible to users

### Playwright Testing Tricks

```javascript
// Click tool button
await page.getByRole('button', { name: 'Evento/Iluminação' }).click();

// Wait for response
await page.waitForTimeout(3000);

// Take snapshot
await page.snapshot();

// Fill form field by field
await page.getByRole('textbox').fill('field value');
await page.getByRole('textbox').press('Enter');
```

### Test Documentation

After testing each tool, document:
1. Tool name and ID
2. All required parameters
3. Expected behavior
4. Any issues found and fixes applied
5. Final test result (PASS/FAIL)

---

## REMEMBER

- **Accuracy** - Must match botserver source code
- **Completeness** - No placeholder sections
- **Clarity** - Accessible to BASIC enthusiasts
- **Keywords** - NEVER use underscores - always spaces
- **NO ASCII art** - Use SVG diagrams only
- **Version 6.2.0** - Always reference 6.2.0
- **GIT WORKFLOW** - ALWAYS push to ALL repositories (github, pragmatismo)
- **TESTING** - Test tools ONE BY ONE, fix ALL errors before moving to next tool
- **NO STOP** - DO NOT STOP testing until ALL tools have ZERO errors - fix immediately and re-test
