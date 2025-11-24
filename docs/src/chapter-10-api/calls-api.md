# Calls API

The Calls API provides endpoints for managing voice and video calls, conference rooms, and real-time communication within BotServer.

## Status

**⚠️ NOT IMPLEMENTED**

This API is planned for future development but is not currently available in BotServer.

## Planned Features

The Calls API will enable:
- Voice call initiation and management
- Video conferencing
- Screen sharing
- Call recording
- Call transcription
- Conference room management
- WebRTC integration

## Planned Endpoints

### Call Management
- `POST /api/v1/calls/initiate` - Start a call
- `GET /api/v1/calls/{call_id}` - Get call details
- `POST /api/v1/calls/{call_id}/end` - End a call
- `GET /api/v1/calls/history` - Call history

### Conference Rooms
- `POST /api/v1/calls/rooms` - Create conference room
- `GET /api/v1/calls/rooms/{room_id}` - Get room details
- `POST /api/v1/calls/rooms/{room_id}/join` - Join room
- `POST /api/v1/calls/rooms/{room_id}/leave` - Leave room
- `GET /api/v1/calls/rooms/{room_id}/participants` - List participants

### Recording
- `POST /api/v1/calls/{call_id}/record/start` - Start recording
- `POST /api/v1/calls/{call_id}/record/stop` - Stop recording
- `GET /api/v1/calls/{call_id}/recordings` - Get recordings

### Transcription
- `POST /api/v1/calls/{call_id}/transcribe` - Enable transcription
- `GET /api/v1/calls/{call_id}/transcript` - Get transcript

## Planned Integration with BASIC

When implemented, call features will be accessible via BASIC keywords:

```basic
' Initiate call (not yet available)
call_id = START CALL "user123"
WAIT FOR CALL ANSWER call_id

' Conference room (not yet available)
room_id = CREATE ROOM "Team Meeting"
INVITE TO ROOM room_id, ["user1", "user2", "user3"]

' Call with bot (not yet available)
ON INCOMING CALL
    ANSWER CALL
    TALK "Hello, how can I help you?"
    response = HEAR
    ' Process voice response
END ON
```

## Planned Data Models

### Call
```json
{
  "call_id": "call_123",
  "type": "video",
  "status": "active",
  "participants": [
    {
      "user_id": "user123",
      "role": "host",
      "audio": true,
      "video": true,
      "joined_at": "2024-01-15T10:00:00Z"
    },
    {
      "user_id": "user456",
      "role": "participant",
      "audio": true,
      "video": false,
      "joined_at": "2024-01-15T10:01:00Z"
    }
  ],
  "started_at": "2024-01-15T10:00:00Z",
  "duration_seconds": 300,
  "recording": false,
  "transcription": true
}
```

### Conference Room
```json
{
  "room_id": "room_456",
  "name": "Daily Standup",
  "type": "persistent",
  "max_participants": 10,
  "settings": {
    "allow_recording": true,
    "auto_transcribe": true,
    "waiting_room": false,
    "require_password": false
  },
  "current_participants": 3,
  "created_at": "2024-01-01T08:00:00Z"
}
```

## Planned Features Detail

### Call Types
- **One-to-One Calls**: Direct calls between two users
- **Group Calls**: Multi-party calls
- **Conference Calls**: Scheduled meetings with rooms
- **Bot Calls**: Voice interaction with bot

### Media Features
- Audio only
- Video with audio
- Screen sharing
- File sharing during calls
- Virtual backgrounds
- Noise suppression

### Recording Options
- Audio only recording
- Video recording
- Selective recording (specific participants)
- Cloud storage integration
- Automatic transcription

### Quality Management
- Adaptive bitrate
- Network quality indicators
- Bandwidth optimization
- Echo cancellation
- Automatic gain control

## Implementation Considerations

When implemented, the Calls API will:

1. **Use WebRTC** for peer-to-peer communication
2. **Integrate with SFU** for scalable group calls
3. **Support TURN/STUN** servers for NAT traversal
4. **Provide end-to-end encryption** for security
5. **Include call analytics** and quality metrics
6. **Support dial-in** via PSTN integration
7. **Enable virtual phone numbers** for bot calling

## Alternative Solutions

Until the Calls API is implemented, consider:

1. **External Services Integration**
   - Integrate with Twilio Voice API
   - Use Zoom SDK
   - Connect to Microsoft Teams
   - Embed Jitsi Meet

2. **WebRTC Libraries**
   ```javascript
   // Use existing WebRTC libraries in frontend
   const peer = new RTCPeerConnection(config);
   // Handle signaling through WebSocket
   ```

3. **Voice Bot Integration**
   - Use external telephony providers
   - Connect via SIP trunk
   - Integrate with cloud PBX systems

## Future Technology Stack

The planned implementation will use:
- **WebRTC** - Real-time communication
- **MediaSoup** or **Janus** - SFU server
- **Coturn** - TURN/STUN server
- **FFmpeg** - Media processing
- **Whisper** - Speech-to-text
- **PostgreSQL** - Call metadata storage
- **S3** - Recording storage

## Workaround Example

Until the Calls API is available, you can implement basic voice interaction:

```basic
' Simple voice bot using external service
FUNCTION HandlePhoneCall(phone_number)
    ' Use external telephony API
    response = CALL EXTERNAL API "twilio", {
        "action": "answer",
        "from": phone_number
    }
    
    ' Convert speech to text
    text = SPEECH TO TEXT response.audio
    
    ' Set the transcribed text as context
    SET CONTEXT "user_question", text
    
    ' System AI responds naturally
    TALK "Let me help you with that question."
    
    ' Convert text to speech
    audio = TEXT TO SPEECH bot_response
    
    ' Send response
    CALL EXTERNAL API "twilio", {
        "action": "play",
        "audio": audio
    }
END FUNCTION
```

## Integration Points

When available, the Calls API will integrate with:
- [Calendar API](./calendar-api.md) - Schedule calls
- [Notifications API](./notifications-api.md) - Call alerts
- [User API](./user-security.md) - User presence
- [Storage API](./storage-api.md) - Recording storage
- [ML API](./ml-api.md) - Transcription and analysis

## Use Cases

### Customer Support
- Voice-enabled bot support
- Call center integration
- Screen sharing for technical support
- Call recording for quality assurance

### Team Collaboration
- Video meetings
- Stand-up calls
- Screen sharing for presentations
- Persistent team rooms

### Education
- Virtual classrooms
- One-on-one tutoring
- Recorded lectures
- Interactive sessions

## Status Updates

Check the [GitHub repository](https://github.com/generalbots/botserver) for updates on Calls API implementation status.

For immediate voice/video needs, consider integrating with established providers like Twilio, Zoom, or Teams rather than waiting for the native implementation.