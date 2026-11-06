/* Meet — microphone, camera, screen share, chat input and reactions. */
(function () {
    "use strict";
    var M = window.GBMeetInternal;
    var sendSignal = M.sendSignal, addChatMessage = M.addChatMessage;
    var showFloatingReaction = M.showFloatingReaction;

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
})();
