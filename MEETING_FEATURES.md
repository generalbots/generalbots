# Meeting and Multimedia Features Implementation

## Overview
This document describes the implementation of enhanced chat features, meeting services, and screen capture capabilities for the General Bots botserver application.

## Features Implemented

### 1. Enhanced Bot Module with Multimedia Support

#### Location: `src/bot/multimedia.rs`
- **Video Messages**: Support for sending and receiving video files with thumbnails
- **Image Messages**: Image sharing with caption support
- **Web Search**: Integrated web search capability with `/search` command
- **Document Sharing**: Support for various document formats
- **Meeting Invites**: Handling meeting invitations and redirects from Teams/WhatsApp

#### Key Components:
- `MultimediaMessage` enum for different message types
- `MultimediaHandler` trait for processing multimedia content
- `DefaultMultimediaHandler` implementation with S3 storage support
- Media upload/download functionality

### 2. Meeting Service Implementation

#### Location: `src/meet/service.rs`
- **Real-time Meeting Rooms**: Support for creating and joining video conference rooms
- **Live Transcription**: Real-time speech-to-text transcription during meetings
- **Bot Integration**: AI assistant that responds to voice commands and meeting context
- **WebSocket Communication**: Real-time messaging between participants
- **Recording Support**: Meeting recording capabilities

#### Key Features:
- Meeting room management with participant tracking
- WebSocket message types for various meeting events
- Transcription service integration
- Bot command processing ("Hey bot" wake word)
- Screen sharing support

### 3. Screen Capture with WebAPI

#### Implementation: Browser-native WebRTC
- **Screen Recording**: Full screen capture using MediaStream Recording API
- **Window Capture**: Capture specific application windows via browser selection
- **Region Selection**: Browser-provided selection interface
- **Screenshot**: Capture video frames from MediaStream
- **WebRTC Streaming**: Direct streaming to meetings via RTCPeerConnection

#### Browser API Usage:
```javascript
// Request screen capture
const stream = await navigator.mediaDevices.getDisplayMedia({
    video: {
        cursor: "always",
        displaySurface: "monitor" // or "window", "browser"
    },
    audio: true
});

// Add to meeting peer connection
stream.getTracks().forEach(track => {
    peerConnection.addTrack(track, stream);
});
```

#### Benefits:
- **Cross-platform**: Works on web, desktop, and mobile browsers
- **No native dependencies**: Pure JavaScript implementation
- **Browser security**: Built-in permission management
- **Standard API**: W3C MediaStream specification

### 4. Web Desktop Meet Component

#### Location: `web/desktop/meet/`
- **Full Meeting UI**: Complete video conferencing interface
- **Video Grid**: Dynamic participant video layout
- **Chat Panel**: In-meeting text chat
- **Transcription Panel**: Live transcription display
- **Bot Assistant Panel**: AI assistant interface
- **Participant Management**: View and manage meeting participants

#### Files:
- `meet.html`: Meeting room interface
- `meet.js`: WebRTC and meeting logic
- `meet.css`: Responsive styling

## Integration Points

### 1. WebSocket Message Types
```javascript
const MessageType = {
    JOIN_MEETING: 'join_meeting',
    LEAVE_MEETING: 'leave_meeting',
    TRANSCRIPTION: 'transcription',
    CHAT_MESSAGE: 'chat_message',
    BOT_MESSAGE: 'bot_message',
    SCREEN_SHARE: 'screen_share',
    STATUS_UPDATE: 'status_update',
    PARTICIPANT_UPDATE: 'participant_update',
    RECORDING_CONTROL: 'recording_control',
    BOT_REQUEST: 'bot_request'
};
```

### 2. API Endpoints
- `POST /api/meet/create` - Create new meeting room
- `POST /api/meet/token` - Get WebRTC connection token
- `POST /api/meet/invite` - Send meeting invitations
- `GET /ws/meet` - WebSocket connection for meeting

### 3. Bot Commands in Meetings
- **Summarize**: Generate meeting summary
- **Action Items**: Extract action items from discussion
- **Key Points**: Highlight important topics
- **Questions**: List pending questions

## Usage Examples

### Creating a Meeting
```javascript
const response = await fetch('/api/meet/create', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
        name: 'Team Standup',
        settings: {
            enable_transcription: true,
            enable_bot: true
        }
    })
});
```

### Sending Multimedia Message
```rust
let message = MultimediaMessage::Image {
    url: "https://example.com/image.jpg".to_string(),
    caption: Some("Check this out!".to_string()),
    mime_type: "image/jpeg".to_string(),
};
```

