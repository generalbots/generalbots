# Tool Compilation

botserver compiles BASIC scripts (`.bas` files) into tool definitions that can be called by the LLM. The compilation process extracts parameters, descriptions, and generates metadata for tool discovery.

## Overview

The compilation process reads `.bas` files from `.gbdialog` directories and parses parameter declarations along with descriptions. It then generates tool definitions in both MCP and OpenAI formats, stores the compiled tools in the database, and makes them available for LLM invocation.

## The Compilation Pipeline

### File Detection

The `DriveMonitor` service watches for changes in `.gbdialog` directories. It monitors `.bas` files in drive storage, detects new or modified scripts, and triggers compilation automatically when changes occur.

### Source Processing

When a `.bas` file changes, the compiler downloads the file from drive and creates a local working directory. It then invokes the `BasicCompiler` to process the script and extract the necessary metadata.

### Parameter Extraction

The compiler parses BASIC script headers for `PARAM` declarations with types and examples, `DESCRIPTION` statements for tool documentation, and variable names with default values.

Example script header:
```basic
PARAM name AS string LIKE "John Smith" DESCRIPTION "User's full name"
PARAM age AS number LIKE 25 DESCRIPTION "User's age"
DESCRIPTION "Processes user registration"
```

### Tool Definition Generation

The compiler creates structured tool definitions from the parsed script. The tool name is derived from the filename without the `.bas` extension. Parameters are extracted from PARAM declarations, the description comes from the DESCRIPTION statement, and the script path provides a reference to the source file.

### Database Storage

Compiled tools are stored in the `basic_tools` table, which contains tool metadata including name, description, and parameters. The table also stores source script content, bot association, and compilation timestamp for tracking when tools were last updated.

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

The compiler also generates OpenAI-compatible function definitions for API compatibility:
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

Tools are recompiled automatically when the source `.bas` file is modified, when the file's ETag changes in drive storage, or when a manual recompilation is triggered through the system.

## Working Directory Structure

The compiler maintains a local working directory structured as `./work/bot-name.gbai/bot-name.gbdialog/` containing the individual tool files like `tool1.bas`, `tool2.bas`, and so on. This directory is used for caching compiled scripts, temporary processing during compilation, and debug inspection when troubleshooting issues.

## Error Handling

Compilation errors are handled gracefully to ensure the system remains stable. Syntax errors are logged with line numbers for easy debugging. Missing parameters are reported clearly, invalid types are highlighted in error messages, and compilation continues for other tools even when one fails. Common compilation errors include missing DESCRIPTION statements, invalid PARAM syntax, unsupported parameter types, and general script parsing failures.

## Tool Activation

After successful compilation, the tool is stored in the database and becomes available for the `USE TOOL` keyword. The LLM can discover the tool through its metadata and invoke it during conversations with users.

## Performance Considerations

Compilation is triggered asynchronously to avoid blocking other operations. Multiple tools can be compiled in parallel for efficiency, and results are cached in the database to avoid redundant processing. Only changed files are recompiled, minimizing unnecessary work.

## Debugging Compilation

To debug compilation issues, check the logs for compilation errors that include file names and line numbers. Inspect the working directory files to see the raw script content. Verify that parameter syntax follows the expected format, and test the tool manually with `USE TOOL` to confirm it functions correctly.

## Best Practices

Always include a DESCRIPTION statement to help the LLM understand the tool's purpose. Use clear parameter names that make the code self-documenting. Provide LIKE examples with realistic values to improve LLM parameter filling accuracy. Test tools after making changes to verify compilation succeeded, and check logs regularly to monitor for compilation errors.

## Limitations

Parameters must be declared at the start of the script before any executable code. The compiler only supports basic types including string, number, and boolean. All parameters are required since optional parameters are not currently supported. Nested object parameters and array parameters are also not available in the current implementation.

## Summary

The compilation process transforms BASIC scripts into callable tools that the LLM can discover and invoke. This automatic compilation ensures that changes to scripts are immediately available for use in conversations, making development iteration fast and seamless.