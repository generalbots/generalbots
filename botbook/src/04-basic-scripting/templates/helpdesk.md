# IT Helpdesk Template

The IT Helpdesk template provides a complete IT support ticketing system that helps users report problems, track ticket status, and get help with common technical issues.

## Topic: IT Support & Ticket Management

This template is perfect for:
- Internal IT support desks
- Technical support teams
- MSP (Managed Service Provider) helpdesks
- Customer technical support
- Self-service IT portals

## The Code

```basic
ADD TOOL "create-ticket"
ADD TOOL "check-ticket-status"
ADD TOOL "my-tickets"
ADD TOOL "update-ticket"
ADD TOOL "close-ticket"

USE KB "helpdesk.gbkb"

SET CONTEXT "it helpdesk" AS "You are an IT helpdesk assistant. Help users create support tickets, check ticket status, and troubleshoot common issues. Gather necessary information before creating tickets: issue description, urgency level, and affected systems."

CLEAR SUGGESTIONS

ADD SUGGESTION "new" AS "Report a problem"
ADD SUGGESTION "status" AS "Check ticket status"
ADD SUGGESTION "password" AS "Reset my password"
ADD SUGGESTION "vpn" AS "VPN issues"
ADD SUGGESTION "email" AS "Email not working"
ADD SUGGESTION "mytickets" AS "View my tickets"

BEGIN TALK
**IT Helpdesk Support**

I can help you with:
• Create a new support ticket
• Check ticket status
• Password resets
• Network and VPN problems
• Email issues
• Hardware and software support

For urgent issues affecting multiple users, mention "urgent" or "critical".

What can I help you with?
END TALK

BEGIN SYSTEM PROMPT
You are an IT Helpdesk support assistant.

Priority levels:
- Critical: System down, security breach, multiple users affected
- High: Single user unable to work, deadline impact
- Medium: Issue with workaround available
- Low: Minor inconvenience, feature requests

Before creating a ticket, collect:
- Clear description of the issue
- When the issue started
- Error messages if any
- Steps already tried

Try to resolve simple issues using the knowledge base before creating tickets.
END SYSTEM PROMPT
```

## Sample Dialogs

These conversations show how the IT Helpdesk template works in real-world scenarios.

### Dialog 1: Creating a Support Ticket

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🛠️</div>
    <div class="wa-header-info">
      <div class="wa-header-name">IT Helpdesk</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">IT Helpdesk</div>
      <p><strong>IT Helpdesk Support</strong></p>
      <p>I can help you with:</p>
      <p>• Create a new support ticket<br>• Check ticket status<br>• Password resets<br>• Network and VPN problems</p>
      <p>What can I help you with?</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>My computer won't turn on</p>
      <div class="wa-time">09:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>I'm sorry to hear that! Let me help you troubleshoot.</p>
      <p><strong>Quick checks:</strong></p>
      <p>1. Is the power cable securely connected?</p>
      <p>2. Is the power strip/outlet working?</p>
      <p>3. Do you see any lights on the computer?</p>
      <p>Have you tried these steps?</p>
      <div class="wa-time">09:01</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Yes, I checked everything. No lights at all.</p>
      <div class="wa-time">09:02 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>I'll create a ticket for hardware support. A few more questions:</p>
      <p>1. What is your computer model/asset tag?</p>
      <p>2. When did this issue start?</p>
      <p>3. Did anything unusual happen before (power outage, etc.)?</p>
      <div class="wa-time">09:02</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Dell Latitude, asset tag IT-4521. Started this morning. We had a storm last night.</p>
      <div class="wa-time">09:03 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>Ticket Created</strong></p>
      <p>🎫 <strong>Ticket #:</strong> INC-2025-0847</p>
      <p>📋 <strong>Issue:</strong> Computer won't power on - possible power surge damage</p>
      <p>⚡ <strong>Priority:</strong> High (user unable to work)</p>
      <p>🖥️ <strong>Asset:</strong> Dell Latitude IT-4521</p>
      <p>A technician will contact you within 2 hours. In the meantime, do you have a spare workstation you can use?</p>
      <div class="wa-time">09:03</div>
    </div>
  </div>
</div>

