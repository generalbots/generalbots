// Singleton instance to prevent multiple initializations
let chatAppInstance = null;

function chatApp() {
  // Return existing instance if already created
  if (chatAppInstance) {
    console.log("Returning existing chatApp instance");
    return chatAppInstance;
  }

  console.log("Creating new chatApp instance");

  // Core state variables (shared via closure)
  let ws = null,
    pendingContextChange = null,
    o,
    isConnecting = false,
    isInitialized = false,
    authPromise = null;
  ((currentSessionId = null),
    (currentUserId = null),
    (currentBotId = "default_bot"),
    (isStreaming = false),
    (voiceRoom = null),
    (isVoiceMode = false),
    (mediaRecorder = null),
    (audioChunks = []),
    (streamingMessageId = null),
    (isThinking = false),
    (currentStreamingContent = ""),
    (hasReceivedInitialMessage = false),
    (reconnectAttempts = 0),
    (reconnectTimeout = null),
    (thinkingTimeout = null),
    (currentTheme = "auto"),
    (themeColor1 = null),
    (themeColor2 = null),
    (customLogoUrl = null),
    (contextUsage = 0),
    (isUserScrolling = false),
    (autoScrollEnabled = true),
    (isContextChange = false));

  const maxReconnectAttempts = 5;

  // DOM references (cached for performance)
  let messagesDiv,
    messageInputEl,
    sendBtn,
    voiceBtn,
    connectionStatus,
    flashOverlay,
    suggestionsContainer,
    floatLogo,
    sidebar,
    themeBtn,
    scrollToBottomBtn,
    sidebarTitle;

  marked.setOptions({ breaks: true, gfm: true });

  return {
    // ----------------------------------------------------------------------
    // UI state (mirrors the structure used in driveApp)
    // ----------------------------------------------------------------------
    current: "All Chats",
    search: "",
    selectedChat: null,
    navItems: [
      { name: "All Chats", icon: "💬" },
      { name: "Direct", icon: "👤" },
      { name: "Groups", icon: "👥" },
      { name: "Archived", icon: "🗄" },
    ],
    chats: [
      {
        id: 1,
        name: "General Bot Support",
        icon: "🤖",
        lastMessage: "How can I help you?",
        time: "10:15 AM",
        status: "Online",
      },
      {
        id: 2,
        name: "Project Alpha",
        icon: "🚀",
        lastMessage: "Launch scheduled for tomorrow.",
        time: "Yesterday",
        status: "Active",
      },
      {
        id: 3,
        name: "Team Stand‑up",
        icon: "🗣️",
        lastMessage: "Done with the UI updates.",
        time: "2 hrs ago",
        status: "Active",
      },
      {
        id: 4,
        name: "Random Chat",
        icon: "🎲",
        lastMessage: "Did you see the game last night?",
        time: "5 hrs ago",
        status: "Idle",
      },
      {
        id: 5,
        name: "Support Ticket #1234",
        icon: "🛠️",
        lastMessage: "Issue resolved, closing ticket.",
        time: "3 days ago",
        status: "Closed",
      },
    ],
    get filteredChats() {
      return this.chats.filter((chat) =>
        chat.name.toLowerCase().includes(this.search.toLowerCase()),
      );
    },

    // ----------------------------------------------------------------------
    // UI helpers (formerly standalone functions)
    // ----------------------------------------------------------------------
    toggleSidebar() {
      sidebar.classList.toggle("open");
    },

    toggleTheme() {
      const themes = ["auto", "dark", "light"];
      const savedTheme = localStorage.getItem("gb-theme") || "auto";
      const idx = themes.indexOf(savedTheme);
      const newTheme = themes[(idx + 1) % themes.length];
      localStorage.setItem("gb-theme", newTheme);
      currentTheme = newTheme;
      this.applyTheme();
      this.updateThemeButton();
    },

    applyTheme() {
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
    },

    // ----------------------------------------------------------------------
    // Lifecycle / event handlers
    // ----------------------------------------------------------------------
    init() {
      // Prevent multiple initializations
      if (isInitialized) {
        console.log("Already initialized, skipping...");
        return;
      }
      isInitialized = true;

      const initializeDOM = () => {
        // Assign DOM elements after the document is ready
        messagesDiv = document.getElementById("messages");

        messageInputEl = document.getElementById("messageInput");
        sendBtn = document.getElementById("sendBtn");
        voiceBtn = document.getElementById("voiceBtn");
        connectionStatus = document.getElementById("connectionStatus");
        flashOverlay = document.getElementById("flashOverlay");
        suggestionsContainer = document.getElementById("suggestions");
        scrollToBottomBtn = document.getElementById("scrollToBottom");

        console.log("Chat DOM elements initialized:", {
          messagesDiv: !!messagesDiv,
          messageInputEl: !!messageInputEl,
          sendBtn: !!sendBtn,
          voiceBtn: !!voiceBtn,
          connectionStatus: !!connectionStatus,
        });

        // Theme initialization and focus
        const savedTheme = localStorage.getItem("gb-theme") || "auto";
        currentTheme = savedTheme;
        this.applyTheme();
        window
          .matchMedia("(prefers-color-scheme: dark)")
          .addEventListener("change", () => {
            if (currentTheme === "auto") {
              this.applyTheme();
            }
          });
        if (messageInputEl) {
          messageInputEl.focus();
        }

        // UI event listeners
        document.addEventListener("click", (e) => {});

        // Scroll detection
        if (messagesDiv && scrollToBottomBtn) {
          messagesDiv.addEventListener("scroll", () => {
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

          scrollToBottomBtn.addEventListener("click", () => {
            this.scrollToBottom();
          });
        }

        if (sendBtn) {
          sendBtn.onclick = () => this.sendMessage();
        }

        if (messageInputEl) {
          messageInputEl.addEventListener("keypress", (e) => {
            if (e.key === "Enter") this.sendMessage();
          });
        }

        // Don't auto-reconnect on focus in browser to prevent multiple connections
        // Tauri doesn't fire focus events the same way

        // Initialize auth only once
        this.initializeAuth();
      };

      // Check if DOM is already loaded (for dynamic script loading)
      if (document.readyState === "loading") {
        window.addEventListener("load", initializeDOM);
      } else {
        // DOM is already loaded, initialize immediately
        initializeDOM();
      }
    },

    flashScreen() {
      gsap.to(flashOverlay, {
        opacity: 0.15,
        duration: 0.1,
        onComplete: () => {
          gsap.to(flashOverlay, { opacity: 0, duration: 0.2 });
        },
      });
    },

    updateConnectionStatus(s) {
      if (!connectionStatus) return;
      connectionStatus.className = `connection-status ${s}`;
      const statusText = {
        connected: "Connected",
        connecting: "Connecting...",
        disconnected: "Disconnected",
      };
      connectionStatus.innerHTML = `<span>${statusText[s] || s}</span>`;
    },

    getWebSocketUrl() {
      const p = "ws:",
        s = currentSessionId || crypto.randomUUID(),
        u = currentUserId || crypto.randomUUID();
      return `${p}//localhost:8080/ws?session_id=${s}&user_id=${u}`;
    },

    async initializeAuth() {
      // Return existing promise if auth is in progress
      if (authPromise) {
        console.log("Auth already in progress, waiting...");
        return authPromise;
      }

      // Already authenticated
      if (
        currentSessionId &&
        currentUserId &&
        ws &&
        ws.readyState === WebSocket.OPEN
      ) {
        console.log("Already authenticated and connected");
        return;
      }

      // Create auth promise to prevent concurrent calls
      authPromise = (async () => {
        try {
          this.updateConnectionStatus("connecting");
          const p = window.location.pathname.split("/").filter((s) => s);
          const b = p.length > 0 ? p[0] : "default";
          const r = await fetch(
            `http://localhost:8080/api/auth?bot_name=${encodeURIComponent(b)}`,
          );
          const a = await r.json();
          currentUserId = a.user_id;
          currentSessionId = a.session_id;
          console.log("Auth successful:", { currentUserId, currentSessionId });
          this.connectWebSocket();
        } catch (e) {
          console.error("Failed to initialize auth:", e);
          this.updateConnectionStatus("disconnected");
          authPromise = null;
          setTimeout(() => this.initializeAuth(), 3000);
        } finally {
          authPromise = null;
        }
      })();

      return authPromise;
    },

    async loadSessions() {
      try {
        const r = await fetch("http://localhost:8080/api/sessions");
        const s = await r.json();
        const h = document.getElementById("history");
        h.innerHTML = "";
        s.forEach((session) => {
          const item = document.createElement("div");
          item.className = "history-item";
          item.textContent =
            session.title || `Session ${session.session_id.substring(0, 8)}`;
          item.onclick = () => this.switchSession(session.session_id);
          h.appendChild(item);
        });
      } catch (e) {
        console.error("Failed to load sessions:", e);
      }
    },

    async createNewSession() {
      try {
        const r = await fetch("http://localhost:8080/api/sessions", {
          method: "POST",
        });
        const s = await r.json();
        currentSessionId = s.session_id;
        hasReceivedInitialMessage = false;
        this.connectWebSocket();
        this.loadSessions();
        messagesDiv.innerHTML = "";
        this.clearSuggestions();
        if (isVoiceMode) {
          await this.stopVoiceSession();
          isVoiceMode = false;
          const v = document.getElementById("voiceToggle");
          v.textContent = "🎤 Voice Mode";
          voiceBtn.classList.remove("recording");
        }
      } catch (e) {
        console.error("Failed to create session:", e);
      }
    },

    switchSession(s) {
      currentSessionId = s;
      hasReceivedInitialMessage = false;
      this.connectWebSocket();
      if (isVoiceMode) {
        this.startVoiceSession();
      }
      sidebar.classList.remove("open");
    },

    connectWebSocket() {
      // Prevent multiple simultaneous connection attempts
      if (isConnecting) {
        console.log("Already connecting to WebSocket, skipping...");
        return;
      }
      if (
        ws &&
        (ws.readyState === WebSocket.OPEN ||
          ws.readyState === WebSocket.CONNECTING)
      ) {
        console.log("WebSocket already connected or connecting");
        return;
      }
      if (ws && ws.readyState !== WebSocket.CLOSED) {
        ws.close();
      }
      clearTimeout(reconnectTimeout);
      isConnecting = true;

      const u = this.getWebSocketUrl();
      console.log("Connecting to WebSocket:", u);
      ws = new WebSocket(u);
      ws.onmessage = (e) => {
        const r = JSON.parse(e.data);

        // Filter out welcome/connection messages that aren't BotResponse
        if (r.type === "connected" || !r.message_type) {
          console.log("Ignoring non-message:", r);
          return;
        }

        if (r.bot_id) {
          currentBotId = r.bot_id;
        }
        // Message type 2 is a bot response (not an event)
        // Message type 5 is context change
        if (r.message_type === 5) {
          isContextChange = true;
          return;
        }
        // Check if this is a special event message (has event field)
        if (r.event) {
          this.handleEvent(r.event, r.data || {});
          return;
        }
        this.processMessageContent(r);
      };
      ws.onopen = () => {
        console.log("Connected to WebSocket");
        isConnecting = false;
        this.updateConnectionStatus("connected");
        reconnectAttempts = 0;
        hasReceivedInitialMessage = false;
      };
      ws.onclose = (e) => {
        console.log("WebSocket disconnected:", e.code, e.reason);
        isConnecting = false;
        this.updateConnectionStatus("disconnected");
        if (isStreaming) {
          this.showContinueButton();
        }
        if (reconnectAttempts < maxReconnectAttempts) {
          reconnectAttempts++;
          const d = Math.min(1000 * reconnectAttempts, 10000);
          reconnectTimeout = setTimeout(() => {
            this.updateConnectionStatus("connecting");
            this.connectWebSocket();
          }, d);
        } else {
          this.updateConnectionStatus("disconnected");
        }
      };
      ws.onerror = (e) => {
        console.error("WebSocket error:", e);
        isConnecting = false;
        this.updateConnectionStatus("disconnected");
      };
    },

    processMessageContent(r) {
      if (isContextChange) {
        isContextChange = false;
        return;
      }

      // Ignore messages without content
      if (!r.content && r.is_complete !== true) {
        return;
      }

      if (r.suggestions && r.suggestions.length > 0) {
        this.handleSuggestions(r.suggestions);
      }
      if (r.is_complete) {
        if (isStreaming) {
          this.finalizeStreamingMessage();
          isStreaming = false;
          streamingMessageId = null;
          currentStreamingContent = "";
        } else if (r.content) {
          // Only add message if there's actual content
          this.addMessage("assistant", r.content, false);
        }
      } else {
        if (!isStreaming) {
          isStreaming = true;
          streamingMessageId = "streaming-" + Date.now();
          currentStreamingContent = r.content || "";
          this.addMessage(
            "assistant",
            currentStreamingContent,
            true,
            streamingMessageId,
          );
        } else {
          currentStreamingContent += r.content || "";
          this.updateStreamingMessage(currentStreamingContent);
        }
      }
    },

    handleEvent(t, d) {
      console.log("Event received:", t, d);
      switch (t) {
        case "thinking_start":
          this.showThinkingIndicator();
          break;
        case "thinking_end":
          this.hideThinkingIndicator();
          break;
        case "warn":
          this.showWarning(d.message);
          break;
        case "context_usage":
          // Context usage removed
          break;
        case "change_theme":
          if (d.color1) themeColor1 = d.color1;
          if (d.color2) themeColor2 = d.color2;
          if (d.logo_url) customLogoUrl = d.logo_url;
          if (d.title) document.title = d.title;
          this.applyTheme();
          break;
      }
    },

    showThinkingIndicator() {
      if (isThinking) return;
      const t = document.createElement("div");
      t.id = "thinking-indicator";
      t.className = "message-container";
      t.innerHTML = `<div class="assistant-message"><div class="assistant-avatar"></div><div class="thinking-indicator"><div class="thinking-dot"></div><div class="thinking-dot"></div><div class="thinking-dot"></div></div></div>`;
      if (messagesDiv) {
        messagesDiv.appendChild(t);
        if (!isUserScrolling) {
          this.scrollToBottom();
        }
      }
      isThinking = true;

      thinkingTimeout = setTimeout(() => {
        if (isThinking) {
          this.hideThinkingIndicator();
          this.showWarning("A resposta está demorando mais que o esperado...");
        }
      }, 30000);
    },

    hideThinkingIndicator() {
      if (!isThinking) return;
      const t = document.getElementById("thinking-indicator");
      if (t && t.parentNode) {
        t.remove();
      }
      isThinking = false;
    },

    showWarning(m) {
      const w = document.createElement("div");
      w.className = "warning-message";
      w.innerHTML = `⚠️ ${m}`;
      if (messagesDiv) {
        messagesDiv.appendChild(w);
        if (!isUserScrolling) {
          this.scrollToBottom();
        }
        setTimeout(() => {
          if (w.parentNode) {
            w.remove();
          }
        }, 5000);
      }
    },

    showContinueButton() {
      const c = document.createElement("div");
      c.className = "message-container";
      c.innerHTML = `<div class="assistant-message"><div class="assistant-avatar"></div><div class="assistant-message-content"><p>A conexão foi interrompida. Clique em "Continuar" para tentar recuperar a resposta.</p><button class="continue-button" onclick="this.parentElement.parentElement.parentElement.remove();">Continuar</button></div></div>`;
      if (messagesDiv) {
        messagesDiv.appendChild(c);
        if (!isUserScrolling) {
          this.scrollToBottom();
        }
      }
    },

    continueInterruptedResponse() {
      if (!ws || ws.readyState !== WebSocket.OPEN) {
        this.connectWebSocket();
      }
      if (ws && ws.readyState === WebSocket.OPEN) {
        const d = {
          bot_id: "default_bot",
          user_id: currentUserId,
          session_id: currentSessionId,
          channel: "web",
          content: "continue",
          message_type: 3,
          media_url: null,
          timestamp: new Date().toISOString(),
        };
        ws.send(JSON.stringify(d));
      }
      document.querySelectorAll(".continue-button").forEach((b) => {
        b.parentElement.parentElement.parentElement.remove();
      });
    },

    addMessage(role, content, streaming = false, msgId = null) {
      const m = document.createElement("div");
      m.className = "message-container";
      if (role === "user") {
        m.innerHTML = `<div class="user-message"><div class="user-message-content">${this.escapeHtml(content)}</div></div>`;
      } else if (role === "assistant") {
        m.innerHTML = `<div class="assistant-message"><div class="assistant-avatar"></div><div class="assistant-message-content markdown-content" id="${msgId || ""}">${streaming ? "" : marked.parse(content)}</div></div>`;
      } else if (role === "voice") {
        m.innerHTML = `<div class="assistant-message"><div class="assistant-avatar">🎤</div><div class="assistant-message-content">${content}</div></div>`;
      } else {
        m.innerHTML = `<div class="assistant-message"><div class="assistant-avatar"></div><div class="assistant-message-content">${content}</div></div>`;
      }
      if (messagesDiv) {
        messagesDiv.appendChild(m);
        if (!isUserScrolling) {
          this.scrollToBottom();
        }
      }
    },

    updateStreamingMessage(c) {
      const m = document.getElementById(streamingMessageId);
      if (m) {
        m.innerHTML = marked.parse(c);
        if (!isUserScrolling) {
          this.scrollToBottom();
        }
      }
    },

    finalizeStreamingMessage() {
      const m = document.getElementById(streamingMessageId);
      if (m) {
        m.innerHTML = marked.parse(currentStreamingContent);
        m.removeAttribute("id");
        if (!isUserScrolling) {
          this.scrollToBottom();
        }
      }
    },

    escapeHtml(t) {
      const d = document.createElement("div");
      d.textContent = t;
      return d.innerHTML;
    },

    clearSuggestions() {
      suggestionsContainer.innerHTML = "";
    },

    handleSuggestions(s) {
      const uniqueSuggestions = s.filter(
        (v, i, a) =>
          i ===
          a.findIndex((t) => t.text === v.text && t.context === v.context),
      );
      suggestionsContainer.innerHTML = "";
      uniqueSuggestions.forEach((v) => {
        const b = document.createElement("button");
        b.textContent = v.text;
        b.className = "suggestion-button";
        b.onclick = () => {
          this.setContext(v.context);
          if (messageInputEl) {
            messageInputEl.value = "";
          }
        };
        if (suggestionsContainer) {
          suggestionsContainer.appendChild(b);
        }
      });
    },

    async setContext(c) {
      try {
        const t = event?.target?.textContent || c;
        this.addMessage("user", t);
        messageInputEl.value = "";
        messageInputEl.value = "";
        if (ws && ws.readyState === WebSocket.OPEN) {
          pendingContextChange = new Promise((r) => {
            const h = (e) => {
              const d = JSON.parse(e.data);
              if (d.message_type === 5 && d.context_name === c) {
                ws.removeEventListener("message", h);
                r();
              }
            };
            ws.addEventListener("message", h);
            const s = {
              bot_id: currentBotId,
              user_id: currentUserId,
              session_id: currentSessionId,
              channel: "web",
              content: t,
              message_type: 4,
              is_suggestion: true,
              context_name: c,
              timestamp: new Date().toISOString(),
            };
            ws.send(JSON.stringify(s));
          });
          await pendingContextChange;
        } else {
          console.warn("WebSocket não está conectado. Tentando reconectar...");
          this.connectWebSocket();
        }
      } catch (err) {
        console.error("Failed to set context:", err);
      }
    },

    async sendMessage() {
      if (pendingContextChange) {
        await pendingContextChange;
        pendingContextChange = null;
      }
      const m = messageInputEl.value.trim();
      if (!m || !ws || ws.readyState !== WebSocket.OPEN) {
        if (!ws || ws.readyState !== WebSocket.OPEN) {
          this.showWarning("Conexão não disponível. Tentando reconectar...");
          this.connectWebSocket();
        }
        return;
      }
      if (isThinking) {
        this.hideThinkingIndicator();
      }
      this.addMessage("user", m);
      const d = {
        bot_id: currentBotId,
        user_id: currentUserId,
        session_id: currentSessionId,
        channel: "web",
        content: m,
        message_type: 1,
        media_url: null,
        timestamp: new Date().toISOString(),
      };
      ws.send(JSON.stringify(d));
      messageInputEl.value = "";
      messageInputEl.focus();
    },

    async toggleVoiceMode() {
      isVoiceMode = !isVoiceMode;
      const v = document.getElementById("voiceToggle");
      if (isVoiceMode) {
        v.textContent = "🔴 Stop Voice";
        v.classList.add("recording");
        await this.startVoiceSession();
      } else {
        v.textContent = "🎤 Voice Mode";
        v.classList.remove("recording");
        await this.stopVoiceSession();
      }
    },

    async startVoiceSession() {
      if (!currentSessionId) return;
      try {
        const r = await fetch("http://localhost:8080/api/voice/start", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            session_id: currentSessionId,
            user_id: currentUserId,
          }),
        });
        const d = await r.json();
        if (d.token) {
          await this.connectToVoiceRoom(d.token);
          this.startVoiceRecording();
        }
      } catch (e) {
        console.error("Failed to start voice session:", e);
        this.showWarning("Falha ao iniciar modo de voz");
      }
    },

    async stopVoiceSession() {
      if (!currentSessionId) return;
      try {
        await fetch("http://localhost:8080/api/voice/stop", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ session_id: currentSessionId }),
        });
        if (voiceRoom) {
          voiceRoom.disconnect();
          voiceRoom = null;
        }
        if (mediaRecorder && mediaRecorder.state === "recording") {
          mediaRecorder.stop();
        }
      } catch (e) {
        console.error("Failed to stop voice session:", e);
      }
    },

    async connectToVoiceRoom(t) {
      try {
        const r = new LiveKitClient.Room();
        const p = "ws:",
          u = `${p}//localhost:8080/voice`;
        await r.connect(u, t);
        voiceRoom = r;
        r.on("dataReceived", (d) => {
          const dc = new TextDecoder(),
            m = dc.decode(d);
          try {
            const j = JSON.parse(m);
            if (j.type === "voice_response") {
              this.addMessage("assistant", j.text);
            }
          } catch (e) {
            console.log("Voice data:", m);
          }
        });
        const l = await LiveKitClient.createLocalTracks({
          audio: true,
          video: false,
        });
        for (const k of l) {
          await r.localParticipant.publishTrack(k);
        }
      } catch (e) {
        console.error("Failed to connect to voice room:", e);
        this.showWarning("Falha na conexão de voz");
      }
    },

    startVoiceRecording() {
      if (!navigator.mediaDevices) {
        console.log("Media devices not supported");
        return;
      }
      navigator.mediaDevices
        .getUserMedia({ audio: true })
        .then((s) => {
          mediaRecorder = new MediaRecorder(s);
          audioChunks = [];
          mediaRecorder.ondataavailable = (e) => {
            audioChunks.push(e.data);
          };
          mediaRecorder.onstop = () => {
            const a = new Blob(audioChunks, { type: "audio/wav" });
            this.simulateVoiceTranscription();
          };
          mediaRecorder.start();
          setTimeout(() => {
            if (mediaRecorder && mediaRecorder.state === "recording") {
              mediaRecorder.stop();
              setTimeout(() => {
                if (isVoiceMode) {
                  this.startVoiceRecording();
                }
              }, 1000);
            }
          }, 5000);
        })
        .catch((e) => {
          console.error("Error accessing microphone:", e);
          this.showWarning("Erro ao acessar microfone");
        });
    },

    simulateVoiceTranscription() {
      const p = [
        "Olá, como posso ajudá-lo hoje?",
        "Entendo o que você está dizendo",
        "Esse é um ponto interessante",
        "Deixe-me pensar sobre isso",
        "Posso ajudá-lo com isso",
        "O que você gostaria de saber?",
        "Isso parece ótimo",
        "Estou ouvindo sua voz",
      ];
      const r = p[Math.floor(Math.random() * p.length)];
      if (voiceRoom) {
        const m = {
          type: "voice_input",
          content: r,
          timestamp: new Date().toISOString(),
        };
        voiceRoom.localParticipant.publishData(
          new TextEncoder().encode(JSON.stringify(m)),
          LiveKitClient.DataPacketKind.RELIABLE,
        );
      }
      this.addMessage("voice", `🎤 ${r}`);
    },

    scrollToBottom() {
      if (messagesDiv) {
        messagesDiv.scrollTop = messagesDiv.scrollHeight;
        isUserScrolling = false;
        if (scrollToBottomBtn) {
          scrollToBottomBtn.classList.remove("visible");
        }
      }
    },
  };

  const returnValue = {
    init: init,
    current: current,
    search: search,
    selectedChat: selectedChat,
    navItems: navItems,
    chats: chats,
    get filteredChats() {
      return chats.filter((chat) =>
        chat.name.toLowerCase().includes(search.toLowerCase()),
      );
    },
    toggleSidebar: toggleSidebar,
    toggleTheme: toggleTheme,
    applyTheme: applyTheme,
    flashScreen: flashScreen,
    updateConnectionStatus: updateConnectionStatus,
    getWebSocketUrl: getWebSocketUrl,
    initializeAuth: initializeAuth,
    loadSessions: loadSessions,
    createNewSession: createNewSession,
    switchSession: switchSession,
    connectWebSocket: connectWebSocket,
    processMessageContent: processMessageContent,
    handleEvent: handleEvent,
    showThinkingIndicator: showThinkingIndicator,
    hideThinkingIndicator: hideThinkingIndicator,
    showWarning: showWarning,
    showContinueButton: showContinueButton,
    continueInterruptedResponse: continueInterruptedResponse,
    addMessage: addMessage,
    updateStreamingMessage: updateStreamingMessage,
    finalizeStreamingMessage: finalizeStreamingMessage,
    escapeHtml: escapeHtml,
    clearSuggestions: clearSuggestions,
    handleSuggestions: handleSuggestions,
    setContext: setContext,
    sendMessage: sendMessage,
    toggleVoiceMode: toggleVoiceMode,
    startVoiceSession: startVoiceSession,
    stopVoiceSession: stopVoiceSession,
    connectToVoiceRoom: connectToVoiceRoom,
    startVoiceRecording: startVoiceRecording,
    simulateVoiceTranscription: simulateVoiceTranscription,
    scrollToBottom: scrollToBottom,
    cleanup: function () {
      // Cleanup WebSocket connection
      if (ws) {
        ws.close();
        ws = null;
      }
      // Clear any pending timeouts/intervals
      isConnecting = false;
      isInitialized = false;
    },
  };

  // Cache and return the singleton instance
  chatAppInstance = returnValue;
  return returnValue;
}

// Initialize the app - expose globally for dynamic loading
window.chatAppInstance = chatApp();

// Auto-initialize if we're already on the chat section
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => {
    const hash = window.location.hash.substring(1);
    if (hash === "chat" || hash === "" || !hash) {
      window.chatAppInstance.init();
    }
  });
} else {
  // If script is loaded dynamically, section-shown event will trigger init
  const chatSection = document.getElementById("section-chat");
  if (chatSection) {
    window.chatAppInstance.init();
  }
}

// Listen for section being shown
document.addEventListener("section-shown", function (e) {
  if (e.target.id === "section-chat" && window.chatAppInstance) {
    console.log("Chat section shown, initializing...");
    window.chatAppInstance.init();
  }
});

// Listen for section changes to cleanup when leaving chat
document.addEventListener("section-hidden", function (e) {
  if (
    e.target.id === "section-chat" &&
    chatAppInstance &&
    chatAppInstance.cleanup
  ) {
    chatAppInstance.cleanup();
  }
});
