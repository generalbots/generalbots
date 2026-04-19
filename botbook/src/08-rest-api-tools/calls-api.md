# Calls API

The Calls API provides endpoints for managing voice and video calls, conference rooms, and real-time communication within botserver.

## Status

**⚠️ NOT IMPLEMENTED**

This API is planned for future development but is not currently available in botserver.

## Planned Features

The Calls API will enable voice call initiation and management, video conferencing, screen sharing, call recording, call transcription, conference room management, and WebRTC integration.

## Planned Endpoints

### Call Management

The call management endpoints will handle the lifecycle of individual calls. Use `POST /api/v1/calls/initiate` to start a call, `GET /api/v1/calls/{call_id}` to retrieve call details, `POST /api/v1/calls/{call_id}/end` to terminate a call, and `GET /api/v1/calls/history` to access call history.

### Conference Rooms

Conference room endpoints manage persistent meeting spaces. Create rooms with `POST /api/v1/calls/rooms`, retrieve room details with `GET /api/v1/calls/rooms/{room_id}`, and manage participation through `POST /api/v1/calls/rooms/{room_id}/join`, `POST /api/v1/calls/rooms/{room_id}/leave`, and `GET /api/v1/calls/rooms/{room_id}/participants`.

### Recording

Recording endpoints control call archival. Start recording with `POST /api/v1/calls/{call_id}/record/start`, stop with `POST /api/v1/calls/{call_id}/record/stop`, and retrieve recordings via `GET /api/v1/calls/{call_id}/recordings`.

### Transcription

Transcription endpoints provide speech-to-text capabilities. Enable transcription with `POST /api/v1/calls/{call_id}/transcribe` and retrieve the transcript using `GET /api/v1/calls/{call_id}/transcript`.

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

The API will support several call types to accommodate different communication needs. One-to-one calls enable direct communication between two users. Group calls allow multi-party conversations with several participants. Conference calls provide scheduled meetings with dedicated rooms. Bot calls enable voice interaction directly with the bot for automated customer service scenarios.

### Media Features

Media capabilities will include audio-only calls, video with audio, and screen sharing for presentations and collaboration. File sharing during calls will allow participants to exchange documents in real-time. Virtual backgrounds will provide privacy and professionalism, while noise suppression will ensure clear audio quality.

### Recording Options

Recording functionality will offer flexibility in how calls are archived. Audio-only recording will minimize storage requirements when video isn't needed. Full video recording will capture the complete visual experience. Selective recording will allow capturing specific participants only. Cloud storage integration will enable automatic upload to configured storage providers. Automatic transcription will convert recorded speech to searchable text.

### Quality Management

Quality features will ensure reliable communication across varying network conditions. Adaptive bitrate will automatically adjust video quality based on available bandwidth. Network quality indicators will inform participants of connection status. Bandwidth optimization will minimize data usage while maintaining quality. Echo cancellation and automatic gain control will ensure clear audio.

## Implementation Considerations

When implemented, the Calls API will use WebRTC for peer-to-peer communication, providing low-latency audio and video. Integration with an SFU (Selective Forwarding Unit) will enable scalable group calls without requiring each participant to send their stream to every other participant. Support for TURN/STUN servers will handle NAT traversal, ensuring connections work across different network configurations. End-to-end encryption will provide security for sensitive conversations. Call analytics and quality metrics will help administrators monitor system health. Dial-in via PSTN integration will allow traditional phone participation. Virtual phone numbers will enable bots to make and receive external calls.

## Alternative Solutions

Until the Calls API is implemented, consider these alternatives for voice and video functionality.

### External Services Integration

You can integrate with established communication platforms through their APIs. Twilio Voice API provides comprehensive telephony features. Zoom SDK enables embedding video meetings. Microsoft Teams integration connects to enterprise communication. Jitsi Meet offers an open-source video conferencing option that can be self-hosted.

### WebRTC Libraries

For custom implementations, you can use existing WebRTC libraries in your frontend:

```javascript
// Use existing WebRTC libraries in frontend
const peer = new RTCPeerConnection(config);
// Handle signaling through WebSocket
```

### Voice Bot Integration

For voice-enabled bots specifically, consider using external telephony providers, connecting via SIP trunk to existing phone systems, or integrating with cloud PBX systems that handle the voice infrastructure.

## Future Technology Stack

The planned implementation will use WebRTC for real-time communication, providing the foundation for peer-to-peer audio and video. MediaSoup or Janus will serve as the SFU server for scalable multi-party calls. Coturn will provide TURN/STUN server functionality for NAT traversal. FFmpeg will handle media processing tasks like transcoding and recording. Whisper will power speech-to-text transcription. PostgreSQL will store call metadata and history. S3-compatible storage will house call recordings.

## Workaround Example

Until the Calls API is available, you can implement basic voice interaction using external services:

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

When available, the Calls API will integrate with the [Calendar API](./calendar-api.md) for scheduling calls, the [Notifications API](./notifications-api.md) for call alerts, the [User API](./user-security.md) for user presence information, the [Storage API](./storage-api.md) for recording storage, and the [ML API](./ml-api.md) for transcription and analysis.

## Use Cases

### Customer Support

Voice-enabled bot support can handle common customer inquiries automatically. Call center integration allows seamless handoff to human agents. Screen sharing enables technical support representatives to guide customers visually. Call recording provides quality assurance data for training and compliance.

### Team Collaboration

Video meetings bring distributed teams together for face-to-face communication. Stand-up calls facilitate daily team synchronization. Screen sharing supports presentations and collaborative work sessions. Persistent team rooms provide always-available meeting spaces.

### Education

Virtual classrooms enable remote learning at scale. One-on-one tutoring provides personalized instruction. Recorded lectures allow students to review material at their own pace. Interactive sessions engage students through real-time participation.

## Status Updates

Check the [GitHub repository](https://github.com/generalbots/botserver) for updates on Calls API implementation status.

For immediate voice and video needs, consider integrating with established providers like Twilio, Zoom, or Teams rather than waiting for the native implementation.