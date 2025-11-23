# Bot Templates

BotServer includes 21 pre-built bot templates for various use cases. Each template is a complete `.gbai` package ready to deploy.

## Template Overview

| Template | Purpose | Key Features | Use Case |
|----------|---------|--------------|----------|
| **default.gbai** | Minimal starter bot | Basic config only | Simple bots, learning |
| **template.gbai** | Reference implementation | Complete structure example | Creating new templates |
| **announcements.gbai** | Company announcements | Multiple KB collections, auth flows | Internal communications |
| **ai-search.gbai** | AI-powered search | QR generation, PDF samples | Document retrieval |
| **api-client.gbai** | External API integration | Climate API, REST patterns | Third-party services |
| **backup.gbai** | Backup automation | Server backup scripts, scheduling | System administration |
| **bi.gbai** | Business Intelligence | Admin/user roles, data viz | Executive dashboards |
| **broadcast.gbai** | Mass messaging | Recipient management, scheduling | Marketing campaigns |
| **crawler.gbai** | Web indexing | Site crawling, content extraction | Search engines |
| **crm.gbai** | Customer Relations | Sentiment analysis, tracking | Sales & support |
| **edu.gbai** | Education platform | Course management, enrollment | Online learning |
| **erp.gbai** | Enterprise Planning | Process automation, integrations | Resource management |
| **law.gbai** | Legal assistant | Document templates, regulations | Legal departments |
| **llm-server.gbai** | LLM hosting | Model serving, GPU config | AI infrastructure |
| **llm-tools.gbai** | LLM utilities | Prompt engineering, testing | AI development |
| **marketing.gbai** | Marketing automation | Campaign tools, lead generation | Marketing teams |
| **public-apis.gbai** | Public API access | Weather, news, data sources | Information services |
| **reminder.gbai** | Task reminders | Scheduling, notifications | Personal assistants |
| **store.gbai** | E-commerce | Product catalog, orders | Online stores |
| **talk-to-data.gbai** | Natural language queries | SQL generation, data viz | Data exploration |
| **whatsapp.gbai** | WhatsApp Business | Meta API, media handling | Mobile messaging |

## Template Structure

All templates follow this standard directory layout:

```
template-name.gbai/
├── template-name.gbdialog/    # BASIC dialog scripts
│   ├── start.bas              # Entry point (required)
│   └── *.bas                  # Tool scripts (auto-discovered)
├── template-name.gbkb/        # Knowledge base collections
│   ├── collection1/           # Documents for USE KB "collection1"
│   └── collection2/           # Documents for USE KB "collection2"
├── template-name.gbdrive/     # File storage (not KB)
│   ├── uploads/               # User uploaded files
│   └── exports/               # Generated files
├── template-name.gbot/        # Configuration
│   └── config.csv             # Bot parameters
└── template-name.gbtheme/     # UI theme (optional)
    └── default.css            # Theme CSS
```

## Quick Start Guide

### 1. Choose a Template

Select based on your needs:
- **Simple chat**: Use `default.gbai`
- **Business app**: Choose `crm.gbai`, `bi.gbai`, or `erp.gbai`
- **AI features**: Pick `ai-search.gbai` or `llm-tools.gbai`
- **Communication**: Select `broadcast.gbai` or `whatsapp.gbai`

### 2. Deploy the Template

```bash
# Templates are auto-deployed during bootstrap
# Access at: http://localhost:8080/template-name
```

### 3. Customize Configuration

Edit `template-name.gbot/config.csv`:

```csv
name,value
bot-name,My Custom Bot
welcome-message,Hello! How can I help?
llm-model,gpt-4
temperature,0.7
```

### 4. Add Knowledge Base

Place documents in `.gbkb` folders:
- Each folder becomes a collection
- Use `USE KB "folder-name"` in scripts
- Documents are automatically indexed

### 5. Create Tools (Optional)

