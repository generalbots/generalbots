# Bot Templates

BotServer comes with pre-built templates for common use cases. Each template is a complete `.gbai` package with dialogs, configurations, and knowledge bases ready to use.

## Available Templates

### Core Templates

#### default.gbai
The foundation template that all bots inherit from. Contains:
- Basic conversation handling
- Session management
- Error handling
- Standard responses
- Core dialog flows

#### template.gbai
A minimal starting point for custom bots with:
- Skeleton structure
- Basic configuration
- Example dialogs
- Placeholder knowledge base

### Business Templates

#### crm.gbai
Customer Relationship Management bot featuring:
- Contact management
- Lead tracking
- Customer inquiries
- Follow-up scheduling
- Sales pipeline integration
- Customer data lookup

#### erp.gbai
Enterprise Resource Planning assistant with:
- Inventory queries
- Order processing
- Supply chain info
- Resource allocation
- Business metrics
- Report generation

#### bi.gbai
Business Intelligence bot providing:
- Data analysis
- Report generation
- Dashboard queries
- KPI tracking
- Trend analysis
- Executive summaries

#### store.gbai
E-commerce assistant offering:
- Product catalog
- Order status
- Shopping cart help
- Payment processing
- Shipping information
- Return handling

### Communication Templates

#### announcements.gbai
Broadcast messaging system for:
- Company announcements
- News distribution
- Event notifications
- Alert broadcasting
- Multi-channel delivery
- Scheduled messages

#### broadcast.gbai
Mass communication bot with:
- Bulk messaging
- Audience segmentation
- Campaign management
- Delivery tracking
- Response collection
- Analytics reporting

#### whatsapp.gbai
WhatsApp-optimized bot featuring:
- WhatsApp Business API integration
- Media handling
- Quick replies
- List messages
- Location sharing
- Contact cards

#### reminder.gbai
Automated reminder system for:
- Task reminders
- Appointment notifications
- Deadline alerts
- Recurring reminders
- Calendar integration
- Follow-up scheduling

### AI & Automation Templates

#### ai-search.gbai
Advanced search assistant with:
- Semantic search
- Multi-source queries
- Result ranking
- Context understanding
- Query refinement
- Search analytics

#### llm-server.gbai
LLM gateway bot providing:
- Model selection
- Prompt management
- Token optimization
- Response caching
- Rate limiting
- Cost tracking

#### llm-tools.gbai
AI tools collection featuring:
- Text generation
- Summarization
- Translation
- Code generation
- Image description
- Sentiment analysis

#### crawler.gbai
Web scraping bot with:
- Site crawling
- Data extraction
- Content indexing
- Change monitoring
- Structured data parsing
- API integration

#### talk-to-data.gbai
Data conversation interface offering:
- Natural language queries
- Database access
- Data visualization
- Export capabilities
- Statistical analysis
- Report generation

### Industry Templates

#### edu.gbai
Education assistant providing:
- Course information
- Student support
- Assignment help
- Schedule queries
- Resource access
- Grade lookup

#### law.gbai
Legal information bot with:
- Legal term definitions
- Document templates
- Case lookup
- Regulation queries
- Compliance checking
- Disclaimer management

#### marketing.gbai
Marketing automation bot featuring:
- Lead generation
- Campaign management
- Content distribution
- Social media integration
- Analytics tracking
- A/B testing

### Integration Templates

#### api-client.gbai
REST API integration bot with:
- API endpoint management
- Authentication handling
- Request formatting
- Response parsing
- Error handling
- Rate limiting

#### public-apis.gbai
Public API aggregator providing:
- Weather information
- News feeds
- Stock prices
- Currency conversion
- Maps/directions
- Public data access

#### backup.gbai
Backup management bot offering:
- Scheduled backups
- Data archiving
- Restore operations
- Backup verification
- Storage management
- Disaster recovery

## Using Templates

### Quick Start

1. **Copy template to your workspace**:
   ```bash
   cp -r templates/crm.gbai mybot.gbai
   ```

2. **Customize configuration**:
   ```bash
   cd mybot.gbai/mybot.gbot
   vim config.csv
   ```

3. **Modify dialogs**:
   ```bash
   cd ../mybot.gbdialog
   vim start.bas
   ```

4. **Add knowledge base**:
   ```bash
   cd ../mybot.gbkb
   # Add your documents
   ```

### Template Structure

Every template follows this structure:

```
template-name.gbai/
├── template-name.gbdialog/
│   ├── start.bas           # Entry point
│   ├── menu.bas            # Menu system
│   └── tools/              # Tool definitions
├── template-name.gbot/
│   └── config.csv          # Configuration
├── template-name.gbkb/
│   ├── docs/               # Documentation
│   └── data/               # Reference data
└── template-name.gbtheme/
    └── style.css           # Optional theming
```

## Customization Guide

### Extending Templates

Templates are designed to be extended:

1. **Inherit from template**:
   ```basic
   INCLUDE "template://default/common.bas"
   ```

2. **Override functions**:
   ```basic
   FUNCTION handle_greeting()
       ' Custom greeting logic
       TALK "Welcome to MyBot!"
   END FUNCTION
   ```

3. **Add new features**:
   ```basic
   ' Add to existing template
   FUNCTION new_feature()
       ' Your custom code
   END FUNCTION
   ```

### Combining Templates

Mix features from multiple templates:

```basic
' Use CRM contact management
INCLUDE "template://crm/contacts.bas"

' Add marketing automation
INCLUDE "template://marketing/campaigns.bas"

' Integrate with APIs
INCLUDE "template://api-client/rest.bas"
```

## Best Practices

### Template Selection

1. **Start with the right template**: Choose based on primary use case
2. **Combine when needed**: Mix templates for complex requirements
3. **Keep core intact**: Don't modify template originals
4. **Document changes**: Track customizations

### Customization Tips

1. **Configuration first**: Adjust config.csv before code
2. **Test incrementally**: Verify each change
3. **Preserve structure**: Maintain template organization
4. **Version control**: Track template modifications

### Performance Considerations

1. **Remove unused features**: Delete unnecessary dialogs
2. **Optimize knowledge base**: Index only needed content
3. **Configure appropriately**: Adjust settings for scale
4. **Monitor resource usage**: Track memory and CPU

## Template Development

### Creating Custom Templates

1. **Start from template.gbai**: Use as foundation
2. **Define clear purpose**: Document template goals
3. **Include examples**: Provide sample data
4. **Write documentation**: Explain usage
5. **Test thoroughly**: Verify all features

### Template Guidelines

- Keep templates focused on specific use cases
- Include comprehensive examples
- Provide clear documentation
- Use consistent naming conventions
- Include error handling
- Make configuration obvious
- Test across channels

## Contributing Templates

To contribute a new template:

1. Create template in `templates/` directory
2. Include README with description
3. Add example configuration
4. Provide sample knowledge base
5. Include test cases
6. Submit pull request

## Template Updates

Templates are versioned and updated regularly:
- Bug fixes
- Security patches
- Feature additions
- Performance improvements
- Documentation updates

Check for updates:
```bash
git pull
diff templates/template-name.gbai
```

## Support

For template-specific help:
- Check template README
- Review example code
- Consult documentation
- Ask in community forums
- Report issues on GitHub