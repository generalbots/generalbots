# Introduction to GeneralBots

GeneralBots is an open-source bot platform that enables users to create, deploy, and manage conversational AI applications using a simple BASIC-like scripting language. The platform provides a comprehensive ecosystem for building intelligent chatbots with knowledge base integration, tool calling, and multi-channel support.

## What is GeneralBots?

GeneralBots allows users to create sophisticated bot applications without extensive programming knowledge. The system uses a package-based architecture where each component serves a specific purpose:

- **.gbai** - Application architecture and structure
- **.gbdialog** - Conversation scripts and dialog flows  
- **.gbkb** - Knowledge base collections for contextual information
- **.gbot** - Bot configuration and parameters
- **.gbtheme** - UI theming and customization
- **.gbdrive** - File storage and management

## Key Features

- **BASIC Scripting**: Simple, English-like syntax for creating bot dialogs
- **Vector Database**: Semantic search and knowledge retrieval using Qdrant
- **Multi-Channel**: Support for web, voice, and messaging platforms
- **Tool Integration**: Extensible tool system for external API calls
- **Automation**: Scheduled tasks and event-driven triggers
- **Theming**: Customizable UI with CSS and HTML templates

## How It Works

GeneralBots processes user messages through a combination of:
1. **Dialog Scripts** (.gbdialog files) that define conversation flow
2. **Knowledge Base** (.gbkb collections) that provide contextual information  
3. **Tools** that extend bot capabilities with external functionality
4. **LLM Integration** for intelligent response generation

The platform manages sessions, maintains conversation history, and provides a consistent experience across different communication channels.
