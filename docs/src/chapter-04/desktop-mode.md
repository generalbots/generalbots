# Desktop Mode & Mobile Apps

BotServer includes a complete desktop interface and mobile app support for rich conversational experiences beyond simple chat.

## Overview

Desktop mode (`--desktop`) transforms BotServer into a full-featured workspace with integrated tools for communication, collaboration, and productivity.

## Launching Desktop Mode

```bash
# Start BotServer in desktop mode
./botserver --desktop

# With custom port
./botserver --desktop --port 8080

# Mobile-optimized interface
./botserver --mobile
```

## Desktop Components

### Chat Interface (`/chat`)
The main conversational interface with enhanced features:
- Multi-session support
- File attachments and sharing
- Rich media rendering
- Conversation history
- Quick actions and suggestions
- Voice input/output
- Screen sharing capabilities

### Attendant (`/attendant`)
AI-powered personal assistant features:
- Calendar integration
- Task management
- Reminders and notifications
- Meeting scheduling
- Contact management
- Email summaries
- Daily briefings

### Drive Integration (`/drive`)
File management and storage interface:
- Browse object storage buckets
- Upload/download files
- Share documents with chat
- Preview documents
- Organize bot resources
- Version control for files
- Collaborative editing support

### Mail Client (`/mail`)
Integrated email functionality:
- Send/receive emails through bots
- AI-powered email composition
- Smart inbox filtering
- Email-to-task conversion
- Automated responses
- Template management
- Thread summarization

### Meeting Room (`/meet`)
Video conferencing and collaboration:
- WebRTC-based video calls
- Screen sharing
- Recording capabilities
- AI meeting notes
- Real-time transcription
- Meeting bot integration
- Calendar sync

### Task Management (`/tasks`)
Project and task tracking:
- Kanban boards
- Sprint planning
- Time tracking
- Bot automation for tasks
- Progress reporting
- Team collaboration
- Integration with chat

### Account Settings (`/account.html`)
User profile and preferences:
- Profile management
- Authentication settings
- API keys management
- Subscription details
- Usage statistics
- Privacy controls
- Data export

### System Settings (`/settings.html`)
Application configuration:
- Theme customization
- Language preferences
- Notification settings
- Bot configurations
- Integration settings
- Performance tuning
- Debug options

## Mobile Application

### Progressive Web App (PWA)
BotServer desktop mode works as a PWA:
- Install on mobile devices
- Offline capabilities
- Push notifications
- Native app experience
- Responsive design
- Touch-optimized UI

### Mobile Features
- Swipe gestures for navigation
- Voice-first interaction
- Location sharing
- Camera integration
- Contact integration
- Mobile-optimized layouts
- Reduced data usage mode

### Installation on Mobile

#### Android
1. Open BotServer URL in Chrome
2. Tap "Add to Home Screen"
3. Accept installation prompt
4. Launch from home screen

#### iOS
1. Open in Safari
2. Tap Share button
3. Select "Add to Home Screen"
4. Name the app and add

## Desktop Interface Structure

```
web/desktop/
├── index.html          # Main desktop dashboard
├── account.html        # User account management
├── settings.html       # Application settings
├── chat/              # Chat interface components
├── attendant/         # AI assistant features
├── drive/             # File management
├── mail/              # Email client
├── meet/              # Video conferencing
├── tasks/             # Task management
├── css/               # Stylesheets
├── js/                # JavaScript modules
└── public/            # Static assets
```

## Features by Screen

### Dashboard (index.html)
- Widget-based layout
- Quick access tiles
- Recent conversations
- Pending tasks
- Calendar view
- System notifications
- Bot status indicators

### Chat Screen
- Conversation list
- Message composer
- Rich text formatting
- Code syntax highlighting
- File attachments
- Emoji picker
- Message reactions
- Thread support

### Drive Screen
- File browser
- Folder navigation
- Upload queue
- Preview pane
- Sharing controls
- Storage metrics
- Search functionality

### Mail Screen
- Inbox/Sent/Drafts
- Message composer
- Rich HTML editor
- Attachment handling
- Contact autocomplete
- Filter and labels
- Bulk operations

## Responsive Design

### Breakpoints
```css
/* Mobile: < 768px */
/* Tablet: 768px - 1024px */
/* Desktop: > 1024px */
/* Wide: > 1440px */
```

### Adaptive Layouts
- Mobile: Single column, bottom navigation
- Tablet: Two-column with collapsible sidebar
- Desktop: Three-column with persistent panels
- Wide: Multi-panel with docked windows

## Theming

### CSS Variables
```css
:root {
  --primary-color: #0d2b55;
  --secondary-color: #fff9c2;
  --background: #ffffff;
  --text-color: #333333;
  --border-color: #e0e0e0;
}
```

### Dark Mode
Automatic dark mode based on:
- System preferences
- Time of day
- User selection
- Per-component overrides

## Performance

### Optimization Strategies
- Lazy loading of components
- Virtual scrolling for long lists
- Image optimization and CDN
- Code splitting by route
- Service worker caching
- WebAssembly for compute tasks

### Resource Management
- Maximum 50MB cache size
- Automatic cleanup of old data
- Compressed asset delivery
- Efficient WebSocket usage
- Battery-aware processing

## Security

### Authentication
- OAuth2/OIDC support
- Biometric authentication (mobile)
- Session management
- Secure token storage
- Auto-logout on inactivity

### Data Protection
- End-to-end encryption for sensitive data
- Local storage encryption
- Secure WebSocket connections
- Content Security Policy
- XSS protection

## Offline Capabilities

### Service Worker
- Cache-first strategy for assets
- Network-first for API calls
- Background sync for messages
- Offline message queue
- Automatic retry logic

### Local Storage
- IndexedDB for structured data
- localStorage for preferences
- Cache API for resources
- File system access (desktop)

## Integration APIs

### JavaScript SDK
```javascript
// Initialize desktop mode
const desktop = new BotDesktop({
  server: 'ws://localhost:8080',
  theme: 'auto',
  modules: ['chat', 'drive', 'mail']
});

// Subscribe to events
desktop.on('message', (msg) => {
  console.log('New message:', msg);
});

// Send commands
desktop.chat.send('Hello from desktop!');
```

## Debugging

### Developer Tools
- Console logging levels
- Network request inspector
- WebSocket frame viewer
- Performance profiler
- Memory leak detector
- Component tree inspector

### Debug Mode
```bash
# Enable debug mode
./botserver --desktop --debug

# Verbose logging
./botserver --desktop --verbose
```

## Deployment

### Web Server Configuration
```nginx
location /desktop {
  proxy_pass http://localhost:8080;
  proxy_http_version 1.1;
  proxy_set_header Upgrade $http_upgrade;
  proxy_set_header Connection "upgrade";
}
```

### CDN Setup
- Static assets on CDN
- Dynamic content from server
- Geographic distribution
- Cache invalidation strategy

## Troubleshooting

### Common Issues

1. **Blank screen on load**
   - Check JavaScript console
   - Verify WebSocket connection
   - Clear browser cache

2. **Slow performance**
   - Reduce active modules
   - Clear local storage
   - Check network latency

3. **PWA not installing**
   - Ensure HTTPS connection
   - Valid manifest.json
   - Service worker registered

## Summary

Desktop mode transforms BotServer from a simple chatbot platform into a comprehensive AI-powered workspace. With mobile PWA support, users can access all features from any device while maintaining a consistent, responsive experience.