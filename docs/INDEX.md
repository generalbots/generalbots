# General Bots Documentation Index

This directory contains comprehensive documentation for the General Bots platform, organized as chapters for easy navigation.

## 📚 Core Documentation

### Chapter 0: Introduction & Getting Started
**[00-README.md](00-README.md)** - Main project overview, quick start guide, and system architecture
- Overview of General Bots platform
- Installation and prerequisites
- Quick start guide
- Core features and capabilities
- KB and TOOL system essentials
- Video tutorials and resources

### Chapter 1: Build & Development Status
**[01-BUILD_STATUS.md](01-BUILD_STATUS.md)** - Current build status, fixes, and development roadmap
- Build status and metrics
- Completed tasks
- Remaining issues and fixes
- Build commands for different configurations
- Feature matrix
- Testing strategy

### Chapter 2: Code of Conduct
**[02-CODE_OF_CONDUCT.md](02-CODE_OF_CONDUCT.md)** - Community guidelines and standards (English)
- Community pledge and standards
- Responsibilities and scope
- Enforcement policies
- Reporting guidelines

### Chapter 3: Código de Conduta (Portuguese)
**[03-CODE_OF_CONDUCT-pt-br.md](03-CODE_OF_CONDUCT-pt-br.md)** - Diretrizes da comunidade (Português)
- Compromisso da comunidade
- Padrões de comportamento
- Responsabilidades
- Aplicação das normas

### Chapter 4: Contributing Guidelines
**[04-CONTRIBUTING.md](04-CONTRIBUTING.md)** - How to contribute to the project
- Logging issues
- Contributing bug fixes
- Contributing features
- Code requirements
- Legal considerations
- Running the entire system

### Chapter 5: Integration Status
**[05-INTEGRATION_STATUS.md](05-INTEGRATION_STATUS.md)** - Complete module integration tracking
- Module activation status
- API surface exposure
- Phase-by-phase integration plan
- Progress metrics (50% complete)
- Priority checklist

### Chapter 6: Security Policy
**[06-SECURITY.md](06-SECURITY.md)** - Security policy and best practices
- IT security evaluation
- Data protection obligations
- Information classification
- Employee security training
- Vulnerability reporting

### Chapter 7: Production Status
**[07-STATUS.md](07-STATUS.md)** - Current production readiness and deployment guide
- Build metrics and achievements
- Active API endpoints
- Configuration requirements
- Architecture overview
- Deployment instructions
- Production checklist

## 🔧 Technical Documentation

### Knowledge Base & Tools
**[KB_AND_TOOLS.md](KB_AND_TOOLS.md)** - Deep dive into the KB and TOOL system
- Core system overview (4 essential keywords)
- USE_KB and CLEAR_KB commands
- USE_TOOL and CLEAR_TOOLS commands
- .gbkb folder structure
- Tool development with BASIC
- Session management
- Advanced patterns and examples

### Quick Start Guide
**[QUICK_START.md](QUICK_START.md)** - Fast-track setup and first bot
- Prerequisites installation
- First bot creation
- Basic conversation flows
- Common patterns
- Troubleshooting

### Security Features
**[SECURITY_FEATURES.md](SECURITY_FEATURES.md)** - Detailed security implementation
- Authentication mechanisms
- OAuth2/OIDC integration
- Data encryption
- Security best practices
- Zitadel integration
- Session security

### Semantic Cache System
**[SEMANTIC_CACHE.md](SEMANTIC_CACHE.md)** - LLM response caching with semantic similarity
- Architecture and benefits
- Implementation details
- Redis integration
- Performance optimization
- Cache invalidation strategies
- 70% cost reduction metrics

### SMB Deployment Guide
**[SMB_DEPLOYMENT_GUIDE.md](SMB_DEPLOYMENT_GUIDE.md)** - Pragmatic deployment for small/medium businesses
- Simple vs Enterprise deployment
- Step-by-step setup
- Configuration examples
- Common SMB use cases
- Troubleshooting for SMB environments

### Universal Messaging System
**[BASIC_UNIVERSAL_MESSAGING.md](BASIC_UNIVERSAL_MESSAGING.md)** - Multi-channel communication
- Channel abstraction layer
- Email integration
- WhatsApp Business API
- Microsoft Teams integration
- Instagram Direct messaging
- Message routing and handling

## 🧹 Maintenance & Cleanup Documentation

