/* Computer Use — consent-gated browser actions (GB7-19): open URL, read page
 * text, fill a field, click an element, scroll. */
"use strict";

(function () {
  // #1644 — the fake canvas "screenshot" action was removed.
  var BROWSER_ACTIONS = {
    open_url: { label: "Open a web page", icon: "🌐", needs_consent: true },
    extract_text: { label: "Read page content", icon: "📄", needs_consent: true },
    click_element: { label: "Click an element", icon: "👆", needs_consent: true },
    fill_field: { label: "Fill a form field", icon: "✏️", needs_consent: true },
    scroll: { label: "Scroll the page", icon: "📜", needs_consent: false },
  };

  var browserFrame = null;

  function getBrowserFrame() {
    if (browserFrame && document.body.contains(browserFrame)) return browserFrame;
    var existing = document.getElementById("gb-browser-frame");
    if (existing) { browserFrame = existing; return existing; }
    var frame = document.createElement("iframe");
    frame.id = "gb-browser-frame";
    frame.sandbox = "allow-scripts allow-forms";
    frame.style.cssText = "position:fixed;bottom:60px;right:24px;width:480px;height:360px;border:1px solid #334155;border-radius:10px;z-index:var(--z-overlay);background:var(--id-surface-solid);display:none;box-shadow:0 8px 32px color-mix(in srgb, var(--palette-000000) 40%, transparent);";
    document.body.appendChild(frame);
    browserFrame = frame;
    return frame;
  }

  function showBrowserPanel() {
    var frame = getBrowserFrame();
    frame.style.display = "block";
  }

  function hideBrowserPanel() {
    var frame = getBrowserFrame();
    frame.style.display = "none";
  }

  function sendResultToChat(actionId, result) {
    // Persist the outcome for reconnect replay; the WS frame is the live path.
    if (typeof clearChatPending === "function") clearChatPending();
    if (typeof hideThinkingIndicator === "function") hideThinkingIndicator();
    if (actionId && /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(actionId)) {
      var token = (typeof window.getChatAccessToken === "function" && window.getChatAccessToken()) ||
        (window.GBSecurity && window.GBSecurity.getToken && window.GBSecurity.getToken()) ||
        sessionStorage.getItem("gb-access-token") || "";
      fetch("/api/computer/actions/" + encodeURIComponent(actionId) + "/result", {
        method: "POST",
        headers: token
          ? { "Content-Type": "application/json", Authorization: "Bearer " + token }
          : { "Content-Type": "application/json" },
        body: JSON.stringify(result),
      }).catch(function () { /* the chat frame remains the live path */ });
    }
    if (ChatState && ChatState.ws && ChatState.ws.readyState === WebSocket.OPEN) {
      ChatState.ws.send(JSON.stringify({
        bot_id: ChatState.currentBotId,
        user_id: ChatState.currentUserId,
        session_id: ChatState.currentSessionId,
        message_type: "browser_result",
        action_id: actionId,
        result: result,
        timestamp: new Date().toISOString(),
      }));
    }
    // Show the outcome now, not on the next prompt.
    var summary;
    if (result && result.status === "ok") {
      summary = result.title
        ? "Opened: " + result.title + "\n" + (result.url || "") + "\n\n" +
          String(result.text || "").slice(0, 800)
        : "Action completed.";
    } else {
      summary = "Action failed: " + ((result && (result.error || result.message)) || "unknown error");
    }
    if (typeof addMessage === "function") addMessage("bot", summary);
  }

  function postJson(path, body) {
    var token = (typeof window.getChatAccessToken === "function" && window.getChatAccessToken()) ||
      (window.GBSecurity && window.GBSecurity.getToken && window.GBSecurity.getToken()) ||
      sessionStorage.getItem("gb-access-token") || "";
    return fetch(path, {
      method: "POST",
      headers: token
        ? { "Content-Type": "application/json", Authorization: "Bearer " + token }
        : { "Content-Type": "application/json" },
      body: JSON.stringify(body || {}),
    }).then(function (res) {
      if (!res.ok) {
        return res.json().catch(function () { return {}; }).then(function (payload) {
          throw new Error(payload.error || ("HTTP " + res.status));
        });
      }
      return res.json();
    });
  }

  function showServerResult(frame, result) {
    showBrowserPanel();
    try {
      var doc = frame.contentDocument || frame.contentWindow.document;
      doc.open();
      doc.write(
        '<div style="font:14px system-ui;padding:16px;color:var(--palette-0f172a);">' +
        '<h3 style="margin:0 0 8px;">' + escapeHtml(result.title || result.url || "Page") + "</h3>" +
        '<p style="color:#475569;margin:0 0 12px;">' + escapeHtml(result.url || "") + "</p>" +
        '<pre style="white-space:pre-wrap;font:12px/1.5 monospace;max-height:260px;overflow:auto;">' +
        escapeHtml((result.text || "").substring(0, 4000)) + "</pre></div>",
      );
      doc.close();
    } catch (e) { /* the panel is progressive; the result still reaches chat */ }
  }

  async function executeAction(action) {
    var actionId = action.action_id;
    // #1644 — approval is a server-side consent record.
    try {
      await postJson("/api/computer/actions/" + encodeURIComponent(actionId) + "/approve", {
        action_id: actionId,
      });
    } catch (e) {
      sendResultToChat(actionId, { status: "error", error: "Approval could not be recorded: " + e.message });
      return;
    }

    if (action.action === "open_url" || action.action === "extract_text") {
      var frame = getBrowserFrame();
      showBrowserPanel();
      try {
        var result = await postJson(
          "/api/computer/actions/" + encodeURIComponent(actionId) + "/execute",
          { approved: true },
        );
        showServerResult(frame, result);
        sendResultToChat(actionId, result);
      } catch (e) {
        sendResultToChat(actionId, { status: "error", error: e.message });
      }
      return;
    }

    switch (action.action) {
      case "click_element":
        try {
          var doc = getBrowserFrame().contentDocument || getBrowserFrame().contentWindow.document;
          var el = doc.querySelector(action.selector);
          if (el) { el.click(); sendResultToChat(actionId, { status: "ok" }); }
          else sendResultToChat(actionId, { status: "error", error: "Element not found: " + action.selector });
        } catch (e) {
          sendResultToChat(actionId, { status: "error", error: e.message });
        }
        break;
      case "fill_field":
        try {
          var frame = getBrowserFrame();
          var doc = frame.contentDocument || frame.contentWindow.document;
          var el = doc.querySelector(action.selector);
          if (el) {
            el.value = action.value || "";
            el.dispatchEvent(new Event("input", { bubbles: true }));
            sendResultToChat(actionId, { status: "ok" });
          } else sendResultToChat(actionId, { status: "error", error: "Field not found: " + action.selector });
        } catch (e) {
          sendResultToChat(actionId, { status: "error", error: e.message });
        }
        break;
      case "scroll":
        try {
          var win = getBrowserFrame().contentWindow;
          if (win) win.scrollBy(0, action.amount || 300);
          sendResultToChat(actionId, { status: "ok" });
        } catch (e) {
          sendResultToChat(actionId, { status: "error", error: e.message });
        }
        break;
      default:
        sendResultToChat(actionId, { status: "error", error: "Unknown action: " + action.action });
    }
  }

  document.addEventListener("gb-ws-frame", function (e) {
    var data = e.detail;
    if (!data) return;
    if (data.action_type !== "browser" && data.message_type !== "browser_action") return;
    var action = data.browser_action || data;
    if (!action.action) return;
    var info = BROWSER_ACTIONS[action.action] || { label: action.action, icon: "🔧", needs_consent: true };
    if (info.needs_consent) {
      var detail = info.icon + " " + info.label;
      if (action.url) detail += ": " + action.url;
      if (action.selector) detail += " [" + action.selector + "]";

      var consentReq = {
        request_id: action.action_id || ("browser-" + Date.now()),
        app_id: "browser",
        action_class: action.action,
        detail: detail,
      };

      if (window.GBSandbox) {
        GBSandbox.addPending({
          action_id: consentReq.request_id,
          category: "browser",
          detail: detail,
          risk: "medium",
        });
      }

      var origResolve = window.GBRenderConsentCard;
      // Waiting for a human decision is not waiting for the model.
      if (typeof clearChatPending === "function") clearChatPending();
      if (typeof hideThinkingIndicator === "function") hideThinkingIndicator();
      var pane = document.getElementById("messages");
      if (pane) {
        var wrap = document.createElement("div");
        wrap.className = "message bot";
        wrap.innerHTML = '<div class="message-content bot-message"></div>';
        var body = wrap.querySelector(".message-content");
        var card = document.createElement("div");
        card.className = "gb-consent-card";
        card.setAttribute("data-request-id", consentReq.request_id);
        card.innerHTML =
          '<div class="gb-consent-head"><span class="gb-consent-glyph">' + info.icon + '</span><div class="gb-consent-headings"><div class="gb-consent-title">Browser Action</div><div class="gb-consent-app">' + escapeHtml(info.label) + '</div></div></div>'
          + '<div class="gb-consent-detail">' + escapeHtml(detail) + '</div>'
          + '<div class="gb-consent-actions"><button type="button" class="gb-consent-btn primary" data-decision="allow">Allow</button><button type="button" class="gb-consent-btn danger" data-decision="deny">Deny</button></div>';
        card.addEventListener("click", function (ev) {
          var btn = ev.target.closest(".gb-consent-btn");
          if (!btn) return;
          var decision = btn.getAttribute("data-decision");
          card.querySelectorAll(".gb-consent-btn").forEach(function (b) { b.disabled = true; });
          card.classList.add("resolved");
          if (decision === "allow") {
            card.innerHTML = '<div class="gb-consent-outcome"><span>✅</span> <span data-i18n="chat-str-allowed-executing"></span></div>';
            executeAction(action);
            if (window.GBSandbox) GBSandbox.removePending(consentReq.request_id);
          } else {
            card.innerHTML = '<div class="gb-consent-outcome"><span>🚫</span> <span data-i18n="chat-str-denied-by-user"></span></div>';
            sendResultToChat(action.action_id || consentReq.request_id, { status: "denied" });
            if (window.GBSandbox) GBSandbox.removePending(consentReq.request_id);
          }
        });
        body.appendChild(card);
        pane.appendChild(wrap);
        if (typeof scrollToBottom === "function") scrollToBottom(true);
      }
    } else {
      executeAction(action);
    }
  });

  // Replay the server-side action queue as ws frames. Hidden/in-flight polls
  // skip; the slow cadence avoids forced layout per consent card.
  var POLL_MS = 10000, polling = false;
  function pollPendingBrowser() {
    if (document.hidden || polling) return;
    var pc = window.productConfig;
    if (!pc || pc.agent_backend_unavailable) return;
    var sid = window.ChatState && ChatState.currentSessionId;
    if (!sid) return;
    // #2945 — use the canonical admin accessor (SSO/localStorage token), not
    // the chat-only one. The central fetch interceptor will not overwrite an
    // Authorization header pre-set with a stale chat token, which 401s on the
    // desktop shell even though the session gate passed.
    var token = (typeof window.getGBAccessToken === "function" && window.getGBAccessToken()) || "";
    polling = true;
    fetch("/api/computer/actions/pending?kind=browser&session_id=" + encodeURIComponent(sid), {
      headers: token ? { Authorization: "Bearer " + token } : {},
    })
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (data) {
        (data && data.actions ? data.actions : []).forEach(function (entry) {
          document.dispatchEvent(new CustomEvent("gb-ws-frame", {
            detail: {
              message_type: "browser_action",
              action_type: "browser",
              browser_action: Object.assign({ action_id: entry.action_id }, entry.action || {}),
            },
          }));
        });
      })
      .catch(function () { /* offline: nothing to collect */ })
      .then(function () { polling = false; });
  }
  // #2943 — arm through the shared session gate; it owns the one sign-in hint
  // and arms nothing without a token. The helper loads lazily on chat tabs.
  var armed = false;
  function armPendingPoll() {
    var s = window.GBCollabPollScheduler;
    if (s && s.schedule) {
      if (!armed) { armed = true; window.GBCollabPollScheduler.schedule(pollPendingBrowser, POLL_MS); }
      return;
    }
    (window.__gbGateWaiters || (window.__gbGateWaiters = [])).push(armPendingPoll);
    if (window.__gbGateLoading) return;
    window.__gbGateLoading = 1;
    var el = document.createElement("script");
    el.src = "/suite/js/collab-comments/01_helpers.js";
    el.onload = function () { window.__gbGateWaiters.forEach(function (f) { f(); }); };
    document.head.appendChild(el);
  }
  armPendingPoll();

  window.GBBrowserUse = {
    show: showBrowserPanel,
    hide: hideBrowserPanel,
    execute: executeAction,
  };
})();
