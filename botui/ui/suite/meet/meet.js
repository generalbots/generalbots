/* Meet - Video Conferencing JavaScript with WebRTC Signaling */

(function() {
    'use strict';

    const ICE_SERVERS_URL = '/api/meet/turn-credentials';

    window._meetState = window._meetState || {
        localStream: null,
        screenStream: null,
        peerConnections: {},
        signalingSocket: null,
        roomId: null,
        participantId: null,
        participantName: null,
        iceServers: [],
        isMuted: false,
        isCameraOff: false,
        isScreenSharing: false,
        isHandRaised: false,
        timerInterval: null,
        timerSeconds: 0
    };

    function generateId() {
        return 'p_' + Math.random().toString(36).substring(2, 10);
    }

    async function fetchIceServers() {
        try {
            const resp = await fetch(ICE_SERVERS_URL);
            if (!resp.ok) return getDefaultIceServers();
            const data = await resp.json();
            return data.ice_servers || getDefaultIceServers();
        } catch (e) {
            console.warn('Failed to fetch TURN credentials:', e);
            return getDefaultIceServers();
        }
    }

    function getDefaultIceServers() {
        return [
            { urls: ['stun:stun.l.google.com:19302'] },
            { urls: ['stun:stun1.l.google.com:19302'] }
        ];
    }

    function createPeerConnection(targetId, targetName) {
        const state = window._meetState;
        const config = { iceServers: state.iceServers };
        const pc = new RTCPeerConnection(config);

        pc.onicecandidate = function(event) {
            if (event.candidate) {
                sendSignal({
                    type: 'ice_candidate',
                    target_id: targetId,
                    candidate: event.candidate.toJSON()
                });
            }
        };

        pc.ontrack = function(event) {
            addRemoteVideo(targetId, targetName, event.streams[0]);
        };

        pc.oniceconnectionstatechange = function() {
            if (pc.iceConnectionState === 'failed' || pc.iceConnectionState === 'disconnected') {
                removeRemoteVideo(targetId);
            }
        };

        if (state.localStream) {
            state.localStream.getTracks().forEach(function(track) {
                pc.addTrack(track, state.localStream);
            });
        }

        state.peerConnections[targetId] = pc;
        return pc;
    }

    async function createOffer(targetId) {
        const state = window._meetState;
        let pc = state.peerConnections[targetId];
        if (!pc) {
            pc = createPeerConnection(targetId, '');
        }

        const offer = await pc.createOffer();
        await pc.setLocalDescription(offer);

        sendSignal({
            type: 'offer',
            target_id: targetId,
            sdp: pc.localDescription.toJSON()
        });
    }

    async function handleOffer(fromId, fromName, sdp) {
        const state = window._meetState;
        let pc = state.peerConnections[fromId];
        if (!pc) {
            pc = createPeerConnection(fromId, fromName);
        }

        await pc.setRemoteDescription(new RTCSessionDescription(sdp));
        const answer = await pc.createAnswer();
        await pc.setLocalDescription(answer);

        sendSignal({
            type: 'answer',
            target_id: fromId,
            sdp: pc.localDescription.toJSON()
        });
    }

    async function handleAnswer(fromId, sdp) {
        const state = window._meetState;
        const pc = state.peerConnections[fromId];
        if (pc) {
            await pc.setRemoteDescription(new RTCSessionDescription(sdp));
        }
    }

    async function handleIceCandidate(fromId, candidate) {
        const state = window._meetState;
        const pc = state.peerConnections[fromId];
        if (pc) {
            try {
                await pc.addIceCandidate(new RTCIceCandidate(candidate));
            } catch (e) {
                console.warn('Error adding ICE candidate:', e);
            }
        }
    }

    function sendSignal(message) {
        const state = window._meetState;
        if (state.signalingSocket && state.signalingSocket.readyState === WebSocket.OPEN) {
            state.signalingSocket.send(JSON.stringify(message));
        }
    }

    function addRemoteVideo(participantId, participantName, stream) {
        const grid = document.getElementById('video-grid');
        if (!grid) return;

        let tile = document.getElementById('remote-' + participantId);
        if (!tile) {
            tile = document.createElement('div');
            tile.id = 'remote-' + participantId;
            tile.className = 'video-tile remote-video';
            tile.innerHTML =
                '<video autoplay playsinline></video>' +
                '<div class="video-overlay">' +
                '<span class="participant-name">' + escapeHtml(participantName || 'Participant') + '</span>' +
                '<div class="video-indicators">' +
                '<span class="indicator mic-on"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"></path><path d="M19 10v2a7 7 0 0 1-14 0v-2"></path></svg></span>' +
                '</div>' +
                '</div>';
            grid.appendChild(tile);
        }

        const video = tile.querySelector('video');
        if (video && stream) {
            video.srcObject = stream;
        }

        updateParticipantCount();
    }

    function removeRemoteVideo(participantId) {
        const tile = document.getElementById('remote-' + participantId);
        if (tile) {
            tile.remove();
        }
        updateParticipantCount();
    }

    function updateParticipantCount() {
        const grid = document.getElementById('video-grid');
        const count = grid ? grid.querySelectorAll('.video-tile').length : 1;
        const badge = document.getElementById('participant-count');
        if (badge) badge.textContent = count;
    }

    function escapeHtml(text) {
        var div = document.createElement('div');
        div.appendChild(document.createTextNode(text));
        return div.innerHTML;
    }

    window.joinRoom = async function(roomId, participantName) {
        const state = window._meetState;
        state.roomId = roomId;
        state.participantId = generateId();
        state.participantName = participantName || 'Anonymous';

        state.iceServers = await fetchIceServers();

        try {
            state.localStream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
            var localVideo = document.getElementById('local-video');
            if (localVideo) localVideo.srcObject = state.localStream;
        } catch (err) {
            console.error('Failed to get media:', err);
        }

        var protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        var wsUrl = protocol + '//' + window.location.host + '/ws/meet/' + roomId;

        state.signalingSocket = new WebSocket(wsUrl);

        state.signalingSocket.onopen = function() {
            sendSignal({
                type: 'join',
                room_id: roomId,
                participant_id: state.participantId,
                participant_name: state.participantName
            });
        };

        state.signalingSocket.onmessage = function(event) {
            var msg;
            try {
                msg = JSON.parse(event.data);
            } catch (e) {
                return;
            }

            switch (msg.type) {
                case 'user_joined':
                    createOffer(msg.participant_id);
                    addChatMessage('System', msg.participant_name + ' joined the meeting');
                    break;
                case 'offer':
                    handleOffer(msg.from_id, '', msg.sdp);
                    break;
                case 'answer':
                    handleAnswer(msg.from_id, msg.sdp);
                    break;
                case 'ice_candidate':
                    handleIceCandidate(msg.from_id, msg.candidate);
                    break;
                case 'user_left':
                    removeRemoteVideo(msg.participant_id);
                    addChatMessage('System', 'Participant left the meeting');
                    break;
                case 'chat_message':
                    addChatMessage(msg.from_name, msg.content);
                    break;
                case 'screen_share_start':
                    addChatMessage('System', msg.participant_id + ' is sharing their screen');
                    break;
                case 'screen_share_stop':
                    addChatMessage('System', 'Screen sharing stopped');
                    break;
                case 'raise_hand':
                    addChatMessage('System', msg.participant_id + ' raised their hand');
                    break;
                case 'reaction':
                    showFloatingReaction(msg.emoji);
                    break;
                case 'error':
                    console.error('Signaling error:', msg.message);
                    break;
            }
        };

        state.signalingSocket.onclose = function() {
            console.log('Signaling socket closed');
        };

        state.signalingSocket.onerror = function(err) {
            console.error('Signaling socket error:', err);
        };
    };

    window.leaveRoom = function() {
        var state = window._meetState;

        sendSignal({ type: 'leave' });

        if (state.signalingSocket) {
            state.signalingSocket.close();
            state.signalingSocket = null;
        }

        Object.keys(state.peerConnections).forEach(function(id) {
            state.peerConnections[id].close();
        });
        state.peerConnections = {};

        if (state.localStream) {
            state.localStream.getTracks().forEach(function(track) { track.stop(); });
            state.localStream = null;
        }

        if (state.screenStream) {
            state.screenStream.getTracks().forEach(function(track) { track.stop(); });
            state.screenStream = null;
        }

        var grid = document.getElementById('video-grid');
        if (grid) {
            var remoteTiles = grid.querySelectorAll('.remote-video');
            remoteTiles.forEach(function(tile) { tile.remove(); });
        }

        stopTimer();
    };

    window.toggleMic = function() {
        var state = window._meetState;
        if (state.localStream) {
            var audioTrack = state.localStream.getAudioTracks()[0];
            if (audioTrack) {
                audioTrack.enabled = !audioTrack.enabled;
                state.isMuted = !audioTrack.enabled;
                var btn = document.getElementById('mic-btn');
                if (btn) btn.classList.toggle('muted', state.isMuted);
            }
        }
    };

    window.toggleCamera = function() {
        var state = window._meetState;
        if (state.localStream) {
            var videoTrack = state.localStream.getVideoTracks()[0];
            if (videoTrack) {
                videoTrack.enabled = !videoTrack.enabled;
                state.isCameraOff = !videoTrack.enabled;
                var btn = document.getElementById('camera-btn');
                if (btn) btn.classList.toggle('muted', state.isCameraOff);
            }
        }
    };

    window.toggleScreenShare = async function() {
        var state = window._meetState;

        if (state.isScreenSharing) {
            if (state.screenStream) {
                state.screenStream.getTracks().forEach(function(track) { track.stop(); });
                state.screenStream = null;
            }

            if (state.localStream) {
                var videoTrack = state.localStream.getVideoTracks()[0];
                if (videoTrack) {
                    Object.keys(state.peerConnections).forEach(function(id) {
                        var sender = state.peerConnections[id].getSenders().find(function(s) {
                            return s.track && s.track.kind === 'video';
                        });
                        if (sender) sender.replaceTrack(videoTrack);
                    });
                }
            }

            sendSignal({ type: 'screen_share_stop' });
            state.isScreenSharing = false;
            var btn = document.getElementById('screen-btn');
            if (btn) btn.classList.remove('muted');
            return;
        }

        try {
            var screenStream = await navigator.mediaDevices.getDisplayMedia({ video: true });
            state.screenStream = screenStream;
            state.isScreenSharing = true;

            var screenTrack = screenStream.getVideoTracks()[0];
            screenTrack.onended = function() {
                window.toggleScreenShare();
            };

            Object.keys(state.peerConnections).forEach(function(id) {
                var sender = state.peerConnections[id].getSenders().find(function(s) {
                    return s.track && s.track.kind === 'video';
                });
                if (sender) sender.replaceTrack(screenTrack);
            });

            sendSignal({ type: 'screen_share_start' });
            var btn = document.getElementById('screen-btn');
            if (btn) btn.classList.add('muted');
        } catch (err) {
            console.error('Screen share failed:', err);
        }
    };

    window.sendChatMessage = function() {
        var input = document.getElementById('chat-input');
        if (!input) return;
        var message = input.value.trim();
        if (!message) return;

        sendSignal({ type: 'chat_message', content: message });
        addChatMessage('You', message);
        input.value = '';
    };

    window.toggleHand = function() {
        var state = window._meetState;
        state.isHandRaised = !state.isHandRaised;
        sendSignal({ type: 'raise_hand' });
    };

    window.sendReaction = function(emoji) {
        sendSignal({ type: 'reaction', emoji: emoji });
        showFloatingReaction(emoji);
        var popup = document.getElementById('reactions-popup');
        if (popup) popup.classList.add('hidden');
    };

    function addChatMessage(sender, content) {
        var container = document.getElementById('chat-messages');
        if (!container) return;

        var msg = document.createElement('div');
        msg.className = 'chat-message';
        msg.innerHTML =
            '<div class="chat-sender">' + escapeHtml(sender) + '</div>' +
            '<div class="chat-text">' + escapeHtml(content) + '</div>';
        container.appendChild(msg);
        container.scrollTop = container.scrollHeight;
    }

    function showFloatingReaction(emoji) {
        var el = document.createElement('div');
        el.textContent = emoji;
        el.style.cssText = 'position:fixed;bottom:120px;font-size:2rem;pointer-events:none;z-index:999;transition:all 2s ease-out;opacity:1;';
        el.style.left = (Math.random() * 60 + 20) + '%';
        document.body.appendChild(el);

        setTimeout(function() {
            el.style.opacity = '0';
            el.style.transform = 'translateY(-200px)';
        }, 50);

        setTimeout(function() { el.remove(); }, 2100);
    }

    function startTimer() {
        var state = window._meetState;
        state.timerInterval = setInterval(function() {
            state.timerSeconds++;
            var h = Math.floor(state.timerSeconds / 3600);
            var m = Math.floor((state.timerSeconds % 3600) / 60);
            var s = state.timerSeconds % 60;
            var display = [h, m, s].map(function(v) { return v.toString().padStart(2, '0'); }).join(':');
            var el = document.getElementById('room-timer');
            if (el) el.textContent = display;
        }, 1000);
    }

    function stopTimer() {
        var state = window._meetState;
        if (state.timerInterval) {
            clearInterval(state.timerInterval);
            state.timerInterval = null;
        }
        state.timerSeconds = 0;
    }

    window.enterMeeting = function() {
        var meetMain = document.querySelector('.meet-main');
        var meetHeader = document.querySelector('.meet-header');
        var meetingRoom = document.getElementById('meeting-room');
        if (meetMain) meetMain.classList.add('hidden');
        if (meetHeader) meetHeader.classList.add('hidden');
        if (meetingRoom) meetingRoom.classList.remove('hidden');
        startTimer();
    };

    window.leaveMeeting = function() {
        window.leaveRoom();
        var meetingRoom = document.getElementById('meeting-room');
        var meetMain = document.querySelector('.meet-main');
        var meetHeader = document.querySelector('.meet-header');
        if (meetingRoom) meetingRoom.classList.add('hidden');
        if (meetMain) meetMain.classList.remove('hidden');
        if (meetHeader) meetHeader.classList.remove('hidden');
    };

    window.showModal = function(id) {
        var modal = document.getElementById(id);
        if (modal) modal.showModal();
    };

    window.hideModal = function(id) {
        var modal = document.getElementById(id);
        if (modal) modal.close();
    };

    window.togglePanel = function(name) {
        var panels = ['participants', 'chat', 'transcription'];
        panels.forEach(function(p) {
            var panel = document.getElementById(p + '-panel');
            if (panel) {
                if (p === name) {
                    panel.classList.toggle('hidden');
                } else {
                    panel.classList.add('hidden');
                }
            }
        });
    };

    window.showReactions = function() {
        var popup = document.getElementById('reactions-popup');
        if (popup) popup.classList.toggle('hidden');
    };

    window.copyMeetingLink = function() {
        var input = document.getElementById('meeting-link');
        if (input) {
            input.select();
            navigator.clipboard.writeText(input.value);
        }
    };

    window.testVideo = async function() {
        try {
            var stream = await navigator.mediaDevices.getUserMedia({ video: true });
            var preview = document.getElementById('preview-video');
            if (preview) preview.srcObject = stream;
        } catch (err) {
            console.error('Error testing video:', err);
        }
    };

    window.testAudio = function() {
        console.log('Testing audio...');
    };

    window.showMoreOptions = function() {
        console.log('More options...');
    };

    window.showNotification = function(message) {
        console.log('Notification:', message);
    };
})();
