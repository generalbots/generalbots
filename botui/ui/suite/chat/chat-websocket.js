var CHAT_FRIENDLY_ERRORS = {
  "Disconnected from chat server": "Chat disconnected. Reconnecting...",
  "Failed to connect to chat server": "Could not reach the chat server. Retrying...",
  "WebSocket connection failed": "Connection lost. Trying to reconnect...",
};

function notify(message, type) {
  type = type || "info";
  var friendly = CHAT_FRIENDLY_ERRORS[message] || message;
  if (window.GBAlerts) {
    if (type === "error") {
      window.GBAlerts.warning("Chat", friendly);
    } else {
      window.GBAlerts.info("Chat", friendly);
    }
  }
}

function updateConnectionStatus(status) {
  var statusEl = document.getElementById("connectionStatus");
  if (!statusEl) return;
  statusEl.className = "connection-status " + status;
  var statusText = statusEl.querySelector(".connection-text");
  if (statusText) {
    switch (status) {
      case "connected":
        statusText.textContent = tr("chat-online", "Connected");
        statusEl.style.display = "none";
        if (window.GBAppLifecycle) GBAppLifecycle.setState(null);
        break;
      case "disconnected":
        statusText.textContent = tr("chat-offline", "Disconnected");
        statusEl.style.display = "flex";
        if (window.GBAppLifecycle) GBAppLifecycle.setState("error", tr("chat-offline", "Disconnected"));
        break;
      case "connecting":
        statusText.textContent = tr("chat-connecting", "Connecting...");
        statusEl.style.display = "flex";
        if (window.GBAppLifecycle) GBAppLifecycle.setState("loading", tr("chat-connecting", "Connecting…"));
        break;
    }
  }
}

function tr(key, fallback) {
  if (window.i18n && typeof window.i18n.t === "function") {
    var value = window.i18n.t(key);
    return value === key ? fallback : value;
  }
  return fallback;
}

// A private bot refuses the WebSocket upgrade with 401, and the browser
// WebSocket API hides that response — the window only saw an endless
// "Disconnected / connecting" flap. Probe the bot first so a private bot with
// an anonymous visitor explains the real reason and offers a way in.
function probeBotAccess(done) {
  if (window.__BOT_IS_PUBLIC__) { ChatState.botAccess = "allowed"; done("allowed"); return; }
  if (ChatState.botAccess) { done(ChatState.botAccess); return; }
  var bot = ChatState.currentBotName || window.__INITIAL_BOT_NAME__ || "default";
  fetch("/api/bot/public?bot_name=" + encodeURIComponent(bot), { credentials: "same-origin" })
    .then(function (r) { return r.ok ? r.json() : {}; })
    .then(function (info) {
      var isPublic = info.is_public === true || String(info.is_public) === "true";
      var token = (window.GBSecurity && typeof GBSecurity.getToken === "function")
        ? GBSecurity.getToken() : null;
      ChatState.botAccess = (isPublic || token) ? "allowed" : "auth_required";
      done(ChatState.botAccess);
    })
    .catch(function () { ChatState.botAccess = "allowed"; done("allowed"); });
}

function showAuthRequired(botName) {
  var statusEl = document.getElementById("connectionStatus");
  if (statusEl) {
    statusEl.className = "connection-status disconnected";
    var statusText = statusEl.querySelector(".connection-text");
    if (statusText) {
      statusText.textContent = (window.i18n && window.i18n.t("chat-private-signin")) || "Private bot — sign in required";
    }
    statusEl.style.display = "flex";
  }
  var login = window.GB_LOGIN_URL || "/login";
  if (statusEl) {
    statusEl.title = "Sign in at " + login;
  }
  if (window.GBAppLifecycle) {
    GBAppLifecycle.setState("error", "This bot is private. Sign in to continue.");
  }
  notify("This bot is private — sign in to continue", "error");
  console.warn("[WS] bot '" + botName + "' is private; WebSocket not attempted");
}

// One dial at a time: several boot modules call connectWebSocket() while the
// first probe is still in flight, and each late callback used to dial another
// socket. The loser stayed open as an orphan while ChatState.ws pointed at a
// dead one, so the composer kept writing into a closed socket.
var chatDialSeq = 0;

