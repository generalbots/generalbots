# Mail - Email Client

> **Your intelligent inbox**

![Mail Flow](../../assets/suite/mail-flow.svg)

---

## Overview

Mail is the email application in General Bots Suite. Read, compose, and organize your emails with AI assistance. Mail helps you write better emails, find important messages, and stay on top of your inbox without the clutter.

---

## Interface Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Mail                                [Compose] [🔍 Search] [⚙️]    [×]  │
├──────────────┬──────────────────────────────────────────────────────────┤
│              │                                                          │
│  FOLDERS     │  Inbox (23)                              [Refresh] [▼]   │
│  ───────────  │  ═══════════════════════════════════════════════════    │
│              │                                                          │
│  📥 Inbox 23 │  ┌─────────────────────────────────────────────────────┐ │
│  ⭐ Starred  │  │ ☐ ⭐ Sarah Johnson                          10:32 AM │ │
│  📤 Sent     │  │      Q2 Report Review                               │ │
│  📝 Drafts 2 │  │      Please review the attached Q2 report and...    │ │
│  🗑️ Trash    │  └─────────────────────────────────────────────────────┘ │
│              │  ┌─────────────────────────────────────────────────────┐ │
│  ───────────  │  │ ☐    Mike Chen                              9:15 AM │ │
│              │  │      Meeting Tomorrow                               │ │
│  LABELS      │  │      Hi, just confirming our meeting tomorrow at... │ │
│  ───────────  │  └─────────────────────────────────────────────────────┘ │
│  🔴 Urgent   │  ┌─────────────────────────────────────────────────────┐ │
│  🟢 Personal │  │ ☐    LinkedIn                            Yesterday │ │
│  🔵 Work     │  │      You have 5 new connection requests             │ │
│  🟡 Finance  │  │      People are looking at your profile...          │ │
│              │  └─────────────────────────────────────────────────────┘ │
│  ───────────  │  ┌─────────────────────────────────────────────────────┐ │
│              │  │ ☐    Newsletter                          Yesterday │ │
│  [+ Label]   │  │      Weekly Tech Digest                             │ │
│              │  │      This week in tech: AI advances, new...         │ │
│              │  └─────────────────────────────────────────────────────┘ │
│              │                                                          │
│              │  ─────────────────────────────────────────────────────   │
│              │  Showing 1-23 of 23                    [◄ Prev] [Next ►] │
│              │                                                          │
└──────────────┴──────────────────────────────────────────────────────────┘
```

---

## Features

### Reading Emails

**Opening an Email**

1. Click on any email in the list
2. The email opens in the reading pane

```
┌─────────────────────────────────────────────────────────────────────────┐
│  ← Back to Inbox                        [Reply] [Forward] [Delete] [⋮]  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Q2 Report Review                                                       │
│  ════════════════════════════════════════════════════════════════════   │
│                                                                         │
│  From:    Sarah Johnson <sarah.johnson@company.com>                     │
│  To:      You <you@company.com>                                         │
│  Date:    May 15, 2025 at 10:32 AM                                      │
│                                                                         │
│  ─────────────────────────────────────────────────────────────────────  │
│                                                                         │
│  Hi,                                                                    │
│                                                                         │
│  Please review the attached Q2 report and let me know if you have       │
│  any questions. I've highlighted the key metrics on page 3.             │
│                                                                         │
│  Key points:                                                            │
│  • Revenue increased 15% from Q1                                        │
│  • Customer acquisition cost decreased by 8%                            │
│  • Retention rate steady at 94%                                         │
│                                                                         │
│  Looking forward to your feedback.                                      │
│                                                                         │
│  Best,                                                                  │
│  Sarah                                                                  │
│                                                                         │
│  ─────────────────────────────────────────────────────────────────────  │
│                                                                         │
│  ATTACHMENTS                                                            │
│  ┌────────────────────────────┐                                        │
│  │ 📄 Q2_Report_2025.pdf      │                                        │
│  │    2.4 MB                  │                                        │
│  │    [Download] [Preview]    │                                        │
│  └────────────────────────────┘                                        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**Email Actions**

| Action | What It Does |
|--------|--------------|
| **Reply** | Respond to the sender |
| **Reply All** | Respond to all recipients |
| **Forward** | Send to someone else |
| **Archive** | Remove from inbox, keep searchable |
| **Delete** | Move to trash |
| **Star** | Mark as important |
| **Mark Unread** | Show as unread again |

---

### Composing Emails

**Starting a New Email**

