# Template for Creating Templates (template.gbai)

A General Bots meta-template that serves as a starting point for creating new bot templates.

---

## Overview

The Template template (yes, it's a template for templates!) provides the essential structure and best practices for creating new General Bots templates. Use this as your foundation when building custom templates for specific use cases.

## Features

- **Standard Structure** - Pre-configured folder hierarchy
- **Best Practices** - Follows General Bots conventions
- **Documentation Ready** - Includes README template
- **Quick Start** - Minimal setup required

---

## Package Structure

```
template.gbai/
├── README.md                    # Template documentation
├── template.gbdialog/           # Dialog scripts
│   └── send.bas                 # Example script (placeholder)
├── template.gbdrive/            # File storage
│   └── (your files here)
├── template.gbkb/               # Knowledge base (optional)
│   └── docs/
└── template.gbot/               # Bot configuration
    └── config.csv
```

---

## Creating a New Template

### Step 1: Copy the Template

```bash
cp -r templates/template.gbai templates/your-template.gbai
```

### Step 2: Rename Internal Folders

Rename all internal folders to match your template name:

```bash
cd templates/your-template.gbai
mv template.gbdialog your-template.gbdialog
mv template.gbdrive your-template.gbdrive
mv template.gbot your-template.gbot
```

### Step 3: Configure Your Bot

Edit `your-template.gbot/config.csv`:

```csv
name,value
Bot Name,Your Bot Name
Theme Color,blue
Answer Mode,default
LLM Provider,openai
```

### Step 4: Create Dialog Scripts

Add your BASIC scripts to `your-template.gbdialog/`:

```basic
' start.bas - Main entry point
ADD TOOL "your-tool"

USE KB "your-template.gbkb"

CLEAR SUGGESTIONS

ADD SUGGESTION "option1" AS "First Option"
ADD SUGGESTION "option2" AS "Second Option"
ADD SUGGESTION "help" AS "Get Help"

BEGIN TALK
**Your Bot Name**

Welcome! I can help you with:
• Feature 1
• Feature 2
• Feature 3

What would you like to do?
END TALK

BEGIN SYSTEM PROMPT
You are a helpful assistant for [your use case].

Guidelines:
- Be helpful and concise
- Use the available tools when appropriate
- Ask clarifying questions when needed
END SYSTEM PROMPT
```

### Step 5: Add Tools

Create tool scripts with proper parameters:

```basic
' your-tool.bas
PARAM input AS STRING LIKE "example" DESCRIPTION "Description of this parameter"
PARAM optional_param AS STRING DESCRIPTION "Optional parameter" OPTIONAL

DESCRIPTION "What this tool does - this helps the LLM decide when to use it"

' Your implementation here
result = DO_SOMETHING(input)

IF result THEN
    RETURN result
ELSE
    RETURN {"error": "Something went wrong"}
END IF
```

### Step 6: Add Knowledge Base (Optional)

If your template needs reference documentation:

```
your-template.gbkb/
└── docs/
    ├── feature1.md
    ├── feature2.md
    └── faq.md
```

### Step 7: Update README

Replace the README with documentation for your template following the standard format.

---

## Template Checklist

Before publishing your template, ensure:

- [ ] All folders renamed to match template name
- [ ] `config.csv` configured with appropriate defaults
- [ ] `start.bas` provides clear entry point
- [ ] All tools have `PARAM` and `DESCRIPTION`
- [ ] System prompt guides LLM behavior
- [ ] README documents all features
- [ ] No hardcoded credentials or secrets
- [ ] Error handling implemented
- [ ] Example conversations documented

---

## Naming Conventions

| Item | Convention | Example |
|------|------------|---------|
| Template folder | `kebab-case.gbai` | `my-crm.gbai` |
| Dialog scripts | `kebab-case.bas` | `add-contact.bas` |
| Tools | `kebab-case` | `search-products` |
| Config keys | `Title Case` | `Theme Color` |
| Table names | `PascalCase` | `CustomerOrders` |

---

## File Templates

### config.csv Template

```csv
name,value
Bot Name,Your Bot Name
Theme Color,blue
Answer Mode,default
LLM Provider,openai
Admin Email,admin@company.com
```

### start.bas Template

```basic
' Register tools
ADD TOOL "tool-name"

' Load knowledge base
USE KB "your-template.gbkb"

' Configure suggestions
CLEAR SUGGESTIONS
ADD SUGGESTION "action" AS "Do Something"

' Welcome message
BEGIN TALK
**Bot Name**

Welcome message here.
END TALK

' System prompt
BEGIN SYSTEM PROMPT
You are a helpful assistant.
Define behavior and guidelines here.
END SYSTEM PROMPT
```

### Tool Template

```basic
PARAM required_param AS STRING LIKE "example" DESCRIPTION "What this is"
PARAM optional_param AS STRING DESCRIPTION "Optional input" OPTIONAL

DESCRIPTION "What this tool does"

' Implementation
result = YOUR_LOGIC_HERE

IF result THEN
    RETURN result
ELSE
    RETURN {"error": "Error message"}
END IF
```

---

## Best Practices

### Dialog Scripts

1. **Clear entry point** - `start.bas` should be the main entry
2. **Register tools** - Use `ADD TOOL` for LLM-callable functions
3. **Set context** - Use `SET CONTEXT` and `BEGIN SYSTEM PROMPT`
4. **Add suggestions** - Guide users with quick actions
5. **Welcome message** - Use `BEGIN TALK` for introduction

### Tools

1. **Descriptive names** - Tool names should indicate function
2. **Parameter examples** - Use `LIKE` to show expected format
3. **Clear descriptions** - Help LLM understand when to use
4. **Return values** - Always return meaningful results
5. **Error handling** - Handle edge cases gracefully

### Configuration

1. **Sensible defaults** - Work out of the box
2. **Document options** - Explain each setting
3. **No secrets** - Use environment variables for credentials
4. **Flexible** - Allow customization without code changes

### Documentation

1. **Clear overview** - Explain what the template does
2. **Feature list** - Highlight capabilities
3. **Usage examples** - Show common scenarios
4. **Configuration guide** - Document all settings
5. **Troubleshooting** - Address common issues

---

## Publishing Your Template

1. **Test thoroughly** - Verify all features work
2. **Document completely** - Users should be self-sufficient
3. **Follow conventions** - Consistent with other templates
4. **Update main README** - Add to templates list
5. **Submit PR** - Include template and documentation

---

## Sample Conversation

```
User: How do I create a new template?

Bot:  To create a new template:

      1. Copy the template.gbai folder
      2. Rename it to your-template.gbai
      3. Rename internal folders to match
      4. Edit config.csv with your settings
      5. Create your dialog scripts in .gbdialog
      6. Add tools as separate .bas files
      7. Update the README
      
      Would you like help with any specific step?

User: How do I add a tool?

Bot:  To add a tool, create a .bas file in your .gbdialog folder:

      ' my-tool.bas
      PARAM input AS STRING LIKE "example" DESCRIPTION "Input parameter"
      
      DESCRIPTION "What this tool does"
      
      ' Your logic here
      result = PROCESS(input)
      RETURN result
      
      Then register it in start.bas:
      
      ADD TOOL "my-tool"
      
      The LLM will automatically call your tool when appropriate
      based on the DESCRIPTION you provided.
```

---

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Tool not found | Not registered | Add `ADD TOOL "name"` in start.bas |
| Tool not called | Poor description | Improve DESCRIPTION text |
| Wrong folder structure | Names don't match | Ensure all folders use same prefix |
| Config not loading | Wrong format | Check CSV syntax |
| KB not found | Wrong path | Verify USE KB path matches folder |

---

## Related Resources

- [Templates Reference](./templates.md) - Full template list
- [Template Samples](./template-samples.md) - Example conversations
- [gbdialog Reference](../chapter-06-gbdialog/README.md) - BASIC scripting guide
- [Bot Configuration](../chapter-08-config/README.md) - Configuration options

---

## Use Cases for Custom Templates

- **Industry-Specific** - Healthcare, legal, finance bots
- **Department-Specific** - HR, IT, sales assistants
- **Process Automation** - Workflow-specific bots
- **Integration Templates** - Connect to specific APIs/systems
- **Vertical Solutions** - Complete solutions for business needs

---

## See Also

- [Templates Reference](./templates.md) - Full template list
- [Template Samples](./template-samples.md) - Example conversations
- [gbdialog Reference](../chapter-06-gbdialog/README.md) - BASIC scripting guide