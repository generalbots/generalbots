/* Split from chat-send.js for #2305 (450-line limit). */
function formatAttachSize(bytes) {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(1) + " MB";
}

function queueOfflineMessage(payload) {
  // A terminal access state will never open a socket: do not queue or promise delivery.
  if (ChatState.botAccess === "auth_required" || ChatState.botAccess === "probe_error" || window.chatAccessTerminal) {
    if (typeof window.clearBlockedChatTurn === "function") window.clearBlockedChatTurn();
    if (typeof window.gbRestoreBlockedComposer === "function") window.gbRestoreBlockedComposer();
    if (ChatState.botAccess === "auth_required" || window.chatAccessTerminal === "auth") notify(chatPrivateSigninMessage(), "error"); return;
  }
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

/* gbgov assistant parity (#2563): the send-busy guard.
 * Classic chat silently finalized the streaming bubble and sent the new turn,
 * so the previous answer vanished without explanation. The assistant shell
 * asks first: keep waiting, or stop the running turn and send now. Kept in the
 * already-loaded send bundle so no loader change is needed. */
(function () {
  "use strict";

  function t(key, fallback) {
    return typeof window.gbT === "function" ? window.gbT(key, fallback) : fallback;
  }
  function ensureStyles() {
    if (document.getElementById("gbChatBusySendStyles")) return;
    var style = document.createElement("style");
    style.id = "gbChatBusySendStyles";
    style.textContent = [
      ".gb-chat-busy-send{position:relative;z-index:var(--z-top,60);margin:8px 12px;padding:10px 12px;border:1px solid var(--id-warn,#d97706);border-radius:10px;background:var(--id-warn-soft,#fff7e6);color:var(--id-warn-text,#7c4a03);font-size:var(--id-text-sm,13px)}",
      ".gb-chat-busy-head{display:flex;align-items:center;gap:8px;font-weight:600}",
      ".gb-chat-busy-text{margin:6px 0 10px;line-height:1.4}",
      ".gb-chat-busy-actions{display:flex;gap:8px;flex-wrap:wrap}",
      ".gb-chat-busy-actions button{padding:5px 11px;border:1px solid var(--id-line,#d7dde5);border-radius:7px;background:var(--id-surface-solid,#fff);color:inherit;cursor:pointer;font:inherit}",
      ".gb-chat-busy-actions button:focus-visible{outline:2px solid var(--id-focus,#2563eb);outline-offset:2px}"
    ].join("");
    (document.head || document.documentElement).appendChild(style);
  }
  function focusComposer() {
    var input = document.getElementById("messageInput");
    if (!input || typeof input.focus !== "function") return;
    try { input.focus({ preventScroll: true }); } catch (e) { input.focus(); }
  }
  function busy() {
    var cs = window.ChatState;
    if (!cs) return false;
    // A terminal access failure has no answer worth waiting for: let chat's
    // own send path clear the stuck turn and show its failure notice.
    if (cs.botAccess === "auth_required" || cs.botAccess === "probe_error"
        || window.chatAccessTerminal === "auth" || window.chatAccessTerminal === "probe") return false;
    if (cs.isStreaming) return true;
    // A stale pendingResponseAt with an idle turn is a stuck turn, not a busy
    // one — chat's send path clears it. Only a live turn deserves the dialog.
    var state = window.GBTurn && typeof GBTurn.state === "function" ? GBTurn.state() : "idle";
    return ["queued", "thinking", "tool", "streaming", "progress"].indexOf(state) !== -1;
  }
  function stopTurn() {
    var cs = window.ChatState;
    // Keep chat's normal cleanup: the streaming bubble would otherwise stay
    // aria-hidden and collide with the bubble of the message being sent.
    if (cs && cs.isStreaming && typeof window.finalizeStreaming === "function") {
      window.finalizeStreaming();
    }
    if (window.GBTurn && typeof window.GBTurn.cancel === "function") window.GBTurn.cancel();
    if (cs) {
      cs.isStreaming = false;
      cs.pendingResponseAt = null;
      cs.streamingMessageId = null;
    }
    if (typeof window.clearChatPending === "function") window.clearChatPending();
  }
  function ask(go) {
    Array.prototype.forEach.call(document.querySelectorAll(".gb-chat-busy-send"), function (old) {
      if (old.parentNode) old.parentNode.removeChild(old);
    });
    ensureStyles();
    var box = document.createElement("div");
    box.className = "gb-chat-busy-send";
    box.setAttribute("role", "alertdialog");
    box.setAttribute("aria-labelledby", "gbChatBusyHead");
    box.setAttribute("aria-describedby", "gbChatBusyText");
    var head = document.createElement("div");
    head.className = "gb-chat-busy-head";
    head.id = "gbChatBusyHead";
    head.textContent = t("chat-busy-send-head", "The answer is still being written");
    var text = document.createElement("p");
    text.className = "gb-chat-busy-text";
    text.id = "gbChatBusyText";
    text.textContent = t("chat-busy-send-text", "Wait for the answer to finish, or stop it and send your message now.");
    var bar = document.createElement("div");
    bar.className = "gb-chat-busy-actions";
    var wait = document.createElement("button");
    wait.type = "button";
    wait.textContent = t("chat-busy-send-wait", "Keep waiting");
    var send = document.createElement("button");
    send.type = "button";
    send.textContent = t("chat-busy-send-stop", "Stop and send");
    function close() {
      if (box.parentNode) box.parentNode.removeChild(box);
      clearInterval(watch);
      document.removeEventListener("keydown", onKey, true);
    }
    function onKey(event) {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      close();
      focusComposer();
    }
    wait.addEventListener("click", function () { close(); focusComposer(); });
    send.addEventListener("click", function () { close(); focusComposer(); go(); });
    bar.appendChild(wait);
    bar.appendChild(send);
    box.appendChild(head);
    box.appendChild(text);
    box.appendChild(bar);
    // The answer may finish while the person is still deciding.
    var watch = setInterval(function () {
      if (!box.isConnected) { clearInterval(watch); return; }
      if (!busy()) close();
    }, 500);
    var form = document.getElementById("chatForm");
    var host = form || document.getElementById("messageInput");
    if (host && host.parentNode) host.parentNode.insertBefore(box, host);
    else document.body.appendChild(box);
    document.addEventListener("keydown", onKey, true);
    wait.focus();
  }

  var original = window.sendMessage;
  if (typeof original !== "function" || original.__gbBusySendGuard) return;
  var guarded = function () {
    if (!busy()) return original.apply(this, arguments);
    var self = this, args = arguments;
    ask(function () { stopTurn(); return original.apply(self, args); });
  };
  guarded.__gbBusySendGuard = true;
  window.sendMessage = guarded;
})();
