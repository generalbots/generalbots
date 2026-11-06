/* Meet — join/leave the room and the signaling socket dispatcher. */
(function () {
    "use strict";
    var M = window.GBMeetInternal;
    var generateId = M.generateId, fetchIceServers = M.fetchIceServers;
    var sendSignal = M.sendSignal, addChatMessage = M.addChatMessage;
    var createOffer = M.createOffer, handleOffer = M.handleOffer;
    var handleAnswer = M.handleAnswer, handleIceCandidate = M.handleIceCandidate;
    var removeRemoteVideo = M.removeRemoteVideo, showFloatingReaction = M.showFloatingReaction;
    var startTimer = M.startTimer, stopTimer = M.stopTimer;

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
})();
