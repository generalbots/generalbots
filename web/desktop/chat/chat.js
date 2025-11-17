function chatApp() {

  // Core state variables (shared via closure)
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
    currentTheme = 'auto',
    themeColor1 = null,
    themeColor2 = null,
    customLogoUrl = null,
    contextUsage = 0,
    isUserScrolling = false,
    autoScrollEnabled = true,
    isContextChange = false;

  const maxReconnectAttempts = 5;

  // DOM references (cached for performance)
  let messagesDiv, input, sendBtn, voiceBtn, connectionStatus, flashOverlay, suggestionsContainer, floatLogo, sidebar, themeBtn, scrollToBottomBtn, contextIndicator, contextPercentage, contextProgressBar, sidebarTitle;

  marked.setOptions({ breaks: true, gfm: true });

  return {
    // ----------------------------------------------------------------------
    // UI state (mirrors the structure used in driveApp)
    // ----------------------------------------------------------------------
    current: 'All Chats',
    search: '',
    selectedChat: null,
    navItems: [
      { name: 'All Chats', icon: '💬' },
      { name: 'Direct', icon: '👤' },
      { name: 'Groups', icon: '👥' },
      { name: 'Archived', icon: '🗄' }
    ],
    chats: [
      { id: 1, name: 'General Bot Support', icon: '🤖', lastMessage: 'How can I help you?', time: '10:15 AM', status: 'Online' },
      { id: 2, name: 'Project Alpha', icon: '🚀', lastMessage: 'Launch scheduled for tomorrow.', time: 'Yesterday', status: 'Active' },
      { id: 3, name: 'Team Stand‑up', icon: '🗣️', lastMessage: 'Done with the UI updates.', time: '2 hrs ago', status: 'Active' },
      { id: 4, name: 'Random Chat', icon: '🎲', lastMessage: 'Did you see the game last night?', time: '5 hrs ago', status: 'Idle' },
      { id: 5, name: 'Support Ticket #1234', icon: '🛠️', lastMessage: 'Issue resolved, closing ticket.', time: '3 days ago', status: 'Closed' }
    ],
    get filteredChats() {
      return this.chats.filter(chat =>
        chat.name.toLowerCase().includes(this.search.toLowerCase())
      );
    },

    // ----------------------------------------------------------------------
    // UI helpers (formerly standalone functions)
    // ----------------------------------------------------------------------
    toggleSidebar() {
      sidebar.classList.toggle('open');
    },

    toggleTheme() {
      const themes = ['auto', 'dark', 'light'];
      const savedTheme = localStorage.getItem('gb-theme') || 'auto';
      const idx = themes.indexOf(savedTheme);
      const newTheme = themes[(idx + 1) % themes.length];
      localStorage.setItem('gb-theme', newTheme);
      currentTheme = newTheme;
      this.applyTheme();
      this.updateThemeButton();
    },

    applyTheme() {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      let theme = currentTheme;
      if (theme === 'auto') {
        theme = prefersDark ? 'dark' : 'light';
      }
      document.documentElement.setAttribute('data-theme', theme);
      if (themeColor1 && themeColor2) {
        const root = document.documentElement;
        root.style.setProperty('--bg', theme === 'dark' ? themeColor2 : themeColor1);
        root.style.setProperty('--fg', theme === 'dark' ? themeColor1 : themeColor2);
      }
      if (customLogoUrl) {
        document.documentElement.style.setProperty('--logo-url', `url('${customLogoUrl}')`);
      }
    },

    // ----------------------------------------------------------------------
    // Lifecycle / event handlers
    // ----------------------------------------------------------------------
    init() {
      document.addEventListener('ready', () => {
        // Assign DOM elements after the document is ready
        messagesDiv = document.getElementById("messages");
        input = document.getElementById("messageInput");
        sendBtn = document.getElementById("sendBtn");
        voiceBtn = document.getElementById("voiceBtn");
        connectionStatus = document.getElementById("connectionStatus");
        flashOverlay = document.getElementById("flashOverlay");
        suggestionsContainer = document.getElementById("suggestions");
        floatLogo = document.getElementById("floatLogo");
        sidebar = document.getElementById("sidebar");
        themeBtn = document.getElementById("themeBtn");
        scrollToBottomBtn = document.getElementById("scrollToBottom");
        contextIndicator = document.getElementById("contextIndicator");
        contextPercentage = document.getElementById("contextPercentage");
        contextProgressBar = document.getElementById("contextProgressBar");
        sidebarTitle = document.getElementById("sidebarTitle");

        // Theme initialization and focus
        const savedTheme = localStorage.getItem('gb-theme') || 'auto';
        currentTheme = savedTheme;
        this.applyTheme();
        window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
          if (currentTheme === 'auto') {
            this.applyTheme();
          }
        });
        input.focus();

        // UI event listeners
        document.addEventListener('click', (e) => {
          if (sidebar.classList.contains('open') && !sidebar.contains(e.target) && !floatLogo.contains(e.target)) {
            sidebar.classList.remove('open');
          }
        });

        messagesDiv.addEventListener('scroll', () => {
          const isAtBottom = messagesDiv.scrollHeight - messagesDiv.scrollTop <= messagesDiv.clientHeight + 100;
          if (!isAtBottom) {
            isUserScrolling = true;
            scrollToBottomBtn.classList.add('visible');
          } else {
            isUserScrolling = false;
            scrollToBottomBtn.classList.remove('visible');
          }
        });

        scrollToBottomBtn.addEventListener('click', () => {
          this.scrollToBottom();
        });

        sendBtn.onclick = () => this.sendMessage();
        input.addEventListener("keypress", e => { if (e.key === "Enter") this.sendMessage(); });
        window.addEventListener("focus", () => {
          if (!ws || ws.readyState !== WebSocket.OPEN) {
            this.connectWebSocket();
          }
        });

        // Start authentication flow
        this.initializeAuth();
      });
    },

    updateContextUsage(u) {
      contextUsage = u;
      const p = Math.min(100, Math.round(u * 100));
      contextPercentage.textContent = `${p}%`;
      contextProgressBar.style.width = `${p}%`;
      contextIndicator.classList.remove('visible');
    },

    flashScreen() {
      gsap.to(flashOverlay, { opacity: 0.15, duration: 0.1, onComplete: () => {
        gsap.to(flashOverlay, { opacity: 0, duration: 0.2 });
      } });
    },

    updateConnectionStatus(s) {
      connectionStatus.className = `connection-status ${s}`;
    },

    getWebSocketUrl() {
      const p = "ws:", s = currentSessionId || crypto.randomUUID(), u = currentUserId || crypto.randomUUID();
      return `${p}//localhost:8080/ws?session_id=${s}&user_id=${u}`;
    },

    async initializeAuth() {
      try {
        this.updateConnectionStatus("connecting");
        const p = window.location.pathname.split('/').filter(s => s);
        const b = p.length > 0 ? p[0] : 'default';
        const r = await fetch(`http://localhost:8080/api/auth?bot_name=${encodeURIComponent(b)}`);
        const a = await r.json();
        currentUserId = a.user_id;
        currentSessionId = a.session_id;
        this.connectWebSocket();
        this.loadSessions();
      } catch (e) {
        console.error("Failed to initialize auth:", e);
        this.updateConnectionStatus("disconnected");
        setTimeout(() => this.initializeAuth(), 3000);
      }
    },

    async loadSessions() {
      try {
        const r = await fetch("http://localhost:8080/api/sessions");
        const s = await r.json();
        const h = document.getElementById("history");
        h.innerHTML = "";
        s.forEach(session => {
          const item = document.createElement('div');
          item.className = 'history-item';
          item.textContent = session.title || `Session ${session.session_id.substring(0, 8)}`;
          item.onclick = () => this.switchSession(session.session_id);
          h.appendChild(item);
        });
      } catch (e) {
        console.error("Failed to load sessions:", e);
      }
    },

    async createNewSession() {
      try {
        const r = await fetch("http://localhost:8080/api/sessions", { method: "POST" });
        const s = await r.json();
        currentSessionId = s.session_id;
        hasReceivedInitialMessage = false;
        this.connectWebSocket();
        this.loadSessions();
        messagesDiv.innerHTML = "";
        this.clearSuggestions();
        this.updateContextUsage(0);
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
      this.loadSessionHistory(s);
      this.connectWebSocket();
      if (isVoiceMode) {
        this.startVoiceSession();
      }
      sidebar.classList.remove('open');
    },

    async loadSessionHistory(s) {
      try {
        const r = await fetch(`http://localhost:8080/api/sessions/${s}`);
        const h = await r.json();
        const m = document.getElementById("messages");
        m.innerHTML = "";
        if (h.length === 0) {
          this.updateContextUsage(0);
        } else {
          h.forEach(([role, content]) => {
            this.addMessage(role, content, false);
          });
          this.updateContextUsage(h.length / 20);
        }
      } catch (e) {
        console.error("Failed to load session history:", e);
      }
    },

    connectWebSocket() {
      if (ws) {
        ws.close();
      }
      clearTimeout(reconnectTimeout);
      const u = this.getWebSocketUrl();
      ws = new WebSocket(u);
      ws.onmessage = (e) => {
        const r = JSON.parse(e.data);
        if (r.bot_id) {
          currentBotId = r.bot_id;
        }
        if (r.message_type === 2) {
          const d = JSON.parse(r.content);
          this.handleEvent(d.event, d.data);
          return;
        }
        if (r.message_type === 5) {
          isContextChange = true;
          return;
        }
        this.processMessageContent(r);
      };
      ws.onopen = () => {
        console.log("Connected to WebSocket");
        this.updateConnectionStatus("connected");
        reconnectAttempts = 0;
        hasReceivedInitialMessage = false;
      };
      ws.onclose = (e) => {
        console.log("WebSocket disconnected:", e.code, e.reason);
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
        this.updateConnectionStatus("disconnected");
      };
    },

    processMessageContent(r) {
      if (isContextChange) {
        isContextChange = false;
        return;
      }
      if (r.context_usage !== undefined) {
        this.updateContextUsage(r.context_usage);
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
        } else {
          this.addMessage("assistant", r.content, false);
        }
      } else {
        if (!isStreaming) {
          isStreaming = true;
          streamingMessageId = "streaming-" + Date.now();
          currentStreamingContent = r.content || "";
          this.addMessage("assistant", currentStreamingContent, true, streamingMessageId);
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
          this.updateContextUsage(d.usage);
          break;
        case "change_theme":
          if (d.color1) themeColor1 = d.color1;
          if (d.color2) themeColor2 = d.color2;
          if (d.logo_url) customLogoUrl = d.logo_url;
          if (d.title) document.title = d.title;
          if (d.logo_text) {
            sidebarTitle.textContent = d.logo_text;
          }
          this.applyTheme();
          break;
      }
    },

    showThinkingIndicator() {
      if (isThinking) return;
      const t = document.createElement("div");
      t.id = "thinking-indicator";
      t.className = "message-container";
      t.innerHTML = `<div class="assistant-message"><div class="assistant-avatar"></div><div class="thinking-indicator"><div class="typing-dots"><div class="typing-dot"></div><div class="typing-dot"></div><div class="typing-dot"></div></div></div></div>`;
      messagesDiv.appendChild(t);
      gsap.to(t, { opacity: 1, y: 0, duration: .3, ease: "power2.out" });
      if (!isUserScrolling) {
        this.scrollToBottom();
      }
      thinkingTimeout = setTimeout(() => {
        if (isThinking) {
          this.hideThinkingIndicator();
          this.showWarning("O servidor pode estar ocupado. A resposta está demorando demais.");
        }
      }, 60000);
      isThinking = true;
    },

    hideThinkingIndicator() {
      if (!isThinking) return;
      const t = document.getElementById("thinking-indicator");
      if (t) {
        gsap.to(t, { opacity: 0, duration: .2, onComplete: () => { if (t.parentNode) { t.remove(); } } });
      }
      if (thinkingTimeout) {
        clearTimeout(thinkingTimeout);
        thinkingTimeout = null;
      }
      isThinking = false;
    },

    showWarning(m) {
      const w = document.createElement("div");
      w.className = "warning-message";
      w.innerHTML = `⚠️ ${m}`;
      messagesDiv.appendChild(w);
      gsap.from(w, { opacity: 0, y: 20, duration: .4, ease: "power2.out" });
      if (!isUserScrolling) {
        this.scrollToBottom();
      }
      setTimeout(() => {
        if (w.parentNode) {
          gsap.to(w, { opacity: 0, duration: .3, onComplete: () => w.remove() });
        }
      }, 5000);
    },

    showContinueButton() {
      const c = document.createElement("div");
      c.className = "message-container";
      c.innerHTML = `<div class="assistant-message"><div class="assistant-avatar"></div><div class="assistant-message-content"><p>A conexão foi interrompida. Clique em "Continuar" para tentar recuperar a resposta.</p><button class="continue-button" onclick="this.parentElement.parentElement.parentElement.remove();">Continuar</button></div></div>`;
      messagesDiv.appendChild(c);
      gsap.to(c, { opacity: 1, y: 0, duration: .5, ease: "power2.out" });
      if (!isUserScrolling) {
        this.scrollToBottom();
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
          timestamp: new Date().toISOString()
        };
        ws.send(JSON.stringify(d));
      }
      document.querySelectorAll(".continue-button").forEach(b => { b.parentElement.parentElement.parentElement.remove(); });
    },

    addMessage(role, content, streaming = false, msgId = null) {
      const m = document.createElement("div");
      m.className = "message-container";
      if (role === "user") {
        m.innerHTML = `<div class="user-message"><div class="user-message-content">${this.escapeHtml(content)}</div></div>`;
        this.updateContextUsage(contextUsage + .05);
      } else if (role === "assistant") {
        m.innerHTML = `<div class="assistant-message"><div class="assistant-avatar"></div><div class="assistant-message-content markdown-content" id="${msgId || ""}">${streaming ? "" : marked.parse(content)}</div></div>`;
        this.updateContextUsage(contextUsage + .03);
      } else if (role === "voice") {
        m.innerHTML = `<div class="assistant-message"><div class="assistant-avatar">🎤</div><div class="assistant-message-content">${content}</div></div>`;
      } else {
        m.innerHTML = `<div class="assistant-message"><div class="assistant-avatar"></div><div class="assistant-message-content">${content}</div></div>`;
      }
      messagesDiv.appendChild(m);
      gsap.to(m, { opacity: 1, y: 0, duration: .5, ease: "power2.out" });
      if (!isUserScrolling) {
        this.scrollToBottom();
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
      suggestionsContainer.innerHTML = '';
    },

    handleSuggestions(s) {
      const uniqueSuggestions = s.filter((v, i, a) => i === a.findIndex(t => t.text === v.text && t.context === v.context));
      suggestionsContainer.innerHTML = '';
      uniqueSuggestions.forEach(v => {
        const b = document.createElement('button');
        b.textContent = v.text;
        b.className = 'suggestion-button';
        b.onclick = () => { this.setContext(v.context); input.value = ''; };
        suggestionsContainer.appendChild(b);
      });
    },

    async setContext(c) {
      try {
        const t = event?.target?.textContent || c;
        this.addMessage("user", t);
        input.value = '';
        if (ws && ws.readyState === WebSocket.OPEN) {
          pendingContextChange = new Promise(r => {
            const h = e => {
              const d = JSON.parse(e.data);
              if (d.message_type === 5 && d.context_name === c) {
                ws.removeEventListener('message', h);
                r();
              }
            };
            ws.addEventListener('message', h);
            const s = { bot_id: currentBotId, user_id: currentUserId, session_id: currentSessionId, channel: "web", content: t, message_type: 4, is_suggestion: true, context_name: c, timestamp: new Date().toISOString() };
            ws.send(JSON.stringify(s));
          });
          await pendingContextChange;
          const x = document.getElementById('contextIndicator');
          if (x) { document.getElementById('contextPercentage').textContent = c; }
        } else {
          console.warn("WebSocket não está conectado. Tentando reconectar...");
          this.connectWebSocket();
        }
      } catch (err) {
        console.error('Failed to set context:', err);
      }
    },

    async sendMessage() {
      if (pendingContextChange) {
        await pendingContextChange;
        pendingContextChange = null;
      }
      const m = input.value.trim();
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
      const d = { bot_id: currentBotId, user_id: currentUserId, session_id: currentSessionId, channel: "web", content: m, message_type: 1, media_url: null, timestamp: new Date().toISOString() };
      ws.send(JSON.stringify(d));
      input.value = "";
      input.focus();
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
          body: JSON.stringify({ session_id: currentSessionId, user_id: currentUserId })
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
          body: JSON.stringify({ session_id: currentSessionId })
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
        const p = "ws:", u = `${p}//localhost:8080/voice`;
        await r.connect(u, t);
        voiceRoom = r;
        r.on("dataReceived", d => {
          const dc = new TextDecoder(), m = dc.decode(d);
          try {
            const j = JSON.parse(m);
            if (j.type === "voice_response") {
              this.addMessage("assistant", j.text);
            }
          } catch (e) {
            console.log("Voice data:", m);
          }
        });
        const l = await LiveKitClient.createLocalTracks({ audio: true, video: false });
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
      navigator.mediaDevices.getUserMedia({ audio: true }).then(s => {
        mediaRecorder = new MediaRecorder(s);
        audioChunks = [];
        mediaRecorder.ondataavailable = e => { audioChunks.push(e.data); };
        mediaRecorder.onstop = () => { const a = new Blob(audioChunks, { type: "audio/wav" }); this.simulateVoiceTranscription(); };
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
      }).catch(e => {
        console.error("Error accessing microphone:", e);
        this.showWarning("Erro ao acessar microfone");
      });
    },

    simulateVoiceTranscription() {
      const p = ["Olá, como posso ajudá-lo hoje?", "Entendo o que você está dizendo", "Esse é um ponto interessante", "Deixe-me pensar sobre isso", "Posso ajudá-lo com isso", "O que você gostaria de saber?", "Isso parece ótimo", "Estou ouvindo sua voz"];
      const r = p[Math.floor(Math.random() * p.length)];
      if (voiceRoom) {
        const m = { type: "voice_input", content: r, timestamp: new Date().toISOString() };
        voiceRoom.localParticipant.publishData(new TextEncoder().encode(JSON.stringify(m)), LiveKitClient.DataPacketKind.RELIABLE);
      }
      this.addMessage("voice", `🎤 ${r}`);
    },

    scrollToBottom() {
      messagesDiv.scrollTop = messagesDiv.scrollHeight;
      isUserScrolling = false;
      scrollToBottomBtn.classList.remove('visible');
    }
  };
}

// Initialize the app
chatApp().init();