### Dialog 2: Password Reset

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🛠️</div>
    <div class="wa-header-info">
      <div class="wa-header-name">IT Helpdesk</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I forgot my password</p>
      <div class="wa-time">10:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔐 <strong>Password Reset</strong></p>
      <p>I can help you reset your password. For security, I need to verify your identity.</p>
      <p>Please confirm your employee ID:</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>EMP-2847</p>
      <div class="wa-time">10:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Thank you, Sarah. I found your account.</p>
      <p>I'm sending a password reset link to your registered email (s***@company.com).</p>
      <p>📧 Check your email for the reset link (valid for 15 minutes).</p>
      <p><strong>Tips for your new password:</strong></p>
      <p>• At least 12 characters</p>
      <p>• Mix of upper/lowercase letters</p>
      <p>• Include numbers and symbols</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Got it, thanks!</p>
      <div class="wa-time">10:33 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>You're welcome! 👍</p>
      <p>If you have trouble logging in after resetting, let me know.</p>
      <div class="wa-time">10:33</div>
    </div>
  </div>
</div>

### Dialog 3: Check Ticket Status

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🛠️</div>
    <div class="wa-header-info">
      <div class="wa-header-name">IT Helpdesk</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>What's the status of my ticket?</p>
      <div class="wa-time">14:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🎫 <strong>Your Open Tickets:</strong></p>
      <p></p>
      <p><strong>INC-2025-0847</strong></p>
      <p>📋 Computer won't power on</p>
      <p>📊 Status: <strong>In Progress</strong></p>
      <p>👤 Assigned to: Mike T.</p>
      <p>📝 Latest update: "Replacement power supply ordered, ETA tomorrow"</p>
      <p></p>
      <p><strong>INC-2025-0812</strong></p>
      <p>📋 Software installation request</p>
      <p>📊 Status: <strong>Pending Approval</strong></p>
      <p>👤 Waiting on: Manager approval</p>
      <p></p>
      <p>Would you like more details on either ticket?</p>
      <div class="wa-time">14:00</div>
    </div>
  </div>
</div>

### Dialog 4: Critical System Issue

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🛠️</div>
    <div class="wa-header-info">
      <div class="wa-header-name">IT Helpdesk</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>URGENT! The entire sales team can't access the CRM system!</p>
      <div class="wa-time">11:15 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🚨 <strong>Critical Issue Detected</strong></p>
      <p>I'm escalating this immediately as it affects multiple users.</p>
      <p>Quick questions:</p>
      <p>1. How many users are affected?</p>
      <p>2. What error message do they see?</p>
      <p>3. When did this start?</p>
      <div class="wa-time">11:15</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>About 15 people. They get "Connection refused" error. Started 10 minutes ago.</p>
      <div class="wa-time">11:16 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🚨 <strong>CRITICAL Ticket Created</strong></p>
      <p>🎫 <strong>Ticket #:</strong> INC-2025-0850</p>
      <p>📋 <strong>Issue:</strong> CRM System Outage - 15 users affected</p>
      <p>⚡ <strong>Priority:</strong> CRITICAL</p>
      <p>⏱️ <strong>Response SLA:</strong> 15 minutes</p>
      <p></p>
      <p>✅ On-call engineer notified</p>
      <p>✅ Infrastructure team alerted</p>
      <p>✅ Incident channel created</p>
      <p></p>
      <p>Someone will contact you within 15 minutes. I'll update you as soon as we know more.</p>
      <div class="wa-time">11:16</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `ADD TOOL` | Register ticket management tools |
| `USE KB` | Load helpdesk knowledge base for troubleshooting |
| `SET CONTEXT` | Define IT support assistant behavior |
| `ADD SUGGESTION` | Create common issue shortcuts |
| `BEGIN TALK` | Welcome message with options |
| `BEGIN SYSTEM PROMPT` | Priority definitions and guidelines |

## Template Structure

```
helpdesk.gbai/
├── helpdesk.gbdialog/
│   ├── start.bas              # Main entry point
│   ├── create-ticket.bas      # Ticket creation
│   ├── check-ticket-status.bas # Status lookup
│   ├── my-tickets.bas         # User's tickets
│   ├── update-ticket.bas      # Ticket updates
│   └── close-ticket.bas       # Ticket resolution
├── helpdesk.gbdrive/
│   └── templates/             # Response templates
├── helpdesk.gbkb/
│   ├── common-issues.md       # Troubleshooting guides
│   └── security-tips.md       # Security best practices
└── helpdesk.gbot/
    └── config.csv             # Bot configuration
```