function connectWebSocket() {
  var seq = ++chatDialSeq;
  if (ChatState.ws) {
    // Detach lifecycles first: the superseded socket must not schedule dials.
    try { ChatState.ws.onclose = null; ChatState.ws.onopen = null; } catch (e) {}
    try { ChatState.ws.close(); } catch (e) {}
  }
  updateConnectionStatus("connecting");
  probeBotAccess(function (state) {
    if (seq !== chatDialSeq) return;
    if (state === "auth_required") {
      showAuthRequired(ChatState.currentBotName);
      return;
    }
    dialChatSocket(seq);
  });
}

function dialChatSocket(seq) {
  // The socket must carry the suite token: the server verifies it and uses
  // that identity over the (possibly stale) user_id the page remembers.
  var socketToken = "";
  try {
    socketToken = (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) ||
      localStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("gb-access-token") || "";
  } catch (e) { socketToken = ""; }
  var url = WS_URL +
    (WS_URL.indexOf("?") === -1 ? "?" : "&") +
    "session_id=" + ChatState.currentSessionId +
    "&user_id=" + ChatState.currentUserId +
    "&bot_name=" + ChatState.currentBotName +
    (socketToken ? "&access_token=" + encodeURIComponent(socketToken) : "");

  var ws = (ChatState.ws = new WebSocket(url));
  if (window.GBAppLifecycle) GBAppLifecycle.socket("chat", ChatState.ws);  ChatState.ws.onmessage = function (event) {
    try {
      var data = JSON.parse(event.data);
      // Any frame proves the socket is alive; the send guard in chat-init
      // uses this to decide whether a message needs to be resent.
      ChatState.lastFrameAt = Date.now();

      // Server receipt for a chat turn. The send guard in chat-init waits for
      // this instead of "any frame", so a message accepted by the socket but
      // lost in a dying connection is resent instead of silently ignored.
      if (data.type === "message_received") {
        if (ChatState.pendingSend) ChatState.pendingSend.acked = true;
        return;
      }

      if (data.type === "connected") {
        ChatState.reconnectAttempts = 0;
        ChatState.currentUserId = data.user_id || ChatState.currentUserId;
        return;
      }

      if (data.event) {
        if (data.event === "change_theme") applyThemeData(data.data || {});
        return;
      }

      if (data.content && typeof data.content === "string") {
        try {
          var contentObj = JSON.parse(data.content);
          if (contentObj.event === "change_theme") {
            applyThemeData(contentObj.data || {});
            return;
          }
        } catch (e) {}
      }

      if (data.type === "chat_status") {
        if (typeof setChatStatus === "function") setChatStatus(data.text || "");
        return;
      }
      if (data.type === "gb.steering_queued") {
        if (typeof window.showToast === "function") {
          window.showToast(data.message || "Queued for the next step.", "info");
        }
        return;
      }
      // #1668 — every run frame feeds the timeline, regardless of agent mode.
      if (data.seq) {
        try {
          window.dispatchEvent(new CustomEvent("gb-run-frame", { detail: { frame: data } }));
        } catch (e) { /* CustomEvent unavailable */ }
      }
      if (data.type === "run_resume_done") {
        try {
          window.dispatchEvent(new CustomEvent("run_resume_done", { detail: data }));
        } catch (e) { /* CustomEvent unavailable */ }
      }
      if (window.RunTimeline) {
        window.RunTimeline.handleFrame(data);
      }
      // #1669 — trusted catalog surfaces and structured questions.
      if (data.type === "CUSTOM" && data.data && window.SurfaceRenderer) {
        if (data.data.name === "gb.surface") {
          window.SurfaceRenderer.renderFromFrame(data);
        } else if (data.data.name === "gb.question") {
          window.SurfaceRenderer.renderQuestion(data);
        } else if (data.data.name === "gb.plan") {
          window.SurfaceRenderer.renderFromFrame(data);
        } else if (data.data.name === "gb.surface_update") {
          window.SurfaceRenderer.applyUpdate(data);
        } else if (data.data.name === "gb.app" && window.AppFrame) {
          window.AppFrame.renderFromFrame(data);
        }
      }
      if (window.AgentMode && data.type &&
        ["thought_process", "terminal_output", "browser_ready", "step_progress", "step_complete", "todo_update", "agent_status", "file_created"].indexOf(data.type) !== -1) {
        window.AgentMode.handleMessage(data);
      }

      if (data.css && typeof data.css === 'string' && data.css.length > 0) {
        var cssId = 'bot-style-' + ChatState.currentBotName;
        var existing = document.getElementById(cssId);
        if (!existing) {
          var styleEl = document.createElement('style');
          styleEl.id = cssId;
          styleEl.textContent = data.css;
          document.head.appendChild(styleEl);
          window.__cssInjected = (window.__cssInjected || 0) + 1;
        }
      }

      if (data.message_type === MessageType.BOT_RESPONSE) {
        var contentPreview = data.content ? data.content.substring(0, 200) : '(empty)';
        console.log("[WS] processMessage: complete=" + data.is_complete + " content_preview=" + contentPreview);
        processMessage(data);
      }

      if (data.message_type === MessageType.UI_ACTION && data.plan) {
        if (window.GBUiOrchestrator) {
          window.GBUiOrchestrator.executePlan(data.plan);
        } else {
          console.warn("[WS] UI_ACTION received but GBUiOrchestrator not loaded");
        }
      }
    } catch (e) { console.error("[WS] onmessage error:", e); }
  };

  ChatState.ws.onclose = function () {
    // A newer socket already replaced this one (deliberate reconnect/
    // takeover): the replacement's lifecycle owns the connection now.
    // Scheduling another dial here would close the healthy replacement
    // and re-arm the loop — the endless 1s connect/close flap seen after
    // chat windows are closed and re-opened (#1288 family).
    if (ChatState.ws !== ws) return;
    updateConnectionStatus("disconnected");
    if (!ChatState.disconnectNotified) {
      notify("Disconnected from chat server", "error");
      ChatState.disconnectNotified = true;
    }
    // Keep dialling forever with a capped backoff. The old code stopped after
    // five attempts and left the app permanently "Disconnected" — no thinking
    // indicator, no reply, and the composer looked broken.
    ChatState.reconnectAttempts++;
    var delay = Math.min(1000 * ChatState.reconnectAttempts, 10000);
    updateConnectionStatus("connecting");
    setTimeout(connectWebSocket, delay);
  };

  ChatState.ws.onopen = function () {
    // Stale socket racing its replacement: ignore its lifecycle entirely.
    if (ChatState.ws !== ws) return;
    ChatState.reconnectAttempts = 0;
    ChatState.disconnectNotified = false;
    // Reset streaming state and reveal the app (the original onopen — its
    // logic was silently DISCARDED by a second onopen assignment below,
    // leaving the loading overlay stuck on every reconnect).
    ChatState.isStreaming = false;
    ChatState.streamingMessageId = null;
    ChatState.currentStreamingContent = "";
    ChatState.streamingBuffer = "";
    var loadingOverlay = document.getElementById("chatLoadingOverlay");
    if (loadingOverlay) loadingOverlay.style.display = "none";
    if (typeof window.showChatApp === "function") {
      window.showChatApp();
    }
    var params = new URLSearchParams(window.location.search);
    var q = params.get("q");
    if (q && typeof window.sendMessage === "function") {
      setTimeout(function () { window.sendMessage(q); }, 300);
    }
    // #1275 — the socket is healthy again: stop the “offline” state, then
    // flush messages queued while disconnected, in order, on THIS socket.
    updateConnectionStatus("connected");
    // #1668 — announce the fresh socket so the timeline can replay the
    // frames this client has not seen yet.
    try {
      window.dispatchEvent(new CustomEvent("gb-chat-connected", {
        detail: { session_id: ChatState.currentSessionId },
      }));
    } catch (e) { /* CustomEvent unavailable */ }
    var queued = Array.isArray(ChatState.offlineQueue) ? ChatState.offlineQueue : [];
    ChatState.offlineQueue = [];
    queued.forEach(function (payload) {
      try {
        // Rebind the queued turn to the session that is live NOW: a message
        // composed while the socket was still connecting captured an empty
        // session id, which the server drops without a trace (the first
        // message after the wizard looked silently lost).
        payload.bot_id = ChatState.currentBotId;
        payload.user_id = ChatState.currentUserId;
        payload.session_id = ChatState.currentSessionId;
        payload.active_switchers = Array.from(ChatState.activeSwitchers || []);
        ChatState.ws.send(JSON.stringify(payload));
        window.dispatchEvent(new CustomEvent("gb-chat-message-sent", {
          detail: { session_id: ChatState.currentSessionId, queued: true },
        }));
      } catch (e) {
        // Socket died again mid-flush: put the rest back and let onclose
        // schedule the next reconnect.
        ChatState.offlineQueue = [payload].concat(queued.slice(queued.indexOf(payload) + 1));
        return;
      }
    });
  };

  ChatState.ws.onerror = function () {
    updateConnectionStatus("disconnected");
  };
}

// cache-bust: 20260905a
