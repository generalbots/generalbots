/* Meet — shared internal namespace and meeting state.
 * Split out of meet.js; load order is fixed by meet.html. */
(function () {
    "use strict";
    if (window.GBAppLifecycle) GBAppLifecycle.begin("meet");

    window.GBMeetInternal = window.GBMeetInternal || {};

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
})();
