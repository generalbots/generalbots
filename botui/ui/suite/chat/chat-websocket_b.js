/* Split from chat-websocket.js for #2305 (450-line limit). #2547: plain replies are not JSON. */
function parseThemeEventContent(c) {
  if (typeof c !== "string" || c.trim().charAt(0) !== "{") return null;
  try { var o = JSON.parse(c); return o && o.event === "change_theme" ? o : null; } catch (e) { return null; }
}
function connectWebSocket() {
  // A bot that does not exist never gains a socket: redialling only replaced
  // the explicit error with an endless "Connecting…" (#2153). Block only the
  // bot the latch refers to so other bots can still connect.
  if (ChatState.botUnavailable && ChatState.botUnavailableFor === ChatState.currentBotName) {
    showBotUnavailable(ChatState.botUnavailableFor);
    return;
  }
  var seq = ++chatDialSeq;
  clearChatConnectTimeout();
  if (ChatState.ws) {
    // Detach lifecycles first: the superseded socket must not schedule dials.
    try { ChatState.ws.onclose = null; ChatState.ws.onopen = null; ChatState.ws.onerror = null; } catch (e) { window.gbLogError ? gbLogError(e, "connectWebSocket") : console.error("connectWebSocket", e); }
    try { ChatState.ws.close(); } catch (e) { window.gbLogError ? gbLogError(e, "connectWebSocket") : console.error("connectWebSocket", e); }
  }
  updateConnectionStatus("connecting");
  probeBotAccess(function (state) {
    if (seq !== chatDialSeq) return;
    if (state === "auth_required") {
      showAuthRequired(ChatState.currentBotName);
      return;
    }
    if (state !== "allowed") {
      showProbeError(ChatState.currentBotName);
      return;
    }
    dialChatSocket(seq);
  });
}

function getChatWsUrl() {
  var base = WS_BASE_URL + window.location.host + "/ws";
  var tok = null;
  try {
    if (typeof window.getChatAccessToken === "function") tok = window.getChatAccessToken();
    else if (window.GBSecurity && window.GBSecurity.getToken) tok = window.GBSecurity.getToken();
    else tok = sessionStorage.getItem("gb-access-token");
  } catch (e) { tok = null; }
  if (tok) base += "?token=" + encodeURIComponent(tok);
  return base;
}

