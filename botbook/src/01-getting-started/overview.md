# Overview

botserver is an open-source conversational AI platform built in Rust that enables developers to create, deploy, and manage intelligent bots with minimal configuration. This chapter provides a comprehensive introduction to the platform's architecture, capabilities, and design philosophy.


## Core Philosophy

botserver was designed around five guiding principles that shape every aspect of the platform. Zero Configuration means the system works out of the box with sensible defaults, eliminating lengthy setup processes. The Package-Based approach ensures bots are self-contained in `.gbai` folders that can be copied and deployed anywhere. BASIC Scripting provides simple, accessible programming for conversation flows that non-programmers can understand and modify. Multi-Channel support means you deploy once and run everywhere across Web, WhatsApp, Teams, and other platforms. Knowledge-First design provides built-in document management and semantic search as core capabilities rather than afterthoughts.


## Architecture Overview

botserver uses a modular architecture organized into three distinct layers that work together to provide a complete conversational AI platform.

### Storage Layer

The storage layer handles all data persistence needs. The SQL database stores structured data including users, sessions, and configurations using PostgreSQL with the Diesel ORM. Object storage provides S3-compatible file storage for documents and uploads, typically using MinIO for self-hosted deployments. The high-performance cache layer handles sessions and frequently accessed data using a Redis-compatible store. An optional vector database enables semantic search capabilities for knowledge bases using Qdrant or similar vector stores.

### Application Layer

The application layer contains the core bot functionality. The Bot Engine processes conversations and manages state across interactions. The BASIC Interpreter executes conversation scripts written in the General Bots dialect of BASIC. The Package Manager handles bot deployment, lifecycle management, and hot-reloading of changes. Channel Adapters connect to various messaging platforms, translating between platform-specific formats and the internal message representation.

### Service Layer

The service layer provides the infrastructure that supports bot operations. The UI Server handles HTTP API requests and WebSocket connections for real-time chat interfaces. The Scheduler executes cron-based tasks for automation and maintenance. LLM Integration connects to language models whether hosted locally or in the cloud. Authentication integrates with directory services for user management and access control.


## Key Features

### Conversation Management

botserver provides comprehensive conversation management capabilities. Sessions persist across interactions, maintaining context and state throughout multi-turn dialogs. The context management system tracks conversation history and user information across interactions. Parallel conversation handling allows a single bot instance to manage thousands of simultaneous conversations efficiently.

### Knowledge Base System

The knowledge base system turns your documents into searchable, AI-accessible information. Document ingestion supports PDF, TXT, MD, and DOCX formats with automatic text extraction. The indexing pipeline processes documents into searchable chunks stored in the vector database. Semantic search finds relevant information based on meaning rather than just keyword matching. Context injection automatically provides relevant document excerpts to the LLM when generating responses.

### BASIC Scripting Language

The BASIC scripting language makes bot development accessible to everyone. The simple syntax allows non-programmers to read and modify conversation flows. Built-in keywords handle common tasks like sending messages, saving data, and calling APIs. The tool integration system lets you create callable functions that the AI can invoke automatically. Event-driven programming support enables reactive bots that respond to schedules, webhooks, and system events.

### Multi-Channel Support

Deploy your bot once and reach users across multiple channels. The web chat interface provides an embeddable widget for websites. WhatsApp Business API integration enables customer service on the world's most popular messaging platform. Microsoft Teams support brings your bot into enterprise collaboration spaces. Email integration allows conversational interactions through traditional email. SMS support via providers enables text message interactions for users without data connectivity.

### Enterprise Features

botserver includes capabilities required for enterprise deployments. Multi-tenancy support allows a single installation to serve multiple organizations with complete isolation. Role-based access control restricts actions based on user roles and permissions. Comprehensive audit logging tracks all actions for compliance and debugging. Horizontal scaling distributes load across multiple instances. High availability configurations ensure continuous operation even during failures.


## System Requirements

### Minimum Requirements

For development and testing purposes, botserver runs comfortably on modest hardware. You need at least 4GB of RAM to run all components. A single CPU core is sufficient for light workloads. Reserve at least 10GB of disk space for the application, databases, and documents. The platform runs on Linux, macOS, or Windows operating systems.

### Recommended for Production

Production deployments benefit from more substantial resources. Plan for 16GB of RAM to handle concurrent users and large knowledge bases. Two or more CPU cores improve response times under load. Use 100GB of SSD storage for better database and file access performance. Linux servers running Ubuntu or Debian provide the most tested and reliable environment. For local LLM hosting, an NVIDIA RTX 3060 or better GPU with at least 12GB of VRAM enables on-premises inference without cloud API dependencies.


## Configuration

Bot configuration uses `config.csv` files with key-value parameters. Server settings like `server_host` and `server_port` control where the UI server listens. LLM configuration through `llm-url` and `llm-model` specifies which language model to use. Email settings including `email-from` and `email-server` enable outbound email functionality. UI customization parameters like `theme-color1`, `theme-color2`, `theme-title`, and `theme-logo` brand the interface. Conversation settings such as `episodic-memory-history` and `episodic-memory-threshold` tune how context is managed. Refer to the config.csv files in bot packages for the complete list of available parameters.


## Bot Package Structure

Each bot is a self-contained `.gbai` folder that includes everything needed for deployment. The structure organizes different aspects of the bot into subfolders with specific naming conventions.

```
mybot.gbai/
  mybot.gbot/
    config.csv
  mybot.gbdialog/
    start.bas
    tools/
  mybot.gbkb/
    documents/
  mybot.gbtheme/
    styles/
```

