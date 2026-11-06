        let ws = null,
            currentSessionId = null,
            currentUserId = null,
            currentBotId = "default_bot",
            isStreaming = false,
            voiceRoom = null,
            isVoiceMode = false,
            mediaRecorder = null,
            audioChunks = [],
            streamingMessageId = null,
            isThinking = false,
            currentStreamingContent = "",
            hasReceivedInitialMessage = false,
            reconnectAttempts = 0,
            reconnectTimeout = null,
            thinkingTimeout = null,
            currentTheme = "auto",
            themeColor1 = null,
            themeColor2 = null,
            customLogoUrl = null,
            contextUsage = 0,
            isUserScrolling = false,
            autoScrollEnabled = true,
            isContextChange = false;
        const maxReconnectAttempts = 5;
        let messagesDiv = document.getElementById("messages");
        let input = document.getElementById("messageInput");
        let sendBtn = document.getElementById("sendBtn");
        let voiceBtn = document.getElementById("voiceBtn");

        // Debug element initialization
        console.log("Element initialization:");
        console.log("messagesDiv:", messagesDiv);
        console.log("input:", input);
        console.log("sendBtn:", sendBtn);
        console.log("voiceBtn:", voiceBtn);

        const connectionStatus =
            document.getElementById("connectionStatus"),
            flashOverlay = document.getElementById("flashOverlay"),
            suggestionsContainer = document.getElementById("suggestions"),
            sidebar = document.getElementById("sidebar"),

            scrollToBottomBtn = document.getElementById("scrollToBottom"),
            contextIndicator = document.getElementById("contextIndicator"),
            contextPercentage =
                document.getElementById("contextPercentage"),
            contextProgressBar =
                document.getElementById("contextProgressBar"),
            sidebarTitle = document.getElementById("sidebarTitle");
        marked.setOptions({ breaks: true, gfm: true });



        function toggleSidebar() {
            sidebar.classList.toggle("open");
        }

        function toggleTheme() {
            const themes = ["auto", "dark", "light"];
            const savedTheme = localStorage.getItem("gb-theme") || "auto";
            const idx = themes.indexOf(savedTheme);
            const newTheme = themes[(idx + 1) % themes.length];
            localStorage.setItem("gb-theme", newTheme);
            currentTheme = newTheme;
            applyTheme();
            updateThemeButton();
        }

        function updateThemeButton() {
            // Theme button removed for non-logged users
        }

        function applyTheme() {
            const prefersDark = window.matchMedia(
                "(prefers-color-scheme: dark)",
            ).matches;
            let theme = currentTheme;
            if (theme === "auto") {
                theme = prefersDark ? "dark" : "light";
            }
            document.documentElement.setAttribute("data-theme", theme);
            if (themeColor1 && themeColor2) {
                const root = document.documentElement;
                root.style.setProperty(
                    "--bg",
                    theme === "dark" ? themeColor2 : themeColor1,
                );
                root.style.setProperty(
                    "--fg",
                    theme === "dark" ? themeColor1 : themeColor2,
                );
            }
            if (customLogoUrl) {
                document.documentElement.style.setProperty(
                    "--logo-url",
                    `url('${customLogoUrl}')`,
                );
            }
        }

        window.addEventListener("load", function () {
            const savedTheme = localStorage.getItem("gb-theme") || "auto";
            currentTheme = savedTheme;
            applyTheme();
            updateThemeButton();
            window
                .matchMedia("(prefers-color-scheme: dark)")
                .addEventListener("change", () => {
                    if (currentTheme === "auto") {
                        applyTheme();
                    }
                });
            input.focus();
        });



        document.addEventListener("click", function (e) {
            if (
                sidebar.classList.contains("open") &&
                !sidebar.contains(e.target)
            ) {
                sidebar.classList.remove("open");
            }
        });

        messagesDiv.addEventListener("scroll", function () {
            const isAtBottom =
                messagesDiv.scrollHeight - messagesDiv.scrollTop <=
                messagesDiv.clientHeight + 100;
            if (!isAtBottom) {
                isUserScrolling = true;
                scrollToBottomBtn.classList.add("visible");
            } else {
                isUserScrolling = false;
                scrollToBottomBtn.classList.remove("visible");
            }
        });

        scrollToBottomBtn.addEventListener("click", function () {
            scrollToBottom();
        });

        function updateContextUsage(u) {
            contextUsage = u;
            const p = Math.min(100, Math.round(u * 100));
            contextPercentage.textContent = `${p}%`;
            contextProgressBar.style.width = `${p}%`;
            contextIndicator.classList.remove("visible");
        }

        function flashScreen() {
            flashOverlay.classList.remove("flash-active");
            void flashOverlay.offsetWidth;
            flashOverlay.classList.add("flash-active");
        }

        function updateConnectionStatus(s) {
            connectionStatus.className = `connection-status ${s}`;
        }

        function getWebSocketUrl() {
            const s = currentSessionId || crypto.randomUUID(),
                u = currentUserId || crypto.randomUUID();
            return `${BOTSERVER_WS_URL}/ws?session_id=${s}&user_id=${u}`;
        }

        async function initializeAuth() {
            try {
                console.log("Starting auth initialization...");
                updateConnectionStatus("connecting");
                const p = window.location.pathname
                    .split("/")
                    .filter((s) => s),
                    b = p.length > 0 ? p[0] : "default";
                console.log("Bot name:", b);
                const r = await fetch(
                    `${BOTSERVER_URL}/api/auth?bot_name=${encodeURIComponent(b)}`,
                ),
                    a = await r.json();
                console.log("Auth response:", a);
                currentUserId = a.user_id;
                currentSessionId = a.session_id;
                currentBotId = a.bot_id || "default_bot";
                console.log(
                    "Auth initialized - User:",
                    currentUserId,
                    "Session:",
                    currentSessionId,
                    "Bot:",
                    currentBotId,
                );
                connectWebSocket();
                loadSessions();
            } catch (e) {
                console.error("Failed to initialize auth:", e);
                updateConnectionStatus("disconnected");
                setTimeout(initializeAuth, 3000);
            }
        }

        async function loadSessions() {
            try {
                const r = await fetch(`${BOTSERVER_URL}/api/sessions`),
                    s = await r.json(),
                    h = document.getElementById("history");
                h.innerHTML = "";
                s.forEach((session) => {
                    const item = document.createElement("div");
                    item.className = "history-item";
                    item.textContent =
                        session.title ||
                        `Session ${session.session_id.substring(0, 8)}`;
                    item.onclick = () => switchSession(session.session_id);
                    h.appendChild(item);
                });
            } catch (e) {
                console.error("Failed to load sessions:", e);
            }
        }

        async function createNewSession() {
            try {
                const r = await fetch(`${BOTSERVER_URL}/api/sessions`, {
                    method: "POST",
                }),
                    s = await r.json();
                currentSessionId = s.session_id;
                hasReceivedInitialMessage = false;
                connectWebSocket();
                loadSessions();
                messagesDiv.innerHTML = "";
                clearSuggestions();
                updateContextUsage(0);
                if (isVoiceMode) {
                    await stopVoiceSession();
                    isVoiceMode = false;
                    const v = document.getElementById("voiceToggle");
                    v.textContent = "🎤 Voice Mode";
                    voiceBtn.classList.remove("recording");
                }
            } catch (e) {
                console.error("Failed to create session:", e);
            }
        }

        function switchSession(s) {
            currentSessionId = s;
            hasReceivedInitialMessage = false;
            loadSessionHistory(s);
            connectWebSocket();
            if (isVoiceMode) {
                startVoiceSession();
            }
            sidebar.classList.remove("open");
        }

        async function loadSessionHistory(s) {
            try {
                const r = await fetch(`${BOTSERVER_URL}/api/sessions/${s}`),
                    h = await r.json(),
                    m = document.getElementById("messages");
                m.innerHTML = "";
                if (h.length === 0) {
                    updateContextUsage(0);
                } else {
                    h.forEach(([role, content]) => {
                        addMessage(role, content, false);
                    });
                    updateContextUsage(h.length / 20);
                }
            } catch (e) {
                console.error("Failed to load session history:", e);
            }
        }

        function connectWebSocket() {
            if (ws) {
                ws.close();
            }
            clearTimeout(reconnectTimeout);
            const u = getWebSocketUrl();
            ws = new WebSocket(u);
            ws.onmessage = function (e) {
                console.log("WebSocket message received:", e.data);
                try {
                    if (!e.data || e.data.trim() === "") {
                        console.warn("Empty WebSocket message received");
                        return;
                    }
                    const r = JSON.parse(e.data);
                    if (r.type === "connected") {
                        console.log("WebSocket welcome message:", r);
                        reconnectAttempts = 0;
                        return;
                    }
                    if (r.bot_id) {
                        currentBotId = r.bot_id;
                    }
                    // BOT_RESPONSE type is used for both regular streaming content and special event messages
                    // Event messages have JSON-encoded content with 'event' and 'data' properties
                    // Regular messages have plain text content that should be displayed directly
                    if (r.message_type === MessageType.BOT_RESPONSE) {
                        // Check if content looks like JSON (starts with { or [)
                        const contentTrimmed = r.content.trim();
                        if (
                            contentTrimmed.startsWith("{") ||
                            contentTrimmed.startsWith("[")
                        ) {
                            try {
                                const d = JSON.parse(r.content);
                                if (d.event && d.data) {
                                    // This is an event message
                                    handleEvent(d.event, d.data);
                                    return;
                                }
                            } catch (parseErr) {
                                // Not a valid event message, treat as regular content
                                console.debug(
                                    "Content is not an event message, processing as regular message",
                                );
                            }
                        }
                        // Process as regular message content
                        processMessageContent(r);
                        return;
                    }
                    if (r.message_type === MessageType.CONTEXT_CHANGE) {
                        isContextChange = true;
                        return;
                    }
                    processMessageContent(r);
                } catch (err) {
                    console.error(
                        "WebSocket message parse error:",
                        err,
                        "Raw data:",
                        e.data,
                    );
                }
            };
            ws.onopen = function () {
                console.log(
                    "Connected to WebSocket, readyState:",
                    ws.readyState,
                );
                updateConnectionStatus("connected");
                hasReceivedInitialMessage = false;
            };
            ws.onclose = function (e) {
                console.log("WebSocket disconnected:", e.code, e.reason);
                updateConnectionStatus("disconnected");
                if (isStreaming) {
                    showContinueButton();
                }
                if (reconnectAttempts < maxReconnectAttempts) {
                    reconnectAttempts++;
                    const d = Math.min(1000 * reconnectAttempts, 10000);
                    reconnectTimeout = setTimeout(() => {
                        updateConnectionStatus("connecting");
                        connectWebSocket();
                    }, d);
                } else {
                    updateConnectionStatus("disconnected");
                }
            };
            ws.onerror = function (e) {
                console.error("WebSocket error:", e);
                updateConnectionStatus("disconnected");
            };
        }
