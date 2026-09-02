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

function connectWebSocket() {
  if (ChatState.ws) ChatState.ws.close();
  updateConnectionStatus("connecting");

  var url = WS_URL +
    "?session_id=" + ChatState.currentSessionId +
    "&user_id=" + ChatState.currentUserId +
    "&bot_name=" + ChatState.currentBotName;

  ChatState.ws = new WebSocket(url);
  if (window.GBAppLifecycle) GBAppLifecycle.socket("chat", ChatState.ws);

ChatState.ws.onopen = function () {
  ChatState.disconnectNotified = false;
  ChatState.isStreaming = false;
  ChatState.streamingMessageId = null;
  ChatState.currentStreamingContent = "";
  ChatState.streamingBuffer = "";
  updateConnectionStatus("connected");
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
};

  ChatState.ws.onmessage = function (event) {
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
    // #1275 — the socket is healthy again: stop the “offline” state, then
    // flush messages queued while disconnected, in order, on THIS socket.
    ChatState.disconnectNotified = false;
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

// cache-bust: 1783796400
