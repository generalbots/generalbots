# Meet - Video Calls

> **Your virtual meeting room**

![Meet Flow](../../assets/suite/meet-flow.svg)

---

## Overview

Meet is the video conferencing app in General Bots Suite. Host video calls, share your screen, collaborate in real-time, and let the AI take notes for you. Meet integrates seamlessly with Calendar so joining meetings is just one click away.

---

## Interface Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Meet                                          [🎤 Mute] [📹 Video] [×] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                                                                 │   │
│  │                                                                 │   │
│  │                         ┌───────────┐                           │   │
│  │                         │           │                           │   │
│  │                         │    📹     │                           │   │
│  │                         │   You     │                           │   │
│  │                         │           │                           │   │
│  │                         └───────────┘                           │   │
│  │                                                                 │   │
│  │               Waiting for others to join...                     │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
├───────────────────────────┬─────────────────────────────────────────────┤
│  PARTICIPANTS (1)         │                                             │
│  ─────────────────        │  ┌─────────────────────────────────────┐   │
│  ● You (Host)             │  │ Chat                                │   │
│                           │  ├─────────────────────────────────────┤   │
│                           │  │                                     │   │
│                           │  │ Meeting started at 10:00 AM         │   │
│                           │  │                                     │   │
│  ─────────────────        │  ├─────────────────────────────────────┤   │
│  [Invite People]          │  │ Type a message...            [Send] │   │
│                           │  └─────────────────────────────────────┘   │
├───────────────────────────┴─────────────────────────────────────────────┤
│  [🎤] [📹] [🖥️ Share] [✋ Raise Hand] [💬 Chat] [⚙️] [📕 End Call]       │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Features

### Starting a Meeting

**Method 1: Instant Meeting**

1. Open the **Meet** app from the apps menu
2. Click **Start Meeting**
3. Your meeting room is ready instantly
4. Share the link with participants

```
┌─────────────────────────────────────────┐
│  Start a Meeting                  [×]   │
├─────────────────────────────────────────┤
│                                         │
│  ┌─────────────────────────────────┐    │
│  │     🎥 Start Instant Meeting    │    │
│  └─────────────────────────────────┘    │
│                                         │
│  ─────────── or ───────────             │
│                                         │
│  Meeting Link:                          │
│  ┌─────────────────────────────────┐    │
│  │ https://meet.bot/abc-defg-hij   │    │
│  └─────────────────────────────────┘    │
│  [Copy Link] [Share via Email]          │
│                                         │
│  ─────────── or ───────────             │
│                                         │
│  Join with Code:                        │
│  ┌─────────────────────────────────┐    │
│  │ Enter meeting code...           │    │
│  └─────────────────────────────────┘    │
│  [Join Meeting]                         │
│                                         │
└─────────────────────────────────────────┘
```

**Method 2: From Calendar**

1. Click on a meeting event in Calendar
2. Click the **Join Meeting** button
3. You're connected automatically

**Method 3: Ask the Bot**

```
You: Start a video call
Bot: ✅ Meeting room created!
     
     🔗 Link: https://meet.bot/abc-defg-hij
     
     [Join Now] [Copy Link] [Send Invites]
```

---

### Joining a Meeting

**From a Link**

1. Click the meeting link you received
2. Allow camera and microphone access
3. Preview your video
4. Click **Join Now**

**From a Code**

1. Open the Meet app
2. Enter the meeting code (e.g., `abc-defg-hij`)
3. Click **Join Meeting**

```
┌─────────────────────────────────────────┐
│  Ready to Join?                   [×]   │
├─────────────────────────────────────────┤
│                                         │
│        ┌─────────────────────┐          │
│        │                     │          │
│        │        📹           │          │
│        │    Your Preview     │          │
│        │                     │          │
│        └─────────────────────┘          │
│                                         │
│        [🎤 On]     [📹 On]              │
│                                         │
│  Your Name:                             │
│  ┌─────────────────────────────────┐    │
│  │ John Smith                      │    │
│  └─────────────────────────────────┘    │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │         Join Now                │    │
│  └─────────────────────────────────┘    │
│                                         │
└─────────────────────────────────────────┘
```

---

### During the Meeting

#### Video Grid

When multiple people join, the video grid adjusts automatically:

**2 Participants**
```
┌──────────────────┬──────────────────┐
│                  │                  │
│       You        │      Sarah       │
│                  │                  │
└──────────────────┴──────────────────┘
```

**4 Participants**
```
┌──────────────────┬──────────────────┐
│       You        │      Sarah       │
├──────────────────┼──────────────────┤
│       John       │      Mike        │
└──────────────────┴──────────────────┘
```