The `.gbot` subfolder contains configuration files including the main `config.csv`. The `.gbdialog` subfolder holds BASIC scripts with `start.bas` serving as the entry point and additional scripts providing tools. The `.gbkb` subfolder stores knowledge base documents organized into topical folders. The optional `.gbtheme` subfolder contains CSS and assets for UI customization.


## Deployment Models

### Standalone Server

The standalone deployment model runs a single botserver instance serving multiple bots. This approach provides the simplest setup with shared resources across bots. Standalone deployment works best for small to medium deployments where isolation between bots is not critical.

### LXC Containers

Linux containers provide lightweight virtualization for bot isolation. Each bot or group of bots runs in its own container with dedicated resources. LXC deployment offers easy management through standard container tooling while maintaining lower overhead than full virtual machines.

### Embedded Mode

Embedded deployment integrates botserver into existing applications as a library. This mode provides programmatic control over bot behavior and direct integration with application logic. Custom integrations can use the embedded mode to add conversational capabilities to any Rust application.


## Getting Started

Installation begins by downloading and running the botserver binary. The bootstrap process automatically downloads all required components to the `botserver-stack/` directory, including database binaries, the object storage server, cache server, LLM runtime, and other dependencies.

Bot deployment uses object storage buckets. Each bot receives its own bucket for file storage. Bots are deployed to the drive rather than the work folder, which is reserved for internal operations as documented in the gbapp chapter.

After startup, access the UI interface at `http://localhost:9000` to interact with your bots and monitor their operation.


## Use Cases

### Customer Support

Customer support bots automate FAQ responses and ticket handling. Load your support documentation, policies, and procedures into knowledge bases. Create tools for ticket creation and status lookup. The result is 24/7 support that handles common questions automatically while escalating complex issues to human agents.

### Internal Tools

Employee assistant bots streamline internal operations. Knowledge bases contain HR policies, IT guides, and company information. Tools enable leave requests, equipment orders, and other common workflows. Employees get instant answers and automated processing for routine requests.

### Educational Applications

Educational bots provide interactive learning experiences. Course materials and reference documents become searchable knowledge bases. Tools handle quiz administration, progress tracking, and enrollment. Students receive personalized guidance and immediate feedback.

### Healthcare Applications

Healthcare bots assist with patient engagement while maintaining compliance. Appointment scheduling, medication reminders, and symptom checking tools automate routine interactions. Knowledge bases contain patient education materials. All interactions maintain audit trails for regulatory compliance.


## Security Features

botserver implements comprehensive security at every layer. Authentication integrates with directory services for centralized user management. SSL/TLS encryption protects all network communications. Session tokens use cryptographically secure generation and validation. Input sanitization prevents injection attacks across all user inputs. SQL injection prevention uses parameterized queries throughout. XSS protection sanitizes output displayed to users. Rate limiting prevents abuse and denial of service attacks. Audit logging records all significant actions for compliance and forensics.


## Monitoring and Operations

### Health Checks

Health monitoring endpoints report component status for operational awareness. Database connectivity checks verify the storage layer is operational. Storage availability checks ensure object storage is accessible. Cache performance metrics track response times and hit rates.

### Metrics

Operational metrics provide visibility into bot performance. Conversation counts show usage patterns over time. Response time measurements identify performance issues. Error rates highlight problems requiring attention. Resource usage tracking helps capacity planning.

### Logging

Structured logging facilitates troubleshooting and analysis. Configurable log levels from ERROR through DEBUG control verbosity. Automatic rotation and archival prevent disk exhaustion. Search and filtering tools help locate specific events in large log files.


## Extensibility

### Channel Adapters

New messaging channels integrate through the adapter system. WebSocket protocols enable real-time bidirectional communication. REST API integration supports request-response style platforms. Custom protocols can be implemented for specialized messaging systems.

### Storage Backends

Storage is abstracted to support multiple backend options. S3-compatible storage works with AWS, MinIO, and other providers. Database adapters could support different SQL databases. Cache providers can be swapped while maintaining the same interface.


## Community and Support

### Documentation

Comprehensive documentation covers all aspects of the platform. The User Guide walks through common tasks and best practices. The API Reference documents all endpoints and parameters. The BASIC Language Reference details every keyword and syntax rule. The Deployment Guide covers production installation and configuration.

### Resources

Example bots in the `templates/` directory demonstrate common patterns. Test suites verify functionality and provide usage examples. Migration tools help transition from other platforms to General Bots.

### Contributing

General Bots is open source under the AGPL (GNU Affero General Public License). The GitHub repository hosts all development activity. Issue tracking manages bug reports and feature requests. Pull requests from the community are welcome and encouraged.


## Codebase Statistics

The General Bots workspace contains the following lines of code by language:

| Language   | Lines of Code |
|------------|---------------|
| HTML       | 318,676       |
| Rust       | 232,015       |
| Markdown   | 135,130       |
| SVG        | 47,196        |
| JSON       | 45,743        |
| CSS        | 40,476        |
| SQL        | 26,242        |
| JavaScript | 16,257        |
| TOML       | 5,640         |
| YAML       | 4,762         |
| Shell      | 3,602         |
| TypeScript | 13            |
| **Total**  | **875,752**   |


## Summary

botserver provides a complete platform for building conversational AI applications. The combination of simple BASIC scripting, automatic setup, and enterprise features bridges the gap between simple chatbots and complex AI systems. The focus on packages, minimal configuration, and multi-channel support makes botserver suitable for both rapid prototyping and production deployments serving millions of users.