### Starting Screen Capture (WebAPI)
```javascript
// Request screen capture with options
const stream = await navigator.mediaDevices.getDisplayMedia({
    video: {
        cursor: "always",
        width: { ideal: 1920 },
        height: { ideal: 1080 },
        frameRate: { ideal: 30 }
    },
    audio: true
});

// Record or stream to meeting
const mediaRecorder = new MediaRecorder(stream, {
    mimeType: 'video/webm;codecs=vp9',
    videoBitsPerSecond: 2500000
});
mediaRecorder.start();
```

## Meeting Redirect Flow

### Handling Teams/WhatsApp Video Calls
1. External platform initiates video call
2. User receives redirect to botserver meeting
3. Redirect handler shows incoming call notification
4. Auto-accept or manual accept/reject
5. Join meeting room with guest credentials

### URL Format for Redirects
```
/meet?meeting=<meeting_id>&from=<platform>

Examples:
/meet?meeting=abc123&from=teams
/meet?meeting=xyz789&from=whatsapp
```

## Configuration

### Environment Variables
```bash
# Search API
SEARCH_API_KEY=your_search_api_key

# WebRTC Server (LiveKit)
LIVEKIT_URL=ws://localhost:7880
LIVEKIT_API_KEY=your_api_key
LIVEKIT_SECRET=your_secret

# Storage for media
DRIVE_SERVER=http://localhost:9000
DRIVE_ACCESSKEY=your_access_key
DRIVE_SECRET=your_secret
```

### Meeting Settings
```rust
pub struct MeetingSettings {
    pub enable_transcription: bool,  // Default: true
    pub enable_recording: bool,      // Default: false
    pub enable_chat: bool,           // Default: true
    pub enable_screen_share: bool,   // Default: true
    pub auto_admit: bool,            // Default: true
    pub waiting_room: bool,          // Default: false
    pub bot_enabled: bool,           // Default: true
    pub bot_id: Option<String>,      // Optional specific bot
}
```

## Security Considerations

1. **Authentication**: All meeting endpoints should verify user authentication
2. **Room Access**: Implement proper room access controls
3. **Recording Consent**: Get participant consent before recording
4. **Data Privacy**: Ensure transcriptions and recordings are properly secured
5. **WebRTC Security**: Use secure signaling and TURN servers

## Performance Optimization

1. **Video Quality**: Adaptive bitrate based on network conditions
2. **Lazy Loading**: Load panels and features on-demand
3. **WebSocket Batching**: Batch multiple messages when possible
4. **Transcription Buffer**: Buffer audio before sending to transcription service
5. **Media Compression**: Compress images/videos before upload

## Future Enhancements

1. **Virtual Backgrounds**: Add background blur/replacement
2. **Breakout Rooms**: Support for sub-meetings
3. **Whiteboard**: Collaborative drawing during meetings
4. **Meeting Analytics**: Track speaking time, participation
5. **Calendar Integration**: Schedule meetings with calendar apps
6. **Mobile Support**: Responsive design for mobile devices
7. **End-to-End Encryption**: Secure meeting content
8. **Custom Layouts**: User-defined video grid layouts
9. **Meeting Templates**: Pre-configured meeting types
10. **Integration APIs**: Webhooks for external integrations

## Testing

### Unit Tests
- Test multimedia message parsing
- Test meeting room creation/joining
- Test transcription processing
- Test bot command handling

### Integration Tests
- Test WebSocket message flow
- Test video call redirects
- Test screen capture with different configurations
- Test meeting recording and playback

### E2E Tests
- Complete meeting flow from creation to end
- Multi-participant interaction
- Screen sharing during meeting
- Bot interaction during meeting

## Deployment

1. Ensure LiveKit or WebRTC server is running
2. Configure S3 or storage for media files
3. Set up transcription service (if using external)
4. Deploy web assets to static server
5. Configure reverse proxy for WebSocket connections
6. Set up SSL certificates for production
7. Configure TURN/STUN servers for NAT traversal

## Troubleshooting

### Common Issues

1. **No Video/Audio**: Check browser permissions and device access
2. **Connection Failed**: Verify WebSocket URL and CORS settings
3. **Transcription Not Working**: Check transcription service credentials
4. **Screen Share Black**: May need elevated permissions on some OS
5. **Bot Not Responding**: Verify bot service is running and connected

### Debug Mode
Enable debug logging in the browser console:
```javascript
localStorage.setItem('debug', 'meet:*');
```

## Support

For issues or questions:
- Check logs in `./logs/meeting.log`
- Review WebSocket messages in browser DevTools
- Contact support with meeting ID and timestamp