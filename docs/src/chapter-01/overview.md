# Overview

BotServer is an open-source conversational AI platform built in Rust that enables developers to create, deploy, and manage intelligent bots with minimal configuration.

## Core Philosophy

BotServer follows these principles:

1. **Zero Configuration**: Works out of the box with sensible defaults
2. **Package-Based**: Bots are self-contained packages (.gbai folders)
3. **BASIC Scripting**: Simple, accessible programming for conversation flows
4. **Multi-Channel**: Deploy once, run everywhere (Web, WhatsApp, Teams, etc.)
5. **Knowledge-First**: Built-in document management and semantic search

## Architecture Overview

BotServer uses a modular architecture with these core components:

### Storage Layer
- **Database**: SQL database for structured data (users, sessions, configurations)
- **Object Storage**: S3-compatible storage for files and documents
- **Cache**: High-performance caching for sessions and frequent data
- **Vector Database**: Optional semantic search for knowledge bases

### Application Layer
- **Bot Engine**: Processes conversations and manages state
- **BASIC Interpreter**: Executes conversation scripts
- **Package Manager**: Handles bot deployment and lifecycle
- **Channel Adapters**: Connects to various messaging platforms

### Service Layer
- **UI Server**: HTTP API and WebSocket connections
- **Scheduler**: Cron-based task scheduling
- **LLM Integration**: Connects to language models (local or cloud)
- **Authentication**: Directory service integration for user management

## Key Features

### Conversation Management
- Stateful conversations with session persistence
- Context management across interactions
- Multi-turn dialog support
- Parallel conversation handling

### Knowledge Base System
- Document ingestion (PDF, TXT, MD, DOCX)
- Automatic text extraction and indexing
- Semantic search capabilities
- Context injection for LLM responses

### BASIC Scripting Language
- Simple syntax for non-programmers
- Built-in keywords for common tasks
- Tool integration system
- Event-driven programming support

### Multi-Channel Support
- Web chat interface
- WhatsApp Business API
- Microsoft Teams
- Email
- SMS (via providers)

### Enterprise Features
- Multi-tenancy support
- Role-based access control
- Audit logging
- Horizontal scaling
- High availability

## System Requirements

### Minimum Requirements
- 4GB RAM
- 1 CPU core (development/testing)
- 10GB disk space
- Linux, macOS, or Windows

### Recommended for Production
- 16GB RAM
- 2+ CPU cores
- 100GB SSD storage
- Linux server (Ubuntu/Debian preferred)
- GPU: RTX 3060 or better (12GB VRAM minimum) for local LLM hosting

## Configuration

Bot configuration is managed through `config.csv` files with parameters like:
- `server_host`, `server_port` - UI server settings
- `llm-url`, `llm-model` - LLM configuration
- `email-from`, `email-server` - Email settings
- `theme-color1`, `theme-color2`, `theme-title`, `theme-logo` - UI customization
- `prompt-history`, `prompt-compact` - Conversation settings

See actual config.csv files in bot packages for available parameters.

## Bot Package Structure

Each bot is a self-contained `.gbai` folder:

```
mybot.gbai/
  mybot.gbot/       # Configuration
    config.csv
  mybot.gbdialog/   # Conversation scripts
    start.bas
    tools/
  mybot.gbkb/       # Knowledge base
    documents/
  mybot.gbtheme/    # Optional UI customization
    styles/
```

## Deployment Models

### Standalone Server
Single instance serving multiple bots:
- Simple setup
- Shared resources
- Best for small to medium deployments

### LXC Containers
Using Linux containers for isolation:
- Lightweight virtualization
- Resource isolation
- Easy management

### Embedded
Integrated into existing applications:
- Library mode
- Custom integrations
- Programmatic control

## Getting Started

1. **Install BotServer**
   ```bash
   # Download and run
   ./botserver
   ```

2. **Bootstrap Components**
   The bootstrap automatically downloads everything to `botserver-stack/`:
   - Database binaries
   - Object storage server
   - Cache server
   - LLM runtime
   - Required dependencies

3. **Deploy a Bot**
   Create a new bucket in object storage:
   ```bash
   # Each bot gets its own storage bucket
   # Bots are deployed to the drive, not work folder
   # The work/ folder is internal (see .gbapp chapter)
   ```

4. **Access UI Interface**
   ```
   http://localhost:8080
   ```

## Use Cases

### Customer Support
- FAQ automation
- Ticket creation and routing
- Knowledge base search
- Live agent handoff

### Internal Tools
- Employee onboarding
- IT helpdesk
- HR inquiries
- Process automation

### Educational
- Interactive tutorials
- Quiz and assessment
- Course navigation
- Student support

### Healthcare
- Appointment scheduling
- Symptom checking
- Medication reminders
- Patient education

## Security Features

- Authentication via directory service
- SSL/TLS encryption
- Session token management
- Input sanitization
- SQL injection prevention
- XSS protection
- Rate limiting
- Audit logging

## Monitoring and Operations

### Health Checks
- Component status endpoints
- Database connectivity
- Storage availability
- Cache performance

### Metrics
- Conversation counts
- Response times
- Error rates
- Resource usage

### Logging
- Structured logging
- Log levels (ERROR, WARN, INFO, DEBUG)
- Rotation and archival
- Search and filtering

## Extensibility

### Channel Adapters
Implement new messaging channels:
- WebSocket protocol
- REST API integration
- Custom protocols

### Storage Backends
- S3-compatible storage
- Database adapters
- Cache providers

## Community and Support

### Documentation
- User Guide
- API Reference
- BASIC Language Reference
- Deployment Guide

### Resources
- Example bots in `templates/`
- Test suites
- Migration tools

### Contributing
- Open source (AGPL - GNU Affero General Public License)
- GitHub repository
- Issue tracking
- Pull requests welcome

## Summary

BotServer provides a complete platform for building conversational AI applications. With its simple BASIC scripting, automatic setup, and enterprise features, it bridges the gap between simple chatbots and complex AI systems.

The focus on packages, minimal configuration, and multi-channel support makes it suitable for both rapid prototyping and production deployments.