function dialChatSocket(seq) {
  // Identity is established via the suite session cookie on the WS handshake.
  // A legacy token query param is included only when a current token is present.
  var base = getChatWsUrl();
  var url = base +
    (base.indexOf("?") === -1 ? "?" : "&") +
    "session_id=" + encodeURIComponent(ChatState.currentSessionId || "") +
    "&user_id=" + encodeURIComponent(ChatState.currentUserId || "") +
    "&bot_name=" + encodeURIComponent(ChatState.currentBotName || "default");

  var ws = (ChatState.ws = new WebSocket(url));
  if (window.GBAppLifecycle) GBAppLifecycle.socket("chat", ChatState.ws);
  clearChatConnectTimeout();
  ChatState.wsConnectTimeout = setTimeout(function () {
    ChatState.wsConnectTimeout = null;
    if (seq !== chatDialSeq || ChatState.ws !== ws) return;
    if (ws.readyState !== WebSocket.CONNECTING) return;
    try { ws.onclose = null; ws.onerror = null; } catch (e) { window.gbLogError ? gbLogError(e, "dialChatSocket") : console.error("dialChatSocket", e); }
    try { ws.close(); } catch (e) { window.gbLogError ? gbLogError(e, "dialChatSocket") : console.error("dialChatSocket", e); }
    ChatState.ws = null;
    updateConnectionStatus("disconnected");
    ChatState.reconnectAttempts++;
    updateConnectionStatus("connecting");
    setTimeout(connectWebSocket, Math.min(1000 * ChatState.reconnectAttempts, 10000));
  }, CHAT_CONNECT_TIMEOUT_MS);
  ChatState.ws.onmessage = function (event) {
    try {
      var data = JSON.parse(event.data);
      // Any frame proves the socket is alive; the send guard in chat-init
      // uses this to decide whether a message needs to be resent.
      ChatState.lastFrameAt = Date.now();

      // Server receipt for a chat turn. The send guard in chat-init waits for
      // this instead of "any frame", so a message accepted by the socket but
      // lost in a dying connection is resent instead of silently ignored.
      if (data.type === "message_received") { if (ChatState.pendingSend) ChatState.pendingSend.acked = true; return; }

      if (data.type === "connected") { ChatState.reconnectAttempts = 0; ChatState.currentUserId = data.user_id || ChatState.currentUserId; return; }

      if (data.event) { if (data.event === "change_theme") applyThemeData(data.data || {}); return; }

      var contentObj = parseThemeEventContent(data.content);
      if (contentObj) { applyThemeData(contentObj.data || {}); return; }

      if (data.type === "chat_status") { if (typeof setChatStatus === "function") setChatStatus(data.text || ""); return; }
      // #2098 — the server accepted our cancel; the turn is over.
      // #2179 — live document deltas streamed over the session WS.
      if (data.event === "gb-doc-delta") {
        try { document.dispatchEvent(new CustomEvent("gb-doc-delta", { detail: data })); } catch (e) { window.gbLogError ? gbLogError(e, "onmessage") : console.error("onmessage", e); }
        return;
      }

      if (data.type === "run_cancelled") { if (window.GBTurn) GBTurn.forceHide(); return; }
      if (data.type === "gb.steering_queued") { notifySteering(data.message || "Queued for the next step."); return; }
      // #1701 — the queued correction is being applied inside the running turn.
      if (data.type === "gb.steer") { notifySteering("Steering: " + (data.message || "")); return; }
      // #2098 — one turn state machine owns the progress element: keepalive
      // frames keep it alive, step/tool frames label it ("Running search…"),
      // and only the completion frame ends the turn. This runs before the
      // timeline so a timeline exception can never strand the indicator.
      if (window.GBTurn) {
        try { GBTurn.observe(data); } catch (e) { console.warn("[WS] turn state failed:", e); }
      }
      // #1668 — every run frame feeds the timeline, regardless of agent mode.
      if (data.seq) { try { window.dispatchEvent(new CustomEvent("gb-run-frame", { detail: { frame: data } })); } catch (e) { window.gbLogError ? gbLogError(e, "onmessage") : console.error("onmessage", e); } }
      if (data.type === "run_resume_done") { try { window.dispatchEvent(new CustomEvent("run_resume_done", { detail: data })); } catch (e) { window.gbLogError ? gbLogError(e, "onmessage") : console.error("onmessage", e); } }
      if (window.RunTimeline) { try { window.RunTimeline.handleFrame(data); } catch (e) { console.warn("[WS] run timeline frame failed:", e); } }
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
        } else if (data.data.name === "gb.subagent" && window.GBSubagents) {
          window.GBSubagents.apply(data.data.payload || {});
        }
      }
      if (window.AgentMode && data.type && ["thought_process", "terminal_output", "browser_ready", "step_progress", "step_complete", "todo_update", "agent_status", "file_created"].indexOf(data.type) !== -1) {
        window.AgentMode.handleMessage(data);
      }

      if (data.css && typeof data.css === "string" && data.css.length > 0) {
        var cssId = "bot-style-" + ChatState.currentBotName;
        if (!document.getElementById(cssId)) {
          var styleEl = document.createElement("style");
          styleEl.id = cssId; styleEl.textContent = data.css;
          document.head.appendChild(styleEl);
          window.__cssInjected = (window.__cssInjected || 0) + 1;
        }
      }

      // While a parallel conversation tab is active, the stock socket's
      // replies belong to the default conversation and must not paint into
      // the parallel pane (that was the "output from one process in another"
      // symptom). They are stored server-side and reappear on the default tab.
      var parallelActive = window.GBMultiChat && GBMultiChat.activeId &&
        GBMultiChat.activeId !== "tab-default";
      if (parallelActive &&
          (data.message_type === MessageType.USER ||
           data.message_type === MessageType.BOT_RESPONSE ||
           data.message_type === MessageType.UI_ACTION)) {
        return;
      }

      // #cross-browser-sync — a user message broadcast from another subscriber
      // (another browser/tab on the same session). Render it as a user bubble
      // unless we are the sender (we already added it locally).
      if (data.message_type === MessageType.USER || data.message_type === 1) {
        // Dedup: skip rendering if this is our own message (we already added
        // it locally). Use client_id, not user_id, because the same user can
        // be connected from multiple browsers.
        if (data.sender_client_id && data.sender_client_id === ChatState.clientId) {
          return;
        }
        if (!data.sender_client_id && data.sender_id && data.sender_id === ChatState.currentUserId) {
          // Fallback for frames without client_id (old server).
          return;
        }
        if (data.content && typeof data.content === "string" && data.content.trim() !== "") {
          if (typeof addMessage === "function") {
            addMessage("user", data.content, data.message_id || data.id || null, null, data.timestamp);
          }
        }
        return;
      }

      if (data.message_type === MessageType.BOT_RESPONSE) {
        // #2084/#2098 — the pending timer and the "Waiting for the model…"
        // status are cleared by the turn state machine: an assistant frame
        // that carries no content (thinking, tool boundary, retry) must keep
        // the progress element, and only the completion frame ends the turn.
        if (window.GBTurn) GBTurn.touch();
        // #2351 — never log reply text (it is user data) in production; keep
        // the frame trace behind an explicit debug flag instead.
        if (window.__GB_DEBUG_CHAT_WS__) {
          console.debug("[WS] frame complete=" + data.is_complete + " len=" + (data.content || "").length);
        }
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
    clearChatConnectTimeout();
    updateConnectionStatus("disconnected");
    if (!ChatState.disconnectNotified) {
      notify((typeof window.gbT === "function" ? window.gbT("chat-disconnected", "Disconnected from chat server") : "Disconnected from chat server"), "error");
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
    clearChatConnectTimeout();
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
      // Consume ?q= once so reconnects do not re-send the same deep-link turn.
      params.delete("q");
      var newSearch = params.toString();
      var newUrl = window.location.pathname + (newSearch ? "?" + newSearch : "") + window.location.hash;
      window.history.replaceState({}, "", newUrl);
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
    clearChatConnectTimeout();
    updateConnectionStatus("disconnected");
  };
}

/* gbgov assistant parity (#2562): optimistic cancellation.
 * GBTurn.cancel() removes the turn immediately, but frames already in flight
 * can still arrive. Without a guard, a late content frame re-enters
 * processMessage(), sets ChatState.isStreaming and strands the composer as if
 * the cancelled turn were running again. Suppress those frames for a short
 * bounded window; the next send clears the marker. */
(function () {
  "use strict";
  var WINDOW_MS = 3000;

  function suppressed(data) {
    var cs = window.ChatState;
    if (!cs || !cs.cancelledAt) return false;
    if (Date.now() - Number(cs.cancelledAt) > WINDOW_MS) return false;
    // A live turn belongs to the new request, never to the cancelled one.
    if (window.GBTurn && typeof GBTurn.active === "function" && GBTurn.active()) return false;
    return !!(data && (
      data.is_complete || data.content || data.thinking || data.message_type === 2
    ));
  }

  var original = window.processMessage;
  if (typeof original !== "function" || original.__gbCancelGuard) return;
  var wrapped = function (data) {
    if (suppressed(data)) return;
    return original.apply(this, arguments);
  };
  wrapped.__gbCancelGuard = true;
  window.processMessage = wrapped;

  window.addEventListener("gb-chat-message-sent", function () {
    if (window.ChatState) window.ChatState.cancelledAt = 0;
  });
})();
