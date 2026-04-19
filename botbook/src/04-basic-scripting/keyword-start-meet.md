# START MEET / JOIN MEET Keywords

The `START MEET` and `JOIN MEET` keywords enable bots to create and participate in video meetings, bringing AI capabilities directly into video conferencing.

## Keywords

| Keyword | Purpose |
|---------|---------|
| `START MEET` | Create a new meeting room and get join link |
| `JOIN MEET` | Add the bot to an existing meeting |
| `LEAVE MEET` | Remove the bot from a meeting |
| `INVITE TO MEET` | Send meeting invitations to participants |

---

## START MEET

Creates a new video meeting room and optionally adds the bot as a participant.

### Syntax

```basic
room = START MEET "room-name"
room = START MEET "room-name" WITH BOT
room = START MEET "room-name" WITH OPTIONS options
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `room-name` | String | Display name for the meeting room |
| `WITH BOT` | Flag | Automatically add the bot to the meeting |
| `options` | JSON | Meeting configuration options |

### Options Object

```basic
' Options can be set as a JSON string
options = '{"recording": true, "transcription": true, "max_participants": 50}'
```

### Example

```basic
' Create a simple meeting
room = START MEET "Team Sync"
TALK "Meeting created! Join here: " + room.url

' Create meeting with bot participant
room = START MEET "AI-Assisted Workshop" WITH BOT
TALK "I've joined the meeting and I'm ready to help!"
TALK "Join link: " + room.url

' Create meeting with full options
options = '{"recording": true, "transcription": true, "bot_persona": "note-taker"}'
room = START MEET "Project Review" WITH OPTIONS options
```

### Return Value

Returns a room object with:

| Property | Description |
|----------|-------------|
| `room.id` | Unique room identifier |
| `room.url` | Join URL for participants |
| `room.name` | Room display name |
| `room.created` | Creation timestamp |
| `room.host_token` | Host access token |

---

## JOIN MEET

Adds the bot to an existing meeting room.

### Syntax

```basic
JOIN MEET room_id
JOIN MEET room_id AS "persona"
JOIN MEET room_url
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `room_id` | String | Meeting room ID |
| `room_url` | String | Meeting join URL |
| `persona` | String | Bot's display name in the meeting |

### Example

```basic
' Join by room ID
JOIN MEET "room-abc123"

' Join with custom persona
JOIN MEET "room-abc123" AS "Meeting Assistant"

' Join by URL
JOIN MEET "https://meet.gb/abc-123"

' Join and announce
JOIN MEET meeting_room AS "AI Note Taker"
TALK TO MEET "Hello everyone! I'm here to take notes. Just say 'note that' followed by anything important."
```

---

## LEAVE MEET

Removes the bot from the current meeting.

### Syntax

```basic
LEAVE MEET
LEAVE MEET room_id
```

### Example

```basic
' Leave current meeting
LEAVE MEET

' Leave specific meeting (when bot is in multiple)
LEAVE MEET "room-abc123"

' Graceful exit
TALK TO MEET "Thanks everyone! I'll send the meeting notes shortly."
WAIT 2
LEAVE MEET
```

---

## INVITE TO MEET

Sends meeting invitations to participants.

### Syntax

```basic
INVITE TO MEET room, participants
INVITE TO MEET room, participants, message
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `room` | Object/String | Room object or room ID |
| `participants` | Array | List of email addresses |
| `message` | String | Optional custom invitation message |

### Example

```basic
' Create room and invite team
room = START MEET "Sprint Planning" WITH BOT
participants = ["alice@company.com", "bob@company.com", "carol@company.com"]
INVITE TO MEET room, participants

TALK "Invitations sent to " + LEN(participants) + " participants"

' With custom message
INVITE TO MEET room, participants, "Join us for sprint planning! The AI assistant will be taking notes."
```

---

## TALK TO MEET

Sends a message to all meeting participants (text-to-speech or chat).

### Syntax

```basic
TALK TO MEET "message"
TALK TO MEET "message" AS CHAT
TALK TO MEET "message" AS VOICE
```

### Example

```basic
' Send as both chat and voice (default)
TALK TO MEET "Let's start with the agenda review."

' Chat only (no voice)
TALK TO MEET "Here's the link to the document: https://..." AS CHAT

' Voice only (no chat message)
TALK TO MEET "I've noted that action item." AS VOICE
```

---

## HEAR FROM MEET

Listens for speech or chat messages from meeting participants.

### Syntax

```basic
HEAR FROM MEET INTO variable
HEAR FROM MEET INTO variable TIMEOUT seconds
```

### Example

```basic
' Listen for meeting input
HEAR FROM MEET INTO participant_message

IF INSTR(participant_message, "note that") > 0 THEN
    note = REPLACE(participant_message, "note that", "")
    notes = notes + "\n- " + note
    TALK TO MEET "Got it! I've noted: " + note