**Speaker View** - Active speaker is shown large:
```
┌─────────────────────────────────────────┐
│                                         │
│              Active Speaker             │
│                 (Sarah)                 │
│                                         │
├─────────┬─────────┬─────────┬──────────┤
│   You   │  John   │  Mike   │  Lisa    │
└─────────┴─────────┴─────────┴──────────┘
```

---

#### Meeting Controls

| Button | Action | Shortcut |
|--------|--------|----------|
| 🎤 | Mute/Unmute microphone | `M` |
| 📹 | Turn camera on/off | `V` |
| 🖥️ | Share your screen | `S` |
| ✋ | Raise your hand | `H` |
| 💬 | Open/close chat | `C` |
| 👥 | Show participants | `P` |
| ⚙️ | Settings | `,` |
| 📕 | Leave/End meeting | `Ctrl+E` |

---

### Screen Sharing

Share your screen, a window, or a browser tab:

1. Click the **🖥️ Share** button
2. Choose what to share:

```
┌─────────────────────────────────────────────────────────────────┐
│  Share Your Screen                                        [×]   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │                 │  │                 │  │                 │ │
│  │   Entire        │  │   Window        │  │   Browser       │ │
│  │   Screen        │  │                 │  │   Tab           │ │
│  │                 │  │                 │  │                 │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│                                                                 │
│  ☑ Share computer audio                                        │
│                                                                 │
│  [Cancel]                                         [Share]       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

3. Select the screen, window, or tab
4. Click **Share**
5. Click **Stop Sharing** when done

💡 **Tip:** Share a specific window to keep other content private.

---

### In-Meeting Chat

Send messages to all participants without interrupting the speaker:

```
┌─────────────────────────────────────┐
│  Meeting Chat                 [×]   │
├─────────────────────────────────────┤
│                                     │
│  Sarah (10:05 AM)                   │
│  Can everyone see my screen?        │
│                                     │
│  John (10:05 AM)                    │
│  Yes, looks good!                   │
│                                     │
│  You (10:06 AM)                     │
│  Perfect, let's continue.           │
│                                     │
├─────────────────────────────────────┤
│  ┌───────────────────────┐ [Send]   │
│  │ Type a message...     │          │
│  └───────────────────────┘          │
└─────────────────────────────────────┘
```

**Chat Features:**
- Send messages to everyone
- Share links and files
- Use emoji reactions
- Chat history is saved

---

### AI Meeting Assistant ✨

Let the AI help during your meetings:

**Live Transcription**

```
┌─────────────────────────────────────────┐
│  📝 Live Transcript                     │
├─────────────────────────────────────────┤
│                                         │
│  Sarah (10:05:23)                       │
│  "Let's review the Q2 numbers first"    │
│                                         │
│  John (10:05:45)                        │
│  "Revenue is up 15% from last quarter"  │
│                                         │
│  Sarah (10:06:12)                       │
│  "Great progress on the new product"    │
│                                         │
│  ─── Transcribing... ───                │
│                                         │
└─────────────────────────────────────────┘
```

**Meeting Notes**

After the meeting, the AI generates:
- Summary of key points
- Action items with assignees
- Decisions made
- Follow-up tasks

```
You: Summarize today's meeting
Bot: 📋 Meeting Summary - Project Review
     Duration: 45 minutes
     Participants: You, Sarah, John, Mike
     
     📌 Key Points:
     • Q2 revenue up 15%
     • New product launch on track for June
     • Marketing budget approved
     
     ✅ Action Items:
     • Sarah: Finalize launch materials by May 20
     • John: Schedule customer interviews
     • You: Review pricing proposal
     
     📅 Next Meeting: May 22 at 10 AM
