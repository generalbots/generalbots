# Announcements Template

The announcements template provides a company communication system for sharing weekly updates, circulars, and organizational news through an AI-powered conversational interface.

## Topic: Company Announcements & Communications

This template is perfect for:
- Weekly company announcements
- Internal circulars distribution
- Multi-topic news aggregation
- Organizational communications
- Employee information portals

## The Code

```basic
resume1 = GET BOT MEMORY("resume")
resume2 = GET BOT MEMORY("auxiliom")
resume3 = GET BOT MEMORY("toolbix")

SET CONTEXT "general"  AS resume1
SET CONTEXT "auxiliom" AS resume2
SET CONTEXT "toolbix"  AS resume3

CLEAR SUGGESTIONS

ADD SUGGESTION "general" AS "Weekly announcements"
ADD SUGGESTION "general" AS "Latest circulars"
ADD SUGGESTION "auxiliom" AS "What is Auxiliom?"
ADD SUGGESTION "auxiliom" AS "Auxiliom services"
ADD SUGGESTION "toolbix" AS "Toolbix features"
ADD SUGGESTION "toolbix" AS "Toolbix for business"

ADD TOOL "change-subject"

TALK resume1
TALK "Ask me about any announcement or circular."
```

## Sample Dialogs

These conversations show how the announcements template works in real-world scenarios.

### Dialog 1: Weekly Announcements

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📢</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Company News</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Company News</div>
      <p>📰 <strong>Weekly Announcements - Week 3</strong></p>
      <p>• Q4 results exceeded expectations by 15%</p>
      <p>• New health benefits program launching Feb 1</p>
      <p>• Office closure Jan 20 for maintenance</p>
      <p></p>
      <p>Ask me about any announcement or circular.</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Tell me more about the health benefits</p>
      <div class="wa-time">09:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🏥 <strong>New Health Benefits Program</strong></p>
      <p><strong>Effective:</strong> February 1, 2025</p>
      <p></p>
      <p><strong>Key Changes:</strong></p>
      <p>• Mental health coverage increased to $5,000/year</p>
      <p>• Dental now covers orthodontics</p>
      <p>• New telemedicine option at no cost</p>
      <p>• Gym membership reimbursement up to $50/month</p>
      <p></p>
      <p>📧 HR will send enrollment forms by Jan 25.</p>
      <div class="wa-time">09:01</div>
    </div>
  </div>
</div>

### Dialog 2: Switching Topics

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📢</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Company News</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>What is Auxiliom?</p>
      <div class="wa-time">10:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔧 <strong>Auxiliom Overview</strong></p>
      <p>Auxiliom is our enterprise support platform that provides:</p>
      <p></p>
      <p>• 24/7 IT infrastructure monitoring</p>
      <p>• Automated incident response</p>
      <p>• Service desk integration</p>
      <p>• Performance analytics dashboard</p>
      <p></p>
      <p>Would you like to know more about specific Auxiliom services?</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Switch to Toolbix</p>
      <div class="wa-time">10:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🛠️ <strong>Toolbix Platform</strong></p>
      <p>Toolbix is our productivity suite designed for modern teams:</p>
      <p></p>
      <p>• Project management & task tracking</p>
      <p>• Team collaboration tools</p>
      <p>• Document sharing & version control</p>
      <p>• Integration with 50+ business apps</p>
      <p></p>
      <p>What would you like to know about Toolbix?</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
</div>

### Dialog 3: Latest Circulars

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">📢</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Company News</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Show me the latest circulars</p>
      <div class="wa-time">14:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📋 <strong>Recent Circulars</strong></p>
      <p></p>
      <p><strong>CIR-2025-003</strong> (Jan 15)</p>
      <p>Updated Travel Expense Policy</p>
      <p></p>
      <p><strong>CIR-2025-002</strong> (Jan 10)</p>
      <p>Remote Work Guidelines 2025</p>
      <p></p>
      <p><strong>CIR-2025-001</strong> (Jan 3)</p>
      <p>Annual Compliance Training Schedule</p>
      <p></p>
      <p>Which circular would you like to read?</p>
      <div class="wa-time">14:00</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `GET BOT MEMORY` | Retrieve stored announcement summaries |
| `SET CONTEXT` | Define contexts for different topics |
| `CLEAR SUGGESTIONS` | Reset quick reply options |
| `ADD SUGGESTION` | Create topic-based quick replies |
| `ADD TOOL` | Register topic switching tool |
| `TALK` | Display announcements to user |

## How It Works

1. **Load Content**: Pre-stored summaries are retrieved from bot memory
2. **Multi-Context Setup**: Different contexts for each announcement topic
3. **Topic Suggestions**: Quick replies organized by topic category
4. **Dynamic Display**: Current announcements shown on start
5. **Topic Switching**: Users can change subjects using the tool

## Template Structure

```
announcements.gbai/
├── announcements.gbdialog/
│   ├── start.bas           # Main entry point
│   ├── auth.bas            # Admin authentication
│   ├── change-subject.bas  # Topic switching
│   └── update-summary.bas  # Update announcements
├── announcements.gbkb/
│   ├── auxiliom/           # Auxiliom topic KB
│   ├── news/               # General news KB
│   └── toolbix/            # Toolbix topic KB
└── announcements.gbot/
    └── config.csv          # Bot configuration
```

