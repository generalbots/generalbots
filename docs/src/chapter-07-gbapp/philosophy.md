# The gbapp Philosophy: Let Machines Do Machine Work

## Core Principle: Automation First

In 2025, the gbapp philosophy is simple and powerful:

**"If a machine can do the work, let it do the work."**

## The Hierarchy of Development

### 1. LLM First (90% of cases)
Let AI write the code for you:
```basic
' Don't write complex logic - describe what you want
result = LLM "Generate a function that validates email addresses and returns true/false: " + email
```

### 2. BASIC for Flow Control (9% of cases)
Use BASIC only for orchestration:
```basic
' BASIC is just glue between AI calls
data = GET "api/data"
processed = LLM "Process this: " + data
SET "results", processed
```

### 3. Rust for Core Only (1% of cases)
Write Rust only when:
- Contributing new keywords to core
- Building fundamental infrastructure
- Optimizing critical performance paths

## What gbapp Really Is

**gbapp is NOT:**
- ❌ External plugin packages
- ❌ Separate npm modules  
- ❌ A way to bypass BASIC
- ❌ Runtime extensions

**gbapp IS:**
- ✅ Virtual crates inside `src/`
- ✅ Rust modules that compile together
- ✅ The bridge between old and new thinking
- ✅ A familiar mental model for contributions
- ✅ A mindset: "Code through automation"

## Real-World Examples

### Wrong Approach (Old Thinking)
```javascript
// 500 lines of custom Node.js, Python or C# code for data validation
function validateComplexBusinessRules(data) {
  // ... hundreds of lines of logic
}
```

### Right Approach (2025 Reality)
```basic
' 3 lines - let AI handle complexity
rules = GET "business-rules.txt"
validation = LLM "Validate this data against these rules: " + data + " Rules: " + rules
IF validation CONTAINS "valid" THEN TALK "Approved" ELSE TALK "Rejected: " + validation
```

## The Multi-SDK Reality

You don't need separate SDKs or plugins. Everything integrates through BASIC + LLM:

### Integrating Any API
```basic
' No SDK needed - just describe what you want
data = GET "https://server/data"
answer = LLM "Do a good report from this json: " + data
TALK data
```

### Working with Any Database
```basic
' No ORM needed - AI understands SQL
results = FIND "users", "all users who logged in today"
```

### Processing Any Format
```basic
' No parser library needed
xml_data = GET "complex.xml"
json = LLM "Convert this XML to JSON: " + xml_data
SET BOT MEMORY "processed_data", json
```

## When to Write Code

### Use LLM When:
- Processing unstructured data
- Implementing business logic
- Transforming between formats
- Making decisions
- Generating content
- Analyzing patterns
- **Basically: 90% of everything**

### Use BASIC When:
- Orchestrating AI calls
- Simple flow control
- Managing state
- Connecting systems
- **Just the glue**

### Use Rust When:
- Building new keywords in your gbapp virtual crate
- Creating a new gbapp module in `src/`
- System-level optimization
- Contributing new features as gbapps
- **Only for core enhancements**

## The gbapp Mindset

Stop thinking about:
- "How do I code this?"
- "What library do I need?"
- "How do I extend the system?"

Start thinking about:
- "How do I describe this to AI?"
- "What's the simplest BASIC flow?"
- "How does this help everyone?"

## Examples of Getting Real

### Data Enrichment (Old Way)
```javascript
// 1000+ lines of code
// Multiple NPM packages
// Complex error handling
// Maintenance nightmare
```

### Data Enrichmentay)
```basic
items = FIND "companies", "needs_enrichment=true"
FOR EACH item IN items
    website = WEBSITE OF item.company
    page = GET website
    enriched = LLM "Extract company info from: " + page
    SET "companies", "id=" + item.id, "data=" + enriched
NEXT
```

### Report Generation (Old Way)
```python
# Custom reporting engine
# Template systems
# Complex formatting logic
# PDF libraries
```

### Report Generation (Get Real Way)
```basic
data = FIND "sales", "month=current"
report = LLM "Create executive summary from: " + data
CREATE SITE "report", "template", report
```

## The Ultimate Test

Before writing ANY code, ask yourself:

1. **Can LLM do this?** (Usually YES)
2. **Can BASIC orchestrate it?** (Almost always YES)
3. **Do I really need Rust?** (Almost never)

## Benefits of This Approach

### For Developers
- 100x faster development
- No dependency management
- No version conflicts
- No maintenance burden
- Focus on business logic, not implementation

### For Organizations
- Reduced complexity
- Lower maintenance costs
- Faster iterations
- No vendor lock-in
- Anyone can contribute

### For the Community
- Shared improvements benefit everyone
- No fragmentation
- Consistent experience
- Collective advancement

## The Future is Already Here

In 2025, this isn't aspirational - it's reality:

- **100% BASIC/LLM applications** are production-ready
- **Zero custom code** for most use cases
- **AI handles complexity** better than humans
- **Machines do machine work** while humans do human work

## Migration Path

### From Extensions to Virtual Crates
```
Old: node_modules/
     └── my-plugin.gbapp/
         ├── index.js (500 lines)
         ├── package.json
         └── complex logic

New: src/
     └── my_feature/        # my_feature.gbapp (virtual crate)
         ├── mod.rs         # 50 lines
         └── keywords.rs    # Register BASIC keywords
     
Plus: my-bot.gbdialog/
      └── logic.bas (5 lines using LLM)
```

### From Code to Descriptions
```
Old: Write algorithm to process data
New: Describe what you want to LLM
```

### From Libraries to LLM
```
Old: Import 20 NPM packages
New: Single LLM call with description
```

## Get Real Guidelines

✅ **DO:**
- Describe problems to LLM
- Use BASIC as glue
- Contribute keywords to core
- Share your patterns
- Think automation-first

❌ **DON'T:**
- Write complex algorithms
- Build separate plugins
- Create custom frameworks
- Maintain separate codebases
- Fight the machine

## The Virtual Crate Architecture

Each gbapp is now a module in `src/`:
```
src/
├── core/           # core.gbapp
├── basic/          # basic.gbapp  
├── channels/       # channels.gbapp
└── your_feature/   # your_feature.gbapp (your contribution!)
```

This elegant mapping preserves the conceptual model while leveraging Rust's power.

## Conclusion

gbapp in 2025 has evolved from external packages to virtual crates - Rust modules inside `src/` that compile into a single, optimized binary. This preserves the familiar mental model while delivering native performance.

The philosophy remains: machines are better at machine work. Your job is to describe what you want, not implement how to do it. The combination of BASIC + LLM eliminates the need for traditional programming in almost all cases.


## Examples Repository

See `/templates/` for real-world examples of 100% BASIC/LLM applications:
- CRM system: 50 lines of BASIC
- Email automation: 30 lines of BASIC
- Data pipeline: 20 lines of BASIC
- Report generator: 15 lines of BASIC

Each would have been thousands of lines in traditional code.
