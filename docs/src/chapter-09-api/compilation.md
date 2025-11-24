# Tool Compilation

BotServer compiles BASIC scripts (`.bas` files) into tool definitions that can be called by the LLM. The compilation process extracts parameters, descriptions, and generates metadata for tool discovery.

## Overview

The compilation process:
1. Reads `.bas` files from `.gbdialog` directories
2. Parses parameter declarations and descriptions
3. Generates tool definitions in MCP and OpenAI formats
4. Stores compiled tools in the database
5. Makes tools available for LLM invocation

## The Compilation Pipeline

### 1. File Detection

The `DriveMonitor` service watches for changes in `.gbdialog` directories:
- Monitors `.bas` files in drive storage
- Detects new or modified scripts
- Triggers compilation automatically

### 2. Source Processing

When a `.bas` file changes, the compiler:
- Downloads the file from drive
- Creates a local working directory
- Invokes the `BasicCompiler` to process the script

### 3. Parameter Extraction

The compiler parses BASIC script headers for:
- `PARAM` declarations with types and examples
- `DESCRIPTION` statements for tool documentation
- Variable names and default values

Example script header:
```basic
PARAM name AS string LIKE "John Smith" DESCRIPTION "User's full name"
PARAM age AS number LIKE 25 DESCRIPTION "User's age"
DESCRIPTION "Processes user registration"
```

### 4. Tool Definition Generation

The compiler creates structured tool definitions:
- **Tool name**: Derived from filename (without `.bas` extension)
- **Parameters**: Extracted from PARAM declarations
- **Description**: From DESCRIPTION statement
- **Script path**: Reference to the source file

### 5. Database Storage

Compiled tools are stored in the `basic_tools` table:
- Tool metadata (name, description, parameters)
- Source script content
- Bot association
- Compilation timestamp

## Compilation Output Formats

### MCP (Model Context Protocol) Format

The compiler generates MCP-compatible tool definitions:
```json
{
  "name": "user_registration",
  "description": "Processes user registration",
  "input_schema": {
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "User's full name"
      },
      "age": {
        "type": "number",
        "description": "User's age"
      }
    },
    "required": ["name", "age"]
  }
}
```

### OpenAI Function Format

Also generates OpenAI-compatible function definitions for API compatibility:
```json
{
  "name": "user_registration",
  "description": "Processes user registration",
  "parameters": {
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "User's full name"
      },
      "age": {
        "type": "number",
        "description": "User's age"
      }
    },
    "required": ["name", "age"]
  }
}
```

## Automatic Recompilation

Tools are recompiled automatically when:
- The source `.bas` file is modified
- The file's ETag changes in drive storage
- A manual recompilation is triggered

## Working Directory Structure

The compiler maintains a local working directory:
```
<svg width="400" height="250" viewBox="0 0 400 250" xmlns="http://www.w3.org/2000/svg" style="background: transparent;">
  <!-- Title -->
  <text x="200" y="20" text-anchor="middle" font-family="system-ui, -apple-system, sans-serif" font-size="14" font-weight="300" fill="currentColor" opacity="0.9">Directory Structure</text>
  
  <!-- Root folder -->
  <rect x="50" y="40" width="100" height="30" fill="none" stroke="currentColor" stroke-width="1.5" rx="4" opacity="0.8"/>
  <text x="100" y="59" text-anchor="middle" font-family="monospace" font-size="11" fill="currentColor">./work/</text>
  
  <!-- Connecting line to bot folder -->
  <path d="M 100 70 L 100 85 L 120 85" stroke="currentColor" stroke-width="1" fill="none" opacity="0.6"/>
  
  <!-- Bot folder -->
  <rect x="120" y="70" width="140" height="30" fill="none" stroke="currentColor" stroke-width="1.5" rx="4" opacity="0.8"/>
  <text x="190" y="89" text-anchor="middle" font-family="monospace" font-size="11" fill="currentColor">bot-name.gbai/</text>
  
  <!-- Connecting line to dialog folder -->
  <path d="M 190 100 L 190 115 L 210 115" stroke="currentColor" stroke-width="1" fill="none" opacity="0.6"/>
  
  <!-- Dialog folder -->
  <rect x="210" y="100" width="160" height="30" fill="none" stroke="currentColor" stroke-width="1.5" rx="4" opacity="0.8"/>
  <text x="290" y="119" text-anchor="middle" font-family="monospace" font-size="11" fill="currentColor">bot-name.gbdialog/</text>
  
  <!-- Connecting lines to tool files -->
  <path d="M 290 130 L 290 140" stroke="currentColor" stroke-width="1" fill="none" opacity="0.6"/>
  <path d="M 290 140 L 280 140 L 280 150" stroke="currentColor" stroke-width="1" fill="none" opacity="0.6"/>
  <path d="M 290 140 L 290 150" stroke="currentColor" stroke-width="1" fill="none" opacity="0.6"/>
  <path d="M 290 140 L 300 140 L 300 150" stroke="currentColor" stroke-width="1" fill="none" opacity="0.6"/>
  
  <!-- Tool files -->
  <rect x="240" y="150" width="80" height="25" fill="none" stroke="currentColor" stroke-width="1" rx="3" opacity="0.7" stroke-dasharray="2,2"/>
  <text x="280" y="167" text-anchor="middle" font-family="monospace" font-size="10" fill="currentColor">tool1.bas</text>
  
  <rect x="250" y="180" width="80" height="25" fill="none" stroke="currentColor" stroke-width="1" rx="3" opacity="0.7" stroke-dasharray="2,2"/>
  <text x="290" y="197" text-anchor="middle" font-family="monospace" font-size="10" fill="currentColor">tool2.bas</text>
  
  <rect x="260" y="210" width="80" height="25" fill="none" stroke="currentColor" stroke-width="1" rx="3" opacity="0.7" stroke-dasharray="2,2"/>
  <text x="300" y="227" text-anchor="middle" font-family="monospace" font-size="10" fill="currentColor">tool3.bas</text>
</svg>
```

This directory is used for:
- Caching compiled scripts
- Temporary processing
- Debug inspection

## Error Handling

Compilation errors are handled gracefully:
- Syntax errors logged with line numbers
- Missing parameters reported
- Invalid types highlighted
- Compilation continues for other tools

Common compilation errors:
- Missing DESCRIPTION statement
- Invalid PARAM syntax
- Unsupported parameter types
- Script parsing failures

## Tool Activation

After successful compilation:
1. Tool is stored in database
2. Available for `USE TOOL` keyword
3. Discoverable by LLM
4. Can be invoked in conversations

## Performance Considerations

- Compilation is triggered asynchronously
- Multiple tools compiled in parallel
- Results cached in database
- Only changed files recompiled

## Debugging Compilation

To debug compilation issues:
1. Check logs for compilation errors
2. Inspect working directory files
3. Verify parameter syntax
4. Test tool manually with `USE TOOL`

## Best Practices

1. **Always include DESCRIPTION**: Helps LLM understand tool purpose
2. **Use clear parameter names**: Self-documenting code
3. **Provide LIKE examples**: Improves LLM parameter filling
4. **Test after changes**: Verify compilation succeeded
5. **Check logs**: Monitor for compilation errors

## Limitations

- Parameters must be declared at script start
- Only supports basic types (string, number, boolean)
- Cannot have optional parameters (all are required)
- No nested object parameters
- No array parameters

## Summary

The compilation process transforms BASIC scripts into callable tools that the LLM can discover and invoke. This automatic compilation ensures that changes to scripts are immediately available for use in conversations, making development iteration fast and seamless.