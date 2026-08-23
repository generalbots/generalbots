// Guard against re-execution - check BEFORE any declarations
    if (!window._meetInlineLoaded) {
        window._meetInlineLoaded = true;

        // Use window object for all state to avoid redeclaration
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
    }