## Create Ticket Tool: create-ticket.bas

```basic
PARAM description AS STRING LIKE "Computer won't start" DESCRIPTION "Issue description"
PARAM category AS STRING LIKE "hardware" DESCRIPTION "Category: hardware, software, network, email, access"
PARAM priority AS STRING LIKE "medium" DESCRIPTION "Priority: critical, high, medium, low" OPTIONAL

DESCRIPTION "Create a new IT support ticket"

' Get user information
user_email = FROM
user_name = USERNAME

' Auto-detect priority if not provided
IF NOT priority THEN
    IF INSTR(LOWER(description), "urgent") > 0 OR INSTR(LOWER(description), "critical") > 0 THEN
        priority = "critical"
    ELSE IF INSTR(LOWER(description), "can't work") > 0 OR INSTR(LOWER(description), "blocked") > 0 THEN
        priority = "high"
    ELSE
        priority = "medium"
    END IF
END IF

' Generate ticket number
ticketNumber = "INC-" + FORMAT(NOW(), "YYYY") + "-" + FORMAT(RANDOM(1000, 9999))

' Set SLA based on priority
SELECT CASE priority
    CASE "critical"
        slaMinutes = 15
        slaText = "15 minutes"
    CASE "high"
        slaMinutes = 120
        slaText = "2 hours"
    CASE "medium"
        slaMinutes = 480
        slaText = "8 hours"
    CASE "low"
        slaMinutes = 1440
        slaText = "24 hours"
END SELECT

' Create ticket record
WITH ticket
    id = ticketNumber
    user_email = user_email
    user_name = user_name
    description = description
    category = category
    priority = priority
    status = "open"
    sla_due = DATEADD(NOW(), slaMinutes, "minutes")
    created_at = NOW()
END WITH

SAVE "tickets.csv", ticket

' Send confirmation email
SEND MAIL user_email, "Ticket Created: " + ticketNumber, 
    "Your support ticket has been created.\n\n" +
    "Ticket: " + ticketNumber + "\n" +
    "Issue: " + description + "\n" +
    "Priority: " + priority + "\n" +
    "Response time: " + slaText

' Notify support team
IF priority = "critical" THEN
    SEND MAIL "oncall@company.com", "🚨 CRITICAL: " + ticketNumber, 
        "Critical ticket requires immediate attention:\n" + description
END IF

TALK "✅ Ticket **" + ticketNumber + "** created!"
TALK "Priority: " + UPPER(priority)
TALK "Expected response: " + slaText

RETURN ticketNumber
```

## My Tickets Tool: my-tickets.bas

```basic
PARAM status AS STRING LIKE "open" DESCRIPTION "Filter by status: open, closed, all" OPTIONAL

DESCRIPTION "View your support tickets"

user_email = FROM

IF NOT status OR status = "all" THEN
    tickets = FIND "tickets.csv", "user_email = '" + user_email + "'"
ELSE
    tickets = FIND "tickets.csv", "user_email = '" + user_email + "' AND status = '" + status + "'"
END IF

IF UBOUND(tickets) = 0 THEN
    TALK "You have no " + IIF(status, status, "") + " tickets."
    RETURN NULL
END IF

TALK "🎫 **Your Tickets:**"
TALK ""

FOR EACH ticket IN tickets
    statusIcon = "🔵"
    IF ticket.status = "open" THEN statusIcon = "🟡"
    IF ticket.status = "in_progress" THEN statusIcon = "🔵"
    IF ticket.status = "resolved" THEN statusIcon = "🟢"
    IF ticket.status = "closed" THEN statusIcon = "⚪"
    
    TALK "**" + ticket.id + "** " + statusIcon
    TALK "📋 " + LEFT(ticket.description, 50) + "..."
    TALK "📊 Status: " + ticket.status
    TALK "📅 Created: " + FORMAT(ticket.created_at, "MMM DD, YYYY")
    TALK ""
NEXT

RETURN tickets
```

## Customization Ideas

### Add Knowledge Base Self-Service

