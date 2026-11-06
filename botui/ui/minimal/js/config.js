        // BotServer URL - configurable via window.BOTSERVER_URL or defaults to same origin
        const BOTSERVER_URL =
            window.BOTSERVER_URL || "";
        const BOTSERVER_WS_URL = BOTSERVER_URL.replace(
            "https://",
            "wss://",
        ).replace("http://", "ws://");

        // Message Type Constants inline since we don't have a /shared route
        const MessageType = {
            EXTERNAL: 0,
            USER: 1,
            BOT_RESPONSE: 2,
            CONTINUE: 3,
            SUGGESTION: 4,
            CONTEXT_CHANGE: 5,
        };