END IF
```

---

## Complete Example: AI Meeting Assistant

```basic
' AI Meeting Assistant Bot
' Joins meetings, takes notes, and provides summaries

TALK "Would you like me to join your meeting? Share the room ID or say 'create new'."
HEAR user_input

IF user_input = "create new" THEN
    TALK "What should we call this meeting?"
    HEAR meeting_name
    
    room = START MEET meeting_name WITH BOT
    TALK "Meeting created! Share this link: " + room.url
    
    TALK "Who should I invite? (comma-separated emails, or 'skip')"
    HEAR invites
    
    IF invites <> "skip" THEN
        participants = SPLIT(invites, ",")
        INVITE TO MEET room, participants
        TALK "Invitations sent!"
    END IF
ELSE
    room_id = user_input
    JOIN MEET room_id AS "AI Assistant"
    TALK "I've joined the meeting!"
END IF

' Initialize notes
notes = "# Meeting Notes\n\n"
notes = notes + "**Date:** " + FORMAT(NOW(), "YYYY-MM-DD HH:mm") + "\n\n"
notes = notes + "## Key Points\n\n"

TALK TO MEET "Hello! I'm your AI assistant. Say 'note that' to capture important points, or 'summarize' when you're done."

' Meeting loop
meeting_active = true

WHILE meeting_active
    HEAR FROM MEET INTO message TIMEOUT 300
    
    IF message = "" THEN
        ' Timeout - check if meeting still active
        CONTINUE
    END IF
    
    ' Process commands
    IF INSTR(LOWER(message), "note that") > 0 THEN
        note_content = REPLACE(LOWER(message), "note that", "")
        notes = notes + "- " + TRIM(note_content) + "\n"
        TALK TO MEET "Noted!" AS VOICE
        
    ELSE IF INSTR(LOWER(message), "action item") > 0 THEN
        action = REPLACE(LOWER(message), "action item", "")
        notes = notes + "- **ACTION:** " + TRIM(action) + "\n"
        TALK TO MEET "Action item recorded!" AS VOICE
        
    ELSE IF INSTR(LOWER(message), "summarize") > 0 THEN
        ' Generate AI summary
        summary = LLM "Summarize these meeting notes concisely:\n\n" + notes
        TALK TO MEET "Here's the summary: " + summary
        
    ELSE IF INSTR(LOWER(message), "end meeting") > 0 THEN
        meeting_active = false
    END IF
WEND

' Save and share notes
filename = "meeting-notes-" + FORMAT(NOW(), "YYYYMMDD-HHmm") + ".md"
SAVE notes TO filename

TALK TO MEET "Meeting ended. I'll send the notes to all participants."
LEAVE MEET

' Email notes to participants
SEND MAIL participants, "Meeting Notes: " + meeting_name, notes
TALK "Notes saved and sent to all participants!"
```

---

## Example: Quick Standup Bot

```basic
' Daily Standup Bot
room = START MEET "Daily Standup" WITH BOT

team = ["dev1@company.com", "dev2@company.com", "dev3@company.com"]
INVITE TO MEET room, team, "Time for standup! Join now."

TALK TO MEET "Good morning team! Let's do a quick round. I'll call on each person."

updates = ""

FOR EACH member IN team
    TALK TO MEET member + ", what did you work on yesterday and what's planned for today?"
    HEAR FROM MEET INTO update TIMEOUT 120
    updates = updates + "**" + member + ":** " + update + "\n\n"
NEXT

TALK TO MEET "Great standup everyone! I'll post the summary to Slack."

' Post to Slack
POST "https://slack.com/api/chat.postMessage" WITH
    channel = "#dev-standup",
    text = "📋 **Standup Summary**\n\n" + updates

LEAVE MEET
```

---

## Configuration

Configure Meet integration in `config.csv`:

```csv
name,value
meet-provider,livekit
meet-server-url,wss://localhost:7880
meet-api-key,vault:gbo/meet/api_key
meet-api-secret,vault:gbo/meet/api_secret
meet-bot-default-persona,AI Assistant
meet-recording-enabled,true
meet-transcription-enabled,true
meet-max-participants,50
```

---

## Bot Capabilities in Meetings

When a bot joins a meeting, it can:

| Capability | Description |
|------------|-------------|
| **Listen** | Transcribe speech from participants |
| **Speak** | Text-to-speech announcements |
| **Chat** | Send text messages to meeting chat |
| **Record** | Capture meeting recording |
| **Screen Share** | Display content (dashboards, docs) |
| **React** | Send emoji reactions |

---

## See Also

- [Meet App](../07-user-interface/apps/meet.md) - User interface for Meet
- [BOOK_MEETING](./keyword-book.md) - Schedule meetings with calendar integration
- [Calls API](../08-rest-api-tools/calls-api.md) - API reference for video calls
- [Multi-Agent Keywords](./keywords-multi-agent.md) - Bot collaboration features