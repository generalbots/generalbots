// Meet Application - Video Conferencing with Bot Integration
const meetApp = (function() {
    'use strict';

    // State management
    let state = {
        room: null,
        localTracks: [],
        participants: new Map(),
        isConnected: false,
        isMuted: false,
        isVideoOff: false,
        isScreenSharing: false,
        isRecording: false,
        isTranscribing: true,
        meetingId: null,
        meetingStartTime: null,
        ws: null,
        botEnabled: true,
        transcriptions: [],
        chatMessages: [],
        unreadCount: 0
    };

    // WebSocket message types
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

    // Initialize the application
    async function init() {
        console.log('Initializing meet application...');

        // Setup event listeners
        setupEventListeners();

        // Check for meeting ID in URL
        const urlParams = new URLSearchParams(window.location.search);
        const meetingIdFromUrl = urlParams.get('meeting');
        const redirectFrom = urlParams.get('from');

        if (redirectFrom) {
            handleRedirect(redirectFrom, meetingIdFromUrl);
        } else if (meetingIdFromUrl) {
            state.meetingId = meetingIdFromUrl;
            showJoinModal();
        } else {
            showCreateModal();
        }

        // Initialize WebSocket connection
        await connectWebSocket();

        // Start timer update
        startTimer();
    }

    // Setup event listeners
    function setupEventListeners() {
        // Control buttons
        document.getElementById('micBtn').addEventListener('click', toggleMicrophone);
        document.getElementById('videoBtn').addEventListener('click', toggleVideo);
        document.getElementById('screenShareBtn').addEventListener('click', toggleScreenShare);
        document.getElementById('leaveBtn').addEventListener('click', leaveMeeting);

        // Top controls
        document.getElementById('recordBtn').addEventListener('click', toggleRecording);
        document.getElementById('transcribeBtn').addEventListener('click', toggleTranscription);
        document.getElementById('participantsBtn').addEventListener('click', () => togglePanel('participants'));
        document.getElementById('chatBtn').addEventListener('click', () => togglePanel('chat'));
        document.getElementById('botBtn').addEventListener('click', () => togglePanel('bot'));

        // Modal buttons
        document.getElementById('joinMeetingBtn').addEventListener('click', joinMeeting);
        document.getElementById('createMeetingBtn').addEventListener('click', createMeeting);
        document.getElementById('sendInvitesBtn').addEventListener('click', sendInvites);

        // Chat
        document.getElementById('chatInput').addEventListener('keypress', (e) => {
            if (e.key === 'Enter') sendChatMessage();
        });
        document.getElementById('sendChatBtn').addEventListener('click', sendChatMessage);

        // Bot commands
        document.querySelectorAll('.bot-cmd-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const command = e.currentTarget.dataset.command;
                sendBotCommand(command);
            });
        });

        // Transcription controls
        document.getElementById('downloadTranscriptBtn').addEventListener('click', downloadTranscript);
        document.getElementById('clearTranscriptBtn').addEventListener('click', clearTranscript);
    }

    // WebSocket connection
    async function connectWebSocket() {
        return new Promise((resolve, reject) => {
            const wsUrl = `ws://localhost:8080/ws/meet`;
            state.ws = new WebSocket(wsUrl);

            state.ws.onopen = () => {
                console.log('WebSocket connected');
                resolve();
            };

            state.ws.onmessage = (event) => {
                handleWebSocketMessage(JSON.parse(event.data));
            };

            state.ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                reject(error);
            };

            state.ws.onclose = () => {
                console.log('WebSocket disconnected');
                // Attempt reconnection
                setTimeout(connectWebSocket, 5000);
            };
        });
    }

    // Handle WebSocket messages
    function handleWebSocketMessage(message) {
        console.log('Received message:', message.type);

        switch (message.type) {
            case MessageType.TRANSCRIPTION:
                handleTranscription(message);
                break;
            case MessageType.CHAT_MESSAGE:
                handleChatMessage(message);
                break;
            case MessageType.BOT_MESSAGE:
                handleBotMessage(message);
                break;
            case MessageType.PARTICIPANT_UPDATE:
                handleParticipantUpdate(message);
                break;
            case MessageType.STATUS_UPDATE:
                handleStatusUpdate(message);
                break;
            default:
                console.log('Unknown message type:', message.type);
        }
    }

    // Send WebSocket message
    function sendMessage(message) {
        if (state.ws && state.ws.readyState === WebSocket.OPEN) {
            state.ws.send(JSON.stringify(message));
        }
    }

    // Meeting controls
    async function createMeeting() {
        const name = document.getElementById('meetingName').value;
        const description = document.getElementById('meetingDescription').value;
        const settings = {
            enable_transcription: document.getElementById('enableTranscription').checked,
            enable_recording: document.getElementById('enableRecording').checked,
            enable_bot: document.getElementById('enableBot').checked,
            waiting_room: document.getElementById('enableWaitingRoom').checked
        };

        try {
            const response = await fetch('/api/meet/create', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name, description, settings })
            });

            const data = await response.json();
            state.meetingId = data.id;

            closeModal('createModal');
            await joinMeetingRoom(data.id, 'Host');

            // Show invite modal
            setTimeout(() => showInviteModal(), 1000);
        } catch (error) {
            console.error('Failed to create meeting:', error);
            alert('Failed to create meeting. Please try again.');
        }
    }

    async function joinMeeting() {
        const userName = document.getElementById('userName').value;
        const meetingCode = document.getElementById('meetingCode').value;

        if (!userName || !meetingCode) {
            alert('Please enter your name and meeting code');
            return;
        }

        closeModal('joinModal');
        await joinMeetingRoom(meetingCode, userName);
    }

    async function joinMeetingRoom(roomId, userName) {
        state.meetingId = roomId;
        state.meetingStartTime = Date.now();

        // Update UI
        document.getElementById('meetingId').textContent = `Meeting ID: ${roomId}`;
        document.getElementById('meetingTitle').textContent = userName + "'s Meeting";

        // Initialize WebRTC
        await initializeWebRTC(roomId, userName);

        // Send join message
        sendMessage({
            type: MessageType.JOIN_MEETING,
            room_id: roomId,
            participant_name: userName
        });

        state.isConnected = true;
    }

    async function leaveMeeting() {
        if (!confirm('Are you sure you want to leave the meeting?')) return;

        // Send leave message
        sendMessage({
            type: MessageType.LEAVE_MEETING,
            room_id: state.meetingId,
            participant_id: 'current-user'
        });

        // Clean up
        if (state.room) {
            state.room.disconnect();
        }

        state.localTracks.forEach(track => track.stop());
        state.localTracks = [];
        state.participants.clear();
        state.isConnected = false;

        // Redirect
        window.location.href = '/chat';
    }

    // WebRTC initialization
    async function initializeWebRTC(roomId, userName) {
        try {
            // For LiveKit integration
            if (window.LiveKitClient) {
                const room = new LiveKitClient.Room({
                    adaptiveStream: true,
                    dynacast: true,
                    videoCaptureDefaults: {
                        resolution: LiveKitClient.VideoPresets.h720.resolution
                    }
                });

                // Get token from server
                const response = await fetch('/api/meet/token', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ room_id: roomId, user_name: userName })
                });

                const { token } = await response.json();

                // Connect to room
                await room.connect('ws://localhost:7880', token);
                state.room = room;

                // Setup event handlers
                room.on('participantConnected', handleParticipantConnected);
                room.on('participantDisconnected', handleParticipantDisconnected);
                room.on('trackSubscribed', handleTrackSubscribed);
                room.on('trackUnsubscribed', handleTrackUnsubscribed);
                room.on('activeSpeakersChanged', handleActiveSpeakersChanged);

                // Publish local tracks
                await publishLocalTracks();
            } else {
                // Fallback to basic WebRTC
                await setupBasicWebRTC(roomId, userName);
            }
        } catch (error) {
            console.error('Failed to initialize WebRTC:', error);
            alert('Failed to connect to meeting. Please check your connection.');
        }
    }

    async function setupBasicWebRTC(roomId, userName) {
        // Get user media
        const stream = await navigator.mediaDevices.getUserMedia({
            video: true,
            audio: true
        });

        // Display local video
        const localVideo = document.getElementById('localVideo');
        localVideo.srcObject = stream;

        state.localTracks = stream.getTracks();
    }

    async function publishLocalTracks() {
        try {
            const tracks = await LiveKitClient.createLocalTracks({
                audio: true,
                video: true
            });

            for (const track of tracks) {
                await state.room.localParticipant.publishTrack(track);
                state.localTracks.push(track);

                if (track.kind === 'video') {
                    const localVideo = document.getElementById('localVideo');
                    track.attach(localVideo);
                }
            }
        } catch (error) {
            console.error('Failed to publish tracks:', error);
        }
    }

    // Media controls
    function toggleMicrophone() {
        state.isMuted = !state.isMuted;

        state.localTracks.forEach(track => {
            if (track.kind === 'audio') {
                track.enabled = !state.isMuted;
            }
        });

        const micBtn = document.getElementById('micBtn');
        micBtn.classList.toggle('muted', state.isMuted);
        micBtn.querySelector('.icon').textContent = state.isMuted ? '🔇' : '🎤';

        updateLocalIndicators();
    }

    function toggleVideo() {
        state.isVideoOff = !state.isVideoOff;

        state.localTracks.forEach(track => {
            if (track.kind === 'video') {
                track.enabled = !state.isVideoOff;
            }
        });

        const videoBtn = document.getElementById('videoBtn');
        videoBtn.classList.toggle('off', state.isVideoOff);
        videoBtn.querySelector('.icon').textContent = state.isVideoOff ? '📷' : '📹';

        updateLocalIndicators();
    }

    async function toggleScreenShare() {
        if (!state.isScreenSharing) {
            try {
                const stream = await navigator.mediaDevices.getDisplayMedia({
                    video: true,
                    audio: false
                });

                if (state.room) {
                    const screenTrack = stream.getVideoTracks()[0];
                    await state.room.localParticipant.publishTrack(screenTrack);

                    screenTrack.onended = () => {
                        stopScreenShare();
                    };
                }

                state.isScreenSharing = true;
                document.getElementById('screenShareBtn').classList.add('active');

                // Show screen share overlay
                const screenShareVideo = document.getElementById('screenShareVideo');
                screenShareVideo.srcObject = stream;
                document.getElementById('screenShareOverlay').classList.remove('hidden');

                // Send screen share status
                sendMessage({
                    type: MessageType.SCREEN_SHARE,
                    room_id: state.meetingId,
                    participant_id: 'current-user',
                    is_sharing: true,
                    share_type: 'screen'
                });
            } catch (error) {
                console.error('Failed to share screen:', error);
                alert('Failed to share screen. Please try again.');
            }
        } else {
            stopScreenShare();
        }
    }

    function stopScreenShare() {
        state.isScreenSharing = false;
        document.getElementById('screenShareBtn').classList.remove('active');
        document.getElementById('screenShareOverlay').classList.add('hidden');

        // Send screen share status
        sendMessage({
            type: MessageType.SCREEN_SHARE,
            room_id: state.meetingId,
            participant_id: 'current-user',
            is_sharing: false
        });
    }

    // Recording and transcription
    function toggleRecording() {
        state.isRecording = !state.isRecording;

        const recordBtn = document.getElementById('recordBtn');
        recordBtn.classList.toggle('recording', state.isRecording);

        sendMessage({
            type: MessageType.RECORDING_CONTROL,
            room_id: state.meetingId,
            action: state.isRecording ? 'start' : 'stop',
            participant_id: 'current-user'
        });

        if (state.isRecording) {
            showNotification('Recording started');
        } else {
            showNotification('Recording stopped');
        }
    }

    function toggleTranscription() {
        state.isTranscribing = !state.isTranscribing;

        const transcribeBtn = document.getElementById('transcribeBtn');
        transcribeBtn.classList.toggle('active', state.isTranscribing);

        if (state.isTranscribing) {
            showNotification('Transcription enabled');
        } else {
            showNotification('Transcription disabled');
        }
    }

    function handleTranscription(message) {
        if (!state.isTranscribing) return;

        const transcription = {
            participant_id: message.participant_id,
            text: message.text,
            timestamp: new Date(message.timestamp),
            is_final: message.is_final
        };

        if (message.is_final) {
            state.transcriptions.push(transcription);
            addTranscriptionToUI(transcription);

            // Check for bot wake words
            if (state.botEnabled && (
                message.text.toLowerCase().includes('hey bot') ||
                message.text.toLowerCase().includes('assistant')
            )) {
                processBotCommand(message.text, message.participant_id);
            }
        }
    }

    function addTranscriptionToUI(transcription) {
        const container = document.getElementById('transcriptionContainer');
        const entry = document.createElement('div');
        entry.className = 'transcription-entry';
        entry.innerHTML = `
            <div class="transcription-header">
                <span class="participant-name">Participant ${transcription.participant_id.substring(0, 8)}</span>
                <span class="timestamp">${transcription.timestamp.toLocaleTimeString()}</span>
            </div>
            <div class="transcription-text">${transcription.text}</div>
        `;
        container.appendChild(entry);
        container.scrollTop = container.scrollHeight;
    }

    // Chat functionality
    function sendChatMessage() {
        const input = document.getElementById('chatInput');
        const content = input.value.trim();

        if (!content) return;

        const message = {
            type: MessageType.CHAT_MESSAGE,
            room_id: state.meetingId,
            participant_id: 'current-user',
            content: content,
            timestamp: new Date().toISOString()
        };

        sendMessage(message);

        // Add to local chat
        addChatMessage({
            ...message,
            is_self: true
        });

        input.value = '';
    }

    function handleChatMessage(message) {
        addChatMessage({
            ...message,
            is_self: false
        });

        // Update unread count if chat panel is hidden
        const chatPanel = document.getElementById('chatPanel');
        if (chatPanel.style.display === 'none') {
            state.unreadCount++;
            updateUnreadBadge();
        }
    }

    function addChatMessage(message) {
        state.chatMessages.push(message);

        const container = document.getElementById('chatMessages');
        const messageEl = document.createElement('div');
        messageEl.className = `chat-message ${message.is_self ? 'self' : ''}`;
        messageEl.innerHTML = `
            <div class="message-header">
                <span class="sender-name">${message.is_self ? 'You' : 'Participant'}</span>
                <span class="message-time">${new Date(message.timestamp).toLocaleTimeString()}</span>
            </div>
            <div class="message-content">${message.content}</div>
        `;
        container.appendChild(messageEl);
        container.scrollTop = container.scrollHeight;
    }

    // Bot integration
    function sendBotCommand(command) {
        const message = {
            type: MessageType.BOT_REQUEST,
            room_id: state.meetingId,
            participant_id: 'current-user',
            command: command,
            parameters: {}
        };

        sendMessage(message);

        // Show loading in bot responses
        const responsesContainer = document.getElementById('botResponses');
        const loadingEl = document.createElement('div');
        loadingEl.className = 'bot-response loading';
        loadingEl.innerHTML = '<span class="loading-dots">Processing...</span>';
        responsesContainer.appendChild(loadingEl);
    }

    function handleBotMessage(message) {
        const responsesContainer = document.getElementById('botResponses');

        // Remove loading indicator
        const loadingEl = responsesContainer.querySelector('.loading');
        if (loadingEl) loadingEl.remove();

        // Add bot response
        const responseEl = document.createElement('div');
        responseEl.className = 'bot-response';
        responseEl.innerHTML = `
            <div class="response-header">
                <span class="bot-icon">🤖</span>
                <span class="response-time">${new Date().toLocaleTimeString()}</span>
            </div>
            <div class="response-content">${marked.parse(message.content)}</div>
        `;
        responsesContainer.appendChild(responseEl);
        responsesContainer.scrollTop = responsesContainer.scrollHeight;
    }

    function processBotCommand(text, participantId) {
        // Process voice command with bot
        sendMessage({
            type: MessageType.BOT_REQUEST,
            room_id: state.meetingId,
            participant_id: participantId,
            command: 'voice_command',
            parameters: { text: text }
        });
    }

    // Participant management
    function handleParticipantConnected(participant) {
        state.participants.set(participant.sid, participant);
        updateParticipantsList();
        updateParticipantCount();

        showNotification(`${participant.identity} joined the meeting`);
    }

    function handleParticipantDisconnected(participant) {
        state.participants.delete(participant.sid);

        // Remove participant video
        const videoContainer = document.getElementById(`video-${participant.sid}`);
        if (videoContainer) videoContainer.remove();

        updateParticipantsList();
        updateParticipantCount();

        showNotification(`${participant.identity} left the meeting`);
    }

    function handleParticipantUpdate(message) {
        // Update participant status
        updateParticipantsList();
    }

    function updateParticipantsList() {
        const listContainer = document.getElementById('participantsList');
        listContainer.innerHTML = '';

        // Add self
        const selfEl = createParticipantElement('You', 'current-user', true);
        listContainer.appendChild(selfEl);

        // Add other participants
        state.participants.forEach((participant, sid) => {
            const el = createParticipantElement(participant.identity, sid, false);
            listContainer.appendChild(el);
        });
    }

    function createParticipantElement(name, id, isSelf) {
        const el = document.createElement('div');
        el.className = 'participant-item';
        el.innerHTML = `
            <div class="participant-info">
                <span class="participant-avatar">${name[0].toUpperCase()}</span>
                <span class="participant-name">${name}${isSelf ? ' (You)' : ''}</span>
            </div>
            <div class="participant-controls">
                <span class="indicator ${state.isMuted && isSelf ? 'muted' : ''}">🎤</span>
                <span class="indicator ${state.isVideoOff && isSelf ? 'off' : ''}">📹</span>
            </div>
        `;
        return el;
    }

    function updateParticipantCount() {
        const count = state.participants.size + 1; // +1 for self
        document.getElementById('participantCount').textContent = count;
    }

    // Track handling
    function handleTrackSubscribed(track, publication, participant) {
        if (track.kind === 'video') {
            // Create video container for participant
            const videoGrid = document.getElementById('videoGrid');
            const container = document.createElement('div');
            container.className = 'video-container';
            container.id = `video-${participant.sid}`;
            container.innerHTML = `
                <video autoplay></video>
                <div class="video-overlay">
                    <span class="participant-name">${participant.identity}</span>
                    <div class="video-indicators">
                        <span class="indicator mic-indicator">🎤</span>
                        <span class="indicator video-indicator">📹</span>
                    </div>
                </div>
                <div class="speaking-indicator hidden"></div>
            `;

            const video = container.querySelector('video');
            track.attach(video);

            videoGrid.appendChild(container);
        }
    }

    function handleTrackUnsubscribed(track, publication, participant) {
        track.detach();
    }

    function handleActiveSpeakersChanged(speakers) {
        // Update speaking indicators
        document.querySelectorAll('.speaking-indicator').forEach(el => {
            el.classList.add('hidden');
        });

        speakers.forEach(participant => {
            const container = document.getElementById(`video-${participant.sid}`);
            if (container) {
                container.querySelector('.speaking-indicator').classList.remove('hidden');
            }
        });
    }

    // UI helpers
    function togglePanel(panelName) {
        const panels = {
            participants: 'participantsPanel',
            chat: 'chatPanel',
            transcription: 'transcriptionPanel',
            bot: 'botPanel'
        };

        const panelId = panels[panelName];
        const panel = document.getElementById(panelId);

        if (panel) {
            const isVisible = panel.style.display !== 'none';

            // Hide all panels
            Object.values(panels).forEach(id => {
                document.getElementById(id).style.display = 'none';
            });

            // Toggle selected panel
            if (!isVisible) {
                panel.style.display = 'block';

                // Clear unread count for chat
                if (panelName === 'chat') {
                    state.unreadCount = 0;
                    updateUnreadBadge();
                }
            }
        }
    }

    function updateLocalIndicators() {
        const micIndicator = document.getElementById('localMicIndicator');
        const videoIndicator = document.getElementById('localVideoIndicator');

        micIndicator.classList.toggle('muted', state.isMuted);
        videoIndicator.classList.toggle('off', state.isVideoOff);
    }

    function updateUnreadBadge() {
        const badge = document.getElementById('unreadCount');
        badge.textContent = state.unreadCount;
        badge.classList.toggle('hidden', state.unreadCount === 0);
    }

    function showNotification(message) {
        // Simple notification - could be enhanced with toast notifications
        console.log('Notification:', message);
    }

    // Modals
    function showJoinModal() {
        document.getElementById('joinModal').classList.remove('hidden');
        setupPreview();
    }

    function showCreateModal() {
        document.getElementById('createModal').classList.remove('hidden');
    }

    function showInviteModal() {
        const meetingLink = `${window.location.origin}/meet?meeting=${state.meetingId}`;
        document.getElementById('meetingLink').value = meetingLink;
        document.getElementById('inviteModal').classList.remove('hidden');
    }

    function closeModal(modalId) {
        document.getElementById(modalId).classList.add('hidden');
    }

    window.closeModal = closeModal;

    async function setupPreview() {
        try {
            const stream = await navigator.mediaDevices.getUserMedia({
                video: true,
                audio: true
            });

            const previewVideo = document.getElementById('previewVideo');
            previewVideo.srcObject = stream;

            // Stop tracks when modal closes
            setTimeout(() => {
                stream.getTracks().forEach(track => track.stop());
            }, 30000);
        } catch (error) {
            console.error('Failed to setup preview:', error);
        }
    }

    // Timer
    function startTimer() {
        setInterval(() => {
            if (state.meetingStartTime) {
                const duration = Date.now() - state.meetingStartTime;
                const hours = Math.floor(duration / 3600000);
                const minutes = Math.floor((duration % 3600000) / 60000);
                const seconds = Math.floor((duration % 60000) / 1000);

                const timerEl = document.getElementById('meetingTimer');
                timerEl.textContent = `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
            }
        }, 1000);
    }

    // Invite functions
    async function sendInvites() {
        const emails = document.getElementById('inviteEmails').value
            .split('\n')
            .map(e => e.trim())
            .filter(e => e);

        if (emails.length === 0) {
            alert('Please enter at least one email address');
            return;
        }

        try {
            const response = await fetch('/api/meet/invite', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    meeting_id: state.meetingId,
                    emails: emails
                })
            });

            if (response.ok) {
                alert('Invitations sent successfully!');
                closeModal('inviteModal');
            }
        } catch (error) {
            console.error('Failed to send invites:', error);
            alert('Failed to send invitations. Please try again.');
        }
    }

    window.copyMeetingLink = function() {
        const linkInput = document.getElementById('meetingLink');
        linkInput.select();
        document.execCommand('copy');
        alert('Meeting link copied to clipboard!');
    };

    window.shareVia = function(platform) {
        const meetingLink = document.getElementById('meetingLink').value;
        const message = `Join my meeting: ${meetingLink}`;

        switch (platform) {
            case 'whatsapp':
                window.open(`https://wa.me/?text=${encodeURIComponent(message)}`);
                break;
            case 'teams':
                // Teams integration would require proper API
                alert('Teams integration coming soon!');
                break;
            case 'email':
                window.location.href = `mailto:?subject=Meeting Invitation&body=${encodeURIComponent(message)}`;
                break;
        }
    };

    // Redirect handling for Teams/WhatsApp
    function handleRedirect(platform, meetingId) {
        document.getElementById('redirectHandler').classList.remove('hidden');
        document.getElementById('callerPlatform').textContent = platform;

        // Auto-accept after 3 seconds
        setTimeout(() => {
            acceptCall();
        }, 3000);
    }

    window.acceptCall = async function() {
        document.getElementById('redirectHandler').classList.add('hidden');

        if (state.meetingId) {
            // Already in a meeting, ask to switch
            if (confirm('You are already in a meeting. Switch to the new call?')) {
                await leaveMeeting();
                await joinMeetingRoom(state.meetingId, 'Guest');
            }
        } else {
            await joinMeetingRoom(state.meetingId || 'redirect-room', 'Guest');
        }
    };

    window.rejectCall = function() {
        document.getElementById('redirectHandler').classList.add('hidden');
        window.location.href = '/chat';
    };

    // Transcript download
    function downloadTranscript() {
        const transcript = state.transcriptions
            .map(t => `[${t.timestamp.toLocaleTimeString()}] ${t.participant_id}: ${t.text}`)
            .join('\n');

        const blob = new Blob([transcript], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `meeting-transcript-${state.meetingId}.txt`;
        a.click();
        URL.revokeObjectURL(url);
    }

    function clearTranscript() {
        if (confirm('Are you sure you want to clear the transcript?')) {
            state.transcriptions = [];
            document.getElementById('transcriptionContainer').innerHTML = '';
        }
    }

    // Status updates
    function handleStatusUpdate(message) {
        console.log('Meeting status update:', message.status);

        if (message.status === 'ended') {
            alert('The meeting has ended.');
            window.location.href = '/chat';
        }
    }

    // Initialize on load
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }

    // Public API
    return {
        joinMeeting: joinMeetingRoom,
        leaveMeeting: leaveMeeting,
        sendMessage: sendMessage,
        toggleMicrophone: toggleMicrophone,
        toggleVideo: toggleVideo,
        toggleScreenShare: toggleScreenShare,
        sendBotCommand: sendBotCommand
    };
})();