1. Click **Compose** button
2. Fill in the fields
3. Click **Send**

```
┌─────────────────────────────────────────────────────────────────────────┐
│  New Message                                                      [×]   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  To:                                                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ john.smith@company.com                                          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│  [Cc] [Bcc]                                                             │
│                                                                         │
│  Subject:                                                               │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Project Update - May 15                                         │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ B  I  U  │ • ≡ │ 🔗 │                                           │   │
│  ├─────────────────────────────────────────────────────────────────┤   │
│  │                                                                 │   │
│  │ Hi John,                                                        │   │
│  │                                                                 │   │
│  │ Here's the weekly project update:                               │   │
│  │                                                                 │   │
│  │ • Completed the user testing phase                              │   │
│  │ • Fixed 12 bugs from QA                                         │   │
│  │ • On track for next milestone                                   │   │
│  │                                                                 │   │
│  │ Let me know if you have questions.                              │   │
│  │                                                                 │   │
│  │ Best,                                                           │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌────────────────────────────┐                                        │
│  │ 📎 project-timeline.pdf    │  [×]                                   │
│  │    156 KB                  │                                        │
│  └────────────────────────────┘                                        │
│  [📎 Attach File]                                                       │
│                                                                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │      Send       │  │  Save Draft     │  │  ✨ AI Help     │         │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘         │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**Compose Fields**

| Field | Purpose | Required |
|-------|---------|----------|
| **To** | Primary recipient(s) | Yes |
| **Cc** | Carbon copy (visible to all) | No |
| **Bcc** | Blind copy (hidden from others) | No |
| **Subject** | Email topic | Recommended |
| **Body** | Your message | Yes |
| **Attachments** | Files to include | No |

---

### AI Email Assistant ✨

Let AI help you write better emails:

**Write an Email**

```
You: Write an email declining a meeting politely
Bot: Here's a draft:

     Subject: Unable to Attend - Project Review Meeting
     
     Hi [Name],
     
     Thank you for the meeting invitation. Unfortunately, I have 
     a scheduling conflict and won't be able to attend the 
     project review meeting on [date].
     
     Would it be possible to reschedule, or could someone share 
     the meeting notes with me afterward?
     
     Thank you for understanding.
     
     Best regards,
     [Your name]
     
     [📧 Use This] [✏️ Edit] [🔄 Regenerate]
```

**Improve Your Draft**

1. Select text in your email
2. Click **✨ AI Help**
3. Choose an option:

```
┌─────────────────────────────────────┐
│  ✨ AI Email Help                   │
├─────────────────────────────────────┤
│                                     │
│  [📝 Improve writing      ]         │
│  [✂️  Make it shorter     ]         │
│  [💼 Make it professional ]         │
│  [😊 Make it friendlier   ]         │
│  [🔧 Fix grammar          ]         │
│  [📋 Add bullet points    ]         │
│  [🌐 Translate to...      ]         │
│                                     │
└─────────────────────────────────────┘
```

**Summarize Long Emails**

```
You: Summarize this email thread
Bot: 📋 Email Thread Summary (15 messages)
     
     Topic: Q3 Budget Approval
     
     Key Points:
     • Marketing requested $50K increase for Q3
     • Finance asked for detailed breakdown
     • VP approved $35K with conditions
     • Final approval pending from CEO
     
     Action Items:
     • You: Submit revised proposal by Friday
     • Sarah: Schedule follow-up meeting
     
     Decision: Partial approval ($35K of $50K requested)