## Change Subject Tool: change-subject.bas

```basic
PARAM subject AS STRING LIKE "toolbix" DESCRIPTION "Topic to switch to: general, auxiliom, toolbix"

DESCRIPTION "Change the current announcement topic"

subject_lower = LCASE(subject)

IF subject_lower = "general" OR INSTR(subject_lower, "news") > 0 OR INSTR(subject_lower, "announcement") > 0 THEN
    resume = GET BOT MEMORY("resume")
    SET CONTEXT "current" AS resume
    TALK "📰 Switched to **General Announcements**"
    TALK resume
ELSE IF subject_lower = "auxiliom" THEN
    resume = GET BOT MEMORY("auxiliom")
    SET CONTEXT "current" AS resume
    TALK "🔧 Switched to **Auxiliom**"
    TALK resume
ELSE IF subject_lower = "toolbix" THEN
    resume = GET BOT MEMORY("toolbix")
    SET CONTEXT "current" AS resume
    TALK "🛠️ Switched to **Toolbix**"
    TALK resume
ELSE
    TALK "Available topics: General Announcements, Auxiliom, Toolbix"
    TALK "Which topic would you like?"
END IF

RETURN subject_lower
```

## Update Summary Tool: update-summary.bas

```basic
PARAM topic AS STRING LIKE "general" DESCRIPTION "Topic to update"
PARAM content AS STRING DESCRIPTION "New summary content"

DESCRIPTION "Update the announcement summary for a topic (admin only)"

' Verify admin access
IF NOT IS_ADMIN(user_id) THEN
    TALK "⚠️ This action requires administrator privileges."
    RETURN NULL
END IF

topic_lower = LCASE(topic)

IF topic_lower = "general" THEN
    SET BOT MEMORY "resume", content
ELSE IF topic_lower = "auxiliom" THEN
    SET BOT MEMORY "auxiliom", content
ELSE IF topic_lower = "toolbix" THEN
    SET BOT MEMORY "toolbix", content
ELSE
    TALK "Unknown topic. Use: general, auxiliom, or toolbix"
    RETURN NULL
END IF

' Log the update
WITH updateLog
    timestamp = NOW()
    updatedBy = user_id
    topicUpdated = topic_lower
    contentLength = LEN(content)
END WITH

SAVE "announcement_log.csv", updateLog

TALK "✅ " + topic + " summary updated successfully!"
TALK "Changes are now live."

RETURN topic_lower
```

## Customization Ideas

### Add Email Distribution

```basic
ADD TOOL "send-announcement"

PARAM announcement AS STRING DESCRIPTION "Announcement to distribute"
PARAM recipients AS STRING LIKE "all" DESCRIPTION "Recipients: all, managers, department name"

' Get recipient list
IF recipients = "all" THEN
    employees = FIND "employees.csv"
ELSE IF recipients = "managers" THEN
    employees = FIND "employees.csv", "role = 'manager'"
ELSE
    employees = FIND "employees.csv", "department = '" + recipients + "'"
END IF

FOR EACH emp IN employees
    SEND MAIL emp.email, "Company Announcement", announcement
    WAIT 1
NEXT

TALK "📧 Announcement sent to " + UBOUND(employees) + " recipients."
```

### Add Announcement Categories

```basic
CLEAR SUGGESTIONS

ADD SUGGESTION "hr" AS "HR Updates"
ADD SUGGESTION "it" AS "IT Announcements"
ADD SUGGESTION "finance" AS "Finance News"
ADD SUGGESTION "events" AS "Upcoming Events"
ADD SUGGESTION "policy" AS "Policy Changes"
ADD SUGGESTION "all" AS "All Announcements"
```

### Add Read Receipts

```basic
' Track who has read announcements
WITH readReceipt
    userId = user_id
    announcementId = current_announcement_id
    readAt = NOW()
END WITH

SAVE "read_receipts.csv", readReceipt

' Check read percentage
total = COUNT("employees.csv")
reads = COUNT("read_receipts.csv", "announcementId = '" + current_announcement_id + "'")
percentage = (reads / total) * 100

TALK "📊 " + FORMAT(percentage, "#0") + "% of employees have read this announcement."
```

### Add Scheduled Announcements

```basic
PARAM schedule_time AS STRING LIKE "2025-01-20 09:00" DESCRIPTION "When to publish"
PARAM announcement AS STRING DESCRIPTION "Announcement content"

SET SCHEDULE schedule_time

SET BOT MEMORY "resume", announcement

' Notify all employees
employees = FIND "employees.csv"
FOR EACH emp IN employees
    TALK TO emp.phone, "📢 New announcement: " + LEFT(announcement, 100) + "..."
NEXT

TALK "Announcement published and distributed."
```

## Best Practices

1. **Keep It Current**: Update announcements regularly
2. **Organize by Topic**: Use clear topic categories
3. **Summarize**: Start with key points, allow drill-down
4. **Archive Old News**: Move outdated items to archive
5. **Track Engagement**: Monitor which topics get most questions

## Related Templates

- [broadcast.bas](./broadcast.md) - Mass messaging to employees
- [edu.bas](./edu.md) - Educational announcements
- [hr-employees.bas](./hr-employees.md) - Employee communications

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