Add `.bas` files to `.gbdialog`:
- Each file becomes a tool
- Auto-discovered by the system
- Called automatically by LLM when needed

## Template Details

### Core Templates

#### default.gbai
- **Files**: Minimal configuration only
- **Best for**: Learning, simple bots
- **Customization**: Start from scratch

#### template.gbai
- **Files**: Complete example structure
- **Best for**: Reference implementation
- **Customization**: Copy and modify

### Business Applications

#### announcements.gbai
- **Files**: `auth.bas`, `start.bas`, multiple KB collections
- **Collections**: auxiliom, news, toolbix
- **Features**: Authentication, summaries

#### bi.gbai
- **Files**: `bi-admin.bas`, `bi-user.bas`
- **Features**: Role separation, dashboards
- **Data**: Report generation

#### crm.gbai
- **Files**: `analyze-customer-sentiment.bas`, `check.bas`
- **Features**: Sentiment analysis
- **Data**: Customer tracking

#### store.gbai
- **Features**: Product catalog, order processing
- **Integration**: E-commerce workflows

### AI & Search

#### ai-search.gbai
- **Files**: `qr.bas`, PDF samples
- **Features**: QR codes, document search
- **Data**: Sample PDFs included

#### talk-to-data.gbai
- **Features**: Natural language to SQL
- **Integration**: Database connections
- **Output**: Data visualization

### Communication

#### broadcast.gbai
- **Files**: `broadcast.bas`
- **Features**: Mass messaging
- **Scheduling**: Message campaigns

#### whatsapp.gbai
- **Config**: Meta Challenge parameter
- **Features**: WhatsApp API integration
- **Media**: Image/video support

### Development Tools

#### api-client.gbai
- **Files**: `climate.vbs`, `msft-partner-center.bas`
- **Examples**: REST API patterns
- **Integration**: External services

#### llm-server.gbai
- **Config**: Model serving parameters
- **Features**: GPU configuration
- **Purpose**: Local LLM hosting

## Best Practices

### Template Selection

1. **Start small**: Begin with `default.gbai`
2. **Match use case**: Choose aligned templates
3. **Combine features**: Mix templates as needed
4. **Keep originals**: Copy before modifying

### Customization Strategy

#### Minimal BASIC Approach
Instead of complex dialog flows, use simple LLM calls:

```basic
' Traditional: 100+ lines of intent matching
' BotServer: Let LLM handle it
response = LLM prompt
TALK response
```

#### Tool Creation
Only create `.bas` files for specific actions:
- API calls
- Database operations
- File processing
- Calculations

#### Knowledge Base Organization
- One folder per topic/collection
- Name folders clearly
- Keep documents updated
- Index automatically

### Performance Tips

- Remove unused template files
- Index only necessary documents
- Configure appropriate cache settings
- Monitor resource usage

## Creating Custom Templates

To create your own template:

1. **Copy `template.gbai`** as starting point
2. **Define clear purpose** - one template, one job
3. **Structure folders** properly:
   - `.gbdialog` for scripts
   - `.gbkb` for knowledge collections
   - `.gbdrive` for general files
   - `.gbot` for configuration
4. **Include examples** - sample data and dialogs
5. **Test thoroughly** - verify all features

## Migration Philosophy

When migrating from traditional platforms:

### Remove Complexity
- ❌ Intent detection → ✅ LLM understands naturally
- ❌ State machines → ✅ LLM maintains context
- ❌ Routing logic → ✅ LLM handles flow
- ❌ Entity extraction → ✅ LLM identifies information

### Embrace Simplicity
- Let LLM handle conversations
- Create tools only for actions
- Use knowledge bases for context
- Trust the system's capabilities

## Template Maintenance

- Templates updated with BotServer releases
- Check repository for latest versions
- Review changes before upgrading
- Test in development first

## Support Resources

- README files in each template folder
- Example configurations included
- Sample knowledge bases provided
- Community forums for discussions