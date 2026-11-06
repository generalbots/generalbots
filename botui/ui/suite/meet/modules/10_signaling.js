/* Meet — WebRTC signaling and peer connections. */
(function () {
    "use strict";
    var M = window.GBMeetInternal;
    var ICE_SERVERS_URL = "/api/meet/turn-credentials";

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
    M.generateId = generateId;
    M.fetchIceServers = fetchIceServers;
    M.getDefaultIceServers = getDefaultIceServers;
    M.createPeerConnection = createPeerConnection;
    M.createOffer = createOffer;
    M.handleOffer = handleOffer;
    M.handleAnswer = handleAnswer;
    M.handleIceCandidate = handleIceCandidate;
    M.sendSignal = sendSignal;
    M.addRemoteVideo = addRemoteVideo;
    M.removeRemoteVideo = removeRemoteVideo;
    M.updateParticipantCount = updateParticipantCount;
    M.escapeHtml = escapeHtml;
})();
