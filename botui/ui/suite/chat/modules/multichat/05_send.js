"use strict";

// Vibe multi-chat (#1282/#1283/#1285-fe) — tab send path, delivery guard and pending queue.
// Split out of the former 22_multichat module for #1731 (450-line limit). Mechanical split:
// same functions, globals and DOM/network behavior. Load order matters — the
// module namespace lives on window.GBMultiChat, created by 01_state.js.

(function () {
  var MC = window.GBMultiChat;

  /* ── Sending from a specific tab ── */

  function clearTabAck(t) {
    if (!t) return;
    if (t.ackTimer) {
      clearTimeout(t.ackTimer);
      t.ackTimer = null;
    }
    t.pendingAck = null;
  }

  // A tab socket can look OPEN after a server restart and still drop the
  // frame. Arm a short delivery guard: if no frame comes back, reconnect and
  // resend once so a message never disappears silently.
  function armTabAck(t, payload) {
    if (!t) return;
    clearTabAck(t);
    var ack = { payload: payload, attempts: 1 };
    t.pendingAck = ack;
    function expire() {
      if (t.pendingAck !== ack) return;
      // An OPEN socket with no frame is usually a slow model, not a lost
      // message. Reconnecting here resent the prompt and produced two
      // answers. Only retry when the socket is actually closed.
      if (t.ws && t.ws.readyState === WebSocket.OPEN) {
        t.pendingAck = null;
        return;
      }
      if (ack.attempts >= 2) {
        t.pendingAck = null;
        if (typeof notify === "function") {
          notify("Message not delivered — reconnecting. Please try again.", "error");
        }
        return;
      }
      ack.attempts += 1;
      try {
        if (t.ws && t.ws.readyState < WebSocket.CLOSING) t.ws.close();
      } catch (e) { /* already closed */ }
      t.pending = t.pending || [];
      t.pending.push(ack.payload);
      MC.connectTab(t);
      t.ackTimer = setTimeout(expire, 8000);
    }
    t.ackTimer = setTimeout(expire, 7000);
  }

  MC.sendInTab = function (tabId, text, fileOrFiles) {
    var t = MC.tabs[tabId];
    if (!t) return;
    var files = Array.isArray(fileOrFiles)
      ? fileOrFiles.filter(Boolean)
      : (fileOrFiles ? [fileOrFiles] : []);
    var file = files[0] || null;
    text = (text || "").trim();
    if (!text && !files.length) return;
    // Parallel tabs ALWAYS use their own socket + session: the stock
    // pipeline's socket is bound to the default conversation, so routing
    // through it sent the message (and received the reply) under the WRONG
    // session — parallel tabs shared one history. The tab's own socket is
    // connected with the tab's session id.
    var display = text || ("📎 " + file.name);
    if (typeof window.armChatPending === "function") {
      window.armChatPending(text || (file && file.name) || "");
    }
    MC.appendToContainer(t.pane, "user", display);
    if (typeof showThinkingIndicator === "function") showThinkingIndicator();
    MC.pushScrollback(t, "user", display);
    MC.schedulePersist();
    var payload = {
      bot_id: t.botId || "default",
      user_id: t.userId || "",
      session_id: t.sessionId,
      channel: "web",
      content: text,
      message_type: 1,
      // #1753 — this tab's formats, never the global/other-tab set.
      active_switchers: Array.from(
        t.switchers instanceof Set ? t.switchers : []
      ),
      timestamp: new Date().toISOString(),
      client_capabilities: window.gbClientCapabilities
        ? window.gbClientCapabilities()
        : undefined,
    };
    function dispatch() {
      t.pending = t.pending || [];
      if (t.ws && t.ws.readyState === WebSocket.OPEN) {
        try {
          t.ws.send(JSON.stringify(payload));
          armTabAck(t, payload);
        } catch (e) {
          t.pending.push(payload);
          MC.connectTab(t);
        }
      } else {
        t.pending.push(payload);
        if (!t.ws || t.ws.readyState >= WebSocket.CLOSING) MC.connectTab(t);
      }
    }
    // Audio recordings and uploaded files in a parallel tab used to be
    // dropped: the router only forwarded text, so the attachment never
    // reached the tab's socket. Read it like the stock path does and send the
    // same `file` payload.
    if (files.length) {
      // The stock send path forwards every attachment as `files[]`; the tab
      // router used to send only the first one and silently drop the rest.
      var readers = files.map(function (attachment) {
        return new Promise(function (resolve, reject) {
          var reader = new FileReader();
          reader.onerror = function () { reject(attachment.name); };
          reader.onload = function () {
            resolve({
              name: attachment.name,
              content_base64: String(reader.result).split(",")[1] || reader.result,
            });
          };
          reader.readAsDataURL(attachment);
        });
      });
      Promise.all(readers)
        .then(function (attachments) {
          payload.files = attachments;
          payload.file = attachments[0];
          dispatch();
        })
        .catch(function () {
          notify("Could not read an attachment. Attach it again.", "error");
        });
    } else {
      dispatch();
    }
  };
  // Flush queued parallel-tab frames once their socket opens.
  (function () {
    // connectTab is private; wrap its socket-open behavior by observing the
    // pending queue on every state change instead of patching the closure.
    setInterval(function () {
      Object.keys(MC.tabs).forEach(function (id) {
        var t = MC.tabs[id];
        if (t && t.pending && t.pending.length && t.ws && t.ws.readyState === WebSocket.OPEN) {
          MC.sendPending(t);
        }
      });
    }, 1000);
  })();

  MC.tabByElement = function (el) {
    var id = el && el.getAttribute ? el.getAttribute("data-tab-id") : null;
    return id ? MC.tabs[id] || null : null;
  };

  MC.sendPending = function (t) {
    if (!t.pending || !t.pending.length) return;
    if (!t.ws || t.ws.readyState !== WebSocket.OPEN) return;
    var qd = t.pending.splice(0, t.pending.length);
    qd.forEach(function (p) {
      try { t.ws.send(JSON.stringify(p)); } catch (e) { t.pending.push(p); }
    });
  };

  /* ── Published for the sibling multichat modules ── */

  MC.clearTabAck = clearTabAck;
})();
