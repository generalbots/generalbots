# General Bots BASIC - Universal Messaging & Multi-Channel Documentation

## Table of Contents
- [Universal Messaging Keywords](#universal-messaging-keywords)
- [Channel Configuration](#channel-configuration)
- [URA System](#ura-system)
- [Complete BASIC Language Reference](#complete-basic-language-reference)

---

## Universal Messaging Keywords

The universal messaging system allows seamless communication across multiple channels (WhatsApp, Instagram, Teams, Web, Email) using intelligent channel detection and routing.

### TALK TO - Universal Message Sending

Send messages to any recipient across any supported channel.

#### Syntax
```basic
TALK TO recipient, message
```

#### Auto-Detection Examples
```basic
' WhatsApp - Auto-detected by phone number format
TALK TO "+5511999999999", "Hello via WhatsApp"
TALK TO "5511999999999", "Message to WhatsApp"

' Email - Auto-detected by email format
TALK TO "user@example.com", "Hello via Email"

' Teams - Auto-detected by domain
TALK TO "user@teams.ms", "Hello via Teams"
TALK TO "user@microsoft.com", "Teams message"

' Web Session - For logged-in users
TALK TO user.id, "Welcome back!"
```

#### Explicit Channel Specification
```basic
' Format: "channel:recipient"
TALK TO "whatsapp:+5511999999999", "WhatsApp message"
TALK TO "teams:user@company.com", "Teams message"
TALK TO "instagram:username", "Instagram DM"
TALK TO "web:session_id", "Web notification"
TALK TO "email:user@example.com", "Email message"
```

### SEND FILE TO - Universal File Sharing

Send files to any recipient across channels with automatic media handling.

#### Syntax
```basic
SEND FILE TO recipient, file
SEND FILE TO recipient, file, caption
```

#### Examples
```basic
' Send file with auto-detection
SEND FILE TO "+5511999999999", document
SEND FILE TO "user@example.com", report_pdf

' Send with caption
SEND FILE TO "+5511999999999", image, "Product photo"
SEND FILE TO "teams:project-channel", spreadsheet, "Monthly report"

' From file path
file = "reports/monthly.pdf"
SEND FILE TO "email:manager@company.com", file, "Monthly Report Attached"

' From generated content
data = FIND "sales.xlsx"
pdf = data AS PDF
SEND FILE TO "+5511999999999", pdf, "Sales Report"
```

### BROADCAST - Multi-Recipient Messaging

Send messages to multiple recipients simultaneously.

#### Syntax
```basic
BROADCAST message TO recipient_list
```

#### Examples
```basic
' Broadcast to contact list
contacts = FIND "contacts.csv"
BROADCAST "Newsletter: New features available!" TO contacts

' Broadcast with filtering
customers = FIND "customers.xlsx", "status='active'"
BROADCAST "Special offer for active customers" TO customers

' Mixed channel broadcast
recipients = ["+5511999999999", "user@email.com", "teams:channel-id"]
BROADCAST "Important announcement" TO recipients
```

### SEND TO - Explicit Channel Routing

Direct channel specification for advanced routing scenarios.

#### Syntax
```basic
SEND TO "channel:recipient", message
```

#### Examples
```basic
' Force specific channel
SEND TO "whatsapp:+5511999999999", "WhatsApp only message"
SEND TO "email:user@example.com", "Email notification"

' Conditional channel selection
IF urgent THEN
    SEND TO "whatsapp:" + customer.phone, alert_message
ELSE
    SEND TO "email:" + customer.email, notification
END IF
```

---

## Channel Configuration

### Configuration Files Location

All channel configurations are stored in the bot configuration database and can be managed via `config.csv`:

```
.gbot/
├── config.csv       # Main configuration
├── ura.csv         # URA routing rules
└── menu.csv        # Interactive menus
```

### WhatsApp Configuration

Add to `config.csv`:
```csv
whatsapp-access-token,YOUR_FACEBOOK_ACCESS_TOKEN
whatsapp-phone-id,YOUR_PHONE_NUMBER_ID
whatsapp-verify-token,YOUR_WEBHOOK_VERIFY_TOKEN
```

### Instagram Configuration

Add to `config.csv`:
```csv
instagram-access-token,YOUR_INSTAGRAM_ACCESS_TOKEN
instagram-page-id,YOUR_PAGE_ID
instagram-verify-token,YOUR_WEBHOOK_VERIFY_TOKEN
instagram-admin-id,ADMIN_USER_ID
```

### Teams Configuration

Add to `config.csv`:
```csv
teams-app-id,YOUR_TEAMS_APP_ID
teams-app-password,YOUR_TEAMS_APP_PASSWORD
teams-service-url,https://smba.trafficmanager.net/br/
teams-tenant-id,YOUR_TENANT_ID
teams-support-channel,SUPPORT_CHANNEL_ID
```

### Email Configuration

Add to `config.csv`:
```csv
email-smtp-host,smtp.gmail.com
email-smtp-port,587
email-smtp-user,your-email@gmail.com
email-smtp-password,YOUR_APP_PASSWORD
email-from-address,your-email@gmail.com
email-from-name,Your Bot Name
```

---

## URA System

The URA (Unidade de Resposta Audível) system provides intelligent message routing and automatic responses.

### URA Configuration (ura.csv)

Format: `rule_type,condition,action_type,action_value`

#### Examples

```csv
keyword,ajuda;help;suporte,transfer,teams
keyword,vendas;orçamento,transfer,sales
time,08:00-18:00,continue,
time,18:01-07:59,message,Estamos fora do horário de atendimento
channel,whatsapp,menu,main_menu
channel,instagram,message,Bem-vindo ao Instagram! Como posso ajudar?
keyword,urgente;emergência,transfer,priority_support
```

### Menu Configuration (menu.csv)

Format: `menu_id,option_key,option_label,action_type,action_value`

#### Examples

```csv
main_menu,1,Suporte Técnico,transfer,technical
main_menu,2,Vendas,transfer,sales
main_menu,3,Financeiro,transfer,finance
main_menu,4,Falar com Atendente,transfer,human
main_menu,0,Encerrar,message,Obrigado por entrar em contato!
```

### Central Attendance Flow

```basic
' Example attendance flow implementation
SET HEAR ON whatsapp

main:
HEAR user_message

' Check URA rules
IF user_message CONTAINS "urgente" THEN
    TALK TO "teams:emergency-support", "Urgent: " + user_message
    TALK "You've been transferred to priority support"
    GOTO main
END IF

' Business hours check
IF TIME() < "08:00" OR TIME() > "18:00" THEN
    TALK "We're currently closed. Business hours: 8AM-6PM"
    GOTO main
END IF

' Show menu
TALK "Choose an option:"
TALK "1 - Technical Support"
TALK "2 - Sales"
TALK "3 - Human Agent"

HEAR option

SELECT CASE option
    CASE "1"
        TALK TO "teams:tech-support", "New ticket from " + user.phone
    CASE "2"
        TALK TO "teams:sales", "Sales inquiry from " + user.phone
    CASE "3"
        TALK TO "teams:human-agents", "Transfer request from " + user.phone
        TALK "You're being transferred to a human agent..."
END SELECT

GOTO main
```

---

## Complete BASIC Language Reference

### User Interaction Commands

| Command | Description | Example |
|---------|-------------|---------|
| `HEAR variable` | Wait for user input | `HEAR name` |
| `TALK message` | Send message to current user | `TALK "Hello " + name` |
| `TALK TO recipient, message` | Send to specific recipient | `TALK TO "+5511999999999", "Hello"` |
| `WAIT seconds` | Pause execution | `WAIT 5` |

### Input Validation

| Command | Description | Example |
|---------|-------------|---------|
| `HEAR var AS EMAIL` | Validate email input | `HEAR email AS EMAIL` |
| `HEAR var AS DATE` | Validate date input | `HEAR birthdate AS DATE` |
| `HEAR var AS NAME` | Validate name input | `HEAR fullname AS NAME` |
| `HEAR var AS INTEGER` | Validate integer | `HEAR age AS INTEGER` |
| `HEAR var AS BOOLEAN` | Validate true/false | `HEAR agree AS BOOLEAN` |
| `HEAR var AS HOUR` | Validate time | `HEAR appointment AS HOUR` |
| `HEAR var AS MONEY` | Validate currency | `HEAR amount AS MONEY` |
| `HEAR var AS MOBILE` | Validate phone | `HEAR phone AS MOBILE` |
| `HEAR var AS ZIPCODE` | Validate ZIP | `HEAR zip AS ZIPCODE` |
| `HEAR var AS "opt1", "opt2"` | Menu selection | `HEAR choice AS "Yes", "No", "Maybe"` |
| `HEAR var AS LANGUAGE` | Language code | `HEAR lang AS LANGUAGE` |
| `HEAR var AS QRCODE` | QR code scan | `HEAR code AS QRCODE` |
| `HEAR var AS FILE` | File upload | `HEAR document AS FILE` |
| `HEAR var AS AUDIO` | Audio upload | `HEAR recording AS AUDIO` |

### Data Operations

| Command | Description | Example |
|---------|-------------|---------|
| `FIND file/table` | Query data | `FIND "customers.xlsx"` |
| `FIND file/table, filter` | Query with filter | `FIND "users", "age>18"` |
| `SAVE table, data` | Save to database | `SAVE "orders", order_data` |
| `GET url` | HTTP GET request | `data = GET "https://api.example.com"` |
| `POST url, data` | HTTP POST request | `POST "https://api.example.com", data` |
| `SELECT ... FROM ...` | SQL operations | `SELECT name, SUM(sales) FROM data GROUP BY name` |

### File Operations

| Command | Description | Example |
|---------|-------------|---------|
| `SEND FILE TO recipient, file` | Send file | `SEND FILE TO "+5511999999999", report` |
| `SAVE file AS path` | Save to disk | `SAVE document AS "reports/monthly.pdf"` |
| `UPLOAD file` | Upload to cloud | `UPLOAD "report.pdf"` |
| `DOWNLOAD url` | Download file | `file = DOWNLOAD "https://example.com/file.pdf"` |
| `INCLUDE file` | Include script | `INCLUDE "functions.gbdialog"` |
| `DIR path` | List directory | `files = DIR "documents/"` |
| `FILL template, data` | Fill template | `doc = FILL "template.docx", customer_data` |

### Data Conversion

| Command | Description | Example |
|---------|-------------|---------|
| `data AS IMAGE` | Convert to image | `chart = data AS IMAGE` |
| `data AS PDF` | Convert to PDF | `report = data AS PDF` |
| `CHART type, data, labels` | Create chart | `img = CHART "pie", [10,20,30], "A;B;C"` |
| `CHART PROMPT data, prompt` | AI chart generation | `chart = CHART PROMPT sales, "monthly bar chart"` |
| `QRCODE text` | Generate QR code | `qr = QRCODE "https://example.com"` |
| `FORMAT value, format` | Format value | `date = FORMAT today, "YYYY-MM-DD"` |
| `CONVERT file` | Convert file format | `html = CONVERT "design.ai"` |

### Web Automation

| Command | Description | Example |
|---------|-------------|---------|
| `OPEN url` | Open webpage | `page = OPEN "https://example.com"` |
| `OPEN url AS session` | Named session | `page = OPEN "https://example.com" AS #login` |
| `GET page, selector` | Get element | `text = GET page, "#title"` |
| `SET page, selector, value` | Set field value | `SET page, "#username", "user123"` |
| `CLICK page, selector` | Click element | `CLICK page, "#submit"` |
| `SCREENSHOT selector` | Take screenshot | `img = SCREENSHOT "body"` |
| `PRESS ENTER ON page` | Press Enter key | `PRESS ENTER ON page` |

### Advanced Operations

| Command | Description | Example |
|---------|-------------|---------|
| `TABLE name ON connection` | Define table | `TABLE "sales" ON "production_db"` |
| `NEW OBJECT` | Create object | `data = NEW OBJECT` |
| `NEW ARRAY` | Create array | `list = NEW ARRAY` |
| `ADD NOTE text` | Add to notes | `ADD NOTE "Customer requested callback"` |
| `ALLOW ROLE role` | Check authorization | `ALLOW ROLE "admin"` |
| `CONTINUATION TOKEN` | Get token | `token = CONTINUATION TOKEN` |
| `SET PARAM name AS value` | Store parameter | `SET PARAM last_contact AS today` |
| `GET PARAM name` | Retrieve parameter | `last = GET PARAM last_contact` |

### Configuration Commands

| Command | Description | Example |
|---------|-------------|---------|
| `SET SCHEDULE cron` | Schedule execution | `SET SCHEDULE "0 9 * * *"` |
| `SET LANGUAGE code` | Set language | `SET LANGUAGE "pt-BR"` |
| `SET TRANSLATOR state` | Toggle translation | `SET TRANSLATOR ON` |
| `SET THEME theme` | Set visual theme | `SET THEME "dark"` |
| `SET MAX LINES n` | Limit output | `SET MAX LINES 100` |
| `SET OPERATOR op` | Set default operator | `SET OPERATOR OR` |
| `SET FILTER TYPE types` | Set filter types | `SET FILTER TYPE date, string` |
| `SET PAGED mode` | Set pagination | `SET PAGED "auto"` |
| `SET WHOLE WORD bool` | Word matching | `SET WHOLE WORD TRUE` |
| `SET HEAR ON channel` | Switch input channel | `SET HEAR ON "+5511999999999"` |

### HTTP Configuration

| Command | Description | Example |
|---------|-------------|---------|
| `SET HTTP HEADER key = value` | Set header | `SET HTTP HEADER Authorization = "Bearer token"` |
| `SET HTTP USERNAME = value` | Set auth user | `SET HTTP USERNAME = "api_user"` |
| `SET HTTP PASSWORD = value` | Set auth pass | `SET HTTP PASSWORD = "secret"` |

### Control Flow

| Command | Description | Example |
|---------|-------------|---------|
| `IF condition THEN` | Conditional | `IF age > 18 THEN TALK "Adult" END IF` |
| `FOR EACH item IN list` | Loop through list | `FOR EACH customer IN customers` |
| `DO WHILE condition` | While loop | `DO WHILE count < 10` |
| `SELECT CASE variable` | Switch statement | `SELECT CASE option` |
| `EXIT` | Exit script | `EXIT` |
| `EXIT FOR` | Exit loop | `EXIT FOR` |
| `GOTO label` | Jump to label | `GOTO menu` |

### Database Connections

Configure external databases in `config.csv`:

```csv
# PostgreSQL
mydb-driver,postgres
mydb-host,localhost
mydb-port,5432
mydb-database,production
mydb-username,dbuser
mydb-password,dbpass

# MySQL/MariaDB
mysql-driver,mysql
mysql-host,localhost
mysql-port,3306
mysql-database,myapp
mysql-username,root
mysql-password,pass

# SQL Server
mssql-driver,mssql
mssql-host,server.database.windows.net
mssql-port,1433
mssql-database,mydb
mssql-username,sa
mssql-password,pass
```

Then use in BASIC:

```basic
TABLE customers ON mydb
    id AS integer PRIMARY KEY
    name AS string(100)
    email AS string(255)
    created_at AS datetime

' Use the table
SAVE customers, customer_data
results = FIND customers, "created_at > '2024-01-01'"
```

## Complete Examples

### Multi-Channel Customer Service Bot

```basic
' Customer service bot with channel routing
SET SCHEDULE "0 9-18 * * 1-5"  ' Business hours only

' Main entry point
main:
SET HEAR ON whatsapp  ' Default to WhatsApp

HEAR initial_message AS TEXT

' Detect urgency
IF initial_message CONTAINS "urgent" OR initial_message CONTAINS "emergency" THEN
    ' Route to priority support
    TALK TO "teams:priority-support", "URGENT from " + user.channel + ": " + initial_message
    TALK "Your request has been marked as urgent. An agent will contact you shortly."
    
    ' Send notification to multiple channels
    SEND TO "email:manager@company.com", "Urgent request received"
    SEND TO "whatsapp:+5511999999999", "Urgent support needed"
END IF

' Show menu based on channel
IF user.channel == "whatsapp" THEN
    TALK "Welcome! Please select an option:"
    HEAR choice AS "1-Support", "2-Sales", "3-Agent", "0-Exit"
ELSE IF user.channel == "instagram" THEN
    TALK "Hi! How can we help you today?"
    HEAR choice AS "Support", "Sales", "Human Agent"
ELSE
    TALK "Hello! Type 'help' for options."
    HEAR choice
END IF

' Process choice
SELECT CASE choice
    CASE "1-Support", "Support", "help"
        GOTO technical_support
    
    CASE "2-Sales", "Sales"
        GOTO sales_inquiry
    
    CASE "3-Agent", "Human Agent", "agent"
        GOTO human_transfer
    
    CASE "0-Exit", "Exit", "bye"
        TALK "Thank you for contacting us!"
        EXIT
        
    CASE ELSE
        TALK "Invalid option. Please try again."
        GOTO main
END SELECT

' Technical support flow
technical_support:
    TALK "Please describe your technical issue:"
    HEAR issue AS TEXT
    
    ' Log to database
    ticket = NEW OBJECT
    ticket.customer = user.id
    ticket.channel = user.channel
    ticket.issue = issue
    ticket.timestamp = NOW()
    
    SAVE "support_tickets", ticket
    
    ' Notify support team
    TALK TO "teams:tech-support", "New ticket from " + user.channel + ": " + issue
    
    TALK "Ticket created. Our team will contact you within 24 hours."
    
    ' Send confirmation
    IF user.channel == "whatsapp" THEN
        SEND FILE TO user.id, ticket AS PDF, "Your support ticket"
    ELSE IF user.channel == "email" THEN
        SEND TO user.id, "Ticket #" + ticket.id + " created: " + issue
    END IF
    
    GOTO main

' Sales inquiry flow
sales_inquiry:
    TALK "What product are you interested in?"
    HEAR product AS TEXT
    
    ' Get product information
    products = FIND "products.xlsx", "name LIKE '" + product + "'"
    
    IF products.length > 0 THEN
        ' Send product catalog
        catalog = products AS PDF
        SEND FILE TO user.id, catalog, "Product Information"
        
        TALK "I've sent you our product catalog. Would you like to speak with sales?"
        HEAR confirm AS BOOLEAN
        
        IF confirm THEN
            GOTO human_transfer
        END IF
    ELSE
        TALK "Product not found. Let me connect you with sales."
        GOTO human_transfer
    END IF
    
    GOTO main

' Human transfer flow
human_transfer:
    TALK "Connecting you to a human agent..."
    
    ' Find available agent based on channel
    agent = GET "https://api.company.com/next-available-agent"
    
    IF agent.available THEN
        ' Create bridge between customer and agent
        TALK TO agent.channel + ":" + agent.id, "New customer from " + user.channel
        TALK TO agent.channel + ":" + agent.id, "Customer: " + user.id
        TALK TO agent.channel + ":" + agent.id, "Initial message: " + initial_message
        
        TALK "You've been connected to " + agent.name
        
        ' Bridge messages
        bridge_loop:
            HEAR customer_msg
            
            IF customer_msg == "end chat" THEN
                TALK "Chat ended. Thank you!"
                GOTO main
            END IF
            
            TALK TO agent.channel + ":" + agent.id, customer_msg
            GOTO bridge_loop
    ELSE
        TALK "All agents are busy. We'll contact you within 1 hour."
        
        ' Queue for callback
        callback = NEW OBJECT
        callback.customer = user.id
        callback.channel = user.channel
        callback.requested_at = NOW()
        
        SAVE "callback_queue", callback
    END IF
    
    GOTO main
```

### Broadcasting Campaign System

```basic
' Marketing campaign broadcaster
SET MAX LINES 1000

' Load campaign data
campaign = FIND "campaign.xlsx"
customers = FIND "customers.csv", "opt_in=true"

' Segment customers by channel preference
whatsapp_list = SELECT * FROM customers WHERE preferred_channel = 'whatsapp'
email_list = SELECT * FROM customers WHERE preferred_channel = 'email'
teams_list = SELECT * FROM customers WHERE preferred_channel = 'teams'

' Prepare personalized messages
FOR EACH customer IN customers
    message = "Hi " + customer.name + "! " + campaign.message
    
    ' Add personalized offer
    IF customer.tier == "gold" THEN
        message = message + " As a Gold member, you get 20% extra discount!"
    END IF
    
    ' Send via preferred channel
    IF customer.preferred_channel == "whatsapp" AND customer.phone != "" THEN
        SEND FILE TO customer.phone, campaign.image, message
    ELSE IF customer.preferred_channel == "email" AND customer.email != "" THEN
        SEND TO "email:" + customer.email, message
    ELSE IF customer.preferred_channel == "teams" AND customer.teams_id != "" THEN
        SEND TO "teams:" + customer.teams_id, message
    END IF
    
    ' Log delivery
    log = NEW OBJECT
    log.customer_id = customer.id
    log.campaign_id = campaign.id
    log.channel = customer.preferred_channel
    log.sent_at = NOW()
    
    SAVE "campaign_log", log
    
    ' Rate limiting
    WAIT 1
NEXT

' Generate report
report = SELECT 
    channel,
    COUNT(*) as total_sent,
    SUM(CASE WHEN status='delivered' THEN 1 ELSE 0 END) as delivered
FROM campaign_log
GROUP BY channel

' Send report to management
report_pdf = report AS PDF
SEND FILE TO "email:marketing@company.com", report_pdf, "Campaign Report"
TALK TO "teams:marketing-channel", "Campaign completed. " + customers.length + " messages sent."
```

### Web Automation with Multi-Channel Notifications

```basic
' Price monitoring with notifications
SET SCHEDULE "0 */6 * * *"  ' Every 6 hours

products = FIND "monitor_products.csv"

FOR EACH product IN products
    ' Open product page
    page = OPEN product.url AS #monitor
    
    ' Get current price
    current_price = GET page, product.price_selector
    current_price = PARSE_NUMBER(current_price)
    
    ' Check for price change
    IF current_price < product.last_price THEN
        discount = ((product.last_price - current_price) / product.last_price) * 100
        
        message = "PRICE DROP! " + product.name + " is now $" + current_price
        message = message + " (" + discount + "% off)"
        
        ' Notify via multiple channels based on discount level
        IF discount > 20 THEN
            ' Big discount - notify everywhere
            BROADCAST message TO product.watchers
            
            ' Send to Telegram group
            TALK TO "telegram:price-alerts", message
            
            ' Send to WhatsApp broadcast list
            FOR EACH watcher IN product.whatsapp_watchers
                TALK TO watcher, message
                SEND FILE TO watcher, SCREENSHOT product.price_selector, "Price proof"
            NEXT
        ELSE
            ' Small discount - email only
            FOR EACH watcher IN product.email_watchers
                SEND TO "email:" + watcher, message
            NEXT
        END IF
        
        ' Update database
        product.last_price = current_price
        product.last_check = NOW()
        SAVE "monitor_products", product
    END IF
    
    WAIT 5  ' Rate limiting between checks
NEXT

TALK TO "teams:monitoring", "Price check completed at " + NOW()
```

## Error Handling

```basic
' Robust error handling example
TRY
    result = GET "https://api.example.com/data"
    
    IF result.error THEN
        THROW "API returned error: " + result.error
    END IF
    
    SAVE "api_data", result
    
CATCH error
    ' Log error
    error_log = NEW OBJECT
    error_log.message = error
    error_log.timestamp = NOW()
    error_log.user = user.id
    
    SAVE "error_log", error_log
    
    ' Notify administrators
    TALK TO "teams:tech-support", "Error occurred: " + error
    TALK TO "email:admin@company.com", "System error logged"
    
    ' Inform user
    TALK "An error occurred. Our team has been notified."
    
FINALLY
    ' Cleanup
    CLOSE page
END TRY
```

## Best Practices

1. **Channel Detection**: Let the system auto-detect channels when possible
2. **Fallback Channels**: Always have a fallback communication method
3. **Rate Limiting**: Use WAIT between bulk operations
4. **Error Recovery**: Implement try-catch for external operations
5. **Logging**: Log all cross-channel communications
6. **User Preferences**: Store and respect user channel preferences
7. **Business Hours**: Check business hours before routing to human agents
8. **Message Templates**: Use templates for consistent multi-channel messaging
9. **Testing**: Test each channel individually before broadcasting
10. **Compliance**: Ensure opt-in consent for each communication channel

## Webhook Endpoints

After configuration, set up these webhook endpoints in each platform:

- **WhatsApp**: `https://your-domain/api/channels/whatsapp/webhook`
- **Instagram**: `https://your-domain/api/channels/instagram/webhook`
- **Teams**: `https://your-domain/api/channels/teams/messages`

## Support

For additional support and updates, visit:
- GitHub: https://github.com/GeneralBots/BotServer
- Documentation: https://docs.generalbots.com
- Community: https://community.generalbots.com