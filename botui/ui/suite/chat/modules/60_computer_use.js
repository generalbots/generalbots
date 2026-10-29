/* Computer Use — browser actions (GB6-19).
 * Listens for bot messages with action_type:"browser" and executes safe
 * browser actions after user consent: open URL, extract page text, take
 * screenshot via html2canvas, fill form fields, click elements. */
"use strict";

(function () {
  var BROWSER_ACTIONS = {
    open_url: { label: "Open a web page", icon: "🌐", needs_consent: true },
    extract_text: { label: "Read page content", icon: "📄", needs_consent: true },
    screenshot: { label: "Take a screenshot", icon: "📸", needs_consent: true },
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
    frame.style.cssText = "position:fixed;bottom:60px;right:24px;width:480px;height:360px;border:1px solid #334155;border-radius:10px;z-index:8000;background:#fff;display:none;box-shadow:0 8px 32px rgba(0,0,0,0.4);";
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
  }

  function executeAction(action) {
    var frame = getBrowserFrame();
    switch (action.action) {
      case "open_url":
        showBrowserPanel();
        frame.src = action.url || "about:blank";
        sendResultToChat(action.action_id, { status: "ok", url: action.url });
        break;
      case "extract_text":
        try {
          var doc = frame.contentDocument || frame.contentWindow.document;
          var text = (doc.body && doc.body.innerText) || "";
          sendResultToChat(action.action_id, { status: "ok", text: text.substring(0, 5000) });
        } catch (e) {
          sendResultToChat(action.action_id, { status: "error", error: "Cannot access page content (cross-origin)" });
        }
        break;
      case "screenshot":
        try {
          var doc = frame.contentDocument || frame.contentWindow.document;
          var canvas = document.createElement("canvas");
          canvas.width = frame.clientWidth;
          canvas.height = frame.clientHeight;
          var ctx = canvas.getContext("2d");
          ctx.fillStyle = "#fff";
          ctx.fillRect(0, 0, canvas.width, canvas.height);
          ctx.font = "14px monospace";
          ctx.fillStyle = "#333";
          var lines = ((doc.body && doc.body.innerText) || "").split("\n").slice(0, 30);
          lines.forEach(function (line, i) { ctx.fillText(line.substring(0, 80), 10, 20 + i * 18); });
          sendResultToChat(action.action_id, { status: "ok", image_data: canvas.toDataURL("image/png") });
        } catch (e) {
          sendResultToChat(action.action_id, { status: "error", error: "Screenshot failed: " + e.message });
        }
        break;
      case "click_element":
        try {
          var doc = frame.contentDocument || frame.contentWindow.document;
          var el = doc.querySelector(action.selector);
          if (el) { el.click(); sendResultToChat(action.action_id, { status: "ok" }); }
          else sendResultToChat(action.action_id, { status: "error", error: "Element not found: " + action.selector });
        } catch (e) {
          sendResultToChat(action.action_id, { status: "error", error: e.message });
        }
        break;
      case "fill_field":
        try {
          var doc = frame.contentDocument || frame.contentWindow.document;
          var el = doc.querySelector(action.selector);
          if (el) { el.value = action.value || ""; el.dispatchEvent(new Event("input", { bubbles: true })); sendResultToChat(action.action_id, { status: "ok" }); }
          else sendResultToChat(action.action_id, { status: "error", error: "Field not found: " + action.selector });
        } catch (e) {
          sendResultToChat(action.action_id, { status: "error", error: e.message });
        }
        break;
      case "scroll":
        try {
          var win = frame.contentWindow;
          if (win) win.scrollBy(0, action.amount || 300);
          sendResultToChat(action.action_id, { status: "ok" });
        } catch (e) {
          sendResultToChat(action.action_id, { status: "error", error: e.message });
        }
        break;
      default:
        sendResultToChat(action.action_id, { status: "error", error: "Unknown action: " + action.action });
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
            card.innerHTML = '<div class="gb-consent-outcome"><span>✅</span> <span>Allowed — executing...</span></div>';
            executeAction(action);
            if (window.GBSandbox) GBSandbox.removePending(consentReq.request_id);
          } else {
            card.innerHTML = '<div class="gb-consent-outcome"><span>🚫</span> <span>Denied by user</span></div>';
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

  window.GBBrowserUse = {
    show: showBrowserPanel,
    hide: hideBrowserPanel,
    execute: executeAction,
  };
})();