```

---

### Raise Hand

Get the speaker's attention without interrupting:

1. Click **✋ Raise Hand**
2. Your hand icon appears next to your name
3. Host sees a notification
4. Click again to lower your hand

```
┌───────────────────────────┐
│  PARTICIPANTS (4)         │
│  ─────────────────        │
│  ● Sarah (Host)           │
│  ● You ✋                  │  ◄── Hand raised
│  ● John                   │
│  ● Mike                   │
└───────────────────────────┘
```

---

### Meeting Settings

```
┌─────────────────────────────────────────┐
│  Meeting Settings                 [×]   │
├─────────────────────────────────────────┤
│                                         │
│  AUDIO                                  │
│  ─────                                  │
│  Microphone: [Built-in Mic        ▼]    │
│  Speaker:    [Built-in Speakers   ▼]    │
│  [🔊 Test Audio]                        │
│                                         │
│  VIDEO                                  │
│  ─────                                  │
│  Camera:     [Built-in Camera     ▼]    │
│  [Mirror my video]  ☑                   │
│  [HD video]         ☐                   │
│                                         │
│  PREFERENCES                            │
│  ──────────                             │
│  [Mute on join]            ☑            │
│  [Camera off on join]      ☐            │
│  [Enable AI transcription] ☑            │
│  [Background blur]         ☐            │
│                                         │
│  ┌─────────────┐  ┌─────────────┐       │
│  │    Save     │  │   Cancel    │       │
│  └─────────────┘  └─────────────┘       │
│                                         │
└─────────────────────────────────────────┘
```

---

### Virtual Backgrounds

Change your background during the meeting:

1. Click **⚙️ Settings** → **Background**
2. Choose an option:

| Option | Description |
|--------|-------------|
| None | Show your real background |
| Blur | Blur your background |
| Office | Virtual office background |
| Nature | Nature scene background |
| Custom | Upload your own image |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `M` | Mute/Unmute microphone |
| `V` | Turn camera on/off |
| `S` | Start/stop screen share |
| `H` | Raise/lower hand |
| `C` | Open/close chat |
| `P` | Show/hide participants |
| `F` | Toggle fullscreen |
| `Space` | Push to talk (while muted) |
| `Ctrl+D` | Leave meeting |
| `Ctrl+E` | End meeting (host only) |

---

## Tips & Tricks

### Before the Meeting

💡 **Test your audio and video** before important meetings

💡 **Use a headset** to avoid echo and improve audio quality

💡 **Check your lighting** - face a window or light source

💡 **Close unnecessary apps** to improve performance

### During the Meeting

💡 **Mute when not speaking** to reduce background noise

💡 **Use the chat** for links and details without interrupting

💡 **Pin important speakers** by clicking their video tile

💡 **Use reactions** (👍 👏 ❤️) for quick feedback

### For Hosts

💡 **Start recording early** so you don't forget

💡 **Admit participants** from the waiting room promptly

💡 **Mute disruptive participants** if needed

💡 **Share meeting notes** after the call

---

## Troubleshooting

### No video showing

**Possible causes:**
1. Camera is blocked by another app
2. Browser doesn't have camera permission
3. Camera is turned off in settings

**Solution:**
1. Close other apps using the camera
2. Click the 🔒 icon in browser address bar → Allow Camera
3. Check that the camera is selected in Settings
4. Try refreshing the page

---

### No one can hear me

**Possible causes:**
1. Microphone is muted
2. Wrong microphone selected
3. Browser doesn't have microphone permission

**Solution:**
1. Click the 🎤 button to unmute
2. Go to Settings → Audio → Select correct microphone
3. Click 🔒 in browser → Allow Microphone
4. Test with "Test Audio" button in settings

---

### Can't hear others

**Possible causes:**
1. Speaker volume is too low
2. Wrong speaker device selected
3. System volume is muted

**Solution:**
1. Check system volume (not muted)
2. Go to Settings → Audio → Select correct speaker
3. Click "Test Audio" to verify
4. Try using headphones

---

### Screen share not working

**Possible causes:**
1. Browser doesn't have screen share permission
2. System privacy settings block screen recording
3. Another app is already sharing

**Solution:**
1. Allow screen share in browser permissions
2. On Mac: System Preferences → Security → Screen Recording → Allow browser
3. On Windows: Check privacy settings
4. Close any other screen sharing software

---

### Poor video quality

**Possible causes:**
1. Slow internet connection
2. Too many participants with video on
3. Computer is overloaded

**Solution:**
1. Close other apps and browser tabs
2. Turn off HD video in settings
3. Ask some participants to turn off video
4. Move closer to your WiFi router

---

## BASIC Integration

Control Meet from your bot dialogs:

### Create a Meeting

```basic
meeting = CREATE MEETING "Project Discussion"

TALK "Meeting created!"
TALK "Link: " + meeting.link
TALK "Code: " + meeting.code
```

### Schedule a Meeting

```basic
meeting = NEW OBJECT
meeting.title = "Weekly Standup"
meeting.date = "2025-05-20"
meeting.time = "10:00"
meeting.duration = 30
meeting.attendees = ["sarah@company.com", "john@company.com"]

result = SCHEDULE MEETING meeting
SEND INVITATION result
```

### Get Meeting Transcript

```basic
transcript = GET MEETING TRANSCRIPT meetingId

TALK "Meeting Summary:"
TALK transcript.summary

TALK "Action Items:"
FOR EACH item IN transcript.actionItems
    TALK "- " + item.task + " (" + item.assignee + ")"
NEXT
```

### Join a Meeting Programmatically

```basic
HEAR code AS TEXT "Enter the meeting code"

IF LEN(code) > 0 THEN
    JOIN MEETING code
    TALK "Joining meeting..."
ELSE
    TALK "No code provided"
END IF
```

---

## See Also

- [Calendar App](./calendar.md) - Schedule meetings
- [Chat App](./chat.md) - Continue conversations after meetings
- [Mail App](./mail.md) - Send meeting invitations
- [How To: Create Your First Bot](../how-to/create-first-bot.md)