```basic
' Before creating a ticket, search KB for solutions
solutions = SEARCH KB description

IF UBOUND(solutions) > 0 THEN
    TALK "I found some articles that might help:"
    FOR EACH solution IN FIRST(solutions, 3)
        TALK "• " + solution.title
    NEXT
    TALK ""
    TALK "Did any of these solve your issue?"
    HEAR resolved
    
    IF LOWER(resolved) = "yes" THEN
        TALK "Great! Let me know if you need anything else."
        RETURN NULL
    END IF
END IF

' Continue to ticket creation...
```

### Add Asset Tracking

```basic
PARAM asset_tag AS STRING DESCRIPTION "Asset tag of affected equipment"

' Look up asset information
asset = FIND "assets.csv", "tag = '" + asset_tag + "'"

IF asset THEN
    ticket.asset_tag = asset_tag
    ticket.asset_type = asset.type
    ticket.asset_model = asset.model
    ticket.warranty_status = asset.warranty_expires > NOW()
    
    IF asset.warranty_expires > NOW() THEN
        TALK "ℹ️ This device is under warranty until " + FORMAT(asset.warranty_expires, "MMM DD, YYYY")
    END IF
END IF
```

### Add Escalation Rules

```basic
' Check if ticket needs escalation
IF ticket.priority = "critical" AND ticket.category = "security" THEN
    ' Escalate to security team
    SEND MAIL "security@company.com", "🔴 Security Incident: " + ticketNumber, description
    ticket.escalated_to = "security"
    ticket.escalation_time = NOW()
END IF

IF ticket.priority = "critical" AND DATEDIFF(NOW(), ticket.created_at, "minutes") > 30 THEN
    ' Escalate if no response in 30 minutes
    SEND MAIL "it-manager@company.com", "⚠️ SLA Breach Risk: " + ticketNumber, 
        "Critical ticket approaching SLA breach"
END IF
```

### Add Satisfaction Survey

```basic
' When closing ticket
IF action = "close" THEN
    ticket.status = "closed"
    ticket.closed_at = NOW()
    ticket.resolution = resolution
    
    UPDATE "tickets.csv", ticket
    
    TALK "Your ticket has been resolved!"
    TALK ""
    TALK "How would you rate your support experience?"
    ADD SUGGESTION "5" AS "⭐⭐⭐⭐⭐ Excellent"
    ADD SUGGESTION "4" AS "⭐⭐⭐⭐ Good"
    ADD SUGGESTION "3" AS "⭐⭐⭐ Average"
    ADD SUGGESTION "2" AS "⭐⭐ Poor"
    ADD SUGGESTION "1" AS "⭐ Very Poor"
    
    HEAR rating
    
    WITH feedback
        ticket_id = ticketNumber
        rating = rating
        timestamp = NOW()
    END WITH
    
    SAVE "satisfaction.csv", feedback
    
    TALK "Thank you for your feedback!"
END IF
```

## Priority Matrix

| Priority | Response Time | Resolution Time | Examples |
|----------|---------------|-----------------|----------|
| Critical | 15 minutes | 4 hours | System outage, security breach, multiple users down |
| High | 2 hours | 8 hours | Single user unable to work, deadline impact |
| Medium | 8 hours | 24 hours | Issue with workaround available |
| Low | 24 hours | 72 hours | Feature requests, minor inconveniences |

## Related Templates

- [hr/employees.bas](./employees.md) - Employee management integration
- [announcements.bas](./announcements.md) - IT announcements
- [backup.bas](./backup.md) - Backup and recovery

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:600px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-chat::after{content:'';display:table;clear:both}
.wa-message{clear:both;margin-bottom:10px;max-width:85%;position:relative}
.wa-message.user{float:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;border-radius:8px 0 8px 8px;margin-left:40px}
.wa-message.bot{float:left}
.wa-message.bot .wa-bubble{background-color:#fff;border-radius:0 8px 8px 8px;margin-right:40px}
.wa-bubble{padding:8px 12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
.wa-message.user .wa-time{color:#61a05e}
.wa-sender{font-size:12px;font-weight:600;color:#06cf9c;margin-bottom:2px}
.wa-status.read::after{content:'✓✓';color:#53bdeb;margin-left:4px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>