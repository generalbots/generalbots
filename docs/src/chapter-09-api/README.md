# Chapter 09: APIs and Tools

General Bots transforms BASIC scripts into powerful tools that LLMs can discover and use automatically. This chapter explains how to create tools, integrate with external APIs, and build sophisticated AI capabilities without complex frameworks.

## The Tool Revolution

In the LLM era, tools are not manually coded integrations - they're capabilities that AI can discover and use intelligently. General Bots makes this simple: write a BASIC script, it becomes a tool.

## How Tools Work in General Bots

### Traditional Approach (Complex)
```javascript
// Define schema
const toolSchema = {
  name: "weather",
  description: "Get weather information",
  parameters: {
    type: "object",
    properties: {
      location: { type: "string" }
    }
  }
};

// Register with framework
framework.registerTool(toolSchema, async (params) => {
  // Implementation
});
```

### General Bots Approach (Simple)
```basic
' weather.bas - This is a complete tool!
PARAM location AS STRING LIKE "New York" DESCRIPTION "City to get weather for"
DESCRIPTION "Gets current weather information"

weather = GET "api/weather/" + location
TALK "The weather in " + location + ": " + weather
```

That's it. The LLM can now use this tool whenever users ask about weather.

## Core Concepts

### Tools Are Just BASIC Files
Every `.bas` file in your `.gbdialog` folder is automatically a potential tool. No registration, no configuration, no boilerplate.

### LLMs Orchestrate Everything
The AI decides when to use tools based on user intent. You don't write decision logic - you provide capabilities.

### Parameters Drive Conversations
`PARAM` declarations tell the LLM what information to collect. The AI handles the conversation naturally.

## What You'll Learn

### Tool Creation
- [Tool Definition](./tool-definition.md) - Creating tools from BASIC scripts
- [PARAM Declaration](./param-declaration.md) - Defining tool parameters
- [Tool Compilation](./compilation.md) - How tools are processed

### API Integration
- [External APIs](./external-apis.md) - Calling web services
- [GET Integration](./get-integration.md) - Using GET for API calls
- [Authentication](./authentication.md) - Handling API keys securely

### Advanced Topics
- [MCP Format](./mcp-format.md) - Model Context Protocol
- [Tool Format](./openai-format.md) - OpenAI-compatible functions
- [Tool Discovery](./discovery.md) - How LLMs find tools
- [Error Handling](./error-handling.md) - Robust tool design

## Quick Example: Complete API Integration

Here's a real-world tool that integrates with an external API:

```basic
' translate.bas - Translation tool using external API
PARAM text AS STRING DESCRIPTION "Text to translate"
PARAM target AS STRING LIKE "es" DESCRIPTION "Target language code (es, fr, de, etc)"

DESCRIPTION "Translates text to another language"

' Get API key from secure storage
api_key = GET BOT MEMORY "translation_api_key"

' Call external translation API
result = GET "https://api.translator.com/v1/translate" WITH {
  "text": text,
  "target": target,
  "key": api_key
}

IF result.error THEN
  TALK "Sorry, translation failed: " + result.error
ELSE
  TALK "Translation: " + result.translated_text
END IF
```

The LLM will:
1. Understand when translation is needed
2. Collect the text and target language through natural conversation
3. Call this tool with the right parameters
4. Present the results conversationally

## Why This Approach Works

### No Glue Code
Traditional systems require layers of integration code. In General Bots, BASIC scripts directly become AI capabilities.

### Natural Conversations
Instead of forms and wizards, users have conversations. The AI handles all the complexity of gathering information.

### Rapid Development
Create a new tool in minutes, not days. Change it instantly. No compilation, no deployment process.

### Universal Compatibility
Tools automatically work with any LLM that supports function calling - Groq, OpenAI, Anthropic, or local models.

## Tool Ecosystem

### Built-in Tools
General Bots includes ready-to-use tools for common tasks:
- Email sending
- Database queries  
- Web scraping
- File operations
- Calendar management

### Custom Tools
Create your own tools for:
- Business processes
- API integrations
- Data transformations
- Workflow automation
- External services

### Tool Libraries
Share and reuse tools across bots. Build a library of capabilities for your organization.

## Best Practices

### Keep Tools Focused
Each tool should do one thing well. Let the LLM compose them for complex tasks.

### Use Clear Descriptions
The DESCRIPTION helps the LLM understand when to use your tool:
```basic
DESCRIPTION "Books a meeting room for the specified date and time"
```

### Handle Errors Gracefully
Always provide meaningful feedback when things go wrong:
```basic
IF NOT result.success THEN
  TALK "I couldn't complete that: " + result.message
END IF
```

### Secure API Keys
Never hardcode credentials. Use BOT MEMORY:
```basic
api_key = GET BOT MEMORY "service_api_key"
```

## Real-World Use Cases

### Customer Service
Tools for checking orders, processing returns, updating accounts - all through natural conversation.

### Data Analysis
Connect to databases, generate reports, visualize data - the AI guides users through complex queries.

### Business Automation
Approval workflows, document processing, notification systems - replacing traditional forms with conversation.

### Integration Hub
Connect legacy systems, modern APIs, and cloud services - unified through conversational AI.

## Getting Started

1. **Create a simple tool** - Start with a basic `.bas` file
2. **Add parameters** - Define what information the tool needs
3. **Test with the LLM** - See how the AI uses your tool
4. **Iterate quickly** - Refine based on real usage

## Summary

APIs and tools in General Bots are refreshingly simple. Write BASIC scripts that become AI capabilities. No frameworks, no schemas, no complex integrations. Just describe what you want in simple code, and let the LLM handle the rest.

The future of API integration isn't about writing more code - it's about providing capabilities that AI can orchestrate intelligently.

## Next Steps

- [Tool Definition](./tool-definition.md) - Deep dive into creating tools
- [External APIs](./external-apis.md) - Connect to any web service
- [PARAM Declaration](./param-declaration.md) - Master parameter definitions
---

<div align="center">
  <img src="https://pragmatismo.com.br/icons/general-bots-text.svg" alt="General Bots" width="200">
</div>
