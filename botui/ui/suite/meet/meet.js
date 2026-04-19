/* Meet - Video Conferencing JavaScript */

// Modal functions
function showModal(id) {
    const modal = document.getElementById(id);
    if (modal) modal.showModal();
}

function hideModal(id) {
    const modal = document.getElementById(id);
    if (modal) modal.close();
}

// Panel toggle
function togglePanel(name) {
    const panels = ['participants', 'chat', 'transcription'];
    panels.forEach(p => {
        const panel = document.getElementById(p + '-panel');
        if (panel) {
            if (p === name) {
                panel.classList.toggle('hidden');
            } else {
                panel.classList.add('hidden');
            }
        }
    });
}

// Meeting functions
function enterMeeting() {
    document.querySelector('.meet-main')?.classList.add('hidden');
    document.querySelector('.meet-header')?.classList.add('hidden');
    document.getElementById('meeting-room')?.classList.remove('hidden');
    startTimer();
    initializeMedia();
}

function leaveMeeting() {
    document.getElementById('meeting-room')?.classList.add('hidden');
    document.querySelector('.meet-main')?.classList.remove('hidden');
    document.querySelector('.meet-header')?.classList.remove('hidden');
    stopTimer();
    stopMedia();
}

// Media controls
let localStream = null;
let isMuted = false;
let isCameraOff = false;

async function initializeMedia() {
    try {
        localStream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
        const localVideo = document.getElementById('local-video');
        if (localVideo) localVideo.srcObject = localStream;
    } catch (err) {
        console.error('Error accessing media devices:', err);
    }
}

function stopMedia() {
    if (localStream) {
        localStream.getTracks().forEach(track => track.stop());
        localStream = null;
    }
}

function toggleMic() {
    if (localStream) {
        const audioTrack = localStream.getAudioTracks()[0];
        if (audioTrack) {
            audioTrack.enabled = !audioTrack.enabled;
            isMuted = !audioTrack.enabled;
            const btn = document.getElementById('mic-btn');
            btn?.classList.toggle('muted', isMuted);
        }
    }
}

function toggleCamera() {
    if (localStream) {
        const videoTrack = localStream.getVideoTracks()[0];
        if (videoTrack) {
            videoTrack.enabled = !videoTrack.enabled;
            isCameraOff = !videoTrack.enabled;
            const btn = document.getElementById('camera-btn');
            btn?.classList.toggle('muted', isCameraOff);
        }
    }
}

async function toggleScreenShare() {
    try {
        const screenStream = await navigator.mediaDevices.getDisplayMedia({ video: true });
        // Handle screen share stream
    } catch (err) {
        console.error('Error sharing screen:', err);
    }
}

// Timer
let timerInterval = null;
let timerSeconds = 0;

function startTimer() {
    timerInterval = setInterval(() => {
        timerSeconds++;
        const hours = Math.floor(timerSeconds / 3600);
        const minutes = Math.floor((timerSeconds % 3600) / 60);
        const seconds = timerSeconds % 60;
        const display = [hours, minutes, seconds]
            .map(v => v.toString().padStart(2, '0'))
            .join(':');
        const timerEl = document.getElementById('room-timer');
        if (timerEl) timerEl.textContent = display;
    }, 1000);
}

function stopTimer() {
    if (timerInterval) {
        clearInterval(timerInterval);
        timerInterval = null;
    }
    timerSeconds = 0;
}

// Reactions
function showReactions() {
    document.getElementById('reactions-popup')?.classList.toggle('hidden');
}

function sendReaction(emoji) {
    // Send via WebSocket
    console.log('Sending reaction:', emoji);
    document.getElementById('reactions-popup')?.classList.add('hidden');
}

// Copy meeting link
function copyMeetingLink() {
    const input = document.getElementById('meeting-link');
    if (input) {
        input.select();
        navigator.clipboard.writeText(input.value);
    }
}

// Preview for join modal
async function testVideo() {
    try {
        const stream = await navigator.mediaDevices.getUserMedia({ video: true });
        const preview = document.getElementById('preview-video');
        if (preview) preview.srcObject = stream;
    } catch (err) {
        console.error('Error testing video:', err);
    }
}

function testAudio() {
    // Test audio input/output
    console.log('Testing audio...');
}

function showMoreOptions() {
    // Show more options menu
    console.log('More options...');
}

function showNotification(message) {
    // Show notification toast
    console.log('Notification:', message);
}
