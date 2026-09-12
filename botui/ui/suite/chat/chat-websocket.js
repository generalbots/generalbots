function notify(message, type) {
  type = type || "info";
  if (window.GBAlerts) {
    if (type === "error") {
      window.GBAlerts.warning("Chat", message);
    } else {
      window.GBAlerts.info("Chat", message);
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
        statusText.textContent = "Connected";
        statusEl.style.display = "none";
        if (window.GBAppLifecycle) GBAppLifecycle.setState(null);
        break;
      case "disconnected":
        statusText.textContent = "Disconnected";
        statusEl.style.display = "flex";
        if (window.GBAppLifecycle) GBAppLifecycle.setState("error", "Disconnected from server. Reconnecting…");
        break;
      case "connecting":
        statusText.textContent = "Connecting...";
        statusEl.style.display = "flex";
        if (window.GBAppLifecycle) GBAppLifecycle.setState("loading", "Connecting…");
        break;
    }
  }
}

// A private bot refuses the WebSocket upgrade with 401, and the browser
// WebSocket API hides that response — the window only saw an endless
// "Disconnected / connecting" flap. Probe the bot first so a private bot with
// an anonymous visitor explains the real reason and offers a way in.
function probeBotAccess(done) {
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
    if (statusText) statusText.textContent = "Private bot — sign in required";
    statusEl.style.display = "flex";
  }
  var login = window.GB_LOGIN_URL || "/login";
  var ret = encodeURIComponent(window.location.href);
  var cta = document.getElementById("publicSignupCta");
  if (cta) {
    cta.innerHTML = '<span>This bot is private. Sign in with an account of its organization.</span>' +
      '<a href="' + login + "?redirect=" + ret + '">Sign in</a>';
    cta.style.display = "flex";
  }
  if (window.GBAppLifecycle) {
    GBAppLifecycle.setState("error", "This bot is private. Sign in to continue.");
  }
  notify("This bot is private — sign in to continue", "error");
  console.warn("[WS] bot '" + botName + "' is private; WebSocket not attempted");
}

function connectWebSocket() {
  if (ChatState.ws) ChatState.ws.close();
  updateConnectionStatus("connecting");
  probeBotAccess(function (state) {
    if (state === "auth_required") {
      showAuthRequired(ChatState.currentBotName);
      return;
    }
    dialChatSocket();
  });
}

function dialChatSocket() {
  var url = WS_URL +
    (WS_URL.indexOf("?") === -1 ? "?" : "&") +
    "session_id=" + ChatState.currentSessionId +
    "&user_id=" + ChatState.currentUserId +
    "&bot_name=" + ChatState.currentBotName;

  var ws = (ChatState.ws = new WebSocket(url));
  if (window.GBAppLifecycle) GBAppLifecycle.socket("chat", ChatState.ws);  ChatState.ws.onmessage = function (event) {
    try {
      var data = JSON.parse(event.data);

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
    if (ChatState.reconnectAttempts < ChatState.maxReconnectAttempts) {
      ChatState.reconnectAttempts++;
      updateConnectionStatus("connecting");
      setTimeout(connectWebSocket, 1000 * ChatState.reconnectAttempts);
    }
  };

  ChatState.ws.onopen = function () {
    // Stale socket racing its replacement: ignore its lifecycle entirely.
    if (ChatState.ws !== ws) return;
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
    var queued = Array.isArray(ChatState.offlineQueue) ? ChatState.offlineQueue : [];
    ChatState.offlineQueue = [];
    queued.forEach(function (payload) {
      try {
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