```

**Smart Reply**

Get quick reply suggestions:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Smart Replies:                                                         │
│                                                                         │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────┐ │
│  │ Sounds good, I'll   │  │ Thanks for the      │  │ Let me check    │ │
│  │ review it today.    │  │ update!             │  │ and get back    │ │
│  │                     │  │                     │  │ to you.         │ │
│  └─────────────────────┘  └─────────────────────┘  └─────────────────┘ │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### Email Organization

#### Folders

| Folder | Purpose |
|--------|---------|
| **Inbox** | Incoming emails |
| **Starred** | Emails you've starred |
| **Sent** | Emails you've sent |
| **Drafts** | Unsent emails |
| **Trash** | Deleted emails (auto-deleted after 30 days) |
| **Archive** | Archived emails (searchable) |
| **Spam** | Suspected spam |

#### Labels

Create custom labels to organize emails:

1. Click **+ Label** in the sidebar
2. Enter a name
3. Choose a color
4. Click **Create**

**Apply Labels**

- Drag email to label in sidebar
- Or right-click email → **Add Label**
- Or use keyboard: `L` then select label

```
┌─────────────────────────────────────┐
│  Add Label                    [×]   │
├─────────────────────────────────────┤
│                                     │
│  🔍 Search labels...                │
│                                     │
│  ☐ 🔴 Urgent                        │
│  ☑ 🟢 Personal                      │
│  ☐ 🔵 Work                          │
│  ☐ 🟡 Finance                       │
│  ☐ 🟣 Projects                      │
│                                     │
│  [+ Create New Label]               │
│                                     │
│  ┌─────────────────────────────┐    │
│  │         Apply               │    │
│  └─────────────────────────────┘    │
│                                     │
└─────────────────────────────────────┘
```

#### Filters

Create rules to automatically organize incoming emails:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Create Filter                                                    [×]   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  WHEN EMAIL MATCHES                                                     │
│  ──────────────────                                                     │
│                                                                         │
│  From:      [newsletter@                    ]                           │
│  To:        [                               ]                           │
│  Subject:   [                               ]                           │
│  Has words: [                               ]                           │
│                                                                         │
│  THEN DO THIS                                                           │
│  ────────────                                                           │
│                                                                         │
│  ☑ Skip inbox (archive)                                                 │
│  ☐ Mark as read                                                         │
│  ☑ Apply label: [Newsletters     ▼]                                    │
│  ☐ Star it                                                              │
│  ☐ Delete it                                                            │
│  ☐ Forward to: [                ]                                       │
│                                                                         │
│  ☑ Also apply to existing emails (45 matches)                           │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      Create Filter                              │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### Search

Find emails quickly with powerful search:

**Basic Search**

Type in the search box to find emails containing those words.

**Advanced Search**

Use operators for precise searches:

| Operator | Example | Finds |
|----------|---------|-------|
| `from:` | `from:sarah` | Emails from Sarah |
| `to:` | `to:john` | Emails sent to John |
| `subject:` | `subject:report` | Emails with "report" in subject |
| `has:attachment` | `has:attachment` | Emails with attachments |
| `is:starred` | `is:starred` | Starred emails |
| `is:unread` | `is:unread` | Unread emails |
| `label:` | `label:work` | Emails with "work" label |
| `after:` | `after:2025-05-01` | Emails after May 1, 2025 |
| `before:` | `before:2025-05-15` | Emails before May 15, 2025 |

**Combine Operators**

```
from:sarah has:attachment after:2025-05-01 subject:report
```

Finds: Emails from Sarah with attachments after May 1, 2025, with "report" in the subject.

---

### Attachments

**Viewing Attachments**

- Click **Preview** to view without downloading
- Click **Download** to save to your device
- Click the attachment name to open

**Supported Preview Types**

| Type | Extensions |
|------|------------|
| Documents | PDF, DOC, DOCX |
| Spreadsheets | XLS, XLSX, CSV |
| Images | JPG, PNG, GIF |
| Text | TXT, MD |

**Attachment Size Limits**

- Maximum single file: 25 MB
- Maximum total per email: 25 MB

---

## Keyboard Shortcuts

### Navigation

| Shortcut | Action |
|----------|--------|
| `J` | Next email |
| `K` | Previous email |
| `O` or `Enter` | Open email |
| `U` | Back to list |
| `G` then `I` | Go to Inbox |
| `G` then `S` | Go to Starred |
| `G` then `T` | Go to Sent |
| `G` then `D` | Go to Drafts |

### Actions

| Shortcut | Action |
|----------|--------|
| `C` | Compose new email |
| `R` | Reply |
| `A` | Reply all |
| `F` | Forward |
| `E` | Archive |
| `#` | Delete |
| `S` | Star/unstar |
| `L` | Add label |
| `V` | Move to folder |
| `Shift+U` | Mark unread |

### Selection

| Shortcut | Action |
|----------|--------|
| `X` | Select email |
| `*` then `A` | Select all |
| `*` then `N` | Deselect all |
| `*` then `R` | Select read |
| `*` then `U` | Select unread |

### Other

| Shortcut | Action |
|----------|--------|
| `/` | Search |
| `?` | Show shortcuts |
| `Escape` | Close dialog |
| `Ctrl+Enter` | Send email |

---

## Tips & Tricks

### Inbox Management

💡 **Use filters** to automatically organize newsletters and notifications

💡 **Archive instead of delete** - keeps emails searchable but clears inbox

💡 **Star important emails** you need to return to

💡 **Process emails once** - reply, archive, or delete immediately

