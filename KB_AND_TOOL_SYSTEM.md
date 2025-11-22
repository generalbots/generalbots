# KB and TOOL System Documentation

## Overview

The General Bots system provides **4 essential keywords** for managing Knowledge Bases (KB) and Tools dynamically during conversation sessions:

1. **ADD_KB** - Load and embed files from `.gbkb` folders into vector database
2. **CLEAR_KB** - Remove KB from current session
3. **ADD_TOOL** - Make a tool available for LLM to call
4. **CLEAR_TOOLS** - Remove all tools from current session

---

## Knowledge Base (KB) System

### What is a KB?

A Knowledge Base (KB) is a **folder containing documents** (`.gbkb` folder structure) that are **vectorized/embedded and stored in a vector database**. The vectorDB retrieves relevant chunks/excerpts to inject into prompts, giving the LLM context-aware responses.

### Folder Structure

```
work/
  {bot_name}/
    {bot_name}.gbkb/          # Knowledge Base root
      circular/               # KB folder 1
        document1.pdf
        document2.md
        document3.txt
      comunicado/             # KB folder 2
        announcement1.txt
        announcement2.pdf
      policies/               # KB folder 3
        policy1.md
        policy2.pdf
      procedures/             # KB folder 4
        procedure1.docx
```

### `ADD_KB "kb-name"`

**Purpose:** Loads and embeds files from the `.gbkb/kb-name` folder into the vector database and makes them available for semantic search in the current session.

**How it works:**
1. Reads all files from `work/{