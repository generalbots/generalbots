/* Split from chat-send.js for #2305 (450-line limit). */
function formatAttachSize(bytes) {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(1) + " MB";
}

function queueOfflineMessage(payload) {
  if (!Array.isArray(ChatState.offlineQueue)) ChatState.offlineQueue = [];
  ChatState.offlineQueue.push(payload);
  if (ChatState.offlineQueue.length > 50) ChatState.offlineQueue.shift();
  notify((typeof gbT === "function") ? gbT("chat-reconnecting-queued", "Reconnecting — your message will be sent when the connection is back.") : "Reconnecting — your message will be sent when the connection is back.", "info");
  if (typeof updateConnectionStatus === "function") updateConnectionStatus("connecting");
  if (ChatState.reconnectAttempts >= ChatState.maxReconnectAttempts) {
    // Reconnect budget was exhausted before the user tried to send: restart
    // it so the queued message actually goes out.
    ChatState.reconnectAttempts = 0;
    if (typeof connectWebSocket === "function") connectWebSocket();
  }
}

// #2155 UI-REG-01b: recover dead/stuck socket after restore/resume.
function ensureChatConnection() {
  if (typeof ChatState === "undefined") return;
  if (ChatState.botUnavailable || ChatState.botAccess === "auth_required") return;
  if (!ChatState.currentSessionId || !ChatState.currentBotName) return;
  var ws = ChatState.ws;
  if (ws && ws.readyState === WebSocket.OPEN) return;
  if (ws && ws.readyState === WebSocket.CONNECTING) return;
  if (typeof connectWebSocket === "function") connectWebSocket();
}

(function installChatReconnectGuards() {
  if (window.__gbChatReconnectGuards) return;
  window.__gbChatReconnectGuards = true;
  function resume() { setTimeout(ensureChatConnection, 0); }
  window.addEventListener("online", resume);
  window.addEventListener("pageshow", resume);
  document.addEventListener("visibilitychange", function () { if (!document.hidden) resume(); });
})();

window.sendMessage = sendMessage;

// #2360 — retry a failed turn using its original structured payload (text +
// attachments) instead of re-reading rendered prose.
window.gbResendChatPayload = function (originalPayload) {
  if (!originalPayload || typeof originalPayload !== "object") return;
  var payload = {};
  try {
    payload = JSON.parse(JSON.stringify(originalPayload));
  } catch (e) {
    payload = {};
    Object.keys(originalPayload).forEach(function (k) { payload[k] = originalPayload[k]; });
  }
  payload.session_id = ChatState.currentSessionId;
  payload.bot_id = ChatState.currentBotId;
  payload.user_id = ChatState.currentUserId;
  payload.client_id = ChatState.clientId;
  payload.timestamp = new Date().toISOString();
  // #2361 — preserve the original message_id for idempotency on retries.
  if (!payload.message_id || typeof payload.message_id !== "string" || payload.message_id === "") {
    try {
      if (typeof crypto !== "undefined" && crypto && typeof crypto.randomUUID === "function") {
        payload.message_id = crypto.randomUUID();
      } else {
        payload.message_id = "m-" + Date.now().toString(36) + "-" + Math.random().toString(36).slice(2, 10);
      }
    } catch (e) {
      payload.message_id = "m-" + Date.now().toString(36) + "-" + Math.random().toString(36).slice(2, 10);
    }
  }

  var text = payload.content || "";
  var hasFiles = !!(payload.files && payload.files.length) || !!payload.file;

  function doResend() {
    var sentPayload = payload;
    ChatState.ws.send(JSON.stringify(sentPayload));
    armChatPending(text || (hasFiles ? "attachment" : ""));
    ChatState.pendingSend = {
      payload: sentPayload,
      at: Date.now(),
      attempts: 1,
      acked: false,
    };
    setTimeout(function () {
      var pending = ChatState.pendingSend;
      if (!pending || pending.payload !== sentPayload) return;
      if (pending.acked) {
        ChatState.pendingSend = null;
        return;
      }
      if (pending.attempts >= 3) {
        ChatState.pendingSend = null;
        notify((typeof gbT === "function") ? gbT("chat-message-not-delivered", "Message not delivered — reconnecting") : "Message not delivered — reconnecting", "error");
        return;
      }
      ChatState.pendingSend = {
        payload: sentPayload,
        at: Date.now(),
        attempts: pending.attempts + 1,
        acked: false,
      };
      if (!Array.isArray(ChatState.offlineQueue)) ChatState.offlineQueue = [];
      ChatState.offlineQueue.push(sentPayload);
      if (typeof connectWebSocket === "function") connectWebSocket();
    }, 6000);
    var retryText = (typeof gbT === "function") ? gbT("chat-retry", "Retry") : "Retry";
    var retryingAttachmentText = (typeof gbT === "function") ? gbT("chat-retrying-attachment", "Retrying attachment") : "Retrying attachment";
    if (window.GBTurn) GBTurn.start(text || (hasFiles ? retryingAttachmentText : retryText));
    else if (typeof showThinkingIndicator === "function") showThinkingIndicator();
  }

  ChatState.lastUserTurnPayload = payload;
  if (ChatState.ws && ChatState.ws.readyState === WebSocket.OPEN) {
    doResend();
  } else {
    queueOfflineMessage(payload);
  }
};

// #1670 — the surface renderer submits forms and plan approvals here.
window.ChatSend = window.ChatSend || {};
window.ChatSend.sendStructured = function (detail) {
  detail = detail || {};
  if (detail.action === "approve_plan") {
    var steps = Array.isArray(detail.steps) ? detail.steps : [];
    var titles = steps.map(function (step) { return step.title || ""; }).filter(Boolean);
    var approvedPrefix = (typeof gbT === "function") ? gbT("chat-approved-plan", "Approved plan: ") : "Approved plan: ";
    sendMessage(approvedPrefix + titles.join(" → "), {
      plan_approval: { surface_id: detail.surface_id || null, steps: steps },
    });
    return;
  }
  var values = detail.values || {};
  var text = Object.keys(values).map(function (key) {
    return key + ": " + values[key];
  }).join("\n");
  sendMessage(text || "Continue.", {
    question_answers: values,
    question_id: detail.surface_id || null,
  });
};
