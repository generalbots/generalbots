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
        break;
      case "disconnected":
        statusText.textContent = "Disconnected";
        statusEl.style.display = "flex";
        break;
      case "connecting":
        statusText.textContent = "Connecting...";
        statusEl.style.display = "flex";
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

  ChatState.ws.onopen = function () {
    ChatState.disconnectNotified = false;
    updateConnectionStatus("connected");
    var loadingOverlay = document.getElementById("chatLoadingOverlay");
    var contentWrapper = document.getElementById("chatContentWrapper");
    if (loadingOverlay) loadingOverlay.style.display = "none";
    if (contentWrapper) contentWrapper.style.display = "flex";
  };

  ChatState.ws.onmessage = function (event) {
    try {
      var data = JSON.parse(event.data);

      if (data.type === "connected") {
        ChatState.reconnectAttempts = 0;
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

      if (data.message_type === MessageType.BOT_RESPONSE) {
        processMessage(data);
      }
    } catch (e) {}
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

  ChatState.ws.onerror = function () {
    updateConnectionStatus("disconnected");
  };
}