### Writing Better Emails

💡 **Use AI to shorten** long emails - busy people appreciate brevity

💡 **Add a clear subject** that summarizes the email's purpose

💡 **Use bullet points** for lists and action items

💡 **Put the ask first** - don't bury your request at the bottom

### Search Tips

💡 **Search by sender** when you remember who sent something

💡 **Search attachments** with `has:attachment filename:report`

💡 **Search date ranges** when you remember when

💡 **Save frequent searches** as filters

### Productivity Tips

💡 **Check email at set times** instead of constantly

💡 **Use Smart Reply** for quick acknowledgments

💡 **Unsubscribe** from newsletters you don't read

💡 **Use templates** for repetitive responses

---

## Troubleshooting

### Emails not loading

**Possible causes:**
1. Internet connection lost
2. Email server temporarily unavailable
3. Browser cache issue

**Solution:**
1. Check your internet connection
2. Click Refresh to reload
3. Try clearing browser cache
4. Wait a few minutes and try again

---

### Can't send email

**Possible causes:**
1. Missing recipient address
2. Attachment too large
3. Email server issue

**Solution:**
1. Verify "To" field has valid email address
2. Reduce attachment size or use Drive link
3. Save as draft and try again later
4. Check email settings are configured

---

### Search not finding emails

**Possible causes:**
1. Typo in search terms
2. Email is in Trash or Spam
3. Using wrong search operators

**Solution:**
1. Try different keywords
2. Check Trash and Spam folders
3. Use simpler search terms
4. Try searching "All Mail"

---

### Attachments won't download

**Possible causes:**
1. File blocked by browser
2. Download folder full
3. File type blocked

**Solution:**
1. Check browser download settings
2. Clear space on your device
3. Right-click and "Save As"
4. Try a different browser

---

## BASIC Integration

Control Mail from your bot dialogs:

### Send an Email

```basic
email = NEW OBJECT
email.to = "john@company.com"
email.subject = "Meeting Reminder"
email.body = "Don't forget our meeting tomorrow at 2 PM."

SEND EMAIL email
TALK "Email sent to John!"
```

### Send with Attachment

```basic
email = NEW OBJECT
email.to = user.email
email.subject = "Your Report"
email.body = "Please find your report attached."
email.attachments = [reportFile]

SEND EMAIL email
TALK "Report sent to your email!"
```

### Check for New Emails

```basic
newEmails = GET EMAILS WHERE "is:unread"

IF COUNT(newEmails) > 0 THEN
    TALK "You have " + COUNT(newEmails) + " unread emails."
    TALK "Most recent from: " + newEmails[0].from
ELSE
    TALK "No new emails!"
END IF
```

### Search Emails

```basic
HEAR query AS TEXT "What should I search for?"

results = SEARCH EMAILS query

IF COUNT(results) > 0 THEN
    TALK "Found " + COUNT(results) + " emails:"
    FOR i = 1 TO MIN(5, COUNT(results))
        TALK "- " + results[i].subject + " from " + results[i].from
    NEXT
ELSE
    TALK "No emails found matching '" + query + "'"
END IF
```

### AI Email Drafting

```basic
HEAR recipient AS EMAIL "Who should I email?"
HEAR topic AS TEXT "What's the email about?"
HEAR tone AS TEXT "What tone? (formal/casual/friendly)"

draft = GENERATE EMAIL
    TO recipient
    ABOUT topic
    TONE tone

TALK "Here's a draft:"
TALK draft.body
TALK ""
HEAR confirm AS BOOLEAN "Should I send it?"

IF confirm THEN
    SEND EMAIL draft
    TALK "Email sent!"
ELSE
    TALK "No problem. Draft saved."
    SAVE DRAFT draft
END IF
```

---

## Email Configuration

Configure email settings in your bot's config.csv:

| Setting | Description | Example |
|---------|-------------|---------|
| `MAIL_PROVIDER` | Email service | `gmail`, `outlook`, `smtp` |
| `MAIL_HOST` | SMTP server | `smtp.gmail.com` |
| `MAIL_PORT` | SMTP port | `587` |
| `MAIL_USER` | Email account | `bot@company.com` |
| `MAIL_FROM_NAME` | Display name | `Company Bot` |

---

## See Also

- [Calendar App](./calendar.md) - Schedule meetings from emails
- [Tasks App](./tasks.md) - Create tasks from emails
- [Paper App](./paper.md) - Draft longer documents
- [How To: Create Your First Bot](../how-to/create-first-bot.md)