### Cleanup Complete
**[CLEANUP_COMPLETE.md](CLEANUP_COMPLETE.md)** - Completed cleanup tasks and achievements
- Refactoring completed
- Code organization improvements
- Documentation consolidation
- Technical debt removed

### Cleanup Warnings
**[CLEANUP_WARNINGS.md](CLEANUP_WARNINGS.md)** - Warning analysis and resolution plan
- Warning categorization
- Resolution strategies
- Priority levels
- Technical decisions

### Fix Warnings Now
**[FIX_WARNINGS_NOW.md](FIX_WARNINGS_NOW.md)** - Immediate action items for warnings
- Critical warnings to fix
- Step-by-step fixes
- Code examples
- Testing verification

### Warnings Summary
**[WARNINGS_SUMMARY.md](WARNINGS_SUMMARY.md)** - Comprehensive warning overview
- Total warning count
- Warning distribution by module
- Intentional vs fixable warnings
- Long-term strategy

## 📖 Detailed Documentation (src subdirectory)

### Book-Style Documentation
Located in `src/` subdirectory - comprehensive book-format documentation:

- **[src/README.md](src/README.md)** - Book introduction
- **[src/SUMMARY.md](src/SUMMARY.md)** - Table of contents

#### Part I: Getting Started
- **Chapter 1:** First Steps
  - Installation
  - First Conversation
  - Sessions

#### Part II: Package System
- **Chapter 2:** Core Packages
  - gbai - AI Package
  - gbdialog - Dialog Package
  - gbdrive - Drive Integration
  - gbkb - Knowledge Base
  - gbot - Bot Package
  - gbtheme - Theme Package

#### Part III: Knowledge Management
- **Chapter 3:** Vector Database & Search
  - Semantic Search
  - Qdrant Integration
  - Caching Strategies
  - Context Compaction
  - Indexing
  - Vector Collections

#### Part IV: User Interface
- **Chapter 4:** Web Interface
  - HTML Structure
  - CSS Styling
  - Web Interface Configuration

#### Part V: BASIC Language
- **Chapter 5:** BASIC Keywords
  - Basics
  - ADD_KB, ADD_TOOL, ADD_WEBSITE
  - CLEAR_TOOLS
  - CREATE_DRAFT, CREATE_SITE
  - EXIT_FOR
  - And 30+ more keywords...

#### Appendices
- **Appendix I:** Database Schema
  - Tables
  - Relationships
  - Schema Documentation

## 📝 Changelog

**CHANGELOG.md** is maintained at the root directory level (not in docs/) and contains:
- Version history
- Release notes
- Breaking changes
- Migration guides

## 🗂️ Documentation Organization Principles

1. **Numbered Chapters (00-07)** - Core project documentation in reading order
2. **Named Documents** - Technical deep-dives, organized alphabetically
3. **src/ Subdirectory** - Book-style comprehensive documentation
4. **Root CHANGELOG.md** - Version history at project root (the truth is in src)

## 🔍 Quick Navigation

### For New Users:
1. Start with **00-README.md** for overview
2. Follow **QUICK_START.md** for setup
3. Read **KB_AND_TOOLS.md** to understand core concepts
4. Check **07-STATUS.md** for current capabilities

### For Contributors:
1. Read **04-CONTRIBUTING.md** for guidelines
2. Check **01-BUILD_STATUS.md** for development status
3. Review **05-INTEGRATION_STATUS.md** for module status
4. Follow **02-CODE_OF_CONDUCT.md** for community standards

### For Deployers:
1. Review **07-STATUS.md** for production readiness
2. Read **SMB_DEPLOYMENT_GUIDE.md** for deployment steps
3. Check **06-SECURITY.md** for security requirements
4. Review **SECURITY_FEATURES.md** for implementation details

### For Developers:
1. Check **01-BUILD_STATUS.md** for build instructions
2. Review **05-INTEGRATION_STATUS.md** for API status
3. Read **KB_AND_TOOLS.md** for system architecture
4. Browse **src/** directory for detailed technical docs

## 📞 Support & Resources

- **GitHub Repository:** https://github.com/GeneralBots/BotServer
- **Documentation Site:** https://docs.pragmatismo.com.br
- **Stack Overflow:** Tag questions with `generalbots`
- **Security Issues:** security@pragmatismo.com.br

---

**Last Updated:** 2024-11-22
**Documentation Version:** 6.0.8
**Status:** Production Ready ✅