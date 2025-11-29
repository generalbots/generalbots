# Chapter 04: User Interface

## Overview

BotServer provides two interface options designed for different use cases:

- **Suite Interface** (`ui/suite/`) - Full productivity workspace with integrated apps
- **Minimal Interface** (`ui/minimal/`) - Simple chat-only interface

## Suite Interface

The suite interface is a complete workspace that brings together all your communication and productivity tools in one place.

### What You Get

When you open the suite interface, you have immediate access to:

- **Chat** - Talk with your AI assistant
- **Drive** - Store and manage your files  
- **Mail** - Send and receive emails
- **Meet** - Start video calls
- **Tasks** - Manage your to-do lists

### How to Use It

1. **Starting a Conversation**
   - Click the Chat icon or press Alt+1
   - Type your message in the input box
   - Press Enter to send
   - The bot responds instantly

2. **Managing Files**
   - Click Drive or press Alt+2
   - Upload files by dragging them to the window
   - Double-click files to preview
   - Share files directly in chat

3. **Email Integration**
   - Click Mail or press Alt+3
   - Connect your email accounts
   - Compose emails with AI assistance
   - Manage multiple inboxes

4. **Video Meetings**
   - Click Meet or press Alt+4
   - Start instant meetings
   - Share screen during calls
   - Record important sessions

5. **Task Management**
   - Click Tasks or press Alt+5
   - Create tasks from chat conversations
   - Set due dates and priorities
   - Track progress visually

### Keyboard Shortcuts

- `Alt+1` - Open Chat
- `Alt+2` - Open Drive
- `Alt+3` - Open Mail
- `Alt+4` - Open Meet
- `Alt+5` - Open Tasks
- `Esc` - Close current dialog
- `/` - Focus search box
- `Ctrl+Enter` - Send message with line break

### Customization

You can personalize your workspace:

- **Theme** - Click the moon/sun icon to switch between light and dark modes
- **Layout** - Resize panels by dragging borders
- **Notifications** - Configure alerts in settings

## Minimal Interface

The minimal interface provides a clean, distraction-free chat experience.

### What You Get

A single-page chat interface with:
- Clean message display
- Voice input support
- File attachments
- Markdown formatting
- Quick suggestions

### How to Use It

1. **Starting** 
   - Open your browser to the bot URL
   - The chat is ready immediately
   - No login required for basic use

2. **Chatting**
   - Type your message
   - Press Enter to send
   - View responses in real-time
   - Scroll up to see history

3. **Voice Input**
   - Click the microphone icon
   - Speak your message
   - Click again to stop
   - Message sends automatically

4. **File Sharing**
   - Click the paperclip icon
   - Select your file
   - File uploads and shares
   - Bot can read and discuss files

### Best For

The minimal interface is perfect for:
- Quick questions
- Mobile devices
- Embedded chat widgets
- Public kiosks
- Simple deployments

## Choosing Your Interface

### Use Suite When You Need:
- Full productivity features
- Multi-tasking capabilities
- File management
- Email integration
- Video meetings
- Task tracking
- Team collaboration

### Use Minimal When You Need:
- Simple chat access
- Mobile-friendly interface
- Quick responses
- Lightweight deployment
- Public access
- Embedded chat

## Mobile Experience

Both interfaces work on mobile devices:

### Suite on Mobile
- Responsive layout adapts to screen size
- Bottom navigation for easy thumb access
- Swipe between apps
- Touch-optimized controls

### Minimal on Mobile
- Full-screen chat experience
- Large touch targets
- Voice input prominent
- Smooth scrolling

## Accessibility

Both interfaces support:
- Keyboard navigation
- Screen readers
- High contrast modes
- Font size adjustment
- Focus indicators
- ARIA labels

## Browser Support

Works in all modern browsers:
- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Mobile browsers

## Getting Started

### First Time Setup

1. **Open the Interface**
   - Suite: `http://your-server:8080`
   - Minimal: `http://your-server:8080/minimal`

2. **Start Chatting**
   - No configuration needed
   - Just type and press Enter
   - The bot responds immediately

3. **Explore Features** (Suite only)
   - Click app icons to explore
   - Try keyboard shortcuts
   - Customize your theme

### Daily Use

**Morning Routine**
1. Open your workspace
2. Check messages in Chat
3. Review emails in Mail
4. Check tasks for the day

**Throughout the Day**
- Ask questions in Chat
- Upload documents to Drive
- Schedule meetings in Meet
- Update task progress

**End of Day**
- Review completed tasks
- Archive important emails
- Save chat conversations

## Tips and Tricks

### Chat Tips
- Use `/` commands for quick actions
- Drag files directly to chat
- Double-click messages to copy
- Use markdown for formatting

### Productivity Tips
- Pin important conversations
- Create task templates
- Set up email filters
- Use keyboard shortcuts

### Organization Tips
- Tag conversations for easy finding
- Create folders in Drive
- Use labels in Mail
- Color-code tasks

## Troubleshooting

### Common Issues

**Chat not responding**
- Refresh the page
- Check internet connection
- Clear browser cache

**Files won't upload**
- Check file size (max 100MB)
- Verify file type is supported
- Ensure sufficient storage

**Video not working**
- Allow camera/microphone permissions
- Check device settings
- Try different browser

## See Also

- [Chapter 1: Getting Started](../chapter-01/README.md) - Initial setup
- [Chapter 2: Packages](../chapter-02/README.md) - Understanding bot packages
- [Chapter 5: Themes](../chapter-05-gbtheme/README.md) - Customizing appearance
- [Chapter 6: Dialogs](../chapter-06-gbdialog/README.md) - Bot conversations